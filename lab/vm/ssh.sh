#!/usr/bin/env bash
# Run a command in a test VM over its admin NIC
# Usage: QG_WORK_DIR=<dir> ssh.sh NAME COMMAND...
set -euo pipefail

name="${1:?usage: ssh.sh NAME COMMAND...}"
shift
work="${QG_WORK_DIR:?QG_WORK_DIR is not set}"

# IPv4 lease given by the "default" network DHCP
ip="$(virsh -c qemu:///system domifaddr "qg-$name" \
    | awk '/ipv4/ { split($4, a, "/"); print a[1] }')"
[ -n "$ip" ] || { echo "qg-$name has no IP yet" >&2; exit 1; }

exec ssh -i "$work/id_ed25519" \
    -o StrictHostKeyChecking=no \
    -o UserKnownHostsFile=/dev/null \
    -o LogLevel=ERROR \
    -o ConnectTimeout=5 \
    "fedora@$ip" "$@"
