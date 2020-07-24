#! /bin/bash

TRACE_PATH=$HOME/trace

[ ! -e  "$TRACE_PATH" ] || rm -f "$TRACE_PATH"

export MOVE_VM_TRACE=$TRACE_PATH

echo "Rebuilding stdlib..."
pushd ../stdlib || exit 1
cargo run
popd || exit 1

echo "Running executor testsuite..."
pushd ../../executor || exit 1
cargo test
popd || exit 1

echo "Running functional testsuite..."
pushd ../functional-tests || exit 1
cargo test
popd || exit 1

echo "Converting trace file..."
cargo run --bin move-trace-conversion -- -f "$TRACE_PATH" -o trace.mvcov

echo "Producing coverage summaries..."
cargo run --bin coverage-summaries -- --summarize-functions -t trace.mvcov -s ../stdlib/staged/stdlib

echo "==========================================================================="
echo "You can check source coverage for a module by running:"
echo "> cargo run --bin source-coverage -- -t trace.mvcov -b ../move-lang/move_build_output/modules/<LOOK_FOR_MODULE_HERE>.mv -s ../../stdlib/modules/<SOURCE_MODULE>.move"
echo "---------------------------------------------------------------------------"
echo "You can can also getter a finer-grained coverage summary for each function by running:"
echo "> cargo run --bin coverage-summaries -- -t trace.mvcov -s ../stdlib/staged/stdlib"
echo "==========================================================================="

unset MOVE_VM_TRACE

echo "DONE"
