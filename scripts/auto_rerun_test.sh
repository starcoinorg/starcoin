#!/bin/bash

STARCOIN_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && cd .. && pwd)"

set -e

TEST_RESULT_FILE="$STARCOIN_DIR/target/debug/test_result.txt"
TEST_RESULT_FAILED_FILE="$STARCOIN_DIR/target/debug/test_result_failed.txt"

RUST_LOG=OFF RUST_BACKTRACE=0 cargo test -q -- --color never --format pretty |tee "$TEST_RESULT_FILE" ||true
grep -e '^test[[:space:]][^[:space:]]*[[:space:]]\.\.\.[[:space:]]FAILED' "$TEST_RESULT_FILE" >"$TEST_RESULT_FAILED_FILE" ||true

IFS=' '
while read -r -a testcase; do
  case_name=${testcase[1]}
  echo "rerun failed test $case_name"
  RUST_LOG=DEBUG RUST_BACKTRACE=full cargo test "$case_name" -- --nocapture
  status=$?
done < "$TEST_RESULT_FAILED_FILE"

exit $status