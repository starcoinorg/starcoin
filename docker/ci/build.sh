#!/bin/bash
# script for builder action hub action runner docker image.

SCRIPT_PATH="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
cd "$SCRIPT_PATH/../.."

docker build . -f docker/ci/Dockerfile -t starcoin/starcoin-builder