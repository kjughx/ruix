use core::arch::asm;

use crate::cpu::{InterruptFrame, Registers};
use crate::paging::PAGE_IS_WRITABLE;
use crate::paging::{pagedirectory::PageDirectory, Paging, PAGE_ACCESS_ALL, PAGE_IS_PRESENT};
use crate::process::Process;
use crate::sync::{Shared, Weak};

pub mod tss;

// TODO: Implement a task-list

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

    #[naked]
    unsafe extern "C" fn task_return(registers: *const Registers) {
        asm!("nop", options(noreturn))
    }
}
