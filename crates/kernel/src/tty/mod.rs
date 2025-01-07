use core::fmt::{self, Write};

#[macro_use]
pub mod terminal;
pub use terminal::Terminal;

pub struct TypeWriter {
    base: *mut u16,
    width: usize,
    height: usize,
    ix: isize,
    iy: isize,
}

static COLOR_WHITE: u8 = 15;
impl TypeWriter {
    pub fn new(base: u32, width: usize, height: usize) -> Self {
        Self {
            base: base as *mut u16,
            width,
            height,
            ix: 0,
            iy: 0,
        }
    }

    pub fn init(&mut self) {
        for y in 0..=self.height as isize {
            for x in 0..=self.width as isize {
                unsafe {
                    *self.base.offset(y * (self.width as isize) + x) = 0;
                }
            }
        }
    }

    fn make_char(c: char, color: u8) -> u16 {
        ((color as u16) << 8) | (c as u16)
    }

    fn backspace(&mut self) {
        if self.ix == 0 && self.iy == 0 {
            return;
        }

        if self.ix == 0 {
            self.iy -= 1;
            self.ix = self.width as isize;
        }

        self.ix -= 1;
        self.put_char(self.ix, self.iy, ' ', COLOR_WHITE);
        self.ix -= 1;
    }

    fn put_char(&mut self, ix: isize, iy: isize, c: char, color: u8) {
        unsafe {
            *self.base.offset(iy * (self.width as isize) + ix) = Self::make_char(c, color);
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

        if self.ix >= self.width as isize {
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

impl Write for TypeWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write(s);
        Ok(())
    }
}
