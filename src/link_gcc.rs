#![allow(non_camel_case_types, dead_code)]

use libc::{c_int, exit, FILE};

#[repr(C, align(16))]
struct f128 {
    a: [u8; 16],
}

#[no_mangle]
extern "C" fn __letf2(_a: f128, _b: f128) -> c_int {
    0
}

#[no_mangle]
extern "C" fn __unordtf2(_a: f128, _b: f128) -> c_int {
    0
}

#[no_mangle]
unsafe extern "C" fn _Unwind_Resume() -> ! {
    exit(2)
}

#[no_mangle]
extern "C" fn __gcc_personality_v0() {}
