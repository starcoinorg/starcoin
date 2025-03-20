#!/bin/bash
# Copyright (c) The Strcoin Core Contributors
# SPDX-License-Identifier: Apache-2.0
# This script sets up the environment for the Starcoin Move Framework build by installing necessary dependencies.
#
# Usage ./dev_setup.sh <options>
#   v - verbose, print all statements

# Assumptions for nix systems:
# 1 The running user is the user who will execute the builds.
# 2 .profile will be used to configure the shell
# 3 ${HOME}/bin/, or ${INSTALL_DIR} is expected to be on the path -etc.  will be installed there on linux systems.

# fast fail.
set -eo pipefail


Z3_VERSION=4.11.2
CVC5_VERSION=0.0.3
DOTNET_VERSION=6.0
BOOGIE_VERSION=2.15.8
MPM_VERSION=v1.13.7


SCRIPT_PATH="$( cd "$( dirname "$0" )" >/dev/null 2>&1 && pwd )"
cd "$SCRIPT_PATH/.." || exit

function usage {
  echo "Usage:"
  echo "Installs or updates necessary dev tools for starcoin move framework."
  echo "-b batch mode, no user interactions and miminal output"
  echo "-p update ${HOME}/.profile"
  echo "-t install build tools"
  echo "-y installs or updates Move prover tools: z3, cvc5, dotnet, boogie"
  echo "-v verbose mode"
  echo "-i installs an individual tool by name"
  echo "-n will target the /opt/ dir rather than the $HOME dir.  /opt/bin/, /opt/rustup/, and /opt/dotnet/ rather than $HOME/bin/, $HOME/.rustup/, and $HOME/.dotnet/"
  echo "If no toolchain component is selected with -t, -o, -y, or -p, the behavior is as if -t had been provided."
  echo "This command must be called from the root folder of the starcoin-frame project."
}

function add_to_profile {
  eval "$1"
  FOUND=$(grep -c "$1" < "${HOME}/.profile" || true)  # grep error return would kill the script.
  if [ "$FOUND" == "0" ]; then
    echo "$1" >> "${HOME}"/.profile
  fi
}


# It is important to keep all path updates together to allow this script to work well when run in github actions
# inside of a docker image created using this script.   GHA wipes the home directory via docker mount options, so
# this profile needs built and sourced on every execution of a job using the docker image.   See the .github/actions/build-setup
# action in this repo, as well as docker/ci/github/Dockerfile.
function update_path_and_profile {
  touch "${HOME}"/.profile

  DOTNET_ROOT="$HOME/.dotnet"
  BIN_DIR="$HOME/bin"
  C_HOME="${HOME}/.cargo"
  if [[ "$OPT_DIR" == "true" ]]; then
    DOTNET_ROOT="/opt/dotnet"
    BIN_DIR="/opt/bin"
    C_HOME="/opt/cargo"
  fi

  mkdir -p "${BIN_DIR}"
  if [ -n "$CARGO_HOME" ]; then
    add_to_profile "export CARGO_HOME=\"${CARGO_HOME}\""
    add_to_profile "export PATH=\"${BIN_DIR}:${CARGO_HOME}/bin:\$PATH\""
  else
    add_to_profile "export PATH=\"${BIN_DIR}:${C_HOME}/bin:\$PATH\""
  fi
  if [[ "$INSTALL_PROVER" == "true" ]]; then
    add_to_profile "export DOTNET_ROOT=\"${DOTNET_ROOT}\""
    add_to_profile "export PATH=\"${DOTNET_ROOT}/tools:\$PATH\""
    add_to_profile "export Z3_EXE=\"${BIN_DIR}/z3\""
    add_to_profile "export CVC5_EXE=\"${BIN_DIR}/cvc5\""
    add_to_profile "export BOOGIE_EXE=\"${DOTNET_ROOT}/tools/boogie\""
  fi
}

function install_rustup {
  echo installing rust.
  BATCH_MODE=$1
  if [[ "$OPT_DIR" == "true" ]]; then
     export RUSTUP_HOME=/opt/rustup/
     mkdir -p "$RUSTUP_HOME" || true
     export CARGO_HOME=/opt/cargo/
     mkdir -p "$CARGO_HOME" || true
  fi

  # Install Rust
  if [[ "${BATCH_MODE}" == "false" ]]; then
    echo "Installing Rust......"
  fi
  VERSION="$(rustup --version || true)"
  if [ -n "$VERSION" ]; then
	  if [[ "${BATCH_MODE}" == "false" ]]; then
      echo "Rustup is already installed, version: $VERSION"
    fi
  else
	  curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain stable
    if [[ -n "${CARGO_HOME}" ]]; then
      PATH="${CARGO_HOME}/bin:${PATH}"
    else
      PATH="${HOME}/.cargo/bin:${PATH}"
    fi
  fi
}


function install_pkg {
  package=$1
  PACKAGE_MANAGER=$2
  PRE_COMMAND=()
  if [ "$(whoami)" != 'root' ]; then
    PRE_COMMAND=(sudo)
  fi
  if command -v "$package" &>/dev/null; then
    echo "$package is already installed"
  else
    echo "Installing ${package}."
    if [[ "$PACKAGE_MANAGER" == "yum" ]]; then
      "${PRE_COMMAND[@]}" yum install "${package}" -y
    elif [[ "$PACKAGE_MANAGER" == "apt-get" ]]; then
      "${PRE_COMMAND[@]}" apt-get install "${package}" --no-install-recommends -y
      echo apt-get install result code: $?
    elif [[ "$PACKAGE_MANAGER" == "pacman" ]]; then
      "${PRE_COMMAND[@]}" pacman -Syu "$package" --noconfirm
    elif [[ "$PACKAGE_MANAGER" == "apk" ]]; then
      apk --update add --no-cache "${package}"
    elif [[ "$PACKAGE_MANAGER" == "dnf" ]]; then
      dnf install "$package"
    elif [[ "$PACKAGE_MANAGER" == "brew" ]]; then
      brew install "$package"
    fi
  fi
}

function install_toolchain {
  version=$1
  FOUND=$(rustup show | grep -c "$version" || true )
  if [[ "$FOUND" == "0" ]]; then
    echo "Installing ${version} of rust toolchain"
    rustup install "$version"
  else
    echo "${version} rust toolchain already installed"
  fi
}


function install_mpm {
  echo "Installing mpm"
  VERSION="$(mpm --version || true)"
  if [ -n "$VERSION" ]; then
    if [[ "${BATCH_MODE}" == "false" ]]; then
      echo "mpm is already installed, version: $VERSION"
    fi
  else
    if [[ $(uname -s) == "Darwin" ]]; then
      mpm_file="mpm-macos-latest";
    else
      if [ "$(. /etc/os-release; echo $NAME)" = "Ubuntu" ]; then
        if [[ $(lsb_release -r | cut -f 2) == '18.04' ]]; then
          mpm_file="mpm-ubuntu-18.04";
        else
          mpm_file="mpm-ubuntu-latest";
        fi
      else
        mpm_file="";
      fi
    fi
    if [[ $mpm_file != "" ]]; then
      curl -sL -o "${INSTALL_DIR}${mpm_file}.zip" "https://github.com/starcoinorg/starcoin/releases/download/${MPM_VERSION}/${mpm_file}.zip"
      unzip -q "${INSTALL_DIR}${mpm_file}.zip" -d "${INSTALL_DIR}"
      mv "${INSTALL_DIR}${mpm_file}/mpm" "${INSTALL_DIR}mpm"
      chmod +x "${INSTALL_DIR}mpm"
      rmdir "${INSTALL_DIR}${mpm_file}"
    else
      echo "Install mpm from source"
      cargo install --git https://github.com/starcoinorg/starcoin move-package-manager --tag $MPM_VERSION --bin mpm --root $HOME
    fi
  fi
}

function install_dotnet {
  echo "Installing .Net"
  mkdir -p "${DOTNET_INSTALL_DIR}" || true
  if [[ $("${DOTNET_INSTALL_DIR}dotnet" --list-sdks | grep -c "^${DOTNET_VERSION}" || true) == "0" ]]; then
    if [[ "$(uname)" == "Linux" ]]; then
        # Install various prerequisites for .dotnet. There are known bugs
        # in the dotnet installer to warn even if they are present. We try
        # to install anyway based on the warnings the dotnet installer creates.
        if [ "$PACKAGE_MANAGER" == "apk" ]; then
          install_pkg icu "$PACKAGE_MANAGER"
          install_pkg zlib "$PACKAGE_MANAGER"
          install_pkg libintl "$PACKAGE_MANAGER"
          install_pkg libcurl "$PACKAGE_MANAGER"
        elif [ "$PACKAGE_MANAGER" == "apt-get" ]; then
          install_pkg gettext "$PACKAGE_MANAGER"
          install_pkg zlib1g "$PACKAGE_MANAGER"
        elif [ "$PACKAGE_MANAGER" == "yum" ] || [ "$PACKAGE_MANAGER" == "dnf" ]; then
          install_pkg icu "$PACKAGE_MANAGER"
          install_pkg zlib "$PACKAGE_MANAGER"
        elif [ "$PACKAGE_MANAGER" == "pacman" ]; then
          install_pkg icu "$PACKAGE_MANAGER"
          install_pkg zlib "$PACKAGE_MANAGER"
        fi
    fi
    # Below we need to (a) set TERM variable because the .net installer expects it and it is not set
    # in some environments (b) use bash not sh because the installer uses bash features.
    curl -sSL https://dot.net/v1/dotnet-install.sh \
        | TERM=linux /bin/bash -s -- --channel $DOTNET_VERSION --install-dir "${DOTNET_INSTALL_DIR}" --version latest
  else
    echo Dotnet already installed.
  fi
}

function install_boogie {
  echo "Installing boogie"
  mkdir -p "${DOTNET_INSTALL_DIR}tools/" || true
  if [[ "$("${DOTNET_INSTALL_DIR}dotnet" tool list --tool-path "${DOTNET_INSTALL_DIR}tools/")" =~ .*boogie.*${BOOGIE_VERSION}.* ]]; then
    echo "Boogie $BOOGIE_VERSION already installed"
  else
    "${DOTNET_INSTALL_DIR}dotnet" tool update --tool-path "${DOTNET_INSTALL_DIR}tools/" Boogie --version $BOOGIE_VERSION
  fi
}

function install_z3 {
  echo "Installing Z3"
  if command -v /usr/local/bin/z3 &>/dev/null; then
    echo "z3 already exists at /usr/local/bin/z3"
    echo "but this install will go to ${INSTALL_DIR}/z3."
    echo "you may want to remove the shared instance to avoid version confusion"
  fi
  if command -v "${INSTALL_DIR}z3" &>/dev/null && [[ "$("${INSTALL_DIR}z3" --version || true)" =~ .*${Z3_VERSION}.* ]]; then
     echo "Z3 ${Z3_VERSION} already installed"
     return
  fi
  if [[ "$(uname)" == "Linux" ]]; then
    Z3_PKG="z3-$Z3_VERSION-x64-glibc-2.31"
  elif [[ "$(uname)" == "Darwin" ]]; then
    Z3_PKG="z3-$Z3_VERSION-x64-osx-10.16"
  else
    echo "Z3 support not configured for this platform (uname=$(uname))"
    return
  fi
  TMPFILE=$(mktemp)
  rm "$TMPFILE"
  mkdir -p "$TMPFILE"/
  (
    cd "$TMPFILE" || exit
    curl -LOs "https://github.com/Z3Prover/z3/releases/download/z3-$Z3_VERSION/$Z3_PKG.zip"
    unzip -q "$Z3_PKG.zip"
    cp "$Z3_PKG/bin/z3" "${INSTALL_DIR}"
    chmod +x "${INSTALL_DIR}z3"
  )
  rm -rf "$TMPFILE"
}

function install_cvc5 {
  echo "Installing cvc5"
  if command -v /usr/local/bin/cvc5 &>/dev/null; then
    echo "cvc5 already exists at /usr/local/bin/cvc5"
    echo "but this install will go to $${INSTALL_DIR}cvc5."
    echo "you may want to remove the shared instance to avoid version confusion"
  fi
  if command -v "${INSTALL_DIR}cvc5" &>/dev/null && [[ "$("${INSTALL_DIR}cvc5" --version || true)" =~ .*${CVC5_VERSION}.* ]]; then
     echo "cvc5 ${CVC5_VERSION} already installed"
     return
  fi
  if [[ "$(uname)" == "Linux" ]]; then
    CVC5_PKG="cvc5-Linux"
  elif [[ "$(uname)" == "Darwin" ]]; then
    CVC5_PKG="cvc5-macOS"
  else
    echo "cvc5 support not configured for this platform (uname=$(uname))"
    return
  fi
  TMPFILE=$(mktemp)
  rm "$TMPFILE"
  mkdir -p "$TMPFILE"/
  (
    cd "$TMPFILE" || exit
    curl -LOs "https://github.com/cvc5/cvc5/releases/download/cvc5-$CVC5_VERSION/$CVC5_PKG"
    cp "$CVC5_PKG" "${INSTALL_DIR}cvc5"
    chmod +x "${INSTALL_DIR}cvc5"
  )
  rm -rf "$TMPFILE"
}


function welcome_message {
cat <<EOF
Welcome to Starcoin Move Framework!

This script will download and install the necessary dependencies needed to
build, test and inspect Starcoin Move Framework.

Based on your selection, these tools will be included:
EOF

  if [[ "$INSTALL_BUILD_TOOLS" == "true" ]]; then
cat <<EOF
Build tools (since -t or no option was provided):
  * Rust
  * mpm
EOF
  fi

  if [[ "$INSTALL_PROVER" == "true" ]]; then
cat <<EOF
Move prover tools (since -y was provided):
  * z3
  * cvc5
  * dotnet
  * boogie
EOF
  fi

  if [[ "$INSTALL_PROFILE" == "true" ]]; then
cat <<EOF
Moreover, ~/.profile will be updated (since -p was provided).
EOF
  fi

cat <<EOF
If you'd prefer to install these dependencies yourself, please exit this script
now with Ctrl-C.
EOF
}

BATCH_MODE=false;
VERBOSE=false;
INSTALL_BUILD_TOOLS=false;
INSTALL_PROFILE=false;
INSTALL_PROVER=false;
INSTALL_INDIVIDUAL=false;
INSTALL_PACKAGES=();
INSTALL_DIR="${HOME}/bin/"
OPT_DIR="false"

#parse args
while getopts "btopvysah:i:n" arg; do
  case "$arg" in
    b)
      BATCH_MODE="true"
      ;;
    t)
      INSTALL_BUILD_TOOLS="true"
      ;;
    p)
      INSTALL_PROFILE="true"
      ;;
    v)
      VERBOSE=true
      ;;
    y)
      INSTALL_PROVER="true"
      ;;
    i)
      INSTALL_INDIVIDUAL="true"
      echo "$OPTARG"
      INSTALL_PACKAGES+=("$OPTARG")
      ;;
    n)
      OPT_DIR="true"
      ;;
    *)
      usage;
      exit 0;
      ;;
  esac
done

if [[ "$VERBOSE" == "true" ]]; then
	set -x
fi

if [[ "$INSTALL_BUILD_TOOLS" == "false" ]] && \
   [[ "$INSTALL_PROFILE" == "false" ]] && \
   [[ "$INSTALL_PROVER" == "false" ]] && \
   [[ "$INSTALL_INDIVIDUAL" == "false" ]]; then
   INSTALL_BUILD_TOOLS="true"
fi

if [[ "${OPT_DIR}" == "true" ]]; then
  INSTALL_DIR="/opt/bin/"
fi
mkdir -p "$INSTALL_DIR" || true

PRE_COMMAND=()
if [ "$(whoami)" != 'root' ]; then
  PRE_COMMAND=(sudo)
fi

PACKAGE_MANAGER=
if [[ "$(uname)" == "Linux" ]]; then
  # check for default package manager for linux
  if [[ -f /etc/redhat-release ]]; then
    # use yum for redhat-releases by default
    if command -v yum &>/dev/null; then
      PACKAGE_MANAGER="yum"
    elif command -v dnf &>/dev/null; then
      # dnf is the updated default since Red Hat Enterprise Linux 8, CentOS 8, Fedora 22, and any distros based on these
      echo "WARNING: dnf package manager support is experimental"
      PACKAGE_MANAGER="dnf"
    fi
  elif [[ -f /etc/debian_version ]] && command -v apt-get &>/dev/null; then
    PACKAGE_MANAGER="apt-get"
  elif [[ -f /etc/arch-release ]] && command -v pacman &>/dev/null; then
    PACKAGE_MANAGER="pacman"
  elif [[ -f /etc/alpine-release ]] && command -v apk &>/dev/null; then
    PACKAGE_MANAGER="apk"
  fi
  # if no default OS specific PACKAGE_MANAGER detected, just pick one that's installed (as best effort)
  if [[ -z $PACKAGE_MANAGER ]]; then
    if command -v yum &>/dev/null; then
      PACKAGE_MANAGER="yum"
    elif command -v apt-get &>/dev/null; then
      PACKAGE_MANAGER="apt-get"
    elif command -v pacman &>/dev/null; then
      PACKAGE_MANAGER="pacman"
    elif command -v apk &>/dev/null; then
      PACKAGE_MANAGER="apk"
    elif command -v dnf &>/dev/null; then
      echo "WARNING: dnf package manager support is experimental"
      PACKAGE_MANAGER="dnf"
    else
      echo "Unable to find supported package manager (yum, apt-get, dnf, or pacman). Abort"
      exit 1
    fi
  fi
elif [[ "$(uname)" == "Darwin" ]]; then
  if command -v brew &>/dev/null; then
    PACKAGE_MANAGER="brew"
  else
    echo "Missing package manager Homebrew (https://brew.sh/). Abort"
    exit 1
  fi
else
  echo "Unknown OS. Abort."
  exit 1
fi

if [[ "$BATCH_MODE" == "false" ]]; then
    welcome_message
    printf "Proceed with installing necessary dependencies? (y/N) > "
    read -e -r input
    if [[ "$input" != "y"* ]]; then
	    echo "Exiting..."
	    exit 0
    fi
fi

if [[ "$PACKAGE_MANAGER" == "apt-get" ]]; then
	if [[ "$BATCH_MODE" == "false" ]]; then
    echo "Updating apt-get......"
  fi
	"${PRE_COMMAND[@]}" apt-get update
  if [[ "$BATCH_MODE" == "false" ]]; then
   echo "Installing ca-certificates......"
  fi
	install_pkg ca-certificates "$PACKAGE_MANAGER"
fi

if [[ "$INSTALL_PROFILE" == "true" ]]; then
  update_path_and_profile
fi

install_pkg curl "$PACKAGE_MANAGER"


if [[ "$INSTALL_BUILD_TOOLS" == "true" ]]; then

  install_rustup "$BATCH_MODE"
  # Add all the components that we need
  rustup component add rustfmt
  rustup component add clippy

  install_pkg git "$PACKAGE_MANAGER"
  install_mpm
  #install_nodejs "$PACKAGE_MANAGER"
fi


if [[ "$INSTALL_INDIVIDUAL" == "true" ]]; then
  for (( i=0; i < ${#INSTALL_PACKAGES[@]}; i++ ));
  do
    PACKAGE=${INSTALL_PACKAGES[$i]}
    if ! command -v "install_${PACKAGE}" &> /dev/null; then
      install_pkg "$PACKAGE" "$PACKAGE_MANAGER"
    else
      "install_${PACKAGE}"
    fi
  done
fi

if [[ "$INSTALL_PROVER" == "true" ]]; then
  export DOTNET_INSTALL_DIR="${HOME}/.dotnet/"
  if [[ "$OPT_DIR" == "true" ]]; then
    export DOTNET_INSTALL_DIR="/opt/dotnet/"
    mkdir -p "$DOTNET_INSTALL_DIR" || true
  fi
  install_z3
  install_cvc5
  install_dotnet
  install_boogie
fi


if [[ "${BATCH_MODE}" == "false" ]]; then
cat <<EOF
Finished installing all dependencies.

You should now be able to build the project by running:
	mpm package build
	mpm package prove
EOF
fi

exit 0
