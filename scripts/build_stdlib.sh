#!/bin/bash

SOURCE_VERSION=$1
TARGET_VERSION=$2

if [ $# -lt 2 ]; then
    echo "Usage: $0 SOURCE_VERSION TARGET_VERSION [ARGS...]"
    exit 1
fi

SOURCE_VERSION=$1
TARGET_VERSION=$2

# check SOURCE_VERSION and TARGET_VERSION has defined
if [ -z "$SOURCE_VERSION" ] || [ -z "$TARGET_VERSION" ]; then
    echo "SOURCE_VERSION and TARGET_VERSION must be specified."
    exit 1
fi

# Check SOURCE_VERSION and TARGET_VERSION is number
if ! [[ "$SOURCE_VERSION" =~ ^[0-9]+$ ]] || ! [[ "$TARGET_VERSION" =~ ^[0-9]+$ ]]; then
    echo "SOURCE_VERSION and TARGET_VERSION must be integers."
    exit 1
fi

cargo run --bin stdlib

cmd="cargo run --bin stdlib -- -v ${TARGET_VERSION} -m StdlibUpgradeScripts -f upgrade_from_v${SOURCE_VERSION}_to_v${TARGET_VERSION}"

args=("${@:3}")

for arg in "${args[@]}"; do
    cmd+=" --arg ${arg}"
done

eval $cmd

# build genesises
cargo run --bin starcoin-genesis