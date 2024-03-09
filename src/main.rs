#![no_std]
#![no_main]

extern crate alloc;
mod kmsg;
mod link;
mod mem;
mod tm;
mod unix;
use errno::Errno;

use kmsg::init_kmsg;
use libc::{
    c_int, EACCES, EINVAL, ENOENT, ENOEXEC, ENOTDIR, EXIT_FAILURE, EXIT_SUCCESS, MNT_DETACH,
    MS_BIND, MS_LAZYTIME, MS_MOVE, MS_NODEV, MS_NOSUID, MS_PRIVATE, MS_RDONLY, MS_REC, MS_RELATIME,
};

use unix::{
    do_chdir, do_execv, do_gettime, do_mkdir, do_mount, do_pivot_root, do_rmdir, do_umount,
};

fn replace_root() -> Result<(), ()> {
    match do_mount(None, Some("/"), None, MS_REC | MS_PRIVATE, None) {
        Err(e) => {
            kprintln!("remount \"/\" with private option failed: {}", e);
            return Err(());
        }
        Ok(_) => kprintln!("\"/\" remounted"),
    };

    match do_mount(
        Some("parentfs"),
        Some("/run"),
        Some("tmpfs"),
        MS_LAZYTIME | MS_NODEV | MS_NOSUID | MS_RELATIME,
        None,
    ) {
        Err(e) => {
            kprintln!("mount \"/run\" failed: {}", e);
            return Err(());
        }
        Ok(_) => kprintln!("\"/run\" mounted"),
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
            Err(e) => {
                kprintln!("mkdir \"{}\" failed: {}", path, e);
                return Err(());
            }
            Ok(_) => kprintln!("\"{}\" created", path),
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
            kprintln!("mount \"/run/overlay/lower\" failed: {}", e);
            return Err(());
        }
        Ok(_) => kprintln!("\"/run/overlay/lower\" mounted"),
    };

    match do_mount(
        Some("rootfs"),
         Some("/run/overlay/merged"), 
         Some("overlay"), MS_RELATIME|MS_LAZYTIME, 
         Some("lowerdir=/run/overlay/lower,upperdir=/run/overlay/upper,workdir=/run/overlay/work,redirect_dir=on,uuid=on,metacopy=on,volatile"),
        ) {
        Err(e) => {kprintln!("mount \"/run/overlay/merged\" failed: {}", e);
        return Err(());
    },
        Ok(_) => kprintln!("\"/run/overlay/merged\" mounted"),
    };

    match do_mkdir("/run/overlay/merged/oldroot", 0o0700) {
        Err(e) => {
            kprintln!("mkdir \"/run/overlay/merged/oldroot\" failed: {}", e);
            return Err(());
        }
        Ok(_) => kprintln!("\"/run/overlay/merged/oldroot\" created"),
    }

    match do_pivot_root("/run/overlay/merged", "/run/overlay/merged/oldroot") {
        Err(e) => {
            kprintln!("pivot_root failed: {}", e);
            return Err(());
        }
        Ok(_) => kprintln!("pivot_root done"),
    }

    match do_chdir("/") {
        Err(e) => {
            kprintln!("chdir \"/\" failed: {}", e);
            return Err(());
        }
        Ok(_) => kprintln!("chdir \"/\" done"),
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
            (Err(e), _) => {
                kprintln!("moving mountpoint \"{}\"  failed: {}", src, e);
                return Err(());
            }
            (Ok(_), _) => kprintln!("mountpoint \"{}\" moved", src),
        };
    }

    match do_umount("/oldroot", MNT_DETACH) {
        Err(e) => {
            kprintln!("umount mountpoint \"/oldroot\"  failed: {}", e);
            return Err(());
        }
        Ok(_) => kprintln!("mountpoint \"/oldroot/run\" unmounted"),
    };

    let unused_dirs = ["/oldroot", "/run/overlay/merged"];
    for path in unused_dirs {
        match do_rmdir(path) {
            Err(e) => {
                kprintln!("remove \"{}\" failed: {}", path, e);
                return Err(());
            }
            Ok(_) => kprintln!("\"{}\" removed", path),
        }
    }

    Ok(())
}

#[no_mangle]
unsafe extern "C" fn main(_argc: c_int, argv: *mut *const u8) -> c_int {
    init_kmsg();
    let start = match do_gettime() {
        Err(err) => {
            kprintln!("{}", err);
            return EXIT_FAILURE;
        }
        Ok(v) => v,
    };

    match replace_root() {
        Err(_) => return EXIT_FAILURE,
        Ok(_) => kprintln!("rootfs replaced with overlayfs!"),
    };

    let end = match do_gettime() {
        Err(err) => {
            kprintln!("{}", err);
            return EXIT_FAILURE;
        }
        Ok(v) => v,
    };
    let elapsed = end - start;
    kprintln!("processed in {:?}", elapsed);

    // overwrite arg0

    let init_candidates = ["/sbin/init", "/usr/sbin/init", "/usr/lib/systemd/systemd"];
    for init_path in init_candidates {
        kprintln!("trying to execute \"{}\"", init_path);
        match do_execv(init_path, argv) {
            Err(Errno(ENOENT)) | Err(Errno(EACCES)) | Err(Errno(ENOEXEC)) | Err(Errno(ENOTDIR)) => {
                continue;
            }
            Err(e) => {
                kprintln!("execution \"{}\" failed: {}", init_path, e);
                break;
            }
            Ok(()) => return EXIT_SUCCESS,
        };
    }
    EXIT_FAILURE
}
