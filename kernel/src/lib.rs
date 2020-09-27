#![no_std]

#[macro_use]
extern crate lazy_static;
extern crate rlibc;

use core::fmt::Write;
use core::panic::PanicInfo;

mod serial;
mod vga;

#[panic_handler]
#[no_mangle]
pub fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn kmain() -> ! {
    // Write "Hello World!" in text to the screen
    {
        let mut writer = serial::SERIAL_WRITER.lock();
        writer.write_str("Hello World!\n").unwrap();
    }

    loop {}
}
