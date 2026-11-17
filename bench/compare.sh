#!/usr/bin/env bash
# Builds the bench twice with plain `cargo build-sbpf`: first with the builtins it links by default,
# then with this branch's crate (`--features pr`). Then prints the compute units of both builds.
set -euo pipefail
cd "$(dirname "$0")"
unset RUSTFLAGS CARGO_ENCODED_RUSTFLAGS CARGO_TARGET_BPFEL_UNKNOWN_NONE_RUSTFLAGS
cargo build-sbpf -V
cargo build-sbpf
cp target/deploy/bench.so target/bench-default.so
cargo build-sbpf -- --features pr
cp target/deploy/bench.so target/bench-pr.so
RUST_LOG=error cargo +nightly test --test compare -- --nocapture
