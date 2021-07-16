#!/usr/bin/env bash
set -eu
cd $(dirname $0)/../..
find -name '*rs' -exec touch {} \;

procs=()
scripts/lint/lint_rust_partial.sh backend &
procs=("${procs[@]}" $!)
scripts/lint/lint_rust_partial.sh frontend &
procs=("${procs[@]}" $!)

for proc in ${procs[@]};
do
		wait $proc
done
