#!/bin/bash -e

# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0

curl -LO https://github.com/Z3Prover/z3/releases/download/z3-4.8.8/z3-4.8.8-x64-ubuntu-16.04.zip
unzip z3-4.8.8-x64-ubuntu-16.04.zip
sudo cp z3-4.8.8-x64-ubuntu-16.04/bin/z3 /usr/local/bin/
rm -rf z3-4.8.8-x64-ubuntu-16.04
rm -rf z3-4.8.8-x64-ubuntu-16.04.zip