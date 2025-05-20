# NeonForge
![Rust](https://img.shields.io/badge/rust-1.84.0_nightly-orange.svg)

![Loading](media/Timeline1.gif)

## About the project
The project is a simple operating system kernel implementation in the Rust programming language, focused on working with a VGA text interface.

## The commands currently supported
* hello – prints HELLO!
* time – displays the system time.
* time_set – sets the system time (example: time_set 12:00:00).
* reboot – restarts the system.
* shutdown – turns off the system.
* clear – clears the terminal.

new:
* date - displays the system date.
* date_set - sets the system date (example: date_set 01.01.2000).

## Kernel capabilities
* Added heap support (alloc).
* Added GPIO support for RPI4.
* Added bar panel.

## Installation

```
sudo apt update
```

```
sudo apt install -y qemu qemu-kvm libvirt-daemon-system libvirt-clients bridge-utils virt-manager
```

install Rust
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

install nightly rust
```
rustup toolchain install nightly-2024-11-08

rustup default nightly-2024-11-08

rustup update nightly
```

```
rustup component add llvm-tools-preview
```

```
rustc --version
```

```
rustup component add rust-src
```

install bootimage
```
cargo install bootimage
```

build kernel
```
cd my_kernel

cargo bootimage
```

### Kernel launch:

```
qemu-system-x86_64 -drive format=raw,file=target/x86_64-blog_os/debug/bootimage-my_kernel.bin
```

## Burning the kernel to disk
### Virtual disk:

```
dd if=/dev/zero of=disk.img bs=1M count=5120
```

```
sudo parted disk.img mklabel msdos

sudo parted disk.img mkpart primary 1M 100%

sudo parted disk.img unit B print
```

#### add fat32
```
LOOP_DEV=$(losetup --find --show --offset 1048576 disk.img)

sudo mkfs.vfat $LOOP_DEV

sudo losetup -d $LOOP_DEV
```

```
dd if=target/x86_64-blog_os/debug/bootimage-my_kernel.bin of=disk.img conv=notrunc
```

```
qemu-system-x86_64 -drive format=raw,file=disk.img
```

### USB disk:

It is also possible to write it to a USB stick and boot it on a real machine, but be careful to choose the correct device name, because everything on that device is overwritten:

```
dd if=target/x86_64-blog_os/debug/bootimage-blog_os.bin of=/dev/sdX && sync
```

Where sdX is the device name of your USB stick.

After writing the image to the USB stick, you can run it on real hardware by booting from it. You probably need to use a special boot menu or change the boot order in your BIOS configuration to boot from the USB stick. Note that it currently doesn’t work for UEFI machines, since the bootloader crate has no UEFI support yet.


## Errors

- [ ] The MBR sector of the disk is erased, which leads to the error "Invalid MBR signature" and does not allow working with the file system.
