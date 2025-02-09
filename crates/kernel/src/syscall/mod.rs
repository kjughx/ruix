use interrupts::interrupt_handler;

use crate::{
    cpu::{InterruptFrame, CPU},
    io::outb,
    paging::KernelPage,
    process::{CurrentProcess, Process},
    syscalls::{gen_syscalls, syscall},
    task::{CurrentTask, Task},
};
use core::arch::naked_asm;

const NUM_SYSCALLS: usize = 1;
gen_syscalls!(1);

#[no_mangle]
static mut SYSCALL_RETURN: usize = 0;

#[interrupt_handler(0x80)]
pub extern "C" fn entry_syscall() {
    unsafe {
        naked_asm!(
            r#"
                cli
                push 0
                pushad

                push esp
                push eax

                call syscall_handler
                mov dword ptr [{0}], eax

                add esp, 8

                popad
                add esp, 4

                mov eax, [{0}]
                iretd
            "#,
            sym SYSCALL_RETURN,
        )
    };
}

/// # Safety
///  Unsafe because derefences raw pointer from CPU
#[no_mangle]
pub unsafe fn syscall_handler(command: usize, frame: *const InterruptFrame) -> usize {
    if command >= NUM_SYSCALLS {
        outb(0x20, 0x20);
        return 0;
    }

    KernelPage::switch();
    Task::save(CurrentTask::get(), *frame);

    let syscall = SYSCALLS[command];
    let frame = unsafe { &*frame };

    let ret = syscall(frame);

    CurrentTask::paging_switch();

    ret
}

#[syscall(0)]
fn exit(_: *const InterruptFrame) -> usize {
    let task = CurrentTask::get();
    let code = Task::copy_stack_item::<usize>(&task, 0);
    Process::mark_dead(CurrentProcess::get(), code);

    unsafe { CPU::return_to_current() };


    unreachable!()
}
