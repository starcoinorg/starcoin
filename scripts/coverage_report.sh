#!/bin/bash
# Copyright (c) The Diem Core Contributors
# SPDX-License-Identifier: Apache-2.0

# Check that the test directory and report path arguments are provided
if [ $# -lt 2 ] || ! [ -d "$1" ]; then
  echo "Usage: $0 <testdir> <outdir> [--batch]"
  echo "All tests in <testdir> and its subdirectories will be run to measure coverage."
  echo "The resulting coverage report will be stored in <outdir>."
  echo "--batch will skip all prompts."
  exit 1
fi

# User prompts will be skipped if '--batch' is given as the third argument
SKIP_PROMPTS=0
if [ $# -eq 3 ] && [ "$3" == "--batch" ]; then
  SKIP_PROMPTS=1
fi

# Set the directory containing the tests to run (includes subdirectories)
TEST_DIR=$1

# Set the directory to which the report will be saved
COVERAGE_DIR=$2

# This needs to run in starcoin
STARCOIN_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && cd .. && pwd)"
if [ "$(pwd)" != "$STARCOIN_DIR" ]; then
  echo "Error: This needs to run from starcoin/, not in $(pwd)" >&2
  exit 1
fi

#set -e

# Check that grcov is installed
if ! [ -x "$(command -v grcov)" ]; then
  echo "Error: grcov is not installed." >&2
  if [ $SKIP_PROMPTS -eq 0 ]; then
    read -p "Install grcov? [yY/*] " -n 1 -r
    echo ""
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
      [[ "$0" == "$BASH_SOURCE" ]] && exit 1 || return 1
    fi
    cargo install grcov
  else
    exit 1
  fi
fi

# Check that lcov is installed
if ! [ -x "$(command -v lcov)" ]; then
  echo "Error: lcov is not installed." >&2
  echo "Documentation for lcov can be found at http://ltp.sourceforge.net/coverage/lcov.php"
  echo "If on macOS and using homebrew, run 'brew install lcov'"
  exit 1
fi

# Warn that cargo clean will happen
if [ $SKIP_PROMPTS -eq 0 ]; then
  read -p "Generate coverage report? This will run cargo clean. [yY/*] " -n 1 -r
  echo ""
  if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    [[ "$0" == "$BASH_SOURCE" ]] && exit 1 || return 1
  fi
fi

# Set the flags necessary for coverage output
export RUSTFLAGS="-Zprofile -Ccodegen-units=1 -Copt-level=0 -Clink-dead-code -Coverflow-checks=off"
export RUSTC_BOOTSTRAP=1
export CARGO_INCREMENTAL=0
export RUST_MIN_STACK=8388608 # 8 * 1024 * 1024

echo check ulimits
ulimit -a

# Clean the project
echo "Cleaning project..."
(
  cd "$TEST_DIR"
  cargo clean
)

# Run tests
echo "Running tests..."
cargo xtest --html-lcov-dir="${COVERAGE_DIR}" --no-fail-fast --lib -j 5 || true

echo "Done. Please view report at ${COVERAGE_DIR}/index.html"
