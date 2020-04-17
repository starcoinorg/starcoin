#!/bin/bash
# This script sets up the environment for the Starcoin build by installing necessary dependencies.
#
# Usage ./dev_setup.sh <options>
#   v - verbose, print all statements

SCRIPT_PATH="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
cd "$SCRIPT_PATH/.."

set -e
OPTIONS="$1"

if [[ $OPTIONS == *"v"* ]]; then
  set -x
fi

if [ ! -f Cargo.toml ]; then
  echo "Unknown location. Please run this from the libra repository. Abort."
  exit 1
fi

PACKAGE_MANAGER=
if [[ "$OSTYPE" == "linux-gnu" ]]; then
  if which yum &>/dev/null; then
    PACKAGE_MANAGER="yum"
  elif which apt-get &>/dev/null; then
    PACKAGE_MANAGER="apt-get"
  elif which pacman &>/dev/null; then
    PACKAGE_MANAGER="pacman"
  else
    echo "Unable to find supported package manager (yum, apt-get, or pacman). Abort"
    exit 1
  fi
elif [[ "$OSTYPE" == "darwin"* ]]; then
  if which brew &>/dev/null; then
    PACKAGE_MANAGER="brew"
  else
    echo "Missing package manager Homebrew (https://brew.sh/). Abort"
    exit 1
  fi
else
  echo "Unknown OS. Abort."
  exit 1
fi

# Install Rust
echo "Installing Rust......"
if rustup --version &>/dev/null; then
  echo "Rust is already installed"
else
  curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain stable
  CARGO_ENV="$HOME/.cargo/env"
  source "$CARGO_ENV"
fi

## Run update in order to download and install the checked in toolchain
#rustup update
#
## Add all the components that we need
#rustup component add rustfmt
#rustup component add clippy

#if [[ $"$PACKAGE_MANAGER" == "apt-get" ]]; then
#	echo "Updating apt-get......"
#	sudo apt-get update
#fi

echo "Installing CMake......"
if which cmake &>/dev/null; then
  echo "CMake is already installed"
else
  if [[ "$PACKAGE_MANAGER" == "yum" ]]; then
    sudo yum install cmake -y
  elif [[ "$PACKAGE_MANAGER" == "apt-get" ]]; then
    sudo apt-get update
    sudo apt-get install cmake -y
  elif [[ "$PACKAGE_MANAGER" == "pacman" ]]; then
    sudo pacman -Syu cmake --noconfirm
  elif [[ "$PACKAGE_MANAGER" == "brew" ]]; then
    brew install cmake
  fi
fi

cat <<EOF
Finished installing all dependencies.

You should now be able to build the project by running:
source $HOME/.cargo/env
cargo build --all
EOF
