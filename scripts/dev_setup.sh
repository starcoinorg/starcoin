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
  echo "Unknown location. Please run this from the starcoin repository. Abort."
  exit 1
fi

bash scripts/dev_setup_rust.sh

PACKAGE_MANAGER=
if [[ "$OSTYPE" == "linux-gnu" ]]; then
  if which yum &>/dev/null; then
    PACKAGE_MANAGER="yum"
    bash scripts/dev_setup_yum.sh
  elif which apt-get &>/dev/null; then
    PACKAGE_MANAGER="apt-get"
    bash scripts/dev_setup_apt.sh
  elif which pacman &>/dev/null; then
    PACKAGE_MANAGER="pacman"
    bash scripts/dev_setup_pacman.sh
  else
    echo "Unable to find supported package manager (yum, apt-get, or pacman). Abort"
    exit 1
  fi
elif [[ "$OSTYPE" == "darwin"* ]]; then
  if which brew &>/dev/null; then
    PACKAGE_MANAGER="brew"
    bash scripts/dev_setup_brew.sh
  else
    echo "Missing package manager Homebrew (https://brew.sh/). Abort"
    exit 1
  fi
else
  echo "Unknown OS. Abort."
  exit 1
fi

cat <<EOF
Finished installing all dependencies.

You should now be able to build the project by running:
source $HOME/.cargo/env
cargo build --all
EOF
