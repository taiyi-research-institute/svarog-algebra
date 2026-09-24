#!/usr/bin/env bash
# Sourced by the release entry points.
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.."

select_packages() {
    release_selection=(--workspace)
    if (( $# )); then
        release_selection=()
        for package in "$@"; do
            case "$package" in
                curve-abstract|svarog-secp256k1-sys|svarog-curve25519|svarog-secp256k1|svarog-lagrange)
                    release_selection+=(-p "$package") ;;
                *) printf 'Unknown release package: %s\n' "$package" >&2; return 2 ;;
            esac
        done
    fi
}
