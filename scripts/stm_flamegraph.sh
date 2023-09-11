#!/bin/bash

STARCOIN_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && cd .. && pwd)"

if [[ "$(uname)" != "Linux" ]]; then
  echo "run flamegraph only in linux. exit"
fi

cmd="cargo run -p starcoin-transaction-benchmarks  --features fuzzing"
echo "run tps with cmd: ${cmd}"
eval "$cmd"

cmd="cargo bench -p starcoin-transaction-benchmarks --bench transaction_benches --features fuzzing,flamegraph -- --profile-time=20"
echo "run flamegraph with cmd: ${cmd}"
eval "$cmd"
