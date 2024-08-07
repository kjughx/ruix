use core::arch::asm;
use core::mem::MaybeUninit;

use crate::cpu::{InterruptFrame, Registers};
use crate::paging::{pagedirectory::PageDirectory, Paging, PAGE_ACCESS_ALL, PAGE_IS_PRESENT};
use crate::paging::{Addr, KernelPage, PAGE_IS_WRITABLE};
use crate::process::{CurrentProcess, Process};
use crate::sync::{Shared, Weak};

pub mod tss;

// TODO: Implement a task-list

pub struct CurrentTask;
impl CurrentTask {
    pub fn get() -> Shared<Task> {
        CurrentProcess::get().with_rlock(|current| current.task())
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
        // TODO: Change current task to @task
        task.with_rlock(|task| Paging::switch(&task.page_directory))
    }

    pub fn save(mut task: Shared<Task>, frame: InterruptFrame) {
        task.with_wlock(|task| task.registers.save(frame));
    }

    pub fn copy_from_task<T: Copy>(task: Shared<Task>, vaddr: Addr) -> T {
        let new_user_page: *mut T = alloc!(core::mem::size_of::<T>());
        task.with_rlock(|task| {
            let mut page_dir = task.page_directory;
            let addr = Addr(new_user_page as usize);
            let old = page_dir.get_entry(addr);
            page_dir.map(
                addr,
                addr,
                PAGE_ACCESS_ALL | PAGE_IS_PRESENT | PAGE_IS_WRITABLE,
            );
            Paging::switch(&page_dir);
            unsafe { new_user_page.write(*(vaddr.0 as *const T)) };
            page_dir.map(addr, old.addr(), old.flags());
        });
        KernelPage::switch();
        let mut t: MaybeUninit<T> = MaybeUninit::uninit();
        unsafe {
            core::ptr::copy_nonoverlapping(new_user_page as *const T, t.as_mut_ptr(), 1);
            free!(new_user_page);
            t.assume_init()
        }
    }

    pub fn copy_stack_item<T: Copy>(task: Shared<Task>, idx: usize) -> T {
        let vaddr = task.with_rlock(|task| task.registers.sp) + idx * core::mem::size_of::<usize>();
        Self::copy_from_task(task, Addr(vaddr))
    }

    #[naked]
    unsafe extern "C" fn task_return(registers: *const Registers) {
        asm!("nop", options(noreturn))
    }
}

impl Drop for Task {
    fn drop(&mut self) {
        self.page_directory.free()
    }
}
