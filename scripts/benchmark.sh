#!/bin/bash

STARCOIN_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && cd .. && pwd)"

#cmd="RUST_LOG=error cargo bench --bench bench_storage --bench bench_chain --bench bench_state_tree --bench bench_vm"
# shellcheck disable=SC2236
#if [ -n "$1" ]
 # then
  #  target_dir="$1"
    # cargo bench criterion use env to detect target dir https://github.com/bheisler/criterion.rs/issues/192
   # export CARGO_TARGET_DIR="${target_dir}"
    #cmd="${cmd} --target-dir ${target_dir}"
#fi
#echo "run benchmark with cmd: ${cmd}"
#eval "$cmd"

if [[ "$(uname)" != "Linux" ]]; then
  echo "run flamegraph only in linux. exit"
fi

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

today=$(date +"%Y%m%d")
#today=""

aws s3api put-object --bucket flamegraph.starcoin.org --key "$today"_"accumulator_append" --body $STARCOIN_DIR/target/criterion/accumulator_append/profile/flamegraph.svg
aws s3api put-object --bucket flamegraph.starcoin.org --key "$today"_"storage_transaction" --body $STARCOIN_DIR/target/criterion/storage_transaction/profile/flamegraph.svg

aws s3api put-object --bucket flamegraph.starcoin.org --key "$today"_"query_block_in10_times100" --body $STARCOIN_DIR/target/criterion/query_block/"query_block_in(10)_times(100)"/profile/flamegraph.svg
aws s3api put-object --bucket flamegraph.starcoin.org --key "$today"_"query_block_in10_times1000" --body $STARCOIN_DIR/target/criterion/query_block/"query_block_in(10)_times(1000)"/profile/flamegraph.svg
aws s3api put-object --bucket flamegraph.starcoin.org --key "$today"_"query_block_in10_times10000" --body $STARCOIN_DIR/target/criterion/query_block/"query_block_in(10)_times(10000)"/profile/flamegraph.svg

aws s3api put-object --bucket flamegraph.starcoin.org --key "$today"_"query_block_in1000_times100" --body $STARCOIN_DIR/target/criterion/query_block/"query_block_in(1000)_times(100)"/profile/flamegraph.svg
aws s3api put-object --bucket flamegraph.starcoin.org --key "$today"_"query_block_in1000_times1000" --body $STARCOIN_DIR/target/criterion/query_block/"query_block_in(1000)_times(1000)"/profile/flamegraph.svg
aws s3api put-object --bucket flamegraph.starcoin.org --key "$today"_"query_block_in1000_times10000" --body $STARCOIN_DIR/target/criterion/query_block/"query_block_in(1000)_times(10000)"/profile/flamegraph.svg

aws s3api put-object --bucket flamegraph.starcoin.org --key "$today"_"get_with_proof_mem_store"  --body $STARCOIN_DIR/target/criterion/get_with_proof/mem_store/profile/flamegraph.svg
aws s3api put-object --bucket flamegraph.starcoin.org --key "$today"_"get_with_proof_db_store"  --body $STARCOIN_DIR/target/criterion/get_with_proof/db_store/profile/flamegraph.svg

aws s3api put-object --bucket flamegraph.starcoin.org --key "$today"_"put_and_commit_db_store_1"  --body $STARCOIN_DIR/target/criterion/put_and_commit/db_store/1/profile/flamegraph.svg
aws s3api put-object --bucket flamegraph.starcoin.org --key "$today"_"put_and_commit_db_store_10"  --body $STARCOIN_DIR/target/criterion/put_and_commit/db_store/10/profile/flamegraph.svg
aws s3api put-object --bucket flamegraph.starcoin.org --key "$today"_"put_and_commit_db_store_100"  --body $STARCOIN_DIR/target/criterion/put_and_commit/db_store/100/profile/flamegraph.svg
aws s3api put-object --bucket flamegraph.starcoin.org --key "$today"_"put_and_commit_db_store_5"  --body $STARCOIN_DIR/target/criterion/put_and_commit/db_store/5/profile/flamegraph.svg
aws s3api put-object --bucket flamegraph.starcoin.org --key "$today"_"put_and_commit_db_store_50"  --body $STARCOIN_DIR/target/criterion/put_and_commit/db_store/50/profile/flamegraph.svg

aws s3api put-object --bucket flamegraph.starcoin.org --key "$today"_"put_and_commit_mem_store_1"  --body $STARCOIN_DIR/target/criterion/put_and_commit/mem_store/1/profile/flamegraph.svg
aws s3api put-object --bucket flamegraph.starcoin.org --key "$today"_"put_and_commit_mem_store_10"  --body $STARCOIN_DIR/target/criterion/put_and_commit/mem_store/10/profile/flamegraph.svg
aws s3api put-object --bucket flamegraph.starcoin.org --key "$today"_"put_and_commit_mem_store_100"  --body $STARCOIN_DIR/target/criterion/put_and_commit/mem_store/100/profile/flamegraph.svg
aws s3api put-object --bucket flamegraph.starcoin.org --key "$today"_"put_and_commit_mem_store_5"  --body $STARCOIN_DIR/target/criterion/put_and_commit/mem_store/5/profile/flamegraph.svg
aws s3api put-object --bucket flamegraph.starcoin.org --key "$today"_"put_and_commit_mem_store_50"  --body $STARCOIN_DIR/target/criterion/put_and_commit/mem_store/50/profile/flamegraph.svg

aws s3api put-object --bucket flamegraph.starcoin.org --key "$today"_"transaction_execution_1"  --body $STARCOIN_DIR/target/criterion/vm/transaction_execution/1/profile/flamegraph.svg
aws s3api put-object --bucket flamegraph.starcoin.org --key "$today"_"transaction_execution_10"  --body $STARCOIN_DIR/target/criterion/vm/transaction_execution/10/profile/flamegraph.svg
aws s3api put-object --bucket flamegraph.starcoin.org --key "$today"_"transaction_execution_20"  --body $STARCOIN_DIR/target/criterion/vm/transaction_execution/20/profile/flamegraph.svg
aws s3api put-object --bucket flamegraph.starcoin.org --key "$today"_"transaction_execution_5"  --body $STARCOIN_DIR/target/criterion/vm/transaction_execution/5/profile/flamegraph.svg
aws s3api put-object --bucket flamegraph.starcoin.org --key "$today"_"transaction_execution_50"  --body $STARCOIN_DIR/target/criterion/vm/transaction_execution/50/profile/flamegraph.svg

cd /tmp
rm -f block_1_10000.csv
wget https://s3.ap-northeast-1.amazonaws.com/main.starcoin.org/block_1_10000.csv
mkdir -p /tmp/$today/main
rm -rf /tmp/$today/main/*
$STARCOIN_DIR/target/release/starcoin_db_exporter apply-block -i /tmp/block_1_10000.csv -n main -o /tmp/$today/main

aws s3api put-object --bucket flamegraph.starcoin.org --key "$today"_"apply_block" --body /tmp/flamegraph.svg