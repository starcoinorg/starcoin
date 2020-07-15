#! /bin/bash

TRACE_PATH=$HOME/trace

[ ! -e  "$TRACE_PATH" ] || rm -f "$TRACE_PATH"

export MOVE_VM_TRACE=$TRACE_PATH

echo "Rebuilding stdlib..."
pushd ../stdlib || exit 1
cargo run
popd || exit 1

echo "Running functional testsuite..."
pushd ../functional-tests || exit 1
cargo test
popd || exit 1

echo "Converting trace file..."
cargo run --bin move-trace-conversion -- -f "$TRACE_PATH" -o trace.mvcov
echo "Producing coverage summaries..."
cargo run --bin coverage-summaries -- -t trace.mvcov -s ../stdlib/staged/stdlib.mv -o "$1"
cat ./baseline/coverage_report > "$2"

unset MOVE_VM_TRACE

echo "DONE"
