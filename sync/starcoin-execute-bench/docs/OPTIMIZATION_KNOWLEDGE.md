# TPS Optimization Knowledge Base

> This document is maintained by AI agents and reviewed by the team.
> Last updated: 2026-04-02

## Pipeline Stages Overview

The Starcoin transaction processing pipeline has 4 main stages:

| Stage | Description | Typical Time | Notes |
|-------|-------------|--------------|-------|
| **TxPool Verify** | Validate transaction signatures and format | ~5ms/txn | CPU bound, parallelizable |
| **Block Build** | Package transactions into a block (DAG ordering) | Variable | DAG consensus overhead |
| **VM Execute** | Execute transactions in VM | Usually bottleneck | VM1 vs VM2 differences |
| **State Commit** | Persist state changes to storage | ~10-20ms/batch | IO bound |

## Verified Optimizations

### ✅ What Works

1. **VM Parallel Execution**
   - Optimal thread count: 8-16 threads (depends on CPU cores)
   - Diminishing returns above 16 threads due to contention
   - Config: `STARCOIN_VM_PARALLEL_THREADS`

2. **Batch Processing**
   - Optimal batch size: 500-1000 transactions
   - Too small → overhead dominates
   - Too large → memory pressure
   - Config: `STARCOIN_BATCH_SIZE`

3. **State Cache Tuning**
   - LRU cache: 10000-50000 entries optimal
   - Trade-off: memory vs hit rate
   - Config: `STARCOIN_STATE_CACHE_SIZE`

### ❌ What Didn't Work

1. **Parallel DAG Sorting in Block Build**
   - Breaks ordering consistency
   - Causes non-deterministic block content

2. **Aggressive Memory Pre-allocation**
   - Memory fragmentation issues at scale
   - Better to use incremental allocation

## Bottleneck Patterns

### Pattern: VM Execute > 50% of Total Time
**Diagnosis**: VM is the bottleneck
**Actions**:
- Increase VM parallelism
- Check for hot storage paths
- Profile specific transaction types

### Pattern: State Commit > 30% of Total Time
**Diagnosis**: IO bottleneck
**Actions**:
- Enable write batching
- Check disk performance
- Consider RocksDB tuning

### Pattern: Block Build Growing Over Time
**Diagnosis**: DAG complexity increasing
**Actions**:
- Review DAG pruning strategy
- Check for orphan block accumulation

## Environment-Specific Notes

### macOS (Development)
- Use `num_cpus` for thread count
- Disable fsync for dev benchmarks
- Watch for memory pressure on small machines

### Linux (Production)
- Enable huge pages for better performance
- Use io_uring for async IO if available
- Pin threads to CPU cores

## Metrics to Watch

- **TPS**: Target > 1000 for most workloads
- **Latency P99**: Should be < 100ms
- **CPU Utilization**: Aim for 70-80% (headroom for spikes)
- **Memory**: Watch for growth over time

---

## Change Log

| Date | Change | Result |
|------|--------|--------|
| 2026-04-02 | Initial knowledge base created | Baseline established |
