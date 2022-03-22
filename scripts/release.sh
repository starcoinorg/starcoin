#!/bin/bash
rm -rf starcoin-artifacts/*
mkdir -p starcoin-artifacts/
cp -v target/release/starcoin starcoin-artifacts/
cp -v target/release/starcoin_miner starcoin-artifacts/
cp -v target/release/starcoin_generator starcoin-artifacts/
cp -v target/release/mpm starcoin-artifacts/
cp -v target/release/starcoin_db_exporter starcoin-artifacts/
cp -v scripts/import_net_block.sh starcoin_artifacts/
cp -v scripts/import_snapshot.sh starcoin_artifacts/
if [ "$1" == "windows-latest" ]; then
  7z a -r starcoin-$1.zip starcoin-artifacts
else
  zip -r starcoin-$1.zip starcoin-artifacts
fi
