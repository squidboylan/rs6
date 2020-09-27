#![no_std]
#![feature(asm)]
#![feature(link_args)]

extern crate rlibc;

use asm::{inb, insl, outb};
use core::panic::PanicInfo;
use core::*;

const ELF_MAGIC: u32 = 0x464C457F;

// See https://en.wikipedia.org/wiki/Executable_and_Linkable_Format
// for detailed breakdown
#[repr(C)]
#[derive(Default, Clone)]
struct ElfHeader {
    magic: u32,     // must contain 0x464C457FU
    elf: [u8; 12],  // various meta things
    file_type: u16, // file object type
    machine: u16,   // arch, 0x03 is x86
    version: u32,   // elf version
    entry: u32,     // entry point for the elf file
    phoff: u32,     // offset where the program header starts
    shoff: u32,     // section header offset
    flags: u32,     // see wikipedia
    ehsize: u16,    // header length
    phentsize: u16, // program header entry size
    phnum: u16,     // number of program header entries
    shentsize: u16, // section header entry size
    shnum: u16,     // number of section header entries
    shstrndx: u16,  // index into the section header table that contains the
                    // section names
}

// See https://en.wikipedia.org/wiki/Executable_and_Linkable_Format
// for detailed breakdown
#[repr(C)]
#[derive(Default, Clone)]
struct ProgHeader {
    prog_type: u32, // see wikipedia
    off: u32,       // offset of the segment in the file image
    vaddr: u32,     // virtual address of the segment in memory
    paddr: u32,     // segments physical address
    filesz: u32,    // size of the segment in file in bytes
    memsz: u32,     // size of the segment in memory in bytes
    flags: u32,     // see wikipedia
    align: u32,     // offset to allign the program on
}

const SECTSIZE: u32 = 512;

#[panic_handler]
#[no_mangle]
pub fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// Takes a u32 which represents the location of the elfHeader in memory
// It then builds a elfHeader on the stack
// This is unsafe because the caller must ensure "elf_loc" contains an address
// with a elf_header in it.
unsafe fn get_elf_header(elf_loc: u32) -> ElfHeader {
    // This should be fairly safe....
    let elf_ref = mem::transmute::<u32, &mut ElfHeader>(elf_loc);
    elf_ref.clone()
}

// Takes a u32 which represents the location of the progHeader in memory
// It then builds a elfHeader on the stack
// This is unsafe because the caller must ensure "prog_loc" contains an address
// with a elf_header in it.
unsafe fn get_prog_header(prog_loc: u32) -> ProgHeader {
    // This should be fairly safe....
    let prog_ref = mem::transmute::<u32, &mut ProgHeader>(prog_loc);
    prog_ref.clone()
}

#[no_mangle]
pub extern "C" fn bootmain() {
    const ELF_LOC: u32 = 0x10000;
    readseg(ELF_LOC, 4096, 0);
    // Copy the elf from the heap to the stack for safe access
    let elf_header = unsafe { get_elf_header(ELF_LOC) };
    if elf_header.magic != ELF_MAGIC {
        loop {}
    }

    let mut prog_entry: u32 = 0;
    while prog_entry < elf_header.phnum as u32 {
        // For each program header entry, load the data it points to into memory at the appropriate
        // physical address, we have no virtual memory yet
        let prog_header_addr =
            ELF_LOC + elf_header.phoff + prog_entry * mem::size_of::<ProgHeader>() as u32;
        let prog_header = unsafe { get_prog_header(prog_header_addr) };
        let physical_address = prog_header.paddr;
        readseg(physical_address, prog_header.filesz, prog_header.off);

        // If the entry takes up extra space in memory, fill that extra space with 0s
        if prog_header.memsz > prog_header.filesz {
            let zeroes = unsafe {
                core::slice::from_raw_parts_mut(
                    (physical_address + prog_header.filesz) as *mut u8,
                    (prog_header.memsz - prog_header.filesz) as usize,
                )
            };
            for i in zeroes {
                *i = 0;
            }
        }
        prog_entry += 1;
    }

    // Cast the entry point to a C function pointer and call into the kernel!
    let entry_ptr = unsafe {
        let tmp = elf_header.entry as *const ();
        mem::transmute::<*const (), extern "C" fn()>(tmp)
    };
    entry_ptr();
}

/// Wait for disk ready.
fn waitdisk() {
    loop {
        if (get_disk_status() & 0xC0) == 0x40 {
            return;
        }
    }
}

fn get_disk_status() -> u8 {
    unsafe { inb(0x1F7) }
}

/// Read a single sector from the disk
fn readsect(dst: &mut [u32], offset: u32) {
    waitdisk();
    unsafe {
        outb(0x1F2, 1); // count = 1
        outb(0x1F3, offset as u8);
        outb(0x1F4, (offset >> 8) as u8);
        outb(0x1F5, (offset >> 16) as u8);
        outb(0x1F6, (offset >> 24) as u8 | 0xE0);
        outb(0x1F7, 0x20); // cmd 0x20 - read sectors
    }

    // Read data.
    waitdisk();
    unsafe {
        insl(0x1F0, dst);
    }
}

/// Read count bytes from the disk starting at offset into pa.
/// Note that we have to read in sectors, so we have to start at the beginning
/// of the sector where the data begins and read the whole sector that has the
/// end of the data. Therefore we may read more than count bytes, and we may
/// start writing into a value less than pa
fn readseg(mut pa: u32, count: u32, mut offset: u32) {
    let epa: u32 = pa + count;

    // Round down to sector boundary.
    pa -= offset % SECTSIZE;

    // Translate from bytes to sectors; kernel starts at sector 1.
    offset = (offset / SECTSIZE) + 1;

    while pa < epa {
        // We need a slice to manipulate since pa is just a u32
        let slice =
            unsafe { core::slice::from_raw_parts_mut(pa as *mut u32, (SECTSIZE / 4) as usize) };
        readsect(slice, offset);
        pa += SECTSIZE;
        offset += 1;
    }
}
