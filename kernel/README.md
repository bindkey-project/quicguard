# kernel

QuicGuard v1 kernel module (Rust for Linux).

- Built with Kbuild, **not** Cargo: this directory is outside the Cargo workspace.
- Contains only the kernel glue: virtual interface, kernel UDP socket, kernel crypto API, netlink.
- QUIC packet logic lives in `crates/qg-engine` and `crates/qg-wire` (core-only), reused here.
- Never load this module on a host machine: test it in a VM.
