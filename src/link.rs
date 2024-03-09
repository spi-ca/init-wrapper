#![allow(non_camel_case_types, dead_code)]

use libc::{exit, FILE};

#[link(name = "c")]
extern "C" {
    static stdin: *mut FILE;
    static stdout: *mut FILE;
    static stderr: *mut FILE;
}

#[no_mangle]
extern "C" fn rust_eh_personality() {}
#[no_mangle]
unsafe extern "C" fn _Unwind_Resume() -> ! {
    exit(2)
}

#[panic_handler]
#[inline(never)]
#[cfg(not(test))]
unsafe fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    exit(2)
}
