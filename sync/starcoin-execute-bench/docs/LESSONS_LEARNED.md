# Lessons Learned

> Document what worked, what didn't, and why. This helps avoid repeating mistakes.

## Successful Optimizations

### [Template]
**Date**: YYYY-MM-DD  
**Change**: What was changed  
**Result**: TPS improvement / latency reduction  
**Why it worked**: Technical explanation  
**Commit**: [link or SHA]

---

## Failed Attempts

### [Template]
**Date**: YYYY-MM-DD  
**Attempted**: What was tried  
**Result**: What happened  
**Why it failed**: Root cause analysis  
**Lesson**: What to avoid in the future

---

## Open Questions

- [ ] What's the optimal balance between VM parallelism and state contention?
- [ ] How does DAG depth affect Block Build performance?
- [ ] Can we batch State Commit across multiple blocks?

---

## Ideas to Explore

1. **Speculative Execution**: Execute transactions before consensus
2. **State Sharding**: Partition state tree for parallel commits
3. **Transaction Prioritization**: Fast-path for simple transfers
