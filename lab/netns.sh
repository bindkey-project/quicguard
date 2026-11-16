#!/usr/bin/env bash
# Labo QuicGuard : deux network namespaces reliés par une paire veth.
#
#   [ qg-client ]                                [ qg-server ]
#    veth-cli 10.99.0.1/24  <==== veth ====>  veth-srv 10.99.0.2/24
#
# Usage : sudo ./lab/netns.sh up|down|status

set -euo pipefail

NS_CLI="qg-client"
NS_SRV="qg-server"
VETH_CLI="veth-cli"
VETH_SRV="veth-srv"
ADDR_CLI="10.99.0.1/24"
ADDR_SRV="10.99.0.2/24"

need_root() {
    if [[ $EUID -ne 0 ]]; then
        echo "erreur : lancer ce script en root (sudo)" >&2
        exit 1
    fi
}

ns_exists() {
    [[ -e "/run/netns/$1" ]]
}

down() {
    # Supprimer un namespace détruit les interfaces qu'il contient ;
    # détruire un bout d'une paire veth détruit aussi l'autre bout.
    for ns in "$NS_CLI" "$NS_SRV"; do
        if ns_exists "$ns"; then
            ip netns del "$ns"
        fi
    done
    # Reste éventuel d'un "up" interrompu : veth restée dans le namespace racine.
    ip link del "$VETH_CLI" 2>/dev/null || true
}

up() {
    down

    # 1. Les deux "machines"
    ip netns add "$NS_CLI"
    ip netns add "$NS_SRV"

    # 2. Le câble, puis un bout dans chaque machine
    ip link add "$VETH_CLI" type veth peer name "$VETH_SRV"
    ip link set "$VETH_CLI" netns "$NS_CLI"
    ip link set "$VETH_SRV" netns "$NS_SRV"

    # 3. Les adresses underlay
    ip -n "$NS_CLI" addr add "$ADDR_CLI" dev "$VETH_CLI"
    ip -n "$NS_SRV" addr add "$ADDR_SRV" dev "$VETH_SRV"

    # 4. Allumer les interfaces (loopback comprise)
    ip -n "$NS_CLI" link set lo up
    ip -n "$NS_SRV" link set lo up
    ip -n "$NS_CLI" link set "$VETH_CLI" up
    ip -n "$NS_SRV" link set "$VETH_SRV" up

    echo "labo prêt : $NS_CLI (${ADDR_CLI%/*}) <-> $NS_SRV (${ADDR_SRV%/*})"
}

status() {
    for ns in "$NS_CLI" "$NS_SRV"; do
        echo "== $ns =="
        if ns_exists "$ns"; then
            ip -n "$ns" -brief addr
            ip -n "$ns" route
        else
            echo "(absent)"
        fi
    done
}

need_root
case "${1:-}" in
    up)     up ;;
    down)   down; echo "labo supprimé" ;;
    status) status ;;
    *)      echo "usage : $0 up|down|status" >&2; exit 2 ;;
esac