#!/usr/bin/env bash
# sweep_bench.sh — Layer 3 orchestrator
# Runs starcoin-execute-bench with different DAG parameter combos,
# collects TPS results into a single CSV.
#
# Usage:
#   ./scripts/sweep_bench.sh [--dry-run] [--output results.csv]
#
# Environment variables (override defaults):
#   BENCH_BIN       — path to starcoin-execute-bench binary
#   ACCOUNT_COUNT   — accounts per run (default: 2000)
#   ROUNDS          — benchmark rounds per run (default: 5)
#   BATCH_USER      — batch user count (default: 2000)
#
# The script reads parameter grids from arrays defined below.
# Edit the arrays to customize the sweep.

set -euo pipefail

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------
BENCH_BIN="${BENCH_BIN:-./target/release/starcoin-execute-bench}"
ACCOUNT_COUNT="${ACCOUNT_COUNT:-2000}"
ROUNDS="${ROUNDS:-5}"
BATCH_USER="${BATCH_USER:-2000}"
OUTPUT=""
DRY_RUN=false

# Parameter grid — edit these arrays
K_VALUES=(16 32)
MAX_PARENTS_VALUES=(10 16)
BLOCK_TIME_VALUES=(500 1000)
MAX_TXN_VALUES=(700 1400)
GAS_LIMIT_VALUES=(500000000 1000000000)

# ---------------------------------------------------------------------------
# Parse CLI args
# ---------------------------------------------------------------------------
while [[ $# -gt 0 ]]; do
    case "$1" in
        --dry-run)   DRY_RUN=true; shift ;;
        --output)    OUTPUT="$2"; shift 2 ;;
        --output=*)  OUTPUT="${1#*=}"; shift ;;
        --k)         IFS=',' read -ra K_VALUES <<< "$2"; shift 2 ;;
        --max-parents) IFS=',' read -ra MAX_PARENTS_VALUES <<< "$2"; shift 2 ;;
        --block-time)  IFS=',' read -ra BLOCK_TIME_VALUES <<< "$2"; shift 2 ;;
        --max-txn)     IFS=',' read -ra MAX_TXN_VALUES <<< "$2"; shift 2 ;;
        --gas-limit)   IFS=',' read -ra GAS_LIMIT_VALUES <<< "$2"; shift 2 ;;
        -h|--help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --dry-run              Print combos without running"
            echo "  --output FILE          Write CSV to FILE (default: stdout)"
            echo "  --k 16,32              K values (comma-separated)"
            echo "  --max-parents 10,16    max_parents values"
            echo "  --block-time 500,1000  block time in ms"
            echo "  --max-txn 700,1400     max txn per block"
            echo "  --gas-limit N,M        gas limit values"
            echo ""
            echo "Environment variables:"
            echo "  BENCH_BIN=$BENCH_BIN"
            echo "  ACCOUNT_COUNT=$ACCOUNT_COUNT"
            echo "  ROUNDS=$ROUNDS"
            echo "  BATCH_USER=$BATCH_USER"
            exit 0
            ;;
        *) echo "Unknown option: $1"; exit 1 ;;
    esac
done

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------
TMPDIR_BASE=$(mktemp -d)
trap 'rm -rf "$TMPDIR_BASE"' EXIT

CSV_HEADER="k,max_parents,block_time_ms,max_txn,gas_limit,tps,stable_tps,block_tps_avg,block_count,total_executed,avg_latency_ms,wall_time_s,status"

log() { echo "[sweep_bench] $(date '+%Y-%m-%d %H:%M:%S') $*" >&2; }

extract_from_json() {
    # Extract fields from benchmark_results.json using python (always available on macOS)
    local logdir="$1"
    local json="$logdir/benchmark_results.json"
    if [[ -f "$json" ]]; then
        python3 -c "
import json, sys
with open('$json') as f:
    d = json.load(f)
s = d.get('summary', {})
print(f\"{s.get('tps',0):.2f},{s.get('stable_tps',0):.2f},{s.get('block_tps_avg',0):.2f},{d.get('block_count',0)},{s.get('total_executed',0)},{s.get('avg_latency_ms',0):.2f}\")
" 2>/dev/null || echo "0,0,0,0,0,0"
    else
        echo "0,0,0,0,0,0"
    fi
}

# ---------------------------------------------------------------------------
# Grid generation (with constraint: max_parents <= K)
# ---------------------------------------------------------------------------
declare -a GRID_K GRID_P GRID_BT GRID_TXN GRID_GAS
idx=0
for k in "${K_VALUES[@]}"; do
    for p in "${MAX_PARENTS_VALUES[@]}"; do
        # Enforce max_parents <= K
        actual_p=$((p < k ? p : k))
        for bt in "${BLOCK_TIME_VALUES[@]}"; do
            for txn in "${MAX_TXN_VALUES[@]}"; do
                for gas in "${GAS_LIMIT_VALUES[@]}"; do
                    GRID_K[$idx]=$k
                    GRID_P[$idx]=$actual_p
                    GRID_BT[$idx]=$bt
                    GRID_TXN[$idx]=$txn
                    GRID_GAS[$idx]=$gas
                    idx=$((idx + 1))
                done
            done
        done
    done
done

TOTAL=$idx
log "Parameter grid: $TOTAL combinations"
log "Binary: $BENCH_BIN"
log "Accounts: $ACCOUNT_COUNT, Rounds: $ROUNDS, Batch: $BATCH_USER"

if [[ ! -x "$BENCH_BIN" ]] && ! $DRY_RUN; then
    log "ERROR: $BENCH_BIN not found or not executable. Build with: cargo build --release -p starcoin-execute-bench"
    exit 1
fi

# ---------------------------------------------------------------------------
# Output setup
# ---------------------------------------------------------------------------
if [[ -n "$OUTPUT" ]]; then
    echo "$CSV_HEADER" > "$OUTPUT"
    log "Results will be written to $OUTPUT"
else
    echo "$CSV_HEADER"
fi

# ---------------------------------------------------------------------------
# Run each combination
# ---------------------------------------------------------------------------
for ((i=0; i<TOTAL; i++)); do
    k=${GRID_K[$i]}
    p=${GRID_P[$i]}
    bt=${GRID_BT[$i]}
    txn=${GRID_TXN[$i]}
    gas=${GRID_GAS[$i]}

    tag="K=${k}_P=${p}_BT=${bt}_TXN=${txn}_GAS=${gas}"
    log "[$((i+1))/$TOTAL] Running: $tag"

    if $DRY_RUN; then
        row="$k,$p,$bt,$txn,$gas,0,0,0,0,0,0,0,dry-run"
        if [[ -n "$OUTPUT" ]]; then
            echo "$row" >> "$OUTPUT"
        else
            echo "$row"
        fi
        continue
    fi

    # Clean state for each run
    rm -rf /Users/manager/dev 2>/dev/null || true
    rm -rf ~/.starcoin 2>/dev/null || true

    run_dir="$TMPDIR_BASE/run_${i}"
    mkdir -p "$run_dir"
    run_log="$run_dir/bench.log"
    start_ts=$(date +%s)
    status="ok"

    if timeout 600 "$BENCH_BIN" \
        -c "$ACCOUNT_COUNT" \
        --rounds "$ROUNDS" \
        --batch-user-count "$BATCH_USER" \
        --fixed-block-time \
        --pipeline-timing \
        --override-block-time "$bt" \
        --override-max-txn "$txn" \
        --override-gas-limit "$gas" \
        --override-k "$k" \
        --override-max-parents "$p" \
        > "$run_log" 2>&1; then
        status="ok"
    else
        status="error($?)"
    fi

    end_ts=$(date +%s)
    wall=$((end_ts - start_ts))

    # Copy benchmark_results.json from CWD into run dir (bench writes it to CWD)
    [[ -f ./benchmark_results.json ]] && cp ./benchmark_results.json "$run_dir/"

    json_fields=$(extract_from_json "$run_dir")

    row="$k,$p,$bt,$txn,$gas,$json_fields,${wall},$status"
    IFS=',' read -r tps stable_tps _ _ _ _ <<< "$json_fields"
    log "[$((i+1))/$TOTAL] Done: TPS=$tps stable_TPS=$stable_tps wall=${wall}s status=$status"

    if [[ -n "$OUTPUT" ]]; then
        echo "$row" >> "$OUTPUT"
    else
        echo "$row"
    fi
done

log "Sweep complete. $TOTAL combinations tested."
if [[ -n "$OUTPUT" ]]; then
    log "Results saved to $OUTPUT"
fi
