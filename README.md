# QuicGuard

A VPN built on QUIC and MASQUE, written in Rust, with its data path in a Linux kernel module.

> **Status: early development.** Nothing here is usable yet. The goals below describe
> where the project is heading, not what it does today.

## Why

WireGuard is small, fast and simple, but it leaves three problems open:

| Problem | WireGuard | QuicGuard |
|---|---|---|
| **Easy to detect and block** | Fixed message formats that a DPI can match with a single rule | Traffic looks like ordinary HTTP/3 on UDP 443 |
| **No key management** | Static keys written by hand: no enrollment, expiry or revocation | TLS 1.3 with certificates, so peers can be enrolled, rotated and revoked |
| **Fixed cryptography** | ChaCha20-Poly1305 only: not FIPS-approved, no native post-quantum option | TLS 1.3 cipher suites, including AES-GCM with hardware acceleration |

Existing MASQUE VPNs run in userspace. QuicGuard puts its data path in the kernel,
like WireGuard, and aims to **beat WireGuard's throughput thanks to AES hardware acceleration**.

## How it works

QuicGuard tunnels IP packets with **MASQUE CONNECT-IP** ([RFC 9484](https://www.rfc-editor.org/rfc/rfc9484.html)):

1. The client opens a QUIC connection (TLS 1.3, ALPN `h3`) to the server.
2. It sends an HTTP/3 `CONNECT-IP` request on a stream.
3. The server answers `200` and assigns the client an address with an `ADDRESS_ASSIGN` capsule.
4. Each IP packet then travels in an unreliable QUIC **DATAGRAM** frame ([RFC 9221](https://www.rfc-editor.org/rfc/rfc9221.html)).

On the wire, a tunnelled packet looks like this:

```
IP │ UDP 443 │ QUIC │ DATAGRAM │ inner IP packet (TCP, UDP, ICMP…)
└── visible ──┘     └──────────── encrypted ────────────┘
```

### Userspace and kernel split

The handshake and HTTP/3 are complex but rare, so they stay in userspace. The per-packet work moves to the kernel:

| Userspace daemon | Kernel module |
|---|---|
| TLS 1.3 handshake, certificates | QUIC 1-RTT packet protection (AEAD + header protection) |
| HTTP/3 and the `CONNECT-IP` request | Packet numbers, replay protection, ACKs |
| Capsules (address and route assignment) | Loss detection and congestion control |
| Connection IDs, migration, key updates | DATAGRAM frames ↔ virtual network interface |

The daemon hands the 1-RTT keys to the kernel over **netlink** once the handshake is done,
the same way kTLS works for TLS over TCP. Frames the kernel does not handle are passed back to the daemon.

## Versions

| Version | What | Purpose |
|---|---|---|
| **v0** | Userspace VPN built on `quinn` + `h3` + a TUN interface | Interop peer to validate the kernel module, and a performance baseline |
| **v1** | Kernel data path + userspace daemon | The actual QuicGuard |

Benchmarks compare three setups on the same hardware: WireGuard, QuicGuard v0 and QuicGuard v1.

## Repository layout

```
quicguard/
├── crates/
│   ├── qg-wire/          # core-only: varints, frames, capsules, netlink messages (shared contract)
│   ├── qg-engine/        # core-only: QUIC 1-RTT packet engine, reused by the kernel module
│   ├── qg-userland-v0/   # v0: quinn-based client and server
│   └── qg-daemon/        # v1: userspace daemon driving the kernel module
├── kernel/               # v1 kernel module (Kbuild, outside the Cargo workspace)
├── tests/                # end-to-end scenarios, interop, benchmarks
├── lab/                  # test lab as code
└── docs/                 # architecture and design decisions
```

`qg-wire` and `qg-engine` are `#![no_std]` and must stay free of dependencies, so the kernel module can reuse them.

## Getting started

Requirements: Linux, and [rustup](https://rustup.rs). The toolchain version is pinned in `rust-toolchain.toml`.

```bash
rustup toolchain install          # installs the pinned toolchain
cargo build --workspace
cargo test --workspace
cargo clippy --workspace
cargo fmt --check
```

The kernel module must only be loaded inside a VM, never on a host machine.

## Test lab

Development runs on one Intel Mac with a hypervisor and two VMs (client and server).
Physical benchmarks on two machines linked by a direct cable come later, once the architecture is stable.

## References

- QUIC: [RFC 9000](https://www.rfc-editor.org/rfc/rfc9000.html), [RFC 9001](https://www.rfc-editor.org/rfc/rfc9001.html), [RFC 9002](https://www.rfc-editor.org/rfc/rfc9002.html), [RFC 9221](https://www.rfc-editor.org/rfc/rfc9221.html)
- MASQUE: [RFC 9484](https://www.rfc-editor.org/rfc/rfc9484.html), [RFC 9297](https://www.rfc-editor.org/rfc/rfc9297.html), [RFC 9220](https://www.rfc-editor.org/rfc/rfc9220.html)
- HTTP/3: [RFC 9114](https://www.rfc-editor.org/rfc/rfc9114.html), [RFC 9204](https://www.rfc-editor.org/rfc/rfc9204.html)
- [WireGuard whitepaper](https://www.wireguard.com/papers/wireguard.pdf)
- [Rust for Linux documentation](https://docs.kernel.org/rust/)

## License

GPL-2.0-only.
