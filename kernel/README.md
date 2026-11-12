# kernel

QuicGuard v1 kernel module (Rust for Linux).

- Built with Kbuild, **not** Cargo: this directory is outside the Cargo workspace.
- Contains only the kernel glue: virtual interface, kernel UDP socket, kernel crypto API, netlink.
- QUIC packet logic lives in `crates/qg-engine` and `crates/qg-wire` (core-only), reused here.
- **Never load this module on your host machine: always test it in a VM.** A bug here can freeze the whole machine.

## Files

| File | Role |
|---|---|
| `quicguard.rs` | Module source |
| `Kbuild` | Tells the kernel build system what to compile (`obj-m := quicguard.o`) |
| `Makefile` | Shortcut so that `make` and `make clean` work from this directory |
| `.gitignore` | Ignores build outputs (`.o`, `.ko`, `.cmd`…) |

## One-time setup

The module is compiled against a kernel that you build yourself with Rust support,
then tested in a VM started with [virtme-ng](https://github.com/arighi/virtme-ng).
Reference versions: **Linux 7.3.4** and **Rust 1.98.1**.

Plan for about 3 GB of disk space and 20 to 40 minutes for the first kernel build.

### 1. Install the tools (Fedora)

```bash
sudo dnf install virtme-ng bindgen-cli clang lld llvm dwarves ncurses-devel \
                 qemu-system-x86 elfutils-libelf-devel openssl-devel flex bison bc
```

Rust must come from [rustup](https://rustup.rs), not from `dnf`.

### 2. Get the kernel sources

Keep them **outside** this repository:

```bash
mkdir -p ~/kernel && cd ~/kernel
curl -LO https://cdn.kernel.org/pub/linux/kernel/v7.x/linux-7.3.4.tar.xz
tar xf linux-7.3.4.tar.xz && rm linux-7.3.4.tar.xz
cd linux-7.3.4
```

### 3. Pin the Rust version

The project's `rust-toolchain.toml` does not apply outside the repository, so pin the version for the kernel directory:

```bash
rustup override set 1.98.1
make LLVM=1 rustavailable      # must print "Rust is available!"
```

### 4. Configure

```bash
vng --kconfig                  # minimal config for virtme-ng (many "override" warnings: normal)
scripts/config --enable RUST
scripts/config --enable SAMPLES --enable SAMPLES_RUST --module SAMPLE_RUST_MINIMAL
scripts/config --disable DEBUG_INFO_DWARF_TOOLCHAIN_DEFAULT --enable DEBUG_INFO_NONE
make LLVM=1 olddefconfig
grep -E 'CONFIG_RUST=|SAMPLE_RUST_MINIMAL|DEBUG_INFO_NONE' .config
```

Expected: `CONFIG_RUST=y`, `CONFIG_SAMPLE_RUST_MINIMAL=m`, `CONFIG_DEBUG_INFO_NONE=y`.
Debug info is turned off to save several GB of disk space.

### 5. Build the kernel

```bash
make LLVM=1 -j$(nproc)
```

`LLVM=1` is required: the module must be built with the same toolchain as the kernel.

### 6. Check the setup with the official sample

```bash
vng --user root                          # boots your kernel, gives you a root shell in the VM
uname -r                                 # 7.3.4
insmod samples/rust/rust_minimal.ko
dmesg | tail                             # "Rust minimal sample (init)"
exit                                     # or Ctrl+D
```

If this works, your kernel, Rust and the VM are ready.

## Daily workflow

**1. Build the module** on your machine, from this directory:

```bash
make                 # produces quicguard.ko
make clean           # removes build outputs
```

If your kernel sources are not in `~/kernel/linux-7.3.4`, run `make KDIR=/path/to/linux`.

**2. Start the VM** (`vng` must be run from the kernel sources):

```bash
cd ~/kernel/linux-7.3.4 && vng --user root
```

**3. Load and test the module** inside the VM. Your home directory is shared with the VM,
so the freshly built `.ko` is there, but use the **full path**: as root, `~` is `/root`.

```bash
insmod /home/<you>/<path-to-repo>/kernel/quicguard.ko
dmesg | tail          # "quicguard: loaded QuicGuard"
rmmod quicguard
dmesg | tail          # "quicguard: unloaded QuicGuard"
```

No need to restart the VM to test a new build: `rmmod`, rebuild on the host, then `insmod` again.
If the module crashes the VM, just start it again.

## Things to know

- **`loading out-of-tree module taints kernel`** is normal: the kernel just records that it runs code from outside its own sources.
- **`Operation not permitted` on `insmod`**: you started `vng` without `--user root`.
- **`module!` needs a comma after every field**, including the last one.
- **Allocation can fail in the kernel**: collections are `KVec`/`KBox`, and every allocation takes flags such as `GFP_KERNEL` and returns a `Result`. See `samples/rust/rust_minimal.rs`.
- **API reference**: the samples in `samples/rust/` of your kernel sources match your exact kernel version, unlike online examples.
