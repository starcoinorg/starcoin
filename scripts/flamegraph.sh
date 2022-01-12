#!/bin/bash

STARCOIN_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && cd .. && pwd)"

PR_NUMBER=$1

if [[ "$(uname)" != "Linux" ]]; then
  echo "run flamegraph only in linux. exit"
fi

echo "PR_NUMBER $PR_NUMBER"

cmd="RUST_LOG=error cargo bench --bench bench_storage -- --profile-time=10"
echo "run flamegraph with cmd: ${cmd}"
eval "$cmd"

cmd="RUST_LOG=error cargo bench --bench bench_chain -- --profile-time=10"
echo "run flamegraph with cmd: ${cmd}"
eval "$cmd"

cmd="RUST_LOG=error cargo bench --bench bench_state_tree -- --profile-time=10"
echo "run flamegraph with cmd: ${cmd}"
eval "$cmd"

cmd="RUST_LOG=error cargo bench --bench bench_vm -- --profile-time=10"
echo "run flamegraph with cmd: ${cmd}"
eval "$cmd"

cmd="RUST_LOG=error cargo build --release --bin=starcoin_db_exporter"
echo "run build with cmd: ${cmd}"
eval "$cmd"


aws s3api put-object --bucket flamegraph.starcoin.org --key "$PR_NUMBER"/"accumulator_append.svg" --body $STARCOIN_DIR/target/criterion/accumulator_append/profile/flamegraph.svg
aws s3api put-object --bucket flamegraph.starcoin.org --key "$PR_NUMBER"/"storage_transaction.svg" --body $STARCOIN_DIR/target/criterion/storage_transaction/profile/flamegraph.svg

aws s3api put-object --bucket flamegraph.starcoin.org --key "$PR_NUMBER"/"query_block_in10_times100.svg" --body $STARCOIN_DIR/target/criterion/query_block/"query_block_in(10)_times(100)"/profile/flamegraph.svg
aws s3api put-object --bucket flamegraph.starcoin.org --key "$PR_NUMBER"/"query_block_in10_times1000.svg" --body $STARCOIN_DIR/target/criterion/query_block/"query_block_in(10)_times(1000)"/profile/flamegraph.svg
aws s3api put-object --bucket flamegraph.starcoin.org --key "$PR_NUMBER"/"query_block_in10_times10000.svg" --body $STARCOIN_DIR/target/criterion/query_block/"query_block_in(10)_times(10000)"/profile/flamegraph.svg

aws s3api put-object --bucket flamegraph.starcoin.org --key "$PR_NUMBER"/"query_block_in1000_times100.svg" --body $STARCOIN_DIR/target/criterion/query_block/"query_block_in(1000)_times(100)"/profile/flamegraph.svg
aws s3api put-object --bucket flamegraph.starcoin.org --key "$PR_NUMBER"/"query_block_in1000_times1000.svg" --body $STARCOIN_DIR/target/criterion/query_block/"query_block_in(1000)_times(1000)"/profile/flamegraph.svg
aws s3api put-object --bucket flamegraph.starcoin.org --key "$PR_NUMBER"/"query_block_in1000_times10000.svg" --body $STARCOIN_DIR/target/criterion/query_block/"query_block_in(1000)_times(10000)"/profile/flamegraph.svg

aws s3api put-object --bucket flamegraph.starcoin.org --key "$PR_NUMBER"/"get_with_proof_mem_store.svg"  --body $STARCOIN_DIR/target/criterion/get_with_proof/mem_store/profile/flamegraph.svg
aws s3api put-object --bucket flamegraph.starcoin.org --key "$PR_NUMBER"/"get_with_proof_db_store.svg"  --body $STARCOIN_DIR/target/criterion/get_with_proof/db_store/profile/flamegraph.svg

aws s3api put-object --bucket flamegraph.starcoin.org --key "$PR_NUMBER"/"put_and_commit_db_store_1.svg"  --body $STARCOIN_DIR/target/criterion/put_and_commit/db_store/1/profile/flamegraph.svg
aws s3api put-object --bucket flamegraph.starcoin.org --key "$PR_NUMBER"/"put_and_commit_db_store_10.svg"  --body $STARCOIN_DIR/target/criterion/put_and_commit/db_store/10/profile/flamegraph.svg
aws s3api put-object --bucket flamegraph.starcoin.org --key "$PR_NUMBER"/"put_and_commit_db_store_100.svg"  --body $STARCOIN_DIR/target/criterion/put_and_commit/db_store/100/profile/flamegraph.svg
aws s3api put-object --bucket flamegraph.starcoin.org --key "$PR_NUMBER"/"put_and_commit_db_store_5.svg"  --body $STARCOIN_DIR/target/criterion/put_and_commit/db_store/5/profile/flamegraph.svg
aws s3api put-object --bucket flamegraph.starcoin.org --key "$PR_NUMBER"/"put_and_commit_db_store_50.svg"  --body $STARCOIN_DIR/target/criterion/put_and_commit/db_store/50/profile/flamegraph.svg

aws s3api put-object --bucket flamegraph.starcoin.org --key "$PR_NUMBER"/"put_and_commit_mem_store_1.svg"  --body $STARCOIN_DIR/target/criterion/put_and_commit/mem_store/1/profile/flamegraph.svg
aws s3api put-object --bucket flamegraph.starcoin.org --key "$PR_NUMBER"/"put_and_commit_mem_store_10.svg"  --body $STARCOIN_DIR/target/criterion/put_and_commit/mem_store/10/profile/flamegraph.svg
aws s3api put-object --bucket flamegraph.starcoin.org --key "$PR_NUMBER"/"put_and_commit_mem_store_100.svg"  --body $STARCOIN_DIR/target/criterion/put_and_commit/mem_store/100/profile/flamegraph.svg
aws s3api put-object --bucket flamegraph.starcoin.org --key "$PR_NUMBER"/"put_and_commit_mem_store_5.svg"  --body $STARCOIN_DIR/target/criterion/put_and_commit/mem_store/5/profile/flamegraph.svg
aws s3api put-object --bucket flamegraph.starcoin.org --key "$PR_NUMBER"/"put_and_commit_mem_store_50.svg"  --body $STARCOIN_DIR/target/criterion/put_and_commit/mem_store/50/profile/flamegraph.svg

aws s3api put-object --bucket flamegraph.starcoin.org --key "$PR_NUMBER"/"transaction_execution_1.svg"  --body $STARCOIN_DIR/target/criterion/vm/transaction_execution/1/profile/flamegraph.svg
aws s3api put-object --bucket flamegraph.starcoin.org --key "$PR_NUMBER"/"transaction_execution_10.svg"  --body $STARCOIN_DIR/target/criterion/vm/transaction_execution/10/profile/flamegraph.svg
aws s3api put-object --bucket flamegraph.starcoin.org --key "$PR_NUMBER"/"transaction_execution_20.svg"  --body $STARCOIN_DIR/target/criterion/vm/transaction_execution/20/profile/flamegraph.svg
aws s3api put-object --bucket flamegraph.starcoin.org --key "$PR_NUMBER"/"transaction_execution_5.svg"  --body $STARCOIN_DIR/target/criterion/vm/transaction_execution/5/profile/flamegraph.svg
aws s3api put-object --bucket flamegraph.starcoin.org --key "$PR_NUMBER"/"transaction_execution_50.svg"  --body $STARCOIN_DIR/target/criterion/vm/transaction_execution/50/profile/flamegraph.svg

cd /tmp
rm -f block_1_10000.csv
wget https://s3.ap-northeast-1.amazonaws.com/main.starcoin.org/block_1_10000.csv
mkdir -p /tmp/$PR_NUMBER/main
rm -rf /tmp/$PR_NUMBER/main/*
$STARCOIN_DIR/target/release/starcoin_db_exporter apply-block -i /tmp/block_1_10000.csv -n main -o /tmp/$PR_NUMBER/main

aws s3api put-object --bucket flamegraph.starcoin.org --key "$PR_NUMBER"/"apply_block.svg" --body /tmp/flamegraph.svg

mkdir -p /tmp/$PR_NUMBER/halley
rm -f block_txn_1_202.csv
wget https://s3.ap-northeast-1.amazonaws.com/main.starcoin.org/block_txn_1_202.csv
rm -rf /tmp/$PR_NUMBER/halley/*
$STARCOIN_DIR/target/release/starcoin_db_exporter apply-block -i /tmp/block_txn_1_202.csv -n halley -o /tmp/$PR_NUMBER/halley
aws s3api put-object --bucket flamegraph.starcoin.org --key "$PR_NUMBER"/"apply_block_txn.svg" --body /tmp/flamegraph.svg
rm -rf /tmp/$PR_NUMBER/halley/*

rm -f block_empty_1_202.csv
wget https://s3.ap-northeast-1.amazonaws.com/main.starcoin.org/block_empty_1_202.csv
rm -rf /tmp/$PR_NUMBER/halley/*
$STARCOIN_DIR/target/release/starcoin_db_exporter apply-block -i /tmp/block_empty_1_202.csv -n halley -o /tmp/$PR_NUMBER/halley
aws s3api put-object --bucket flamegraph.starcoin.org --key "$PR_NUMBER"/"apply_block_empty.svg" --body /tmp/flamegraph.svg
rm -rf /tmp/$PR_NUMBER/halley/*

rm -f block_fixed_1_60.csv
wget https://s3.ap-northeast-1.amazonaws.com/main.starcoin.org/block_fixed_1_60.csv
rm -rf /tmp/$PR_NUMBER/halley/*
$STARCOIN_DIR/target/release/starcoin_db_exporter apply-block -i /tmp/block_fixed_1_60.csv -n halley -o /tmp/$PR_NUMBER/halley
aws s3api put-object --bucket flamegraph.starcoin.org --key "$PR_NUMBER"/"apply_block_fixed.svg" --body /tmp/flamegraph.svg
rm -rf /tmp/$PR_NUMBER/halley/*

rm -rf /tmp/$PR_NUMBER/