#!/bin/bash
rm -rf starcoin-artifacts/*
mkdir -p starcoin-artifacts/
cp -v target/release/starcoin starcoin-artifacts/starcoin
cp -v target/release/starcoin_miner starcoin-artifacts/starcoin_miner
cp -v target/release/faucet starcoin-artifacts/faucet

if [ "$1" == "windows-latest" ]; then
  7z a -r starcoin-$1.zip starcoin-artifacts
else
  zip -r starcoin-$1.zip starcoin-artifacts
fi