#!/usr/bin/env bash
# Wait until a test VM answers over SSH (max ~3 min)
# Usage: QG_WORK_DIR=<dir> wait-ssh.sh NAME
set -euo pipefail

name="${1:?usage: wait-ssh.sh NAME}"
dir="$(dirname "$0")"

for _ in $(seq 36); do
    if "$dir/ssh.sh" "$name" true 2>/dev/null; then
        echo "qg-$name is up"
        exit 0
    fi
    sleep 5
done

echo "qg-$name did not answer over SSH" >&2
exit 1
