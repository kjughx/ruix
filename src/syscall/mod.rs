use crate::{
    cpu::InterruptFrame,
    io::outb,
    paging::KernelPage,
    syscall_macro::{syscall, syscalls},
    task::{CurrentTask, Task},
    traceln,
};
use core::arch::asm;

const NUM_SYSCALLS: usize = 2;
syscalls!(2);

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
fn print(_frame: &InterruptFrame) -> usize {
    let int = Task::copy_stack_item::<u32>(CurrentTask::get(), 0);

    traceln!("Got from userland: 0x{:x}", int);

    1
}

#[syscall(1)]
fn exit(frame: &InterruptFrame) -> usize {
    traceln!("{}", frame);

    2
}
