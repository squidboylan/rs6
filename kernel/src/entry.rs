use crate::kmain;

#[no_mangle]
pub unsafe extern "C" fn _start() {
    kmain();
}