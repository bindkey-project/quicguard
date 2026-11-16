# qg-userland-v0

QuicGuard v0: a userspace MASQUE CONNECT-IP VPN built on `quinn`.

This file is the work specification for v0. It says **what** must be built and **why**;
the implementation choices are up to the v0 team.

## Why v0 exists

v0 is not the final product: the final data path is the kernel module (v1). v0 has two jobs:

1. **Interop peer.** The kernel module will be tested against v0. `quinn` follows the RFCs,
   so if `kernel client ↔ v0 server` works, the module really speaks QUIC and MASQUE.
   Testing the module only against itself would hide bugs made the same way on both sides.
2. **Performance baseline.** Benchmarks compare WireGuard, v0 and v1 on the same machine.
   v0 shows what MASQUE costs in userspace, so we can measure what the kernel brings.

So v0 must be **standard-compliant first**, simple second. Speed is not a goal.

## Scope

| In v0 | Out of v0 |
|---|---|
| Client and server binaries | Key management (enrollment, rotation, revocation) |
| QUIC + HTTP/3 with `quinn` and `h3` | Stealth (decoy website, TLS fingerprint) |
| `CONNECT-IP` request and response | Performance tuning |
| `ADDRESS_ASSIGN` capsule (+ `ROUTE_ADVERTISEMENT` if time allows) | HTTP/2 fallback over TCP |
| IP packets in QUIC DATAGRAM frames, TUN interface on both sides | IPv6 (IPv4 is enough for v0) |
| Server certificate check by the client; mTLS as a bonus | |

## Protocol requirements

References: [RFC 9484](https://www.rfc-editor.org/rfc/rfc9484.html) (CONNECT-IP),
[RFC 9297](https://www.rfc-editor.org/rfc/rfc9297.html) (HTTP Datagrams and capsules),
[RFC 9220](https://www.rfc-editor.org/rfc/rfc9220.html) (Extended CONNECT over HTTP/3),
[RFC 9221](https://www.rfc-editor.org/rfc/rfc9221.html) (QUIC DATAGRAM).

### QUIC and HTTP/3 setup

- ALPN: `h3`.
- QUIC DATAGRAM support enabled on both sides (transport parameter `max_datagram_frame_size`;
  in `quinn`, the datagram buffer settings of `TransportConfig`).
- HTTP/3 settings: `SETTINGS_H3_DATAGRAM = 1` on both sides,
  `SETTINGS_ENABLE_CONNECT_PROTOCOL = 1` on the server.

### The request (client → server, on a bidirectional stream)

```
:method    = CONNECT
:protocol  = connect-ip
:scheme    = https
:authority = <server name>
:path      = /.well-known/masque/ip/*/*/
capsule-protocol = ?1
```

The server answers with a `2xx` status to accept, or an error status (for example `403`) to refuse.
After a `2xx`, the request stream stays open for the whole session: closing it closes the tunnel.

### Capsules (on the request stream)

The request stream body is a sequence of capsules (RFC 9297 §3.2), carried in HTTP/3 DATA frames:

```
Capsule {
  Type   (varint),
  Length (varint),
  Value  (Length bytes),
}
```

`ADDRESS_ASSIGN` (type `0x01`, RFC 9484 §4.7.1), sent by the server after the `2xx`, contains a list of:

```
Assigned Address {
  Request ID        (varint),   // 0 when not answering an ADDRESS_REQUEST
  IP Version        (8),        // 4 or 6
  IP Address        (32 or 128),
  IP Prefix Length  (8),
}
```

The client configures its TUN interface with this address. Unknown capsule types must be ignored, not treated as errors.

### Datagrams (the tunnelled packets)

Each IP packet travels in one QUIC DATAGRAM frame, whose payload is:

```
Quarter Stream ID (varint),   // request stream ID / 4
Context ID        (varint),   // 0 = the payload is a full IP packet
IP packet         (..),
```

- Only Context ID `0` is used in v0. Datagrams with another Context ID are dropped silently.
- A packet larger than the maximum datagram size of the connection is dropped, not fragmented.
  Set the TUN MTU so that this is rare (the connection exposes its current maximum datagram size).

## Shared code: `qg-wire`

The encodings above (varints, capsule header, `ADDRESS_ASSIGN`, the datagram header)
must be written in **`crates/qg-wire`**, not in this crate.

The kernel module will reuse `qg-wire`, so the rules there are strict:

- `#![no_std]`, **no dependencies**, no `alloc`: functions read from `&[u8]` and write into `&mut [u8]`.
- Errors are plain enums, not `anyhow`.
- Unit tests for every encoder and decoder, including the examples from the RFCs.

Any change to `qg-wire` must be reviewed by both the v0 and v1 teams,
because both versions must encode exactly the same bytes.

## Expected command line

Indicative, adjust if needed:

```bash
qg-userland-v0 server --listen 0.0.0.0:4433 --cert server.der --key server.key --pool 10.0.0.0/24
qg-userland-v0 client --server 10.99.0.2:4433 --name localhost --ca server.der
```

Both sides create a TUN interface, so they must run as root (or with `CAP_NET_ADMIN`).

## Development tips

- Test with two network namespaces linked by a veth pair rather than two VMs: it is lighter
  and isolates the routes of the two TUN interfaces.
- Support `SSLKEYLOGFILE` (rustls `KeyLogFile`) so Wireshark can decrypt the traffic:
  it is the fastest way to check the `CONNECT` request, the capsules and the datagram format.
- Follow the workspace conventions: `anyhow` in binaries, typed errors in libraries,
  no `unwrap` (Clippy warns about it).

## Milestones and acceptance tests

1. **QUIC datagram tunnel without HTTP/3**: TUN on both sides, IP packets in raw QUIC datagrams.
   ✅ `ping` from the client namespace to the server's tunnel address works.
2. **HTTP/3 + CONNECT-IP**: same tunnel, now opened by the `CONNECT-IP` request,
   with the datagram format above and the address given by `ADDRESS_ASSIGN`.
   ✅ `ping` works, and Wireshark (with the key log) shows the request, the `2xx`, the capsule and the datagrams.
3. **Throughput test**: ✅ `iperf3` runs through the tunnel, and the numbers are written down as the first v0 baseline.
4. **Interop** (with the v1 team, later): ✅ the v1 client reaches the v0 server, and the reverse.
