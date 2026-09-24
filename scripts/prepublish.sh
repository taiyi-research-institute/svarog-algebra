#!/usr/bin/env bash
set -euo pipefail
source "$(dirname -- "${BASH_SOURCE[0]}")/release-common.sh"
select_packages "$@"

cargo test --workspace --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --all-features --no-deps --locked
cargo package "${release_selection[@]}" --locked
cargo publish "${release_selection[@]}" --dry-run --locked
