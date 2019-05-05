use core::fmt::{ Write, Result };
use x86::io::{ outb, inb };
use spin::Mutex;

// COM1 ports
const SERIAL_DATA_PORT: u16 = 0x3F8;
const SERIAL_FIFO_COMMAND_PORT: u16 = 0x3FA;
const SERIAL_LINE_COMMAND_PORT: u16 = 0x3FB;
const SERIAL_MODEM_COMMAND_PORT: u16 = 0x3FC;
const SERIAL_LINE_STATUS_PORT: u16 = 0x3FD;

const SERIAL_LINE_ENABLE_DLAB: u8 = 0x80;

lazy_static! {
    pub static ref SERIAL_WRITER: Mutex<Serial> = Mutex::new(Serial::new());
}

pub struct Serial{}

impl Serial {
    fn new() -> Self {
        Serial::serial_configure_baud_rate(2);
        Serial::serial_configure_line();
        Serial{}
    }

    fn serial_configure_baud_rate(divisor: u16) {
        unsafe {
            outb(SERIAL_LINE_COMMAND_PORT, SERIAL_LINE_ENABLE_DLAB);
            outb(SERIAL_DATA_PORT, (divisor >> 8) as u8);
            outb(SERIAL_DATA_PORT, divisor as u8);
        }
    }

    fn serial_configure_line() {
        unsafe {
            outb(SERIAL_LINE_COMMAND_PORT, 0x03);
        }
    }

    fn ready(&mut self) -> bool {
        unsafe {
            inb(SERIAL_LINE_STATUS_PORT) & 0x20 != 0
        }
    }

    fn write(&mut self, c: u8) {
        unsafe {
            outb(SERIAL_DATA_PORT, c);
        }
    }
}

impl Write for Serial {
    fn write_str(&mut self, s: &str) -> Result {
        let mut count = 0;
        for c in s.as_bytes() {
            if count == 14 {
                while !self.ready() {}
                count = 0;
            }
            self.write(*c);
        }
        Ok(())
    }
}
