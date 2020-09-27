/// This represents A page directory used for Virtual Memory, each entry points to a PageTable
/// which has entries that point to physical locations.
/// This must be aligned to 4KB boundaries because their pointer is represented by 20 high bits and
/// followed by 12 0s
#[repr(align(4096))]
struct PageDir {
    entries: [PageTableEntry; 1024],
}

/// This represents a page table used by the virtual memory hardware and page directory to map a
/// virtual address to a physical address
/// This must be aligned to 4KB boundaries because their pointer is represented by 20 high bits and
/// followed by 12 0s
#[repr(align(4096))]
struct PageTable {
    entries: [PageTableEntry; 1024],
}

impl PageDir {
    pub fn new() -> Self {
        PageDir { entries: [0; 1024] }
    }
}

bitfield! {
    struct PageTableEntry(u32);
    impl Debug;
    get_present, set_present: 0;
    get_writable, set_writable: 1;
    get_user, set_user: 2;
    get_write_through, set_write_through: 3;
    get_cache_disabled, set_cache_disabled: 4;
    get_accessed, set_accessed: 5;
    get_dirty, set_dirty: 6;
    get_available, set_available: 11, 9;
    get_ppn, set_ppn: 31, 12;

}
