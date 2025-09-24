#!/bin/bash

SCRIPT_PATH="$( cd "$( dirname "$0" )" >/dev/null 2>&1 && pwd )"
cd "$SCRIPT_PATH/.." || exit

cargo build -p move-package-manager2
MPM=./target/debug/mpm2

# Unit test
$MPM package test -p ./vm2/framework/move-stdlib -t 8 -i 400000000|| exit
$MPM package test -p ./vm2/framework/starcoin-stdlib -t 8 -i 400000000 || exit
$MPM package test -p ./vm2/framework/starcoin-token -t 8 -i 400000000 || exit
$MPM package test -p ./vm2/framework/starcoin-token-objects -t 8 -i 400000000 || exit
$MPM package test -p ./vm2/framework/starcoin-framework  -t 8 -i 400000000 || exit

# Integration test
export RUST_TEST_THREADS=32
$MPM integration-test -p ./vm2/framework/starcoin-framework || exit