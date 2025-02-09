use global::global;

use core::fmt::{self, Write};

use super::{TypeWriter, COLOR_WHITE};

const VGA_WIDTH: usize = 80;
const VGA_HEIGHT: usize = 25;

global! {
    Terminal,
    TypeWriter,
    TypeWriter::new(0xB8000, VGA_WIDTH, VGA_HEIGHT),
    "TERMINAL"
}

impl Terminal {
    pub fn init() {
        let terminal = Self::get_mut();
        terminal.with_wlock(|terminal| terminal.init());
    }
    pub fn print(args: fmt::Arguments) {
        let terminal = Self::get_mut();
        terminal.with_wlock(|terminal| terminal.write_fmt(args).unwrap());
    }
    pub fn write(chars: &[u8]) {
        let terminal = Self::get_mut();
        terminal.with_wlock(|terminal| for c in chars {
            terminal.write_char((*c) as char, COLOR_WHITE);
        })
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::tty::terminal::Terminal::print(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($fmt:expr) => ($crate::print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::print!(
        concat!($fmt, "\n"), $($arg)*));
}
