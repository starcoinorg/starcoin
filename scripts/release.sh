#!/bin/bash
rm -rf starcoin-artifacts/*
mkdir -p starcoin-artifacts/
cp -v target/release/starcoin starcoin-artifacts/
cp -v target/release/starcoin_miner starcoin-artifacts/
cp -v target/release/starcoin_faucet starcoin-artifacts/
cp -v target/release/starcoin_txfactory starcoin-artifacts/
cp -v target/release/starcoin_generator starcoin-artifacts/
cp -v target/release/starcoin_indexer starcoin-artifacts/
if [ "$1" == "windows-latest" ]; then
  7z a -r starcoin-$1.zip starcoin-artifacts
else
  zip -r starcoin-$1.zip starcoin-artifacts
fi
