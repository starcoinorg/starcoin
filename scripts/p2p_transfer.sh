#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 ]]; then
    echo "Usage: $0 <network> [--no-create]" >&2
    echo "Example: $0 proxima" >&2
    echo "Example (reuse existing accounts): $0 proxima --no-create" >&2
    exit 1
fi

NETWORK=$1
NAMESPACE="starcoin-${NETWORK}"
shift

CREATE_ACCOUNTS=1
if [[ ${1:-} == "--no-create" ]]; then
    CREATE_ACCOUNTS=0
    shift
fi

declare -a RECEIVER_ACCOUNTS=()
if [[ $CREATE_ACCOUNTS -eq 1 ]]; then
    for i in $(seq 1 10); do
        OUTPUT=$(kubectl exec -n "$NAMESPACE" starcoin-0 -- /starcoin/starcoin -n "$NETWORK" -d /sc-data/ account create -p "")
        ADDRESS=$(printf '%s\n' "$OUTPUT" | sed -n 's/.*"address": "\(0x[0-9a-fA-F]*\)".*/\1/p' | tail -n1)
        if [[ -z "$ADDRESS" ]]; then
            echo "Failed to parse address from account create #$i output:" >&2
            printf '%s\n' "$OUTPUT" >&2
            exit 1
        fi
        echo "New account $i: $ADDRESS"
        RECEIVER_ACCOUNTS+=("$ADDRESS")
    done
else
    OUTPUT=$(kubectl exec -n "$NAMESPACE" starcoin-0 -- /starcoin/starcoin -n "$NETWORK" -d /sc-data/ account list)
    echo "Existing accounts:"
    printf '%s\n' "$OUTPUT"
    RECEIVER_ACCOUNTS=($(printf '%s\n' "$OUTPUT" | sed -n 's/.*"address": "\(0x[0-9a-fA-F]*\)".*/\1/p'))
fi

if [[ ${#RECEIVER_ACCOUNTS[@]} -eq 0 ]]; then
    echo "No account addresses were collected, exiting." >&2
    exit 1
fi


while true; do
    RECEIVER=${RECEIVER_ACCOUNTS[$((RANDOM % ${#RECEIVER_ACCOUNTS[@]}))]}
    kubectl exec -n "$NAMESPACE" starcoin-$((RANDOM % 3)) -- /starcoin/starcoin -n "$NETWORK" -d /sc-data/ account unlock || true
    kubectl exec -n "$NAMESPACE" starcoin-$((RANDOM % 3)) -- /starcoin/starcoin -n "$NETWORK" -d /sc-data/ account transfer -r "$RECEIVER" -v 1000000000 --blocking
    sleep 0.5
done
