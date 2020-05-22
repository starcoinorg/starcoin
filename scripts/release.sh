#!/bin/bash
rm -rf artifacts/*
mkdir -p artifacts/
#cp -v target/release/starcoin artifacts/starcoin
#cp -v target/release/starcoin_miner artifacts/starcoin_miner
#cp -v target/release/faucet artifacts/faucet
tar -czvf starcoin-$1.tar.gz  artifacts