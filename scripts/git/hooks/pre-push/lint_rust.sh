#!/usr/bin/env bash
set -eu
cd $(dirname $0)/../../../..

git diff --name-only $GIT_HOOKS_SHA_RANGE | grep '.*\.rs' | xargs -I target_file touch target_file

scripts/lint/lint_rust.sh
