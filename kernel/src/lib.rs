#![no_std]

use core::panic::PanicInfo;

#[panic_handler]
#[no_mangle]
pub fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn kmain() -> ! {
    // Write "Hello World!" in blue text over vga!
    // This should be replaced by a vga module, however for now it helps
    // us know our thing boots
    let vga_ptr = 0xb8000 as *mut u8;
    let vga_size = 80 * 25 * 2;
    let vga = unsafe { core::slice::from_raw_parts_mut(vga_ptr, vga_size) };
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
