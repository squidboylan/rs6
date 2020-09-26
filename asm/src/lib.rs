#![feature(asm)]
#![no_std]

#[inline(always)]
pub unsafe fn outb(port: u16, byte: u8) {
    asm!("out dx, al", in("dx") port, in("al") byte);
}

#[inline(always)]
pub unsafe fn inb(port: u16) -> u8 {
    let r;
    asm!("in al, dx", out("al") r, in("dx") port);
    r
}

#[inline(always)]
pub unsafe fn outdw(port: u16, dword: u32) {
    asm!("out dx, eax", in("dx") port, in("eax") dword)
}

#[inline(always)]
pub unsafe fn indw(port: u16) -> u32 {
    let r;
    asm!("in eax, dx", out("eax") r, in("dx") port);
    r
}

#[inline(always)]
pub unsafe fn insl(port: u16, buf: &mut [u32]) {
    // Because I can't figure out the REP or INS syntax for Rust
    // inline ASM we'll just fake it.
    buf.iter_mut().for_each(|dw| *dw = indw(port));
}