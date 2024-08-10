use crate::platform::Paging_;
use core::mem::MaybeUninit;

use crate::platform::x86::{
    cpu::{InterruptFrame, Registers},
    paging::{
        pagedirectory::PageDirectory, Addr, KernelPage, Paging, PAGE_ACCESS_ALL, PAGE_IS_PRESENT,
        PAGE_IS_WRITABLE,
    },
    process::{CurrentProcess, Process},
};
use crate::sync::{Shared, Weak};

pub mod tss;

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

    pub fn copy_from_task<T: Copy>(task: Shared<Task>, vaddr: Addr) -> T {
        let new_user_page: *mut T = alloc!(core::mem::size_of::<T>());
        task.with_rlock(|task| {
            let mut page_dir = task.page_directory;
            let addr = Addr(new_user_page as usize);
            let old = page_dir.get_entry(addr);
            page_dir.map(
                addr.0,
                addr.0,
                (PAGE_ACCESS_ALL | PAGE_IS_PRESENT | PAGE_IS_WRITABLE) as usize,
            );
            Paging::switch(&page_dir);
            unsafe { new_user_page.write(*(vaddr.0 as *const T)) };
            page_dir.map(addr.0, old.addr().0, old.flags() as usize);
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
}

impl Drop for Task {
    fn drop(&mut self) {
        self.page_directory.free()
    }
}
