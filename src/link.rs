#![allow(non_camel_case_types, dead_code)]

use libc::FILE;

#[link(name = "c")]
extern "C" {
    static stdin: *mut FILE;
    static stdout: *mut FILE;
    static stderr: *mut FILE;
}

#[panic_handler]
#[inline(never)]
#[cfg(not(test))]
unsafe fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    libc::exit(2)
}
