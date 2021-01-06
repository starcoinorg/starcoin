#!/bin/bash -e

# Copyright (c) The Diem Core Contributors
# SPDX-License-Identifier: Apache-2.0

wget -q https://github.com/Z3Prover/z3/releases/download/z3-4.8.9/z3-4.8.9-x64-ubuntu-16.04.zip -O /tmp/z3-4.8.9-x64-ubuntu-16.04.zip
unzip /tmp/z3-4.8.9-x64-ubuntu-16.04.zip -d /tmp/
sudo cp /tmp/z3-4.8.9-x64-ubuntu-16.04/bin/z3 /usr/local/bin/
rm -rf /tmp/z3-4.8.9-x64-ubuntu-16.04
rm -rf /tmp/z3-4.8.9-x64-ubuntu-16.04.zip
