#!/usr/bin/env bash
set -eu
cd $(dirname $0)/../../../..

scripts/lint/lint_rust.sh
