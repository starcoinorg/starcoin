#!/bin/bash

# Install Rust
echo "Installing Rust......"
if rustup --version &>/dev/null; then
  echo "Rust is already installed"
else
  curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain stable
  CARGO_ENV="$HOME/.cargo/env"
  source "$CARGO_ENV"
fi

sudo apt-get update

echo "Installing CMake......"
if which cmake &>/dev/null; then
  echo "CMake is already installed"
else
   sudo pacman -Syu cmake --noconfirm
fi

echo "Installing Clang......"
if which clang &>/dev/null; then
  echo "Clang is already installed"
else
  sudo pacman -Syu clang --noconfirm
fi

echo "Move prover tools do not support install via pacman"