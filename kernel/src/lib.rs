#![feature(asm)]
#![no_std]

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate bitfield;

use core::fmt::Write;
use core::panic::PanicInfo;

mod asm;
mod entry;
mod paging;
mod serial;
mod vga;

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
