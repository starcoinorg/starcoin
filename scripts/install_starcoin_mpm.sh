#!/bin/bash

# Provide a script to download the latest starcoin and mpm from gitHub release page.

if [ -z "$1" ]; then
    echo "Usage: $0 <version>"
    exit 1
fi

INSTALL_DIR="/usr/local/bin"
VERSION=$1
SYSTEM=$(uname -s)

if [ $SYSTEM == "Darwin" ]; then
  PLATFORM="macos"
elif [ $SYSTEM == "Linux" ]; then
  PLATFORM="ubuntu"
else
  echo "Unsupported system"
  exit 1
fi

PACKAGE_DIR="starcoin-${PLATFORM}-latest"
unzip_dir="starcoin-artifacts"
PACKAGE="https://github.com/starcoinorg/starcoin/releases/download/${VERSION}/${PACKAGE_DIR}.zip"
curl -sL -O "${PACKAGE}" && unzip "${PACKAGE_DIR}"
mv -f "${unzip_dir}/mpm" "${INSTALL_DIR}" && mv -f "${unzip_dir}/starcoin" "${INSTALL_DIR}"
chmod +x "${INSTALL_DIR}/mpm" && chmod +x "${INSTALL_DIR}/starcoin"
rm -f "${PACKAGE_DIR}.zip" && rm -fr ${unzip_dir}
