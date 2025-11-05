#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 ]]; then
    echo "Usage: $0 <network> [--no-create]" >&2
    exit 1
fi

NETWORK=$1
shift

CREATE_ACCOUNTS=1
if [[ ${1:-} == "--no-create" ]]; then
    CREATE_ACCOUNTS=0
    shift
fi

COMPOSE_FILE="docker/docker-compose.yml"
if [[ ! -f "$COMPOSE_FILE" ]]; then
    echo "Missing $COMPOSE_FILE" >&2
    exit 1
fi

if docker compose version >/dev/null 2>&1; then
    COMPOSE_CMD=(docker compose -f "$COMPOSE_FILE")
elif command -v docker-compose >/dev/null 2>&1; then
    COMPOSE_CMD=(docker-compose -f "$COMPOSE_FILE")
else
    echo "Need docker compose installed." >&2
    exit 1
fi

SERVICES=(starcoin_1 starcoin_2 starcoin_3 starcoin_4)
PRIMARY_SERVICE=${SERVICES[0]}
DATA_DIR=/starcoin/data

if ! "${COMPOSE_CMD[@]}" exec -T "$PRIMARY_SERVICE" /bin/sh -c true >/dev/null 2>&1; then
    echo "Compose stack not running. Start it first." >&2
    exit 1
fi

run_starcoin() {
    local svc=$1
    shift
    "${COMPOSE_CMD[@]}" exec -T "$svc" /starcoin/starcoin -n "$NETWORK" -d "$DATA_DIR" "$@"
}

declare -a RECEIVERS=()

if [[ $CREATE_ACCOUNTS -eq 1 ]]; then
    for i in $(seq 1 10); do
        OUTPUT=$(run_starcoin "$PRIMARY_SERVICE" account create -p "")
        ADDRESS=$(printf '%s\n' "$OUTPUT" | sed -n 's/.*"address": "\(0x[0-9a-fA-F]*\)".*/\1/p' | tail -n1)
        if [[ -z "$ADDRESS" ]]; then
            echo "Failed to parse address #$i" >&2
            printf '%s\n' "$OUTPUT" >&2
            exit 1
        fi
        echo "New account $i: $ADDRESS"
        RECEIVERS+=("$ADDRESS")
    done
else
    OUTPUT=$(run_starcoin "$PRIMARY_SERVICE" account list)
    printf '%s\n' "$OUTPUT"
    RECEIVERS=($(printf '%s\n' "$OUTPUT" | sed -n 's/.*"address": "\(0x[0-9a-fA-F]*\)".*/\1/p'))
fi

if [[ ${#RECEIVERS[@]} -eq 0 ]]; then
    echo "No receiver accounts found." >&2
    exit 1
fi

while true; do
    SERVICE=${SERVICES[$((RANDOM % ${#SERVICES[@]}))]}
    RECEIVER=${RECEIVERS[$((RANDOM % ${#RECEIVERS[@]}))]}
    run_starcoin "$SERVICE" account unlock || true
    run_starcoin "$SERVICE" account transfer -r "$RECEIVER" -v 1000000000 --blocking
    sleep 0.5
done
