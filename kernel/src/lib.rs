#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
#[no_mangle]
pub fn panic(info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    let vga = 0xb8000 as *mut u8;
    let mut vga = unsafe { core::slice::from_raw_parts_mut(vga, 50) };
    let mut count = 0;
    let chars = b"Hello World!";
    while count < chars.len()*2 {
        vga[count] = chars[count/2];
        count += 1;
        vga[count] = 1;
        count += 1;
    }
    loop {}
}
