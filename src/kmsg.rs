#![allow(unused_imports)]

use alloc::format;
use core::ffi::CStr;
use libc::{c_char, c_int, open, FILE, O_APPEND, O_CLOEXEC, O_SYNC, O_WRONLY, STDERR_FILENO};
use libc_print::__LibCWriter;
pub(crate) static mut LOG_FD: c_int = STDERR_FILENO;
pub(crate) static mut LOG_FD_KMSG: bool = false;

#[inline]
pub(crate) fn init_kmsg() {
    const DEV_KMSG_PATH: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"/dev/kmsg\0").as_ptr() };
    const LOG_FLAGS: c_int = O_APPEND | O_WRONLY | O_CLOEXEC | O_SYNC;

    let fd = unsafe { open(DEV_KMSG_PATH, LOG_FLAGS) };
    match fd {
        -1 => (),
        fd => unsafe {
            crate::kmsg::LOG_FD = fd;
            LOG_FD_KMSG = true;
        },
    }
}

#[macro_export]
macro_rules! kprintln {
    () => { $crate::kprintln!("") };
    ($fmt: expr) => {
        {
            #[allow(unused_must_use)]
            unsafe {
                let mut stm = libc_print::__LibCWriter::new($crate::kmsg::LOG_FD);
                let buf=alloc::format!("init-wrapper: {}",$fmt);
                for line in buf.lines(){
                    stm.write_str(&line);
                }
                if (!$crate::kmsg::LOG_FD_KMSG) {
                    stm.write_nl();
                }
            }
        }
   };
    ($fmt: tt) => {
        {
            #[allow(unused_must_use)]
            unsafe {
                let mut stm = libc_print::__LibCWriter::new($crate::kmsg::LOG_FD);
                stm.write_str(concat!("init-wrapper: ",$fmt));
                if (!$crate::kmsg::LOG_FD_KMSG) {
                    stm.write_nl();
                }
            }
        }
   };
    ($fmt: tt, $($arg:tt)*) =>{
        {

            #[allow(unused_must_use)]
            unsafe {
                let mut stm = libc_print::__LibCWriter::new($crate::kmsg::LOG_FD);
                if ($crate::kmsg::LOG_FD_KMSG) {
                    let buf=alloc::format!(concat!("init-wrapper: ",$fmt),$($arg)*);
                    for line in buf.lines(){
                        stm.write_str(&line);
                    }
                }else{
                    stm.write_fmt(format_args!(concat!("init-wrapper: ",$fmt),$($arg)*));
                    stm.write_nl();
                }
            }
        }
    };
}
