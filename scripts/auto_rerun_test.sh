#!/bin/bash

STARCOIN_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && cd .. && pwd)"

TEST_RESULT_FILE="$STARCOIN_DIR/target/debug/test_result.txt"
TEST_RESULT_FAILED_FILE="$STARCOIN_DIR/target/debug/test_result_failed.txt"

if [[ "$(whoami)" == "root" ]]; then
  BOOGIE_PATH="/root/.dotnet/tools/boogie"
else
  BOOGIE_PATH="/home/$(whoami)/.dotnet/tools/boogie"
fi
export BOOGIE_EXE=$BOOGIE_PATH;
export Z3_EXE=/usr/local/bin/z3;

export RUSTFLAGS='-Ccodegen-units=1 -Copt-level=0'
export RUSTC_BOOTSTRAP=1
export CARGO_INCREMENTAL=0
export RUST_MIN_STACK=8388608 # 8 * 1024 * 1024

echo check ulimits
ulimit -a

#pleanse ensure tow test command's argument is same.
cargo xtest --no-run --no-fail-fast -j 1 -- --color never --format pretty
RUST_LOG=OFF RUST_BACKTRACE=0 cargo xtest --no-fail-fast -j 1 -- --test-threads=2 --color never --format pretty |tee "$TEST_RESULT_FILE" ||true
grep -e '^test[[:space:]][^[:space:]]*[[:space:]]\.\.\.[[:space:]]FAILED' "$TEST_RESULT_FILE" >"$TEST_RESULT_FAILED_FILE" ||true

status=0
IFS=' '
while read -r -a testcase; do
  case_name=${testcase[1]}
  echo "rerun failed test $case_name"
  RUST_LOG=DEBUG RUST_BACKTRACE=full cargo xtest --no-fail-fast -j 1 "$case_name" -- --nocapture
  case_status=$?
  if [ $case_status -ne 0 ]; then
    status=$case_status
  fi
done < "$TEST_RESULT_FAILED_FILE"

exit $status
