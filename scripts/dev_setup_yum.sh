#!/bin/bash

sudo yum update
sudo yum install -y wget unzip curl

# Install Rust
echo "Installing Rust......"
if rustup --version &>/dev/null; then
  echo "Rust is already installed"
else
  curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain stable
  CARGO_ENV="$HOME/.cargo/env"
  source "$CARGO_ENV"
fi

echo "Installing CMake......"
if which cmake &>/dev/null; then
  echo "CMake is already installed"
else
   sudo yum install cmake -y
fi

echo "Installing Clang......"
if which clang &>/dev/null; then
  echo "Clang is already installed"
else
  sudo yum install clang -y
fi

echo "Install openssl dev ...."
sudo yum install openssl-devel -y


echo "Move prover tools do not support install via yum"