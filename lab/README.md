# lab

Test lab as code: VM images, network setup scripts, and later Ansible playbooks.

Current setup: one Intel Mac running a hypervisor with two VMs (client and server).
Physical tests on two Macs come later.

# Local lab: network namespaces (`netns.sh`)

For day-to-day development of `qg-userland-v0`, two network namespaces linked by a veth pair
replace the two VMs: lighter, and each side gets its own routes for its TUN interface.

```
[ qg-client ]                                [ qg-server ]
 veth-cli 10.99.0.1/24  <==== veth ====>  veth-srv 10.99.0.2/24
```

`10.99.0.0/24` is the underlay (carries QUIC/UDP). The tunnel (overlay) uses `10.0.0.0/24`.

```bash
sudo ./lab/netns.sh up       # create (or recreate) the lab
sudo ./lab/netns.sh status   # show addresses and routes
sudo ./lab/netns.sh down     # remove everything

# run any command inside a namespace
sudo ip netns exec qg-client ping -c 3 10.99.0.2
sudo ip netns exec qg-server ./target/debug/qg-userland-v0 server --listen 10.99.0.2:4433 ...
```

Build with `cargo build` as a normal user, run the binary with `sudo` (never `sudo cargo`).
Namespaces do not survive a reboot.
