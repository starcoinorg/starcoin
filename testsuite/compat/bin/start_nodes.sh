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
# get seed_id from node1
sleep 2s
SEED=$(./starcoin-artifacts/starcoin --connect ws://127.0.0.1:9870 -ojson node info|grep self_address |awk '{print $2}'| tr -d '"')
echo "node1 startup ok!seed: $SEED"
popd || exit 1

sleep 5s

echo "get  node2..."
pushd ./target2 || exit 1

wget "https://github.com/starcoinorg/starcoin/releases/download/$tag2/starcoin-macos-latest.zip"
unzip starcoin-macos-latest.zip

# second start up node on DEV mode
if [ "$SEED" ]; then
  echo "start node2 by seed: $SEED"
  ./starcoin-artifacts/starcoin -n dev  -d ./dev --seed "$SEED" &
else
  ./starcoin-artifacts/starcoin -n dev  -d ./dev &
fi

echo "node2 startup ok!"

popd || exit 1

echo "start compat nodes ok!"
