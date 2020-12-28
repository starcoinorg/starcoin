#!/bin/bash

STARCOIN_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && cd .. && pwd)"

if [ -z "$1" ]
  then
    target_dir="benchmarks/target/"
  else
    target_dir="$1"
fi
echo "benchmark use target_dir ${target_dir}"
cargo bench --bench bench_storage --bench bench_chain --bench bench_state_tree --bench bench_vm --target-dir "$target_dir"
