use crate::{
    cpu::InterruptFrame,
    io::outb,
    paging::{KernelPage, Paging},
    syscall_macro::{syscall, syscalls},
    task::CurrentTask,
    traceln,
};
use core::arch::asm;

const NUM_SYSCALLS: usize = 1;
syscalls!(1);

#[no_mangle]
static mut SYSCALL_RETURN: usize = 0;

#[naked]
#[no_mangle]
pub extern "C" fn entry_syscall() {
    unsafe {
        asm!(
            r#"
                cli
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

    let syscall = SYSCALLS[command];
    let frame = unsafe { &*frame };

    let ret = syscall(frame);

    CurrentTask::get().with_rlock(|task| {
        Paging::switch(&task.page_directory);
    });

    ret
}

#[syscall(0)]
fn print(frame: &InterruptFrame) -> usize {
    traceln!("{}", frame);

    1
}
