#!/usr/bin/env bash
# This scripts runs various CI-like checks in a convenient way.
set -eux

cargo check --quiet --workspace --all-targets
cargo fmt --all -- --check
cargo clippy --quiet --workspace --all-targets --all-features --  -D warnings -W clippy::all

# cargo install --locked cargo-deny --version 0.19.0
command -v cargo-deny >/dev/null 2>&1 || { echo "error: cargo-deny not found; install with: cargo install --locked cargo-deny --version 0.19.0"; exit 1; }
cargo deny check -d

# cargo install typos-cli
typos

# cargo test
cargo test --quiet --workspace --all-targets --all-features
cargo test --quiet --workspace --doc
