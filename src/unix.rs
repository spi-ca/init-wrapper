#![cfg_attr(test, allow(dead_code))]

use alloc::ffi::CString;
#[cfg(test)]
use alloc::string::String;
use core::ptr::null;
use errno::{errno, Errno};
use libc::{
    c_void, chdir, execv, mkdir, mode_t, mount, rmdir, syscall, umount2, SYS_pivot_root, EINVAL,
};

#[cfg(test)]
use crate::tm::{new_timespec, Timespec};
#[cfg(test)]
use core::borrow::BorrowMut;
#[cfg(test)]
use libc::{
    c_uint, clock_gettime, makedev, mknod, readlinkat, size_t, unlink, AT_FDCWD, CLOCK_BOOTTIME,
    S_IFCHR,
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
    let raw_src = optional_c_string(source)?;
    let raw_tgt = optional_c_string(target)?;
    let raw_fs = optional_c_string(fs)?;
    let raw_fs_opt = optional_c_string(opt)?;

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
pub(crate) fn do_readlink(path: &str) -> Result<String, Errno> {
    // has ownership
    let raw_path = CString::new(path).unwrap();
    unsafe {
        const BUF_SZ: usize = 1024;
        let mut buf = [0u8; BUF_SZ];
        let len = readlinkat(
            AT_FDCWD,
            raw_path.as_ref().as_ptr(),
            buf.as_mut_ptr() as *mut _,
            buf.len() as size_t,
        );
        if len == -1 {
            return Err(errno());
        }
        Ok(String::from_utf8_lossy(&buf[..len as usize]).into_owned())
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

fn optional_c_string(value: Option<&str>) -> Result<Option<CString>, Errno> {
    value
        .map(CString::new)
        .transpose()
        .map_err(|_| Errno(EINVAL))
}

pub(crate) fn do_execv(path: &str, argv: *mut *const i8) -> SystemResult {
    if argv.is_null() {
        return Err(Errno(EINVAL));
    }

    let raw_init_path = CString::new(path).map_err(|_| Errno(EINVAL))?;
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
            Ok(time)
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;
    use std::format;
    use std::fs;
    use std::os::unix::fs::symlink;
    use std::path::PathBuf;
    use std::string::ToString;

    fn temp_path(name: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!("init-wrapper-{}-{}", std::process::id(), name));
        let _ = fs::remove_file(&path);
        let _ = fs::remove_dir_all(&path);
        path
    }

    #[test]
    fn mkdir_and_rmdir_forward_success_and_errno() {
        let dir = temp_path("mkdir-rmdir");
        let dir_str = dir.to_string_lossy().to_string();

        do_mkdir(&dir_str, 0o700).expect("mkdir should create temp dir");
        assert!(dir.is_dir());
        assert!(do_mkdir(&dir_str, 0o700).is_err());
        do_rmdir(&dir_str).expect("rmdir should remove temp dir");
        assert!(!dir.exists());
        assert!(do_rmdir(&dir_str).is_err());
    }

    #[test]
    fn unlink_removes_files_and_reports_missing_paths() {
        let file = temp_path("unlink");
        fs::write(&file, b"x").expect("write temp file");
        let file_str = file.to_string_lossy().to_string();

        do_unlink(&file_str).expect("unlink should remove temp file");
        assert!(!file.exists());
        assert!(do_unlink(&file_str).is_err());
    }

    #[test]
    fn readlink_returns_owned_target_without_requiring_nul_termination() {
        let link = temp_path("readlink");
        symlink("target-without-nul", &link).expect("create symlink");
        let link_str = link.to_string_lossy().to_string();

        let target = do_readlink(&link_str).expect("readlink should succeed");
        assert_eq!(target, "target-without-nul");
        fs::remove_file(&link).expect("cleanup symlink");
    }

    #[test]
    fn readlink_reports_missing_path() {
        let missing = temp_path("missing-readlink");
        let missing_str = missing.to_string_lossy().to_string();
        assert!(do_readlink(&missing_str).is_err());
    }

    #[test]
    fn chdir_changes_directory_and_reports_missing_paths() {
        let original = std::env::current_dir().expect("current dir");
        let dir = temp_path("chdir");
        fs::create_dir(&dir).expect("create temp dir");
        let dir_str = dir.to_string_lossy().to_string();

        do_chdir(&dir_str).expect("chdir should enter temp dir");
        assert_eq!(std::env::current_dir().expect("new cwd"), dir);
        std::env::set_current_dir(&original).expect("restore cwd");
        fs::remove_dir(&dir).expect("cleanup temp dir");
        assert!(do_chdir(&dir_str).is_err());
    }

    #[test]
    fn mount_rejects_nul_bytes_before_syscall() {
        assert_eq!(
            do_mount(Some("bad\0source"), None, None, 0, None),
            Err(Errno(EINVAL))
        );
        assert_eq!(
            do_mount(None, Some("bad\0target"), None, 0, None),
            Err(Errno(EINVAL))
        );
        assert_eq!(
            do_mount(None, None, Some("bad\0fs"), 0, None),
            Err(Errno(EINVAL))
        );
        assert_eq!(
            do_mount(None, None, None, 0, Some("bad\0option")),
            Err(Errno(EINVAL))
        );
    }

    #[test]
    fn execv_rejects_null_argv_and_nul_path_before_syscall() {
        assert_eq!(
            do_execv("/sbin/init", core::ptr::null_mut()),
            Err(Errno(EINVAL))
        );

        let mut argv = [core::ptr::null()];
        assert_eq!(do_execv("bad\0init", argv.as_mut_ptr()), Err(Errno(EINVAL)));
    }

    #[test]
    fn gettime_returns_boottime_value() {
        let time = do_gettime().expect("clock_gettime(CLOCK_BOOTTIME) should work");
        assert!(time.ts.tv_sec >= 0);
        assert!(time.ts.tv_nsec >= 0);
        assert!(time.ts.tv_nsec < 1_000_000_000);
    }
}
