use core::marker::PhantomData;

use global::global;

use crate::boxed::Array;
use crate::cpu::CPU;
use crate::fs::{FileMode, VFS};
use crate::loader;
use crate::loader::elf::Elf;
use crate::paging::{Addr, PAGE_SIZE};
use crate::paging::{PAGE_ACCESS_ALL, PAGE_IS_PRESENT, PAGE_IS_WRITABLE};
use crate::path::Path;
use crate::sync::{Shared, Weak};
use crate::task::Task;

const USER_STACK_SIZE: usize = 16 * 1024;
const USER_STACK_START: usize = 0x3FF000;
const USER_STACK_END: usize = USER_STACK_START - USER_STACK_SIZE;
const USER_VIRTUAL_START: usize = 0x400000;
const MAX_PROCESSES: usize = 12;

global!(
    ProcessList,
    [Option<Shared<Process>>; MAX_PROCESSES],
    [const { None }; MAX_PROCESSES],
    "PROCESSES"
);
global!(Current, usize, 0, "CURRENT_PROCESS");

impl Current {
    pub fn assign(id: usize) {
        Current::get_mut().with_wlock(|current| *current = id)
    }
}

pub struct CurrentProcess;
impl CurrentProcess {
    pub fn get() -> Shared<Process> {
        Current::get().with_rlock(|id| {
            ProcessList::get().with_rlock(|processes| {
                if let Some(ref proc) = processes[*id] {
                    proc.clone()
                } else {
                    processes[0].as_ref().unwrap().clone()
                }
            })
        })
    }
}

#[derive(Debug)]
pub enum ProcessError {
    InvalidFormat,
    Other,
}

// Pointer to the data and its size
enum ProcessData {
    Binary(Array<u8>, PhantomData<[u8]>),
    Elf(Elf),
}

pub struct Processes;
impl Processes {
    pub fn get(id: usize) -> Option<Shared<Process>> {
        if id >= MAX_PROCESSES {
            return None;
        }

        ProcessList::get().with_rlock(|array| -> Option<Shared<Process>> {
            // Clippy doesn't understand what's going on
            #[allow(clippy::useless_asref)]
            array[id].as_ref().map(|process| process.clone())
        })
    }

    /// # Safety: Unsafe because it trusts the PROCESSES is locked
    unsafe fn find_slot() -> Option<usize> {
        for i in 0..MAX_PROCESSES {
            unsafe {
                if ProcessList::get_mut().force()[i].is_none() {
                    return Some(i);
                }
            }
        }
        None
    }

    fn insert(mut process: Shared<Process>) -> Option<usize> {
        unsafe {
            ProcessList::get_mut().with_wlock(|list| -> Option<usize> {
                let id = Self::find_slot()?;

                process.with_wlock(|process| process.id = id);
                list[id] = Some(process);
                Some(id)
            })
        }
    }
}

struct ProcessBare {
    task: Task,
    data: ProcessData,
    bss: Option<*const ()>,
}

pub struct Process {
    id: usize,
    task: Shared<Task>,
    // TODO: Track allocations
    data: ProcessData,
    bss: Option<*const ()>,
    stack: *const (),
    _bss_marker: PhantomData<[u8]>,
    _stack_marker: PhantomData<[u8]>,

    _mark_dead: bool, // If true, the process is effectively dead and should be cleaned-up
}

impl Process {
    pub fn new(filename: &str) -> Result<Shared<Process>, ProcessError> {
        let bare = match Self::new_elf(filename) {
            Err(ProcessError::InvalidFormat) => Self::new_binary(filename)?,
            bare => bare?,
        };

        let process = Self::from_bare(bare);

        if let Some(id) = Processes::insert(process.clone()) {
            Current::assign(id);
        }

        Ok(process)
    }

    fn from_bare(mut bare: ProcessBare) -> Shared<Self> {
        // Stack
        let stack: *const () = alloc!(USER_STACK_SIZE);
        bare.task.page_directory.map_range(
            Addr(USER_STACK_END),
            Addr(stack as usize),
            Addr(stack as usize + USER_STACK_SIZE).align_upper(),
            PAGE_IS_PRESENT | PAGE_ACCESS_ALL | PAGE_IS_WRITABLE,
        );

        let mut process = Shared::new(Self {
            id: 0,
            task: Shared::new(bare.task),
            data: bare.data,
            bss: bare.bss,
            stack,
            _bss_marker: PhantomData,
            _stack_marker: PhantomData,
            _mark_dead: false,
        });

        let weak = Shared::weak(&process);

        process.with_wlock(|process| process.task.with_wlock(|task| task.process = weak));

        process
    }

    pub fn exec(proc: Shared<Self>) {
        let task = proc.with_rlock(|inner| inner.task());
        unsafe { CPU::return_to_task(task) }
    }

    pub fn task(&self) -> Shared<Task> {
        self.task.clone()
    }

    pub fn idle() -> Shared<Process> {
        let mut task = Task::new(Weak::new(), None);

        let s = &[235, 254];
        let program_data: Array<u8> = Array::from(&s[..]);

        // Map the memory
        {
            let directory = &mut task.page_directory;
            // Code
            directory.map_range(
                Addr(USER_VIRTUAL_START),
                Addr(program_data.as_ptr() as usize),
                Addr(program_data.as_ptr() as usize + 2).align_upper(),
                PAGE_IS_PRESENT | PAGE_ACCESS_ALL,
            );
        }

        let bare = ProcessBare {
            task,
            data: ProcessData::Binary(program_data, PhantomData),
            bss: None,
        };

        Self::from_bare(bare)
    }

    fn new_elf(filename: &str) -> Result<ProcessBare, ProcessError> {
        let elf = match Elf::load(filename) {
            Err(loader::Error::BadFormat) => return Err(ProcessError::InvalidFormat),
            a => a.unwrap(),
        };

        let mut task = Task::new(Weak::new(), Some(elf.entry_point()));

        // Map the memory
        let bss = {
            let mut bss: *const u8 = core::ptr::null();
            let directory = &mut task.page_directory;
            for pheader in &elf.pheaders() {
                let flags = {
                    let mut f = PAGE_IS_PRESENT | PAGE_ACCESS_ALL;
                    if pheader.is_writable() {
                        f |= PAGE_IS_WRITABLE;
                    }
                    f
                };

                if pheader.vaddr() % PAGE_SIZE != 0 {
                    continue;
                }

                // BSS Section
                if pheader.filesz() == 0 && pheader.memsz() > 0 {
                    assert!(bss.is_null(), "Many BSS sections? :O");
                    bss = alloc!(pheader.memsz());
                }

                directory.map_range(
                    Addr(pheader.vaddr()).align_lower(),
                    Addr(pheader.paddr()).align_lower(),
                    Addr(pheader.paddr() + pheader.memsz()).align_upper(),
                    flags,
                )
            }
            bss
        };

        Ok(ProcessBare {
            task,
            data: ProcessData::Elf(elf),
            bss: Some(bss as *const ()),
        })
    }

    fn new_binary(filename: &str) -> Result<ProcessBare, ProcessError> {
        let fd = VFS::open(Path::new(filename), FileMode::ReadOnly).unwrap();
        let size = fd.stat().size;

        let program_data = fd.read_all().unwrap();

        let mut task = Task::new(Weak::new(), None);

        // Map the memory
        {
            let directory = &mut task.page_directory;
            // Code
            directory.map_range(
                Addr(USER_VIRTUAL_START),
                Addr(program_data.as_ptr() as usize),
                Addr(program_data.as_ptr() as usize + size).align_upper(),
                PAGE_IS_PRESENT | PAGE_ACCESS_ALL,
            );
        }

        Ok(ProcessBare {
            task,
            data: ProcessData::Binary(program_data, PhantomData),
            bss: None,
        })
    }

    /// This marks the process as dead.
    /// Touching it after this is undefined behaviour.
    pub fn mark_dead(mut this: Shared<Process>, _: i32) {
        this.with_wlock(|process| process._mark_dead = true);

        // TODO: How do we clean up the memory of the process?
        //  This is called in the syscall handler so cannot block
        // Make a garbage-collecting worker thread.

        // TODO: Change to a more reasonable process.
        Current::assign(0);
    }
}
