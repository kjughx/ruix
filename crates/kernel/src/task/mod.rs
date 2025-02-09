use crate::boxed::Array;
use core::arch::naked_asm;
use core::mem::MaybeUninit;

use crate::cpu::{InterruptFrame, Registers};
use crate::paging::{pagedirectory::PageDirectory, Paging, PAGE_ACCESS_ALL, PAGE_IS_PRESENT};
use crate::paging::{Addr, KernelPage, PAGE_IS_WRITABLE, PAGE_SIZE};
use crate::process::{CurrentProcess, Process};
use crate::sync::{Shared, Weak};

// TODO: Implement a task-list

pub struct CurrentTask;
impl CurrentTask {
    pub fn get() -> Shared<Task> {
        CurrentProcess::get().with_rlock(|current| current.task())
    }

    pub fn paging_switch() {
        CurrentTask::get().with_rlock(|task| {
            Paging::switch(&task.page_directory);
        });
    }
}

pub struct Task {
    pub page_directory: PageDirectory,
    pub registers: Registers,
    pub process: Weak<Process>,
}

impl Task {
    pub fn new(process: Weak<Process>, entry: Option<usize>) -> Self {
        let page_directory =
            PageDirectory::new(PAGE_IS_PRESENT | PAGE_IS_WRITABLE | PAGE_ACCESS_ALL);

        let mut registers = Registers::user_default();
        if let Some(entry) = entry {
            registers.ip = entry;
        }

        Self {
            page_directory,
            registers,
            process,
        }
    }

    pub fn switch(task: Shared<Task>) {
        task.with_rlock(|task| Paging::switch(&task.page_directory))
    }

    pub fn save(mut task: Shared<Task>, frame: InterruptFrame) {
        task.with_wlock(|task| task.registers.save(frame));
    }

    pub fn copy_from_task<T: Copy>(task: &Shared<Task>, vaddr: Addr) -> T {
        let tmp: *mut T = alloc!(core::mem::size_of::<T>());
        let tmp_addr = Addr(tmp as usize);

        task.with_rlock(|task| {
            let mut pages = task.page_directory;
            let old = pages.get_entry(tmp_addr);
            pages.map(
                tmp_addr,
                tmp_addr,
                PAGE_ACCESS_ALL | PAGE_IS_PRESENT | PAGE_IS_WRITABLE,
            );
            Paging::switch(&task.page_directory);
            unsafe { tmp.write(*(vaddr.0 as *const T)) };

            pages.map(tmp_addr, old.addr(), old.flags());
        });

        KernelPage::switch();
        let mut t = MaybeUninit::uninit();
        unsafe {
            t.write(tmp);
            free!(tmp);
            *t.assume_init()
        }
    }

    pub fn copy_slice_from_task<T: Copy>(task: Shared<Task>, start: Addr, n: usize) -> Array<T> {
        // FIXME: tmp cannot span more than a page
        if n * n * core::mem::size_of::<T>() > PAGE_SIZE {
            unimplemented!("Copies over page boundaries");
        }

        let tmp: *mut T = alloc!(n * core::mem::size_of::<T>());
        let tmp_addr = Addr(tmp as usize);

        task.with_rlock(|task| {
            let mut pages = task.page_directory;
            let old = pages.get_entry(start);

            pages.map(
                tmp_addr,
                tmp_addr,
                PAGE_ACCESS_ALL | PAGE_IS_PRESENT | PAGE_IS_WRITABLE,
            );

            Paging::switch(&task.page_directory);
            for i in 0..n {
                unsafe { tmp.add(i).write(*((start.0 as *const T).add(i))) }
            }
            pages.map(tmp_addr, old.addr(), old.flags());
        });

        KernelPage::switch();
        let mut t = Array::new(n);
        unsafe {
            for i in 0..n {
                t[i] = *tmp.offset(i as isize);
            }
        }
        free!(tmp);
        t
    }

    pub fn copy_stack_item<T: Copy>(task: &Shared<Task>, idx: usize) -> T {
        let vaddr = task.with_rlock(|task| task.registers.sp) + idx * core::mem::size_of::<usize>();
        Self::copy_from_task(&task, Addr(vaddr))
    }

    #[naked]
    unsafe extern "C" fn task_return(registers: *const Registers) {
        naked_asm!("nop")
    }
}

impl Drop for Task {
    fn drop(&mut self) {
        self.page_directory.free()
    }
}
