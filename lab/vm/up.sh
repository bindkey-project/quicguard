#!/usr/bin/env bash
# Create one test VM: overlay disk and cloud-init files
# Usage: QG_WORK_DIR=<dir> up.sh NAME
set -euo pipefail

POOL=qg-images
BASE=Fedora-Cloud-Base-Generic-45-1.4.x86_64.qcow2
DISK_SIZE=10G

# Every virsh call talks to the system libvirt
virsh_sys() {
    virsh -c qemu:///system "$@"
}

name="${1:?usage: up.sh NAME}"

# Per-job work dir, set by the workflow
work="${QG_WORK_DIR:?QG_WORK_DIR is not set}"
vm_dir="$work/$name"
mkdir -p "$vm_dir"

# Overlay disk on top of the base image (copy-on-write)
virsh_sys vol-create-as "$POOL" "qg-$name.qcow2" "$DISK_SIZE" \
    --format qcow2 \
    --backing-vol "$BASE" --backing-vol-format qcow2

# cloud-init: VM identity
cat > "$vm_dir/meta-data" <<EOF
instance-id: qg-$name
local-hostname: qg-$name
EOF

# cloud-init: SSH access with the per-job key
pubkey="$(cat "$work/id_ed25519.pub")"

cat > "$vm_dir/user-data" <<EOF
#cloud-config
ssh_authorized_keys:
    - $pubkey
EOF

# cloud-init seed: an ISO labelled "cidata" (NoCloud)
seed="$vm_dir/seed.iso"
xorriso -as mkisofs -quiet -output "$seed" \
    -volid cidata -joliet -rock \
    "$vm_dir/user-data" "$vm_dir/meta-data"

# Upload it into the pool so that QEMU can read it
seed_vol="qg-$name-seed.iso"
virsh_sys vol-create-as "$POOL" "$seed_vol" \
    "$(stat -c %s "$seed")" --format raw
virsh_sys vol-upload "$seed_vol" "$seed" --pool "$POOL"

# Define and boot the VM (admin NIC on "default" for now)
virt-install --connect qemu:///system \
    --name "qg-$name" \
    --memory 1024 --vcpus 1 \
    --cpu host-passthrough \
    --osinfo linux2024 \
    --import \
    --disk vol="$POOL/qg-$name.qcow2",bus=virtio \
    --disk vol="$POOL/$seed_vol",device=cdrom \
    --network network=default,model=virtio \
    --graphics none \
    --noautoconsole
