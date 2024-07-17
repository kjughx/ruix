#![no_std]
#![no_main]

use core::hint;

use ruix::{
    traceln,
    tty::{init_screen, print},
};

#[no_mangle] // don't mangle the name of this function
pub extern "C" fn kernel_main() -> ! {
    init_screen();
    print("Hello, World!");
    traceln!("Hello, World!");
    loop {
        hint::spin_loop()
    }
}
