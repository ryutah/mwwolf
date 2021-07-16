#!/usr/bin/env bash
set -eu
cd $1
cargo clippy --all-features -- -D clippy::all -D warnings --no-deps
cargo fmt -- --check
