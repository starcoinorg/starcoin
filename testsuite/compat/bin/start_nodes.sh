#!/bin/sh

# first get tag release package, and unzip to assign dir
tag1=$1
tag2=$2
mkdir target1
mkdir target2

echo "get  node1..."
pushd ./target1 || exit 1

wget "https://github.com/starcoinorg/starcoin/releases/download/$tag1/starcoin-macos-latest.zip"
unzip starcoin-macos-latest.zip

# second start up node on DEV mode
./starcoin-artifacts/starcoin -n dev  -d ./dev &

popd || exit 1


echo "get  node2..."
pushd ./target2 || exit 1

wget "https://github.com/starcoinorg/starcoin/releases/download/$tag2/starcoin-macos-latest.zip"
unzip starcoin-macos-latest.zip

# second start up node on DEV mode
./starcoin-artifacts/starcoin -n dev  -d ./dev &

popd || exit 1

echo "start compat nodes ok!"
