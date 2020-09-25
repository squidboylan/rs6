#![no_std]

#[macro_use]
extern crate lazy_static;

use core::panic::PanicInfo;
use core::fmt::Write;

mod entry;
mod vga;
mod serial;

#[panic_handler]
#[no_mangle]
pub fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

fn kmain() -> ! {
    // Write "Hello World!" in text to the screen
    {
        let mut writer = serial::SERIAL_WRITER.lock();
        writer.write_str("Hello World!\n").unwrap();
    }

    loop {}
}
