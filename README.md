# my_kernel
![Rust](https://img.shields.io/badge/rust-1.84.0_nightly-orange.svg)

![Loading](media\Timeline1.gif)

### About the project:
The project is a simple operating system kernel implementation in the Rust programming language, focused on working with a VGA text interface.

### The commands currently supported:
* hello – prints HELLO!
* time – displays the system time.
* time_set – sets the system time (example: time_set 12:00:00).
* reboot – restarts the system.
* shutdown – turns off the system.
* clear – clears the terminal.

### Installation:

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
rustup install nightly

rustup default nightly

rustup update nightly
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
