# $STARCOIN_DIR = (get-item $PSScriptRoot).parent.FullName
# $TEST_RESULT_FILE="$STARCOIN_DIR/target/debug/test_result.txt"
# $TEST_RESULT_FAILED_FILE="$STARCOIN_DIR/target/debug/test_result_failed.txt"
# Write-Host "Starcoin root dir: $STARCOIN_DIR"

# $env:RUSTFLAGS="-Ccodegen-units=1 -Copt-level=0"
# $env:RUSTC_BOOTSTRAP=1
# $env:RUST_MIN_STACK=8*1024*1024
# $env:RUST_LOG="OFF"
# $env:RUST_BACKTRACE=0
# cargo xtest --no-fail-fast --exclude starcoin-move-prover -j 15 -- --test-threads=10 --color never --format pretty | Tee-Object "$TEST_RESULT_FILE"

# $failed_tests=Select-String -Pattern 'test .* +\.\.\. FAILED' -Path "./target/debug/test_result.txt" -AllMatches
# Write-Host "All failed tests are redirected to file: $TEST_RESULT_FAILED_FILE" -ForegroundColor Green
# $failed_tests > "$TEST_RESULT_FAILED_FILE"

cargo nextest -V >/dev/null 2>&1 || cargo install cargo-nextest

cargo nextest run --workspace --retries 2 --build-jobs 8 --test-threads 12 --failure-output immediate-final

# Write-Host "Retrying failed test cases" -ForegroundColor Green
# $env:RUST_LOG="DEBUG"
# $env:RUST_BACKTRACE="FULL"
# $case_status=0
# $failed_tests | ForEach-Object {
#     $test=$_ -split ' '
#     $test=$test[1]
#     Write-Host "Rerunning test failed case: $test" -ForegroundColor Red
#     cargo xtest -j 15 "$test" -- --test-threads=1 --nocapture
#     if ($LASTEXITCODE -ne 0) {
#         $case_status=$LASTEXITCODE
#         Write-Host "Test case $test failed with $case_status" -ForegroundColor Red
#     }
# }
# exit $case_status
