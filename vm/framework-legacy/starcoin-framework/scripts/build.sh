#!/bin/bash

# Copyright (c) The Strcoin Core Contributors
# SPDX-License-Identifier: Apache-2.0

set -e

SCRIPT_PATH="$( cd "$( dirname "$0" )" >/dev/null 2>&1 && pwd )"
cd "$SCRIPT_PATH/.." || exit

mpm package build --doc --abi --force