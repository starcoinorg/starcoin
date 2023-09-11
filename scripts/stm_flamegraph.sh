#!/bin/bash

STARCOIN_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && cd .. && pwd)"

if [[ "$(uname)" != "Linux" ]]; then
  echo "run flamegraph only in linux. exit"
fi

cmd="cargo run -p starcoin-transaction-benchmarks  --features fuzzing -- -n 2"
echo "run tps with cmd: ${cmd}"
eval "$cmd"

cmd="cargo run -p starcoin-transaction-benchmarks  --features fuzzing -- -n 4"
echo "run tps with cmd: ${cmd}"
eval "$cmd"

cmd="cargo run -p starcoin-transaction-benchmarks  --features fuzzing -- -n 6"
echo "run tps with cmd: ${cmd}"
eval "$cmd"

cmd="cargo run -p starcoin-transaction-benchmarks  --features fuzzing -- -n 8"
echo "run tps with cmd: ${cmd}"
eval "$cmd"

cmd="cargo run -p starcoin-transaction-benchmarks  --features fuzzing -- -n 10"
echo "run tps with cmd: ${cmd}"
eval "$cmd"

cmd="cargo run -p starcoin-transaction-benchmarks  --features fuzzing -- -n 12"
echo "run tps with cmd: ${cmd}"
eval "$cmd"

cmd="cargo run -p starcoin-transaction-benchmarks  --features fuzzing -- -n 14"
echo "run tps with cmd: ${cmd}"
eval "$cmd"

cmd="cargo run -p starcoin-transaction-benchmarks  --features fuzzing -- -n 16"
echo "run tps with cmd: ${cmd}"
eval "$cmd"

cmd="cargo run -p starcoin-transaction-benchmarks  --features fuzzing -- -n 18"
echo "run tps with cmd: ${cmd}"
eval "$cmd"

cmd="cargo run -p starcoin-transaction-benchmarks  --features fuzzing -- -n 20"
echo "run tps with cmd: ${cmd}"
eval "$cmd"

cmd="cargo run -p starcoin-transaction-benchmarks  --features fuzzing -- -n 22"
echo "run tps with cmd: ${cmd}"
eval "$cmd"

cmd="cargo run -p starcoin-transaction-benchmarks  --features fuzzing -- -n 24"
echo "run tps with cmd: ${cmd}"
eval "$cmd"

cmd="cargo bench -p starcoin-transaction-benchmarks --bench transaction_benches --features fuzzing,flamegraph -- --profile-time=20"
echo "run flamegraph with cmd: ${cmd}"
eval "$cmd"
