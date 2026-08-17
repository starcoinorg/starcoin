#!/usr/bin/env bash
set -euo pipefail

namespace="${NAMESPACE:-starcoin-main}"

set_resources() {
  local deployment="$1"
  local container="$2"
  local requests="$3"
  local limits="$4"

  kubectl -n "$namespace" set resources "deployment/$deployment" \
    --containers="$container" \
    --requests="$requests" \
    --limits="$limits"
}

set_resources starcoin-pricereporter-deployment \
  starcoin-pricereporter cpu=50m,memory=1536Mi memory=2560Mi
set_resources starcoin-pricereporter-deployment-barnard \
  starcoin-pricereporter-barnard cpu=50m,memory=1536Mi memory=2560Mi

set_resources starswap-api-deployment-aptos-testnet \
  starswap-api-aptos-testnet cpu=100m,memory=2Gi memory=3Gi
set_resources starswap-api-deployment-aptos-mainnet \
  starswap-api-aptos-mainnet cpu=100m,memory=1Gi memory=2Gi
set_resources starswap-api-deployment-aptos-devnet \
  starswap-api-aptos-devnet cpu=100m,memory=1Gi memory=2Gi

set_resources starswap-api-deployment \
  starswap-api cpu=100m,memory=768Mi memory=2Gi
set_resources starswap-api-deployment-barnard \
  starswap-api-barnard cpu=100m,memory=768Mi memory=2Gi
set_resources poll-api-deployment \
  poll-api cpu=100m,memory=768Mi memory=2Gi
set_resources dao-api-deployment \
  dao-api cpu=100m,memory=768Mi memory=2Gi
set_resources dao-api-deployment-barnard \
  dao-api-barnard cpu=100m,memory=768Mi memory=2Gi
set_resources swap-stat-api-deployment \
  swap-stat-api cpu=100m,memory=768Mi memory=2Gi

set_resources starcoin-indexer-swap-deployment \
  starcoin-indexer-swap cpu=50m,memory=256Mi memory=1Gi
