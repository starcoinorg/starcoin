# Starcoin TPS Benchmark

A benchmark tool for measuring Starcoin transaction throughput (TPS).

## Quick Start

### Basic Benchmark (Recommended for CI)

```bash
cargo run -rp starcoin-execute-bench -- \
    --account-count 4000 \
    --simple-transfer \
    --agent-mode
```

This will:
- Create 4000 test accounts
- Use simple P2P transfers (lower gas, more txns per block)
- Output detailed analysis including TPS, latency, pipeline stages

### Expected Output

```text
========== Benchmark Results ==========
TPS (executed-time): ~900-1100
TPS (per-block, block_ts->exec) - Min/Max/Avg
TPS (per-block, mined->exec) - Min/Max/Avg (peak ~8000+)
Total Executed: 2000
Latency - Min/Max/Avg/Median (ms)
========================================
```

## CLI Options

| Option | Default | Description |
|--------|---------|-------------|
| `--account-count`, `-c` | 20 | Number of test accounts to create |
| `--simple-transfer` | false | Use simple P2P transfer (recommended) |
| `--agent-mode` | false | Enable detailed analysis output |
| `--network`, `-n` | custom | Network: custom, halley, proxima, etc. |
| `--balance-wait-timeout-secs` | 600 | Timeout for funding phase |

### Full Options

```bash
cargo run -rp starcoin-execute-bench -- --help
```

## Output Files

- `benchmark_results.json` - Detailed benchmark results
- `agent_output.json` - Agent analysis (when `--agent-mode` enabled)

## CI Integration

For CI pipelines, compare benchmark results between branches:

1. Run benchmark on the target branch
2. Run benchmark on the base branch (e.g., `dual-verse-dag`)
3. Compare `benchmark_results.json` outputs

Key metrics to monitor:
- **TPS (executed-time)**: Primary throughput metric
- **Latency avg**: Transaction confirmation time
- **Pipeline stages**: Identify bottlenecks

## Architecture

The benchmark measures 4 pipeline stages:

| Stage | Description |
|-------|-------------|
| TxPool Verify | Transaction validation in mempool |
| Block Build | Block template construction |
| VM Execute | Transaction execution |
| State Commit | State persistence |

## Typical Results

On a standard development machine:

| Metric | Value |
|--------|-------|
| End-to-End TPS | ~900-1100 |
| Block TPS (peak) | ~8000+ |
| VM Execute | ~1M+ TPS |
| Latency (avg) | ~3-4 seconds |
