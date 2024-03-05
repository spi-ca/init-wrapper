#![no_std]
#![no_main]

extern crate alloc;
mod link;
mod mem;

use alloc::ffi::CString;
use core::ffi::CStr;
use core::ops::Sub;
use core::ptr::null;
use core::time::Duration;
use errno::{errno, Errno};
use libc::{
    c_int, c_void, chdir, clock_gettime, execvp, mkdir, mode_t, mount, rmdir, syscall, timespec,
    umount2, SYS_pivot_root, CLOCK_BOOTTIME, EXIT_FAILURE, EXIT_SUCCESS, MNT_DETACH, MS_BIND,
    MS_LAZYTIME, MS_MOVE, MS_NODEV, MS_NOSUID, MS_PRIVATE, MS_RDONLY, MS_REC, MS_RELATIME,
};
use libc_print::std_name::eprintln;

struct Timespec {
    ts: timespec,
}

type SystemResult = Result<(), Errno>;

impl Sub<Timespec> for Timespec {
    type Output = Duration;

    fn sub(self, other: Timespec) -> Duration {
        let sec = self.ts.tv_sec - other.ts.tv_sec;
        let nsec = self.ts.tv_nsec - other.ts.tv_nsec;
        if nsec < 0 {
            Duration::from_secs(sec as u64) - Duration::from_nanos(nsec.unsigned_abs())
        } else {
            Duration::from_secs(sec as u64) + Duration::from_nanos(nsec.unsigned_abs())
        }
    }
}

impl From<timespec> for Timespec {
    fn from(item: timespec) -> Self {
        Timespec { ts: item }
    }
}

impl Into<timespec> for Timespec {
    fn into(self) -> timespec {
        self.ts
    }
}

fn do_mount(
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
fn do_umount(path: &str, flags: i32) -> SystemResult {
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

fn do_mkdir(path: &str, mode: mode_t) -> SystemResult {
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
fn do_rmdir(path: &str) -> SystemResult {
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
fn do_chdir(path: &str) -> SystemResult {
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

fn pivot_root(new_root: &str, put_old: &str) -> SystemResult {
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

fn do_gettime() -> Result<Timespec, Errno> {
    let mut time = timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };

    unsafe {
        if clock_gettime(CLOCK_BOOTTIME, &mut time) == -1 {
            Err(errno())
        } else {
            Ok(time.into())
        }
    }
}

fn replace_root() -> Result<(), ()> {
    match do_mount(None, Some("/"), None, MS_REC | MS_PRIVATE, None) {
        Err(e) => {
            eprintln!("remount \"/\" with private option failed: {}", e);
            return Err(());
        }
        Ok(_) => eprintln!("\"/\" remounted"),
    };

    match do_mount(
        Some("parentfs"),
        Some("/run"),
        Some("tmpfs"),
        MS_LAZYTIME | MS_NODEV | MS_NOSUID | MS_RELATIME,
        None,
    ) {
        Err(e) => {
            eprintln!("mount \"/run\" failed: {}", e);
            return Err(());
        }
        Ok(_) => eprintln!("\"/run\" mounted"),
    };

    let ovr_dir_structures = [
        "/run/overlay",
        "/run/overlay/lower",
        "/run/overlay/upper",
        "/run/overlay/work",
        "/run/overlay/merged",
    ];
    for path in ovr_dir_structures {
        match do_mkdir(path, 0o0700) {
            Err(e) => {
                eprintln!("mkdir \"{}\" failed: {}", path, e);
                return Err(());
            }
            Ok(_) => eprintln!("\"{}\" created", path),
        }
    }

    match do_mount(
        Some("/"),
        Some("/run/overlay/lower"),
        None,
        MS_BIND | MS_RDONLY,
        None,
    ) {
        Err(e) => {
            eprintln!("mount \"/run/overlay/lower\" failed: {}", e);
            return Err(());
        }
        Ok(_) => eprintln!("\"/run/overlay/lower\" mounted"),
    };

    match do_mount(
        Some("rootfs"),
         Some("/run/overlay/merged"), 
         Some("overlay"), MS_RELATIME|MS_LAZYTIME, 
         Some("lowerdir=/run/overlay/lower,upperdir=/run/overlay/upper,workdir=/run/overlay/work,redirect_dir=on,uuid=on,metacopy=on,volatile"),
        ) {
        Err(e) => {eprintln!("mount \"/run/overlay/merged\" failed: {}", e);
        return Err(());
    },
        Ok(_) => eprintln!("\"/run/overlay/merged\" mounted"),
    };

    match do_mkdir("/run/overlay/merged/oldroot", 0o0700) {
        Err(e) => {
            eprintln!("mkdir \"/run/overlay/merged/oldroot\" failed: {}", e);
            return Err(());
        }
        Ok(_) => eprintln!("\"/run/overlay/merged/oldroot\" created"),
    }

    match pivot_root("/run/overlay/merged", "/run/overlay/merged/oldroot") {
        Err(e) => {
            eprintln!("pivot_root failed: {}", e);
            return Err(());
        }
        Ok(_) => eprintln!("pivot_root done"),
    }

    match do_chdir("/") {
        Err(e) => {
            eprintln!("chdir \"/\" failed: {}", e);
            return Err(());
        }
        Ok(_) => eprintln!("chdir \"/\" done"),
    }

    match do_mount(
        Some("/oldroot/run"),
        Some("/run"),
        None,
        MS_REC | MS_MOVE,
        None,
    ) {
        Err(e) => {
            eprintln!("moving mountpoint \"/oldroot/run\"  failed: {}", e);
            return Err(());
        }
        Ok(_) => eprintln!("mountpoint \"/oldroot/run\" moved"),
    };

    match do_umount("/oldroot", MNT_DETACH) {
        Err(e) => {
            eprintln!("umount mountpoint \"/oldroot\"  failed: {}", e);
            return Err(());
        }
        Ok(_) => eprintln!("mountpoint \"/oldroot/run\" unmounted"),
    };

    let unused_dirs = ["/oldroot", "/run/overlay/merged"];
    for path in unused_dirs {
        match do_rmdir(path) {
            Err(e) => {
                eprintln!("remove \"{}\" failed: {}", path, e);
                return Err(());
            }
            Ok(_) => eprintln!("\"{}\" removed", path),
        }
    }

    Ok(())
}

#[no_mangle]
unsafe extern "C" fn main(_argc: c_int, argv: *const *const u8) -> c_int {
    let start = match do_gettime() {
        Err(err) => {
            eprintln!("{}", err);
            return EXIT_FAILURE;
        }
        Ok(v) => v,
    };

    match replace_root() {
        Err(_) => return EXIT_FAILURE,
        Ok(_) => eprintln!("rootfs replaced with overlayfs!"),
    };

    let end = match do_gettime() {
        Err(err) => {
            eprintln!("{}", err);
            return EXIT_FAILURE;
        }
        Ok(v) => v,
    };
    let elapsed = end - start;
    eprintln!("processed in {:?}", elapsed);

    let init_path = CStr::from_bytes_with_nul_unchecked(b"/sbin/init\0");
    execvp(init_path.as_ptr(), argv);

    EXIT_SUCCESS
}
