#![allow(unused_imports)]
#![cfg_attr(test, allow(dead_code))]

use alloc::format;
use core::ffi::CStr;
use core::fmt::Arguments;
use libc::{c_char, c_int, open, FILE, O_APPEND, O_CLOEXEC, O_SYNC, O_WRONLY, STDERR_FILENO};
use libc_print::__LibCWriter;
pub(crate) static mut LOG_FD: c_int = STDERR_FILENO;
pub(crate) static mut LOG_FD_KMSG: bool = false;
pub(crate) const DEV_KMSG_PATH_BYTES: &[u8] = b"/dev/kmsg\0";
pub(crate) const LOG_FLAGS: c_int = O_APPEND | O_WRONLY | O_CLOEXEC | O_SYNC;

#[inline]
pub(crate) fn init_kmsg() {
    const DEV_KMSG_PATH: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(DEV_KMSG_PATH_BYTES).as_ptr() };

    let fd = unsafe { open(DEV_KMSG_PATH, LOG_FLAGS) };
    match fd {
        -1 => (),
        fd => unsafe {
            crate::kmsg::LOG_FD = fd;
            LOG_FD_KMSG = true;
        },
    }
}

#[inline]
pub(crate) fn write_log_line(args: Arguments<'_>) {
    #[allow(unused_must_use)]
    unsafe {
        let mut stm = libc_print::__LibCWriter::new(crate::kmsg::LOG_FD);
        if crate::kmsg::LOG_FD_KMSG {
            let buf = alloc::format!("init-wrapper: {}", args);
            for line in buf.lines() {
                stm.write_str(line);
            }
        } else {
            stm.write_str("init-wrapper: ");
            stm.write_fmt(args);
            stm.write_nl();
        }
    }
}

#[macro_export]
macro_rules! kprintln {
    () => { $crate::kprintln!("") };
    ($fmt: tt) => {
        $crate::kmsg::write_log_line(format_args!($fmt))
    };
    ($fmt: tt, $($arg:tt)*) =>{
        $crate::kmsg::write_log_line(format_args!($fmt, $($arg)*))
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kmsg_path_is_nul_terminated_dev_kmsg() {
        assert_eq!(DEV_KMSG_PATH_BYTES, b"/dev/kmsg\0");
        assert_eq!(*DEV_KMSG_PATH_BYTES.last().unwrap(), 0);
    }

    #[test]
    fn log_flags_open_kmsg_for_synchronous_append_only_writes() {
        assert_eq!(LOG_FLAGS & O_APPEND, O_APPEND);
        assert_eq!(LOG_FLAGS & O_WRONLY, O_WRONLY);
        assert_eq!(LOG_FLAGS & O_CLOEXEC, O_CLOEXEC);
        assert_eq!(LOG_FLAGS & O_SYNC, O_SYNC);
    }

    #[test]
    fn logger_defaults_to_stderr_until_kmsg_opens() {
        unsafe {
            assert_eq!(core::ptr::addr_of!(LOG_FD).read_volatile(), STDERR_FILENO);
            assert!(!core::ptr::addr_of!(LOG_FD_KMSG).read_volatile());
        }
    }
}
