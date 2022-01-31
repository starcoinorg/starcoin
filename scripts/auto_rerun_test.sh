#!/bin/bash

set -eo pipefail

STARCOIN_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && cd .. && pwd)"

TEST_RESULT_FILE="$STARCOIN_DIR/target/debug/test_result.txt"
TEST_RESULT_FAILED_FILE="$STARCOIN_DIR/target/debug/test_result_failed.txt"

export RUSTFLAGS='-Ccodegen-units=1 -Copt-level=0'
export RUSTC_BOOTSTRAP=1
export RUST_MIN_STACK=8388608 # 8 * 1024 * 1024

echo check ulimits
ulimit -a

#pleanse ensure tow test command's argument is same.
RUST_LOG=OFF RUST_BACKTRACE=0 cargo xtest --exclude starcoin-move-prover -j 15 -- --test-threads=10 --color never --format pretty |tee "$TEST_RESULT_FILE" ||true
grep -e '^test[[:space:]][^[:space:]]*[[:space:]]\.\.\.[[:space:]]FAILED' "$TEST_RESULT_FILE" >"$TEST_RESULT_FAILED_FILE" ||true

status=0
IFS=' '
while read -r -a testcase; do
  case_name=${testcase[1]}
  echo "rerun failed test $case_name"
  RUST_LOG=DEBUG RUST_BACKTRACE=full cargo xtest -j 15 "$case_name" -- --test-threads=1 --nocapture
  case_status=$?
  if [ $case_status -ne 0 ]; then
    status=$case_status
  fi
done < "$TEST_RESULT_FAILED_FILE"

exit $status
