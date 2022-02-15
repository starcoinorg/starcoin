#!/bin/bash

# Provide a script for fast check the commit.

set -eo pipefail

STARCOIN_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && cd .. && pwd)"

# cargo fmt check
cargo fmt -- --check
# cargo clippy check
cargo clippy --all-targets -- -D warnings
# generate stdlib
cargo run -p stdlib
# generate genesis
cargo run -p starcoin-genesis
# generate rpc schema document
cargo run -p starcoin-rpc-api -- -d ./rpc/generated_rpc_schema
# test config file
cargo test -p starcoin-config  test_example_config_compact
# check changed files
"${STARCOIN_DIR}"/scripts/changed-files.sh
