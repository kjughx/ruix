use core::arch::asm;

pub extern "C" fn insb(port: u16) -> u8 {
    let val: u8;
    unsafe {
        asm!(
            "in al, dx", in("dx") port, out("al") val
        )
    }

    val
}

pub extern "C" fn insw(port: u16) -> u16 {
    let val: u16;
    unsafe {
        asm!(
            "in ax, dx", in("dx") port, out("ax") val
        )
    }
    val
}

pub extern "C" fn outb(port: u16, val: u8) {
    unsafe {
        asm!(
            "out dx, al", in("dx") port, in("al") val
        )
    }
}

pub extern "C" fn outw(port: u16, val: u16) {
    unsafe {
        asm!(
            "out dx, ax", in("dx") port, in("ax") val
        )
    }
}
