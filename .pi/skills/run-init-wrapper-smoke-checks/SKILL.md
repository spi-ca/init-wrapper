---
name: "run-init-wrapper-smoke-checks"
description: "Run and record focused smoke checks for init-wrapper in a disposable early-boot environment"
version: 1
created: "2026-06-14"
updated: "2026-06-16"
---
## When to Use
Use when changes affect init-wrapper boot flow, mount namespace behavior, overlayfs options, init fallback, or operator-facing documentation for this repo.

## Procedure
1. Build the release binary for the target rootfs. Use `cargo build --release` for the host default target, or pass an explicit target such as `--target aarch64-unknown-linux-musl` / install a matching `.cargo/config.toml` when the target rootfs differs. If an explicit target is used, copy the binary from `target/<triple>/release/init-wrapper`; otherwise use `target/release/init-wrapper`. `Dockerfile` / `Dockerfile.debian` may also be used, but verify their output path matches the selected target.
2. Prepare a disposable VM, test image, or initramfs where the binary is installed as `/sbin/init-wrapper`, then boot the kernel with `init=/sbin/init-wrapper`.
3. Confirm the environment has the privileges and kernel features required for `mount`, `tmpfs`, `overlayfs`, `pivot_root`, and the configured overlay mount options (`redirect_dir=on`, `uuid=on`, `metacopy=on`, `volatile`).
4. Capture early boot logs from `/dev/kmsg`, serial console, or stderr and verify that root replacement succeeded before the real init hand-off.
5. Inspect the running system with `findmnt` or `mount` to confirm `/` is the overlay mount, `/run` is tmpfs, and overlay upper/work live under `/run/overlay`.
6. If the change touches init hand-off behavior, test both the normal path and at least one fallback path by controlling which init candidate exists in the image.

## Pitfalls
- Never run these checks against a production rootfs. Use only disposable boot environments because the task intentionally rewires the active root mount.
- Do not treat a successful build as proof that the target kernel supports every overlay option used by the binary.
- Remember that writes land in tmpfs-backed upper/work and are expected to disappear after reboot.
- If logs are missing from `/dev/kmsg`, check stderr or serial output because logging falls back when `/dev/kmsg` is unavailable.

## Verification
1. The selected build command succeeds when dependencies are available, and the produced artifact path matches the chosen target (`target/release/init-wrapper` for host default or `target/<triple>/release/init-wrapper` for explicit targets).
2. A disposable boot with `init=/sbin/init-wrapper` reaches a real init process and logs `rootfs replaced with overlayfs!`.
3. `findmnt / /run` or equivalent confirms overlay root plus tmpfs `/run`.
4. A test write under `/` appears during the booted session and disappears after reboot, confirming the volatile upper/work behavior.
