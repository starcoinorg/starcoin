set -eo pipefail

# STARCOIN_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && cd .. && pwd)"

# TEST_RESULT_FILE="$STARCOIN_DIR/target/debug/test_result.txt"
# TEST_RESULT_FAILED_FILE="$STARCOIN_DIR/target/debug/test_result_failed.txt"

# export RUSTFLAGS='-Ccodegen-units=1 -Copt-level=0'
# export RUSTC_BOOTSTRAP=1
# export RUST_MIN_STACK=8388608 # 8 * 1024 * 1024

echo check ulimits
ulimit -a

# install cargo-nextest
echo "Setup cargo-nextest."
cargo nextest -V >/dev/null 2>&1 || cargo install cargo-nextest --version "0.9.57" --locked

# following options are tuned for current self hosted CI machine
# --test-threads 12, proper test concurrency level, balance failure rate and test speed
# --failure-output immediate-final, make error log output immediate & at the end of the run
# --retries 2, a correct test case usually takes no more than 3 tries to pass
# --build-jobs 8, a little (~20s) faster than 5 or 10 build jobs 
cargo nextest run --workspace \
-E 'not (test(block_connector::test_write_dag_block_chain::test_block_dag_chain_switch_main)) and not (test(block_connector::test_write_dag_block_chain::test_full_sync_continue)) and not (test(tasks::tests::test_full_sync_continue)) and not (test(tasks::tests::test_full_sync_fork))  and not (test(tasks::tests::test_full_sync_fork_from_genesis))  and not (test(tasks::tests::test_full_sync_new_node))  and not (test(tasks::tests::test_sync_block_in_async_connection))  and not (test(tasks::tests_dag::test_continue_sync_dag_blocks)) and not (test(tasks::tests_dag::test_sync_dag_blocks))  and not (test(test_sync_and_notification))  and not (test(block_connector::test_write_dag_block_chain::test_block_dag_chain_switch_main)) and not (test(block_connector::test_illegal_block::test_verify_consensus_failed)) and not (test(tasks::tests::test_full_sync_cancel))'
--retries 2 --build-jobs 8 --test-threads 12 --no-fail-fast --failure-output immediate-final


# please ensure the two test commands' arguments (e.g. `-j 15`) are the same to avoid recompilation
# RUST_LOG=OFF RUST_BACKTRACE=0 cargo xtest --no-fail-fast --exclude starcoin-move-prover -j 15 -- --test-threads=10 --color never --format pretty |tee "$TEST_RESULT_FILE" ||true
# grep -e '^test[[:space:]][^[:space:]]*[[:space:]]\+\.\.\.[[:space:]]FAILED' "$TEST_RESULT_FILE" >"$TEST_RESULT_FAILED_FILE" ||true

# status=0
# IFS=' '
# while read -r -a testcase; do
# 	case_name=${testcase[1]}
# 	echo "rerun failed test $case_name"
# 	RUST_LOG=DEBUG RUST_BACKTRACE=full cargo xtest -j 15 "$case_name" -- --test-threads=1 --nocapture
# 	case_status=$?
# 	if [ $case_status -ne 0 ]; then
# 		status=$case_status
# 	fi
# done <"$TEST_RESULT_FAILED_FILE"
# exit $status

