#!/bin/bash

STARCOIN_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && cd .. && pwd)"

cmd="RUST_LOG=error cargo bench --bench bench_storage --bench bench_chain --bench bench_state_tree --bench bench_vm"
# shellcheck disable=SC2236
if [ -n "$1" ]
  then
    target_dir="$1"
    # cargo bench criterion use env to detect target dir https://github.com/bheisler/criterion.rs/issues/192
    export CARGO_TARGET_DIR="${target_dir}"
    cmd="${cmd} --target-dir ${target_dir}"
fi
echo "run benchmark with cmd: ${cmd}"
eval "$cmd"
