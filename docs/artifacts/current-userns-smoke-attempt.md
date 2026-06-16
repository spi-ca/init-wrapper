# Current user-namespace smoke attempt

Date: 2026-06-16

This artifact records the non-privileged smoke attempt that was possible in the current agent environment. It is **not** a replacement for the required disposable VM/initramfs early-boot smoke test because the real target runs as privileged PID 1 during boot.

## Environment checks

```text
id: uid=1000(spi-ca) gid=1000(spi-ca) groups=1000(spi-ca),967(realtime),994(input),998(wheel)
sudo -n true: sudo requires a password
findmnt -T /: / btrfs ro,relatime,lazytime,compress=zstd:1,ssd,discard=async,space_cache=v2,commit=60,subvolid=274,subvol=/@
findmnt -T /run: /run tmpfs rw,nosuid,nodev,relatime,lazytime,size=6467604k,nr_inodes=819200,mode=755,inode64
/proc/filesystems: overlay available
```

A minimal user+mount+PID namespace mount probe succeeded:

```bash
tmp=$(mktemp -d /tmp/init-wrapper-smoke.XXXXXX)
unshare -Urmpf --mount-proc sh -c \
  'set -eu; mount -t tmpfs tmpfs "$1"; touch "$1/ok"; test -f "$1/ok"; umount "$1"' sh "$tmp"
rmdir "$tmp"
```

## init-wrapper attempt

Command:

```bash
timeout 20s unshare -Urmpf --mount-proc ./target/release/init-wrapper \
  > /tmp/init-wrapper-smoke.stdout \
  2> /tmp/init-wrapper-smoke.stderr
```

Result:

```text
rc=1
stdout: <empty>
stderr: init-wrapper: failed to replace rootfs: mount "/run/overlay/lower" failed: Invalid argument
```

## Interpretation

The current environment can create a user/mount/PID namespace and mount tmpfs, but it cannot complete the `init-wrapper` root replacement path. The failure occurs before overlay mount and `pivot_root`, at the whole-root bind mount for `/run/overlay/lower`.

Completion evidence still requires a privileged disposable VM/initramfs boot with `init=/sbin/init-wrapper`, followed by mount tree and init hand-off inspection.
