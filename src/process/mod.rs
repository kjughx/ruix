use crate::fs::{FileMode, Vfs};
use crate::heap::alloc;
use crate::paging::Addr;
use crate::paging::{PAGE_ACCESS_ALL, PAGE_IS_PRESENT, PAGE_IS_WRITABLE};
use crate::path::Path;
use crate::string::Str;
use crate::sync::{Global, Shared, Weak};
use crate::task::Task;

const USER_STACK_SIZE: usize = 16 * 1024;
const USER_STACK_START: usize = 0x3FF000;
const USER_STACK_END: usize = USER_STACK_START - USER_STACK_SIZE;
const USER_VIRTUAL_START: usize = 0x400000;
const MAX_PROCESSES: usize = 12;
static mut PROCESSES: Global<[Option<Shared<Process>>; MAX_PROCESSES]> =
    Global::new(|| [const { None }; MAX_PROCESSES], "PROCESSES");

#[derive(Debug)]
pub enum ProcessError {
    Other,
}

// Pointer to the data and its size
enum ProcessData {
    Binary(*const u8, usize),
}

pub struct Processes;
impl Processes {
    pub fn get(id: usize) -> Option<Shared<Process>> {
        if id >= MAX_PROCESSES {
            return None;
        }

        unsafe {
            PROCESSES.with_rlock(|array| -> Option<Shared<Process>> {
                // Clippy doesn't understand what's going on
                #[allow(clippy::useless_asref)]
                array[id].as_ref().map(|process| process.clone())
            })
        }
    }

    /// # Safety: Unsafe because it trusts the PROCESSES is locked
    unsafe fn find_slot() -> Option<usize> {
        for i in 0..MAX_PROCESSES {
            unsafe {
                if PROCESSES.force()[i].is_none() {
                    return Some(i);
                }
            }
        }
        None
    }

    /// # Safety: Unsafe because it trusts the PROCESSES is locked
    unsafe fn insert(id: usize, process: Process) {
        unsafe {
            PROCESSES.force()[id] = Some(Shared::new(process));
        }
    }
}

pub struct Process {
    id: u16,
    filename: Str,
    pub task: Task,
    // TODO: Track allocations
    data: ProcessData,
    stack: *const (),
}

impl Process {
    pub fn new(filename: &str) -> Result<Process, ProcessError> {
        Self::new_binary(filename)
    }

    fn new_binary(filename: &str) -> Result<Process, ProcessError> {
        // No races for this
        unsafe { PROCESSES.wlock() };
        let id = unsafe { Processes::find_slot().expect("Available slots") };

        let fd = Vfs::open(Path::new(filename), FileMode::ReadOnly).unwrap();
        let size = fd.stat().size;

        let program_data = unsafe {
            let ptr = alloc(size);
            let slice = core::ptr::slice_from_raw_parts_mut(ptr, size)
                .as_mut()
                .unwrap();
            fd.read(size, 1, slice).unwrap();
            ptr
        };

        let stack = alloc(USER_STACK_SIZE);
        let mut task = Task::new(Weak::new());

        {
            let directory = &mut task.page_directory;
            // Code
            directory.map_range(
                Addr(USER_VIRTUAL_START),
                Addr(program_data as usize),
                Addr(program_data as usize + size).align_upper(),
                PAGE_IS_PRESENT | PAGE_ACCESS_ALL,
            );

            // Stack
            directory.map_range(
                Addr(USER_STACK_END),
                Addr(stack as usize),
                Addr(stack as usize + USER_STACK_SIZE).align_upper(),
                PAGE_IS_PRESENT | PAGE_ACCESS_ALL | PAGE_IS_WRITABLE,
            );
        }

        Ok(Self {
            id: id as u16,
            filename: Str::from(filename),
            task,
            data: ProcessData::Binary(program_data, size),
            stack,
        })
    }
}
