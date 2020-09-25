use crate::asm::{CR0_PE, CR0_WP, CR0_PG, CR4_PSE};
use crate::kmain;

#[no_mangle]
pub unsafe extern "C" fn _start() {
    asm!(
        "mov eax, cr4",
        "or  eax, {CR4_PSE}",
        "mov cr4, eax",
        CR4_PSE = const CR4_PSE,
    );

    kmain();
}