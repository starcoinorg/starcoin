#!/usr/bin/env bash
# Builds and runs the libFuzzer targets with the repo toolchain (1.66.1).
#
# cargo-fuzz 0.12+ passes cargo config keys that cargo 1.66 rejects, and sanitizers need
# nightly, so this applies cargo-fuzz's coverage flags directly (sanitizer: none).
#
#   ./fuzz.sh seeds                      write seed inputs into corpus/<target>/
#   ./fuzz.sh build                      build every fuzz target
#   ./fuzz.sh run <target> [secs] [forks]  fuzz one target (default 600 s, 8 forks)
#   ./fuzz.sh repro <target> <input>     run one input, e.g. a crash from artifacts/
set -euo pipefail
cd "$(dirname "$0")"

TRIPLE=x86_64-unknown-linux-gnu
FUZZ_RUSTFLAGS="-Cpasses=sancov-module -Cllvm-args=-sanitizer-coverage-level=4 \
-Cllvm-args=-sanitizer-coverage-inline-8bit-counters -Cllvm-args=-sanitizer-coverage-pc-table \
-Cllvm-args=-sanitizer-coverage-trace-compares -Cllvm-args=-sanitizer-coverage-stack-depth \
--cfg fuzzing -Cdebug-assertions"

build() {
    RUSTFLAGS="$FUZZ_RUSTFLAGS ${RUSTFLAGS:-}" cargo build --release --target "$TRIPLE" --bins
}

case "${1:-}" in
seeds)
    cargo test --release --test seed_corpus -- --ignored
    ;;
build)
    build
    ;;
run)
    target="$2"
    mkdir -p "corpus/$target" "artifacts/$target"
    build
    "target/$TRIPLE/release/$target" "corpus/$target" -artifact_prefix="artifacts/$target/" \
        -fork="${4:-8}" -ignore_crashes=1 -ignore_timeouts=1 -ignore_ooms=1 \
        -max_total_time="${3:-600}" -timeout=10 -rss_limit_mb=2048 -max_len=16384
    ;;
repro)
    build
    "target/$TRIPLE/release/$2" "$3"
    ;;
*)
    sed -n '2,10p' "$0"
    exit 2
    ;;
esac
