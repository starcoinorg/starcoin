#!/bin/bash

# Default flag value for simulation mode
simulate=true

# Function to display script usage
display_help() {
    echo "Usage: ./update_version.sh <old_version> <new_version> [--execute]"
    echo "       ./update_version.sh -h|--help"
    echo ""
    echo "Options:"
    echo "  --execute  Perform actual changes in files"
    echo "  -h, --help Display this help message"
}

# Check if -h or --help flag is provided to display help
if [ "$1" == "-h" ] || [ "$1" == "--help" ]; then
    display_help
    exit 0
fi

# Check if input parameters are empty
if [ -z "$1" ] || [ -z "$2" ]; then
    echo "Error: Please provide the old version number and new version number as parameters"
    display_help
    exit 1
fi

# Check if --execute flag is provided to perform actual changes
if [ "$3" == "--execute" ]; then
    simulate=false
fi

if [[ "$OSTYPE" == "darwin"* ]]; then
  if ! command -v gsed >/dev/null; then
    echo "gsed not found, installing..."
    brew install gnu-sed
  fi
  SED=gsed
else
  SED=sed
fi

# Get input parameters
old_version=$1
new_version=$2

# Get the absolute path of the script directory
script_dir=$(cd "$(dirname "$0")" && pwd)

# Get the parent directory path of the script location
base_dir=$(dirname "$script_dir")

# Find all Cargo.toml files (excluding target directory) and process each file
find "$base_dir" -name "target" -prune -o -name "Cargo.toml" -type f | while read -r cargo_file; do
    # Use sed command to find and replace version number in [package] section
    if [ "$simulate" = true ]; then
        $SED -n "/\[package\]/,/^$/p" "$cargo_file" | $SED "s/version = \"$old_version\"/version = \"$new_version\"/g"
    else
        $SED -i "/\[package\]/,/^$/ s/version = \"$old_version\"/version = \"$new_version\"/" "$cargo_file"
        echo "Version number in $cargo_file has been changed to $new_version"
    fi
done
