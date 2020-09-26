use core::fmt::{ Write, Result };

use asm::{ outb, inb };
use spin::Mutex;
use volatile::Volatile;

lazy_static! {
    pub static ref VGA_WRITER: Mutex<VGA> = Mutex::new(VGA::new());
}

#[repr(C)]
#[derive(Copy, Clone)]
struct VGACell {
    character: u8,
    color: u8,
}

impl VGACell {
    pub fn new(character: u8, fg_color: Color, bg_color: Color) -> Self {
        VGACell {
            character,
            color: ((bg_color as u8) << 4) | fg_color as u8,
        }
    }
}

const BUFF_HEIGHT: usize = 25;
const BUFF_WIDTH: usize = 80;

struct Buffer {
    chars: [[Volatile<VGACell>; BUFF_WIDTH]; BUFF_HEIGHT],
}

#[allow(dead_code)]
#[repr(u8)]
#[derive(Copy, Clone)]
enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGrey = 7,
    DarkGrey = 8,
    LightBlue = 9,
    Lightgreen = 10,
    LightCyan = 11,
    LightRed = 12,
    LightMagenta = 13,
    LightBrown = 14,
    White = 15,
}

pub struct VGA {
    cursor: usize,
    buff: &'static mut Buffer,
    fg_color: Color,
    bg_color: Color,
}

impl VGA {
    pub fn new() -> Self {
        VGA {
            cursor: 0,
            buff: unsafe { &mut *(0x000B8000 as *mut Buffer) },
            fg_color: Color::LightGrey,
            bg_color: Color::Black,
        }
    }

    /// Fill the line with spaces until we get to the next line
    fn new_line(&mut self) {
        let blank_spaces = BUFF_WIDTH - (self.cursor % BUFF_WIDTH);
        for _ in 0..blank_spaces {
            self.write_char(' ').unwrap();
        }
    }

    /// Fills the screen with ' ' and sets cursor back to 0,0
    pub fn clear_screen(&mut self) {
        for l in &mut self.buff.chars {
            for c in l.iter_mut() {
                c.write(VGACell::new(' ' as u8, self.fg_color, self.bg_color));
            }
        }
        self.cursor = 0;
        self.update_cursor();
    }

    /// Moves all rows up one, doesnt change cursor
    pub fn scroll(&mut self) {
        for y in 0..BUFF_HEIGHT-1 {
            for x in 0..BUFF_WIDTH {
                self.buff.chars[y][x].write(self.buff.chars[y+1][x].read());
            }
        }
        for x in 0..BUFF_WIDTH {
            self.buff.chars[BUFF_HEIGHT-1][x].write(VGACell::new(' ' as u8, self.fg_color, self.bg_color));
        }
    }

    /// Update the cursor location on screen to match our cursor state
    pub fn update_cursor(&self) {
        unsafe {
            // The inbs are to wait for IO to happen, may not be necessary but doesnt really hurt
            // to have
            outb(0x3D4, 14);
            inb(0x64);
            outb(0x3D5, (self.cursor >> 8) as u8);
            inb(0x64);
            outb(0x3D4, 15);
            inb(0x64);
            outb(0x3D5, self.cursor as u8);
            inb(0x64);
        }
    }
}

impl Write for VGA {
    fn write_str(&mut self, s: &str) -> Result {
        for c in s.as_bytes() {
            if *c == '\n' as u8 {
                self.new_line();
            } else {
                let x = self.cursor % BUFF_WIDTH;
                let y = self.cursor / BUFF_WIDTH;
                self.buff.chars[y][x].write(VGACell::new(*c, self.fg_color, self.bg_color));
                self.cursor += 1;
                if self.cursor == BUFF_HEIGHT * BUFF_WIDTH {
                    self.scroll();
                    self.cursor -= BUFF_WIDTH;
                }
            }
        }
        self.update_cursor();
        Ok(())
    }
}
