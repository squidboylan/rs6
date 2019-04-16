#![no_std]
#![no_main]
#![feature(asm)]
#![feature(link_args)]

use core::panic::PanicInfo;
use core::*;
use x86::io::{ inb, outb, insl };

const ELF_MAGIC: u32 = 0x464C457F;

// See https://en.wikipedia.org/wiki/Executable_and_Linkable_Format
// for detailed breakdown
#[repr(C)]
#[derive(Default, Clone)]
struct elfHeader {
    magic: u32,       // must contain 0x464C457FU
    elf: [u8; 12],    // various meta things
    file_type: u16,   // file object type
    machine: u16,     // arch, 0x03 is x86
    version: u32,     // elf version
    entry: u32,       // entry point for the elf file
    phoff: u32,       // offset where the program header starts
    shoff: u32,       // section header offset
    flags: u32,       // see wikipedia
    ehsize: u16,      // header length
    phentsize: u16,   // program header entry size
    phnum: u16,       // number of program header entries
    shentsize: u16,   // section header entry size
    shnum: u16,       // number of section header entries
    shstrndx: u16,    // index into the section header table that contains the
                      // section names
}

// See https://en.wikipedia.org/wiki/Executable_and_Linkable_Format
// for detailed breakdown
#[repr(C)]
#[derive(Default, Clone)]
struct progHeader {
  prog_type: u32,     // see wikipedia
  off: u32,           // offset of the segment in the file image
  vaddr: u32,         // virtual address of the segment in memory
  paddr: u32,         // segments physical address
  filesz: u32,        // size of the segment in file in bytes
  memsz: u32,         // size of the segment in memory in bytes
  flags: u32,         // see wikipedia
  align: u32,         // offset to allign the program on
}

const SECTSIZE: u32 = 512;

#[panic_handler]
#[no_mangle]
pub fn panic(info: &PanicInfo) -> ! {
    loop {}
}


// Takes a u32 which represents the location of the elfHeader in memory
// It then builds a elfHeader on the stack
// This is unsafe because the caller must ensure "elf_loc" contains an address
// with a elf_header in it.
unsafe fn get_elf_header(elf_loc: u32) -> elfHeader {
    // This should be fairly safe....
    let elf_ref = mem::transmute::<u32, &mut elfHeader>(elf_loc);
    elf_ref.clone()
}


// Takes a u32 which represents the location of the progHeader in memory
// It then builds a elfHeader on the stack
// This is unsafe because the caller must ensure "prog_loc" contains an address
// with a elf_header in it.
unsafe fn get_prog_header(prog_loc: u32) -> progHeader {
    // This should be fairly safe....
    let prog_ref = mem::transmute::<u32, &mut progHeader>(prog_loc);
    prog_ref.clone()
}

#[no_mangle]
pub extern "C" fn bootmain() {
    const elf_loc: u32 = 0x10000;
    readseg(elf_loc, 4096, 0);
    // Copy the elf from the heap to the stack for safe access
    let elf_header = unsafe{ get_elf_header(elf_loc) };
    /*
    if elf_header.magic != ELF_MAGIC {
        // The image is broken somehow, maybe we should print something to the
        // screen and hlt instead?
        return;
    }
    */

    let mut prog_entry: u32 = 0;
    while prog_entry < elf_header.phnum as u32 {
        let prog_header_addr = elf_loc + elf_header.phoff + prog_entry * mem::size_of::<progHeader>() as u32;
        let prog_header = unsafe { get_prog_header(prog_header_addr) };
        let physical_address = prog_header.paddr;
        readseg(physical_address, prog_header.filesz, prog_header.off);
        if prog_header.memsz > prog_header.filesz {
            physical_address + prog_header.filesz;
            let zeroes = unsafe {
                core::slice::from_raw_parts_mut(
                    (physical_address + prog_header.filesz) as *mut u8,
                    (prog_header.memsz - prog_header.filesz) as usize
                )
            };
            for i in zeroes {
                *i = 0;
            }
        }

    }

    // Cast the entry point to a C function pointer
    let entry_ptr = unsafe {
        let tmp = elf_header.entry as *const ();
        mem::transmute::<*const (), extern "C" fn()>(tmp)
    };
    entry_ptr();
}


fn waitdisk() {
    // Wait for disk ready.

    let mut not_ready = (get_disk_status() & 0xC0) != 0x40;

    while not_ready {
        not_ready = (get_disk_status() & 0xC0) != 0x40;
    }
}

fn get_disk_status() -> u8 {
    unsafe { inb(0x1F7) }
}

fn readsect(dst: &mut [u32], offset: u32) {
    // Issue command.
    waitdisk();
    unsafe {
        outb(0x1F2, 1);   // count = 1
        outb(0x1F3, offset as u8);
        outb(0x1F4, (offset >> 8) as u8);
        outb(0x1F5, (offset >> 16) as u8);
        outb(0x1F6, (offset >> 24) as u8 | 0xE0);
        outb(0x1F7, 0x20);  // cmd 0x20 - read sectors
    }

    // Read data.
    waitdisk();
    unsafe {
        insl(0x1F0, dst);
    }
}

fn readseg(mut pa: u32, count: u32, mut offset: u32) {
    let epa: u32 = pa + count;

    // Round down to sector boundary.
    pa -= offset % SECTSIZE;

    // Translate from bytes to sectors; kernel starts at sector 1.
    offset = (offset / SECTSIZE) + 1;

    // If this is too slow, we could read lots of sectors at a time.
    // We'd write more to memory than asked, but it doesn't matter --
    // we load in increasing order.
    while pa < epa {
        // We need a slice to manipulate since pa is just a u32
        let mut slice = unsafe {
            core::slice::from_raw_parts_mut(pa as *mut u32, (SECTSIZE/4) as usize)
        };
        readsect(slice, offset);
        pa += SECTSIZE;
        offset += 1;
    }
}
