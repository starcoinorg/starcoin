#!/bin/bash

get_latest_release() {
  curl --silent "https://api.github.com/repos/$1/releases/latest" | # Get latest release from GitHub api
    grep '"tag_name":' |                                            # Get tag line
    sed -E 's/.*"([^"]+)".*/\1/'                                    # Pluck JSON value
}

// TODO support -h

if [ -z "$1" ]; then
    echo "A script to download the latest starcoin and mpm from gitHub release page."
    echo "Usage: $0 [version]"
    echo "version "
    exit 1
fi

// same as dev_setup.sh default INSTALL_DIR, no need for `sudo` to operate on
INSTALL_DIR="${HOME}/bin"

VERSION=$1
SYSTEM=$(uname -s)

if [ $SYSTEM == "Darwin" ]; then
  PLATFORM="macos"
elif [ $SYSTEM == "Linux" ]; then
  if 
  PLATFORM="ubuntu"
else
  echo "Unsupported system"
  exit 1
fi

PACKAGE_DIR="starcoin-${PLATFORM}-latest"
unzip_dir="starcoin-artifacts"
PACKAGE="https://github.com/starcoinorg/starcoin/releases/download/${VERSION}/${PACKAGE_DIR}.zip"
curl -L -O "${PACKAGE}" && unzip "${PACKAGE_DIR}"
mv -f "${unzip_dir}/mpm" "${INSTALL_DIR}" && mv -f "${unzip_dir}/starcoin" "${INSTALL_DIR}"
chmod +x "${INSTALL_DIR}/mpm" && chmod +x "${INSTALL_DIR}/starcoin"
rm -f "${PACKAGE_DIR}.zip" && rm -fr ${unzip_dir}
