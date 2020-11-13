#!/bin/bash

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

echo "Install Dotnet Core......"
if which dotnet &>/dev/null; then
  echo "Dotnet Core is already installed"
else
  if [[ "$PACKAGE_MANAGER" == "apt-get" ]]; then
    bash vm/move-prover/scripts/linux/install-dotnet.sh
  elif [[ "$PACKAGE_MANAGER" == "brew" ]]; then
    bash vm/move-prover/scripts/macos/install-dotnet.sh
  else
    echo "do not support dotnet installation via $PACKAGE_MANAGER"
  fi
fi

echo "Install Boogie......"
if [[ "$OSTYPE" == "linux-gnu" ]]; then
  BOOGIE_PATH="/home/$(whoami)/.dotnet/tools/boogie"
elif  [[ "$OSTYPE" == "darwin"* ]]; then
  BOOGIE_PATH="/Users/$(whoami)/.dotnet/tools/boogie"
else
  BOOGIE_PATH=
fi

if [[ -f $BOOGIE_PATH ]]; then
  echo "Boogie is already installed"
else
  if [[ "$PACKAGE_MANAGER" == "apt-get" ]]; then
    bash vm/move-prover/scripts/linux/install-boogie.sh
  elif [[ "$PACKAGE_MANAGER" == "brew" ]]; then
    bash vm/move-prover/scripts/macos/install-boogie.sh
  else
    echo "do not support dotnet installation via $PACKAGE_MANAGER"
  fi
fi

echo "Install Z3......"
if which z3 &>/dev/null; then
  echo "Z3 is already installed"
else
  if [[ "$PACKAGE_MANAGER" == "apt-get" ]]; then
    bash vm/move-prover/scripts/linux/install-z3.sh
  elif [[ "$PACKAGE_MANAGER" == "brew" ]]; then
    bash vm/move-prover/scripts/macos/install-z3.sh
  else
    echo "do not support dotnet installation via $PACKAGE_MANAGER"
  fi
fi