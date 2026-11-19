#!/usr/bin/env bash
# Destroy one test VM and its volumes. Safe to run twice.
# Usage: down.sh NAME
set -euo pipefail

POOL=qg-images

virsh_sys() {
    virsh -c qemu:///system "$@"
}

name="${1:?usage: down.sh NAME}"
dom="qg-$name"

# Power off (may already be off), then drop the definition
if virsh_sys dominfo "$dom" >/dev/null 2>&1; then
    virsh_sys destroy "$dom" >/dev/null 2>&1 || true
    virsh_sys undefine "$dom"
fi

# Delete the per-VM volumes, never the base image
for vol in "$dom.qcow2" "$dom-seed.iso"; do
    if virsh_sys vol-info "$vol" --pool "$POOL" >/dev/null 2>&1
    then
        virsh_sys vol-delete "$vol" --pool "$POOL"
    fi
done
