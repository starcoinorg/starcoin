# Copilot Instructions for Starcoin

## TPS Optimization Knowledge

When working on TPS optimization tasks, **always read the optimization documentation first**:

```
sync/starcoin-execute-bench/docs/
├── OPTIMIZATION_KNOWLEDGE.md    # Read this first - learned strategies
├── BENCHMARK_HISTORY.md         # Historical results
├── LESSONS_LEARNED.md           # What worked and what didn't
└── adr/                         # Architecture decisions
```

### Workflow for AI Agents

1. **Before optimizing**: Read `OPTIMIZATION_KNOWLEDGE.md` to understand existing knowledge
2. **Run benchmarks**: Use `--agent-mode` flag for full analysis
3. **After success**: Update the docs with new learnings
4. **For big changes**: Create an ADR in `docs/adr/`

### Running Benchmarks

```bash
# Basic benchmark with agent analysis
cargo run -p starcoin-execute-bench -- --agent-mode

# With custom parameters
cargo run -p starcoin-execute-bench -- \
  --agent-mode \
  --batch-user-count 4000 \
  --account-count 20 \
  --tags "experiment,vm-tuning"
```

### Pipeline Stages

The transaction pipeline has 4 stages tracked by `starcoin-pipeline-timing`:

1. **TxPool Verify** - Transaction validation
2. **Block Build** - DAG block packaging  
3. **VM Execute** - Transaction execution (usually bottleneck)
4. **State Commit** - State persistence

### Key Files

- `sync/starcoin-execute-bench/` - Benchmark tool with agent mode
- `commons/pipeline-timing/` - Pipeline timing instrumentation
- `vm2/vm-runtime/` - VM execution (often the bottleneck)
- `state/statedb-v2/` - State storage

## Code Style

- Use `anyhow::Result` for error handling
- Prefer `log` macros over `println!`
- Run `cargo fmt` before committing
- Add `#[cfg(test)]` for test modules

## Dual VM Architecture

Starcoin has two VMs:
- **VM1**: Legacy Move VM (in `vm/`)
- **VM2**: New optimized VM (in `vm2/`)

Benchmark focuses on VM2 performance.
