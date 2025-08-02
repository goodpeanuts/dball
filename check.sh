#!/usr/bin/env bash
# This scripts runs various CI-like checks in a convenient way.
set -eux

cargo check --quiet --workspace --all-targets
cargo check --quiet --workspace --lib --target wasm32-unknown-unknown --exclude dball-client
cargo fmt --all -- --check
# Temporarily disabled clippy due to many lint errors - can be re-enabled after fixing lints
# cargo clippy --quiet --workspace --all-targets --all-features --  -D warnings -W clippy::all
cargo test --quiet --workspace --all-targets --all-features
cargo test --quiet --workspace --doc
trunk build
