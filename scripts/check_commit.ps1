$STARCOIN_DIR = (get-item $PSScriptRoot).parent.FullName

# cargo fmt check
cargo fmt -- --check
# cargo clippy check
cargo clippy --all-targets -- -D warnings
# generate stdlib
cargo run -p stdlib
# generate genesis
cargo run -p starcoin-genesis
# generate rpc schema document
cargo run -p starcoin-rpc-api -- -d ./rpc/api/generated_rpc_schema
# test config file
cargo test -p starcoin-config  test_example_config_compact
# check changed files
. "${STARCOIN_DIR}/scripts/changed_files.ps1"
