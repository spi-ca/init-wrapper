# Current QEMU boot smoke

Date: 2026-06-16

This artifact records the disposable QEMU/KVM smoke check used to validate the current `init-wrapper` root replacement path. It is a boot-path smoke, not a production certification.

## Purpose

Validate that the release `init-wrapper` binary can:

1. run in an early boot PID 1 path,
2. mount `/run` as tmpfs,
3. bind-mount the original root to `/run/overlay/lower`,
4. remount that lower read-only,
5. mount the overlay root,
6. complete `pivot_root`,
7. hand off to the real init candidate, and
8. allow writes on the volatile overlay root.

## Environment

```text
host kernel: Linux 7.1.0-1-spica-git x86_64
qemu: /usr/bin/qemu-system-x86_64
accelerator: kvm
kernel: /usr/lib/modules/7.1.0-1-spica-git/vmlinuz
initramfs: generated disposable newc archive under /tmp
```

The host `qemu-system-x86_64` binary required older nettle/hogweed shared libraries. For this smoke only, the matching Arch package was downloaded to `/tmp` and exposed through `LD_LIBRARY_PATH`.

The boot kernel has `overlayfs` as a module. The generated initramfs therefore used a tiny `/init` loader to load `overlay.ko` with `finit_module(2)` and then `execv("/sbin/init-wrapper", ...)`. After that exec, `init-wrapper` remained PID 1 and exercised the normal root replacement path.

## Command shape

```bash
qemu-system-x86_64 \
  -enable-kvm \
  -cpu host \
  -m 512M \
  -nographic \
  -no-reboot \
  -kernel /usr/lib/modules/$(uname -r)/vmlinuz \
  -initrd /tmp/init-wrapper-qemu.*/initramfs.cpio \
  -append 'console=ttyS0 panic=-1 oops=panic'
```

The initramfs contained:

- `/init`: module-loader helper that loads `/overlay.ko` and execs `/sbin/init-wrapper`
- `/sbin/init-wrapper`: current release binary from `target/release/init-wrapper`
- `/sbin/init`: smoke init helper
- dynamic loader and libc needed by the test binaries
- `/overlay.ko`: decompressed module for the boot kernel
- `/dev/console` and `/dev/null` device nodes

## Result

This smoke was re-run after the expanded unit-test pass and produced the same success transcript.

```text
LOADER_FINIT_OVERLAY_OK
init-wrapper: rootfs replaced with overlayfs!
init-wrapper: trying to execute "/sbin/init"
REAL_INIT_REACHED pid=1
SMOKE_MOUNT parentfs /run tmpfs rw,lazytime,nosuid,nodev,relatime,inode64 0 0
SMOKE_MOUNT rootfs /run/overlay/lower rootfs ro,size=237832k,nr_inodes=59458,inode64 0 0
SMOKE_MOUNT rootfs / overlay rw,lazytime,relatime,lowerdir=/run/overlay/lower,upperdir=/run/overlay/upper,workdir=/run/overlay/work,redirect_dir=on,uuid=on,metacopy=on,fsync=volatile 0 0
SMOKE_WRITE_OK /smoke-written
reboot: Power down
```

## Interpretation

- `init-wrapper` completed root replacement and logged `rootfs replaced with overlayfs!`.
- The smoke init ran as PID 1 after `init-wrapper` executed `/sbin/init`.
- `/run` was tmpfs.
- `/run/overlay/lower` was mounted read-only (`ro`).
- `/` was overlayfs with the expected lower, upper, work, redirect, metacopy, and volatile/fsync options.
- A write to `/smoke-written` succeeded on the overlay root.

## Limits

- The initramfs preloaded `overlay.ko` before `execv("/sbin/init-wrapper")` because the selected kernel ships overlayfs as a module. A kernel with built-in overlayfs can boot directly into `init-wrapper` without this helper.
- The smoke uses a synthetic initramfs and smoke init helper, not a full distribution rootfs.
- The test validates the current x86_64 release binary path. Cross-target deployments still need target-specific build and boot evidence.
