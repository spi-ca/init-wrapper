#![no_std]
#![no_main]

extern crate alloc;
mod kmsg;
mod link;
mod mem;
mod tm;
mod unix;
use alloc::format;
use alloc::string::String;
use errno::Errno;
use kmsg::init_kmsg;
use libc::{
    c_int, EACCES, EINVAL, ENOENT, ENOEXEC, ENOTDIR, EXIT_FAILURE, EXIT_SUCCESS, MNT_DETACH,
    MS_BIND, MS_LAZYTIME, MS_MOVE, MS_NODEV, MS_NOSUID, MS_PRIVATE, MS_RDONLY, MS_REC, MS_RELATIME,
};

use unix::{do_chdir, do_execv, do_mkdir, do_mount, do_pivot_root, do_rmdir, do_umount};

fn replace_root() -> Result<(), String> {
    match do_mount(None, Some("/"), None, MS_REC | MS_PRIVATE, None) {
        Err(e) => return Err(format!("remount \"/\" with private option failed: {}", e)),
        Ok(_) => (),
    };

    match do_mount(
        Some("parentfs"),
        Some("/run"),
        Some("tmpfs"),
        MS_LAZYTIME | MS_NODEV | MS_NOSUID | MS_RELATIME,
        None,
    ) {
        Err(e) => return Err(format!("mount \"/run\" failed: {}", e)),
        Ok(_) => (),
    };

    let ovr_dir_structures = [
        ("/run/overlay", 0o0700),
        ("/run/overlay/lower", 0o0700),
        ("/run/overlay/upper", 0o0755),
        ("/run/overlay/work", 0o0755),
        ("/run/overlay/merged", 0o0755),
    ];

    for (path, mode) in ovr_dir_structures {
        match do_mkdir(path, mode) {
            Err(e) => return Err(format!("mkdir \"{}\" failed: {}", path, e)),
            Ok(_) => (),
        }
    }

    match do_mount(
        Some("/"),
        Some("/run/overlay/lower"),
        None,
        MS_BIND | MS_RDONLY,
        None,
    ) {
        Err(e) => return Err(format!("mount \"/run/overlay/lower\" failed: {}", e)),
        Ok(_) => (),
    };

    match do_mount(
        Some("rootfs"),
         Some("/run/overlay/merged"),
         Some("overlay"), MS_RELATIME|MS_LAZYTIME,
         Some("lowerdir=/run/overlay/lower,upperdir=/run/overlay/upper,workdir=/run/overlay/work,redirect_dir=on,uuid=on,metacopy=on,volatile"),
        ) {
            Err(e) =>return Err(format!("mount \"/run/overlay/merged\" failed: {}", e)),
            Ok(_) => (),
        };

    match do_mkdir("/run/overlay/merged/oldroot", 0o0700) {
        Err(e) => {
            return Err(format!(
                "mkdir \"/run/overlay/merged/oldroot\" failed: {}",
                e
            ))
        }
        Ok(_) => (),
    }

    match do_pivot_root("/run/overlay/merged", "/run/overlay/merged/oldroot") {
        Err(e) => return Err(format!("pivot_root failed: {}", e)),
        Ok(_) => (),
    }

    match do_chdir("/") {
        Err(e) => return Err(format!("chdir \"/\" failed: {}", e)),
        Ok(_) => (),
    }

    let move_mnt_pairs = [
        ("/oldroot/run", "/run", true),
        ("/oldroot/dev", "/dev", false),
    ];
    for (src, dst, required) in move_mnt_pairs {
        match (
            do_mount(Some(src), Some(dst), None, MS_REC | MS_MOVE, None),
            required,
        ) {
            (Err(Errno(EINVAL)), true) => (),
            (Err(e), _) => return Err(format!("moving mountpoint \"{}\"  failed: {}", src, e)),
            (Ok(_), _) => (),
        };
    }

    match do_umount("/oldroot", MNT_DETACH) {
        Err(e) => return Err(format!("umount mountpoint \"/oldroot\"  failed: {}", e)),
        Ok(_) => (),
    };

    let unused_dirs = ["/oldroot", "/run/overlay/merged"];
    for path in unused_dirs {
        match do_rmdir(path) {
            Err(e) => return Err(format!("remove \"{}\" failed: {}", path, e)),
            Ok(_) => (),
        }
    }

    Ok(())
}

#[no_mangle]
unsafe extern "C" fn main(_argc: c_int, argv: *mut *const u8) -> c_int {
    init_kmsg();

    match replace_root() {
        Err(err) => {
            kprintln!("failed to replace rootfs: {}", err);
            return EXIT_FAILURE;
        }
        Ok(_) => kprintln!("rootfs replaced with overlayfs!"),
    };

    let init_candidates = ["/sbin/init", "/usr/sbin/init", "/usr/lib/systemd/systemd"];
    for init_path in init_candidates {
        kprintln!("trying to execute \"{}\"", init_path);
        match do_execv(init_path, argv) {
            Err(Errno(ENOENT)) | Err(Errno(EACCES)) | Err(Errno(ENOEXEC)) | Err(Errno(ENOTDIR)) => {
                continue;
            }
            Err(e) => {
                kprintln!("execution \"{}\" failed: {}", init_path, e);
                return EXIT_FAILURE;
            }
            Ok(()) => return EXIT_SUCCESS,
        };
    }
    EXIT_FAILURE
}
