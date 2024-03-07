# init-wrapper
It performs the same operation as the shell at the bottom.

```
#!/bin/sh
set -eu
mount --make-private /
mount -t tmpfs -o lazytime,relatime,nodev,nosuid parentfs /run
mkdir -p /run/overlay
mkdir -p /run/overlay/lower
mkdir -p /run/overlay/upper
mkdir -p /run/overlay/work
mkdir -p /run/overlay/merged
mount -o bind / /run/overlay/lower
mount -t overlay -o rw,relatime,lowerdir=/run/overlay/lower,upperdir=/run/overlay/upper,workdir=/run/overlay/work,redirect_dir=on,uuid=on,metacopy=on,volatile rootfs /run/overlay/merged
mkdir -p /run/overlay/merged/oldroot
pivot_root /run/overlay/merged /run/overlay/merged/oldroot
cd /
mount  --move /oldroot/run /run
umount -l /oldroot
rmdir /oldroot
rmdir /run/overlay/merged
exec /sbin/init
```


## USAGE
When booting the kernel, add the following line to the append parameter: 

```
init=/sbin/init-wrapper
```