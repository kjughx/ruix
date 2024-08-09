use crate::{
    cpu::{InterruptFrame, CPU},
    io::outb,
    paging::KernelPage,
    process::{CurrentProcess, Process},
    syscalls::{gen_syscalls, syscall},
    task::{CurrentTask, Task},
    traceln,
};
use core::arch::asm;

const NUM_SYSCALLS: usize = 1;
gen_syscalls!(1);

#[no_mangle]
static mut SYSCALL_RETURN: usize = 0;

#[naked]
#[no_mangle]
pub extern "C" fn entry_syscall() {
    unsafe {
        asm!(
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

                mov eax, [{0}]
                iretd
            "#,
            sym SYSCALL_RETURN,
            options(noreturn)
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
fn exit(code: i32) -> usize {
    Process::mark_dead(CurrentProcess::get(), code);

    unsafe { CPU::return_to_current() };

    unreachable!()
}
