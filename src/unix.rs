use alloc::ffi::CString;
use core::ptr::null;
use errno::{errno, Errno};
use libc::{c_void, chdir, execv, mkdir, mode_t, mount, rmdir, syscall, umount2, SYS_pivot_root};

#[cfg(test)]
use crate::tm::{new_timespec, Timespec};
#[cfg(test)]
use core::borrow::BorrowMut;
#[cfg(test)]
use libc::{
    c_uint, clock_gettime, makedev, mkdir, mknod, mode_t, mount, readlinkat, rmdir, strlen,
    syscall, umount2, unlink, SYS_pivot_root, AT_FDCWD, CLOCK_BOOTTIME, CLOCK_BOOTTIME, S_IFCHR,
};

pub(crate) type SystemResult = Result<(), Errno>;

#[cfg(test)]
pub(crate) fn do_mknod(path: &str, major: c_uint, minor: c_uint) -> SystemResult {
    // has ownership
    let raw_path = CString::new(path).unwrap();

    unsafe {
        let dev = makedev(major, minor);

        if mknod(raw_path.as_ref().as_ptr(), S_IFCHR, dev) == -1 {
            Err(errno())
        } else {
            Ok(())
        }
    }
}

pub(crate) fn do_mount(
    source: Option<&str>,
    target: Option<&str>,
    fs: Option<&str>,
    flags: u64,
    opt: Option<&str>,
) -> SystemResult {
    // has ownership
    let raw_src = source.map(|v| CString::new(v).ok()).flatten();
    let raw_tgt = target.map(|v| CString::new(v).ok()).flatten();
    let raw_fs = fs.map(|v| CString::new(v).ok()).flatten();
    let raw_fs_opt = opt.map(|v| CString::new(v).ok()).flatten();

    unsafe {
        if mount(
            raw_src.as_ref().map(|v| v.as_ptr()).unwrap_or_else(null),
            raw_tgt.as_ref().map(|v| v.as_ptr()).unwrap_or_else(null),
            raw_fs.as_ref().map(|v| v.as_ptr()).unwrap_or_else(null),
            flags,
            raw_fs_opt.as_ref().map(|v| v.as_ptr()).unwrap_or_else(null) as *const c_void,
        ) == -1
        {
            Err(errno())
        } else {
            Ok(())
        }
    }
}

pub(crate) fn do_umount(path: &str, flags: i32) -> SystemResult {
    // has ownership
    let raw_path = CString::new(path).unwrap();
    unsafe {
        if umount2(raw_path.as_ref().as_ptr(), flags) == -1 {
            Err(errno())
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
pub(crate) fn do_readlink(path: &str) -> Result<&str, Errno> {
    // has ownership
    let raw_path = CString::new(path).unwrap();
    unsafe {
        const BUF_SZ: usize = 1024;
        let mut buf = [0u8; BUF_SZ];
        if readlinkat(
            AT_FDCWD,
            raw_path.as_ref().as_ptr(),
            buf.as_mut_ptr() as *mut _,
            buf.len() as size_t,
        ) == -1
        {
            return Err(errno());
        }
        let c_str_len = strlen(buf.as_ptr() as *const _);
        Ok(from_utf8_lossy(&buf[..c_str_len]))
    }
}

pub(crate) fn do_mkdir(path: &str, mode: mode_t) -> SystemResult {
    // has ownership
    let raw_path = CString::new(path).unwrap();
    unsafe {
        if mkdir(raw_path.as_ref().as_ptr(), mode) == -1 {
            Err(errno())
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
pub(crate) fn do_unlink(path: &str) -> SystemResult {
    // has ownership
    let raw_path = CString::new(path).unwrap();
    unsafe {
        if unlink(raw_path.as_ref().as_ptr()) == -1 {
            Err(errno())
        } else {
            Ok(())
        }
    }
}

pub(crate) fn do_rmdir(path: &str) -> SystemResult {
    // has ownership
    let raw_path = CString::new(path).unwrap();
    unsafe {
        if rmdir(raw_path.as_ref().as_ptr()) == -1 {
            Err(errno())
        } else {
            Ok(())
        }
    }
}

pub(crate) fn do_execv(path: &str, argv: *mut *const u8) -> SystemResult {
    let raw_init_path = CString::new(path).unwrap();
    unsafe {
        *argv = raw_init_path.as_ref().as_ptr();
        if execv(raw_init_path.as_ref().as_ptr(), argv) == -1 {
            Err(errno())
        } else {
            Ok(())
        }
    }
}

pub(crate) fn do_chdir(path: &str) -> SystemResult {
    // has ownership
    let raw_path = CString::new(path).unwrap();
    unsafe {
        if chdir(raw_path.as_ref().as_ptr()) == -1 {
            Err(errno())
        } else {
            Ok(())
        }
    }
}

pub(crate) fn do_pivot_root(new_root: &str, put_old: &str) -> SystemResult {
    let raw_new_root = CString::new(new_root).unwrap();
    let raw_put_old = CString::new(put_old).unwrap();
    unsafe {
        if syscall(
            SYS_pivot_root,
            raw_new_root.as_ref().as_ptr(),
            raw_put_old.as_ref().as_ptr(),
        ) == -1
        {
            Err(errno())
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
pub(crate) fn do_gettime() -> Result<Timespec, Errno> {
    let mut time = new_timespec();

    unsafe {
        if clock_gettime(CLOCK_BOOTTIME, time.ts.borrow_mut()) == -1 {
            Err(errno())
        } else {
            Ok(time.into())
        }
    }
}

#[cfg(test)]
#[inline]
fn from_utf8_lossy(input: &[u8]) -> &str {
    match str::from_utf8(input) {
        Ok(valid) => valid,
        Err(error) => unsafe { str::from_utf8_unchecked(&input[..error.valid_up_to()]) },
    }
}
