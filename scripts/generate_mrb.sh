#!/bin/bash

# Script to generate and manage MRB files
# Usage: ./generate_mrb.sh <version> <network>
# Example: ./generate_mrb.sh 13 main
#          ./generate_mrb.sh latest dev
# 
# Version: number (e.g., 13, 14) or 'latest'
# Network: BuiltinNetworkID values - test, dev, halley, proxima, barnard, main

set -e

# Check arguments
if [ $# -ne 2 ]; then
    echo "Usage: $0 <version> <network>"
    echo "Example: $0 13 main"
    echo "         $0 latest dev"
    echo "Version: number or 'latest'"
    echo "Network: test, dev, halley, proxima, barnard, main"
    exit 1
fi

VERSION=$1
NETWORK=$2

# Validate arguments
if [[ ! "$VERSION" =~ ^([0-9]+|latest)$ ]]; then
    echo "Error: version must be a number or 'latest'"
    exit 1
fi

if [[ ! "$NETWORK" =~ ^(test|dev|halley|proxima|barnard|main)$ ]]; then
    echo "Error: network must be one of test, dev, halley, proxima, barnard, or main (BuiltinNetworkID values)"
    exit 1
fi

# Define paths
FRAMEWORK_DIR="vm2/framework"
TARGET_DIR="vm/stdlib/compiled/$VERSION"
TARGET_FILE="$TARGET_DIR/${NETWORK}.mrb"

echo "=== Generating MRB File ==="
echo "Version: $VERSION"
echo "Network: $NETWORK"
echo "Target file: $TARGET_FILE"
echo

# Create target directory (if it doesn't exist)
if [ ! -d "vm/stdlib/compiled" ]; then
    echo "Error: vm/stdlib/compiled base directory does not exist"
    exit 1
fi

echo "Creating target directory: $TARGET_DIR"
mkdir -p "$TARGET_DIR"

# Enter framework directory
echo "Entering directory: $FRAMEWORK_DIR"
cd "$FRAMEWORK_DIR"

# Clean previous build artifacts
echo "Cleaning previous head.mrb file..."
rm -f head.mrb

# Generate head.mrb file
echo "Generating head.mrb file..."
echo "Running command: cargo run --release -- release --target head"
cargo run --release -- release --target head

# Check if generation was successful
if [ ! -f "head.mrb" ]; then
    echo "Error: head.mrb file generation failed"
    exit 1
fi

echo "head.mrb file generated successfully"

# Move and rename file
echo "Moving file to target directory..."
mv head.mrb "../../$TARGET_FILE"

if [ -f "../../$TARGET_FILE" ]; then
    echo "Success! MRB file generated: $TARGET_FILE"
    echo "File size: $(du -h "../../$TARGET_FILE" | cut -f1)"
else
    echo "Error: file move failed"
    exit 1
fi

echo "=== Completed ==="