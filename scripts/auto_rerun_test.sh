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

RUST_LOG=OFF RUST_BACKTRACE=0 cargo test -q --no-fail-fast -- --color never --format pretty |tee "$TEST_RESULT_FILE" ||true
grep -e '^test[[:space:]][^[:space:]]*[[:space:]]\.\.\.[[:space:]]FAILED' "$TEST_RESULT_FILE" >"$TEST_RESULT_FAILED_FILE" ||true

status=0
IFS=' '
while read -r -a testcase; do
  case_name=${testcase[1]}
  echo "rerun failed test $case_name"
  RUST_LOG=DEBUG RUST_BACKTRACE=full cargo test "$case_name" -- --nocapture
  case_status=$?
  if [ $case_status -ne 0 ]; then
    status=$case_status
  fi
done < "$TEST_RESULT_FAILED_FILE"

exit $status
