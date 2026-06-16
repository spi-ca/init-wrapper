#![no_std]
#![cfg_attr(not(test), no_main)]

extern crate alloc;
mod kmsg;
#[cfg(not(test))]
mod link;
#[cfg(not(test))]
mod mem;
mod tm;
mod unix;
#[cfg(not(test))]
use alloc::format;
#[cfg(not(test))]
use alloc::string::String;
use errno::Errno;
#[cfg(not(test))]
use kmsg::init_kmsg;
#[cfg(not(test))]
use libc::{c_int, EXIT_FAILURE, EXIT_SUCCESS};
use libc::{
    EACCES, EINVAL, ENOENT, ENOEXEC, ENOTDIR, MNT_DETACH, MS_BIND, MS_LAZYTIME, MS_MOVE, MS_NODEV,
    MS_NOSUID, MS_PRIVATE, MS_RDONLY, MS_REC, MS_RELATIME, MS_REMOUNT,
};

#[cfg(not(test))]
use unix::{do_chdir, do_execv, do_mkdir, do_mount, do_pivot_root, do_rmdir, do_umount};

const PRIVATE_ROOT_FLAGS: u64 = MS_REC | MS_PRIVATE;
const RUN_TMPFS_FLAGS: u64 = MS_LAZYTIME | MS_NODEV | MS_NOSUID | MS_RELATIME;
const OVERLAY_ROOT_FLAGS: u64 = MS_RELATIME | MS_LAZYTIME;
const LOWER_BIND_FLAGS: u64 = MS_BIND;
const LOWER_READONLY_REMOUNT_FLAGS: u64 = MS_BIND | MS_REMOUNT | MS_RDONLY;
const MOVE_MOUNT_FLAGS: u64 = MS_REC | MS_MOVE;
const OLDROOT_UMOUNT_FLAGS: i32 = MNT_DETACH;
const OVERLAY_OPTIONS: &str = "lowerdir=/run/overlay/lower,upperdir=/run/overlay/upper,workdir=/run/overlay/work,redirect_dir=on,uuid=on,metacopy=on,volatile";
const OVERLAY_DIRS: [(&str, u32); 5] = [
    ("/run/overlay", 0o0700),
    ("/run/overlay/lower", 0o0700),
    ("/run/overlay/upper", 0o0755),
    ("/run/overlay/work", 0o0755),
    ("/run/overlay/merged", 0o0755),
];
const UNUSED_DIRS: [&str; 2] = ["/oldroot", "/run/overlay/merged"];
const MOVE_MNT_PAIRS: [(&str, &str, bool); 2] = [
    ("/oldroot/run", "/run", true),
    ("/oldroot/dev", "/dev", false),
];
const INIT_CANDIDATES: [&str; 3] = ["/sbin/init", "/usr/sbin/init", "/usr/lib/systemd/systemd"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RootAction {
    MakeRootPrivate,
    MountRunTmpfs,
    Mkdir {
        path: &'static str,
        mode: u32,
    },
    BindLower,
    RemountLowerReadonly,
    MountOverlay,
    PivotRoot,
    ChdirRoot,
    MoveMount {
        src: &'static str,
        dst: &'static str,
        required: bool,
    },
    UmountOldRoot,
    Rmdir(&'static str),
}

const ROOT_REPLACEMENT_PLAN: [RootAction; 18] = [
    RootAction::MakeRootPrivate,
    RootAction::MountRunTmpfs,
    RootAction::Mkdir {
        path: OVERLAY_DIRS[0].0,
        mode: OVERLAY_DIRS[0].1,
    },
    RootAction::Mkdir {
        path: OVERLAY_DIRS[1].0,
        mode: OVERLAY_DIRS[1].1,
    },
    RootAction::Mkdir {
        path: OVERLAY_DIRS[2].0,
        mode: OVERLAY_DIRS[2].1,
    },
    RootAction::Mkdir {
        path: OVERLAY_DIRS[3].0,
        mode: OVERLAY_DIRS[3].1,
    },
    RootAction::Mkdir {
        path: OVERLAY_DIRS[4].0,
        mode: OVERLAY_DIRS[4].1,
    },
    RootAction::BindLower,
    RootAction::RemountLowerReadonly,
    RootAction::MountOverlay,
    RootAction::Mkdir {
        path: "/run/overlay/merged/oldroot",
        mode: 0o0700,
    },
    RootAction::PivotRoot,
    RootAction::ChdirRoot,
    RootAction::MoveMount {
        src: MOVE_MNT_PAIRS[0].0,
        dst: MOVE_MNT_PAIRS[0].1,
        required: MOVE_MNT_PAIRS[0].2,
    },
    RootAction::MoveMount {
        src: MOVE_MNT_PAIRS[1].0,
        dst: MOVE_MNT_PAIRS[1].1,
        required: MOVE_MNT_PAIRS[1].2,
    },
    RootAction::UmountOldRoot,
    RootAction::Rmdir(UNUSED_DIRS[0]),
    RootAction::Rmdir(UNUSED_DIRS[1]),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MoveMountDecision {
    Continue,
    Fail,
}

fn classify_move_mount_result(result: &Result<(), Errno>, required: bool) -> MoveMountDecision {
    match (result, required) {
        (Ok(_), _) => MoveMountDecision::Continue,
        (Err(Errno(EINVAL)), true) => MoveMountDecision::Continue,
        (Err(_), false) => MoveMountDecision::Continue,
        (Err(_), true) => MoveMountDecision::Fail,
    }
}

fn should_try_next_init(error: Errno) -> bool {
    matches!(
        error,
        Errno(ENOENT) | Errno(EACCES) | Errno(ENOEXEC) | Errno(ENOTDIR)
    )
}

#[cfg(not(test))]
fn execute_root_action(action: RootAction) -> Result<(), String> {
    match action {
        RootAction::MakeRootPrivate => do_mount(None, Some("/"), None, PRIVATE_ROOT_FLAGS, None)
            .map_err(|e| format!("remount \"/\" with private option failed: {}", e)),
        RootAction::MountRunTmpfs => do_mount(
            Some("parentfs"),
            Some("/run"),
            Some("tmpfs"),
            RUN_TMPFS_FLAGS,
            None,
        )
        .map_err(|e| format!("mount \"/run\" failed: {}", e)),
        RootAction::Mkdir { path, mode } => {
            do_mkdir(path, mode).map_err(|e| format!("mkdir \"{}\" failed: {}", path, e))
        }
        RootAction::BindLower => do_mount(
            Some("/"),
            Some("/run/overlay/lower"),
            None,
            LOWER_BIND_FLAGS,
            None,
        )
        .map_err(|e| format!("mount \"/run/overlay/lower\" failed: {}", e)),
        RootAction::RemountLowerReadonly => do_mount(
            None,
            Some("/run/overlay/lower"),
            None,
            LOWER_READONLY_REMOUNT_FLAGS,
            None,
        )
        .map_err(|e| format!("remount \"/run/overlay/lower\" readonly failed: {}", e)),
        RootAction::MountOverlay => do_mount(
            Some("rootfs"),
            Some("/run/overlay/merged"),
            Some("overlay"),
            OVERLAY_ROOT_FLAGS,
            Some(OVERLAY_OPTIONS),
        )
        .map_err(|e| format!("mount \"/run/overlay/merged\" failed: {}", e)),
        RootAction::PivotRoot => {
            do_pivot_root("/run/overlay/merged", "/run/overlay/merged/oldroot")
                .map_err(|e| format!("pivot_root failed: {}", e))
        }
        RootAction::ChdirRoot => do_chdir("/").map_err(|e| format!("chdir \"/\" failed: {}", e)),
        RootAction::MoveMount { src, dst, required } => {
            let move_result = do_mount(Some(src), Some(dst), None, MOVE_MOUNT_FLAGS, None);
            match classify_move_mount_result(&move_result, required) {
                MoveMountDecision::Continue => Ok(()),
                MoveMountDecision::Fail => match move_result {
                    Err(e) => Err(format!("moving mountpoint \"{}\"  failed: {}", src, e)),
                    Ok(()) => Ok(()),
                },
            }
        }
        RootAction::UmountOldRoot => do_umount("/oldroot", OLDROOT_UMOUNT_FLAGS)
            .map_err(|e| format!("umount mountpoint \"/oldroot\"  failed: {}", e)),
        RootAction::Rmdir(path) => {
            do_rmdir(path).map_err(|e| format!("remove \"{}\" failed: {}", path, e))
        }
    }
}

#[cfg(not(test))]
fn replace_root() -> Result<(), String> {
    for action in ROOT_REPLACEMENT_PLAN {
        execute_root_action(action)?;
    }
    Ok(())
}

#[cfg(not(test))]
#[no_mangle]
unsafe extern "C" fn main(_argc: c_int, argv: *mut *const i8) -> c_int {
    init_kmsg();

    match replace_root() {
        Err(err) => {
            kprintln!("failed to replace rootfs: {}", err);
            return EXIT_FAILURE;
        }
        Ok(_) => kprintln!("rootfs replaced with overlayfs!"),
    };

    for init_path in INIT_CANDIDATES {
        kprintln!("trying to execute \"{}\"", init_path);
        match do_execv(init_path, argv) {
            Err(e) if should_try_next_init(e) => continue,
            Err(e) => {
                kprintln!("execution \"{}\" failed: {}", init_path, e);
                return EXIT_FAILURE;
            }
            Ok(()) => return EXIT_SUCCESS,
        };
    }
    EXIT_FAILURE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_replacement_flags_match_documented_mount_contract() {
        assert_eq!(PRIVATE_ROOT_FLAGS, MS_REC | MS_PRIVATE);
        assert_eq!(
            RUN_TMPFS_FLAGS,
            MS_LAZYTIME | MS_NODEV | MS_NOSUID | MS_RELATIME
        );
        assert_eq!(LOWER_BIND_FLAGS, MS_BIND);
        assert_eq!(
            LOWER_READONLY_REMOUNT_FLAGS,
            MS_BIND | MS_REMOUNT | MS_RDONLY
        );
        assert_eq!(OVERLAY_ROOT_FLAGS, MS_RELATIME | MS_LAZYTIME);
        assert_eq!(MOVE_MOUNT_FLAGS, MS_REC | MS_MOVE);
        assert_eq!(OLDROOT_UMOUNT_FLAGS, MNT_DETACH);
    }

    #[test]
    fn root_replacement_plan_pins_privileged_syscall_order() {
        assert_eq!(
            ROOT_REPLACEMENT_PLAN,
            [
                RootAction::MakeRootPrivate,
                RootAction::MountRunTmpfs,
                RootAction::Mkdir {
                    path: "/run/overlay",
                    mode: 0o0700,
                },
                RootAction::Mkdir {
                    path: "/run/overlay/lower",
                    mode: 0o0700,
                },
                RootAction::Mkdir {
                    path: "/run/overlay/upper",
                    mode: 0o0755,
                },
                RootAction::Mkdir {
                    path: "/run/overlay/work",
                    mode: 0o0755,
                },
                RootAction::Mkdir {
                    path: "/run/overlay/merged",
                    mode: 0o0755,
                },
                RootAction::BindLower,
                RootAction::RemountLowerReadonly,
                RootAction::MountOverlay,
                RootAction::Mkdir {
                    path: "/run/overlay/merged/oldroot",
                    mode: 0o0700,
                },
                RootAction::PivotRoot,
                RootAction::ChdirRoot,
                RootAction::MoveMount {
                    src: "/oldroot/run",
                    dst: "/run",
                    required: true,
                },
                RootAction::MoveMount {
                    src: "/oldroot/dev",
                    dst: "/dev",
                    required: false,
                },
                RootAction::UmountOldRoot,
                RootAction::Rmdir("/oldroot"),
                RootAction::Rmdir("/run/overlay/merged"),
            ]
        );
    }

    #[test]
    fn overlay_directory_layout_matches_runtime_sequence() {
        assert_eq!(
            OVERLAY_DIRS,
            [
                ("/run/overlay", 0o0700),
                ("/run/overlay/lower", 0o0700),
                ("/run/overlay/upper", 0o0755),
                ("/run/overlay/work", 0o0755),
                ("/run/overlay/merged", 0o0755),
            ]
        );
        assert_eq!(UNUSED_DIRS, ["/oldroot", "/run/overlay/merged"]);
    }

    #[test]
    fn overlay_options_pin_lower_upper_work_and_volatile_features() {
        for required in [
            "lowerdir=/run/overlay/lower",
            "upperdir=/run/overlay/upper",
            "workdir=/run/overlay/work",
            "redirect_dir=on",
            "uuid=on",
            "metacopy=on",
            "volatile",
        ] {
            assert!(OVERLAY_OPTIONS.split(',').any(|option| option == required));
        }
    }

    #[test]
    fn init_fallback_order_matches_documented_order() {
        assert_eq!(
            INIT_CANDIDATES,
            ["/sbin/init", "/usr/sbin/init", "/usr/lib/systemd/systemd"]
        );
    }

    #[test]
    fn init_retry_policy_only_retries_missing_or_unusable_candidates() {
        for errno in [ENOENT, EACCES, ENOEXEC, ENOTDIR] {
            assert!(should_try_next_init(Errno(errno)));
        }
        for errno in [EINVAL, libc::EIO, libc::EPERM, libc::ENOMEM] {
            assert!(!should_try_next_init(Errno(errno)));
        }
    }

    #[test]
    fn run_mount_move_is_required_and_dev_move_is_best_effort() {
        assert_eq!(MOVE_MNT_PAIRS[0], ("/oldroot/run", "/run", true));
        assert_eq!(MOVE_MNT_PAIRS[1], ("/oldroot/dev", "/dev", false));
    }

    #[test]
    fn move_mount_error_policy_matches_early_boot_contract() {
        assert_eq!(
            classify_move_mount_result(&Ok(()), true),
            MoveMountDecision::Continue
        );
        assert_eq!(
            classify_move_mount_result(&Ok(()), false),
            MoveMountDecision::Continue
        );
        assert_eq!(
            classify_move_mount_result(&Err(Errno(EINVAL)), true),
            MoveMountDecision::Continue
        );
        assert_eq!(
            classify_move_mount_result(&Err(Errno(libc::EIO)), true),
            MoveMountDecision::Fail
        );
        assert_eq!(
            classify_move_mount_result(&Err(Errno(libc::EIO)), false),
            MoveMountDecision::Continue
        );
    }
}
