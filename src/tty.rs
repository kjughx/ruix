use core::cell::LazyCell;
use crate::types::Mutex;

const VGA_WIDTH: isize = 80;
const VGA_HEIGHT: isize = 25;

static mut TERMINAL: LazyCell<Mutex<TypeWriter>> = LazyCell::new(|| {
    Mutex::new(TypeWriter {
        base: 0xB8000,
        width: VGA_WIDTH,
        height: VGA_HEIGHT,
        ix: 0,
        iy: 0,
    })
});

struct TypeWriter {
    base: u32,
    width: isize,
    height: isize,
    ix: isize,
    iy: isize,
}

static COLOR_WHITE: u8 = 15;
impl TypeWriter {
    fn init(&mut self) {
        for y in 0..=self.height {
            for x in 0..=self.width {
                unsafe {
                    *(self.base as *mut u16).offset(y * self.width + x) = 0;
                }
            }
        }
    }

    fn make_char(c: char, color: u8) -> u16 {
        return (color as u16) << 8 | (c as u16);
    }

    fn backspace(&mut self) {
        if self.ix == 0 && self.iy == 0 {
            return;
        }

        if self.ix == 0 {
            self.iy -= 1;
            self.ix = self.width;
        }

        self.ix -= 1;
        self.put_char(self.ix, self.iy, ' ', COLOR_WHITE);
        self.ix -= 1;
    }

    fn put_char(&mut self, ix: isize, iy: isize, c: char, color: u8) {
        unsafe {
            *(self.base as *mut u16).offset(iy * self.width + ix) = Self::make_char(c, color);
        }
    }

    fn write_char(&mut self, c: char, color: u8) {
        if c as u8 == 0x08 {
            self.backspace();
            return;
        }

        if c == '\n' {
            self.ix = 0;
            self.iy += 1;
            return;
        }

        self.put_char(self.ix, self.iy, c, color);
        self.ix += 1;

        if self.ix >= self.width {
            self.ix = 0;
            self.iy += 1;
        }
    }

    fn write(&mut self, msg: &str) {
        for c in msg.chars() {
            self.write_char(c, COLOR_WHITE);
        }
    }
}

pub fn init_screen() {
    unsafe { TERMINAL.lock().init() };
}

pub fn print(msg: &str) {
    unsafe { TERMINAL.lock().write(msg) };
}
