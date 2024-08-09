use crate::__trace;
use core::arch::asm;
#[inline(never)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    if let Some(loc) = info.location() {
        __trace!("[{}:{}] panic - {}", loc.file(), loc.line(), info.message());
    } else {
        __trace!("Kernel panic somwhere!");
    }

    unsafe { asm!("hlt", options(noreturn)) }
}
