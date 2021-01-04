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

## Run update in order to download and install the checked in toolchain
rustup update
#
## Add all the components that we need
rustup component add rustfmt
rustup component add clippy
