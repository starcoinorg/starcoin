#!/bin/bash
rm -rf starcoin-artifacts/*
mkdir -p starcoin-artifacts/
cp -v target/release/starcoin starcoin-artifacts/
cp -v target/release/starcoin_miner starcoin-artifacts/
cp -v target/release/starcoin_generator starcoin-artifacts/
cp -v target/release/mpm starcoin-artifacts/
cp -v target/release/starcoin_txfactory starcoin-artifacts/
cp -v target/release/starcoin_db_exporter starcoin-artifacts/
cp -v scripts/import_block.sh starcoin-artifacts/
cp -v scripts/verify_header.sh starcoin-artifacts/
cp -v README.md starcoin-artifacts/
if [ "$1" == "windows-latest" ]; then
  7z a -r starcoin-$1.zip starcoin-artifacts
else
  zip -r starcoin-$1.zip starcoin-artifacts
fi
