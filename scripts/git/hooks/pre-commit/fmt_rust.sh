#!/usr/bin/env bash
file_path=$1
cargo fmt -- $file_path
exit 0
