/// Protected Mode
pub const CR0_PE: u32 = 0x00000001;

/// Write Protect
pub const CR0_WP: u32 = 1 << 5;

/// Paging
pub const CR0_PG: u32 = 1 << 31;

/// Page Size Extension (Use 4MiB-sized pages)
pub const CR4_PSE: u32 = 1 << 4;