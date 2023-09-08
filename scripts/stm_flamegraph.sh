#!/bin/bash

STARCOIN_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && cd .. && pwd)"

if [[ "$(uname)" != "Linux" ]]; then
  echo "run flamegraph only in linux. exit"
fi


cmd="cargo bench -p starcoin-transaction-benchmarks --features fuzzing"
echo "run flamegraph with cmd: ${cmd}"
eval "$cmd"

