#!/bin/sh

# first get tag release package, and unzip to assign dir
tag1=$1
tag2=$2
rm -rf ./testsuite/compat/target1
mkdir ./testsuite/compat/target1


echo " build node1 ..."
#compile node1
git checkout $tag1
cargo build --target-dir ./testsuite/compat/target1

# second start up node on DEV mode
./testsuite/compat/target1/debug/starcoin -n dev  -d ./testsuite/compat/target1/dev &
echo "node1 startup ok!"

#checkout tag2
export STARCOIN_WS=ws://127.0.0.1:9870
git checkout $tag2
cargo test --test integration -- --nocapture -e compat_remote



