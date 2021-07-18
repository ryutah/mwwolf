#!/usr/bin/env bash

cd $(dirname $0)/../..

set -eu
workspace=$1


find ./$workspace -name '*rs' -exec touch {} \;
scripts/lint/lint_rust_partial.sh $workspace
