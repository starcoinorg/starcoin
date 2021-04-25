#!/bin/bash

SCRIPT_PATH="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"

sudo apt-get update
sudo apt-get install wget unzip curl -y

echo "Installing CMake......"
if which cmake &>/dev/null; then
  echo "CMake is already installed"
else
  sudo apt-get install cmake -y
fi

echo "Installing Clang......"
if which clang &>/dev/null; then
  echo "Clang is already installed"
else
  sudo apt-get install clang -y
fi

echo "Install openssl dev ...."
sudo apt-get install pkg-config libssl-dev -y

echo "Install tools for move prover......"

echo "Install Dotnet Core......"
if which dotnet &>/dev/null; then
  echo "Dotnet Core is already installed"
else
  bash vm/move-prover/scripts/linux/install-dotnet.sh
fi

echo "Install Boogie......"
BOOGIE_PATH="/home/$(whoami)/.dotnet/tools/boogie"

if [[ -f $BOOGIE_PATH ]]; then
  echo "Boogie is already installed"
else
  bash vm/move-prover/scripts/linux/install-boogie.sh
fi

echo "Install Z3......"
if which z3 &>/dev/null; then
  echo "Z3 is already installed"
else
  bash vm/move-prover/scripts/linux/install-z3.sh
fi