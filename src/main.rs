#![no_std]
#![no_main]

use ruix::{
    tty::{init_screen, print},
    traceln,
};

#[no_mangle] // don't mangle the name of this function
pub extern "C" fn kernel_main() -> ! {
    init_screen();
    print("Hello, World!");
    traceln!("Hello, World!");
    loop {}
}
