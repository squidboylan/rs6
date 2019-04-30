#![no_std]

#[macro_use]
extern crate lazy_static;

use core::panic::PanicInfo;
use core::fmt::Write;

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
        let mut vga_writer = vga::VGA_WRITER.lock();
        vga_writer.clear_screen();
        vga_writer.write_str("Hello World!\n").unwrap();
    }

    loop {}
}
