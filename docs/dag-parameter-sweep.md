# DAG Parameter Sweep Toolchain

用于 DAG 共识参数调优的数据采集工具链，提供"参数→吞吐量/安全指标"映射，不做结论。

## 架构概览

```
Layer 2: simnet DAG 模拟 (纯内存，无 VM)
  └─ sweep_dag_params CLI → CSV (red_rate, blocks/s, avg_parents, ...)

Layer 1: execute-bench VM 吞吐 (完整节点，单机)
  └─ starcoin-execute-bench --override-* → benchmark_results.json (tps, stable_tps, ...)

Layer 3: 编排脚本 (批量运行 Layer 1)
  └─ sweep_bench.sh → CSV (tps, stable_tps, block_tps_avg, ...)
```

## 快速开始

### Layer 2: DAG 纯模拟（秒级）

```bash
# 构建
cargo build -p simnet --bin sweep_dag_params

# 默认 24 组合网格
./target/debug/sweep_dag_params --default-grid --output dag_sweep.csv

# 自定义参数
./target/debug/sweep_dag_params \
  --k 8,16,32 \
  --max-parents 4,8,16 \
  --block-interval 100,500,1000 \
  --network-delay 50,200 \
  --miner-count 5 \
  --total-time 30000 \
  --output custom_sweep.csv
```

输出指标：
- `red_rate` — 红块率（安全性指标，越低越好）
- `blocks_per_second` — 出块吞吐
- `avg_parents` — 平均父块数
- `max_dag_width` — DAG 最大宽度
- `avg_commit_ms` — 平均 commit 延迟

### Layer 1: 单次 VM 吞吐测试（分钟级）

```bash
cargo build --release -p starcoin-execute-bench

./target/release/starcoin-execute-bench \
  -c 2000 --rounds 5 --batch-user-count 2000 \
  --fixed-block-time --pipeline-timing \
  --override-block-time 1000 \
  --override-max-txn 700 \
  --override-gas-limit 500000000 \
  --override-k 16 \
  --override-max-parents 10
```

结果写入 `./benchmark_results.json`，关键字段：
- `summary.tps` — 整体 TPS
- `summary.stable_tps` — 稳态 TPS（去掉首尾）
- `summary.block_tps_avg` — 每块平均 TPS

### Layer 3: 批量 VM 测试（小时级）

```bash
# 编辑脚本顶部数组或用 CLI 覆盖参数
bash scripts/sweep_bench.sh \
  --k 16,32 \
  --max-parents 10,16 \
  --block-time 500,1000 \
  --max-txn 700,1400 \
  --gas-limit 500000000 \
  --output results.csv

# dry-run 预览组合数
bash scripts/sweep_bench.sh --dry-run --k 16,32 --max-parents 10,16

# 使用 release 构建（推荐）
BENCH_BIN=./target/release/starcoin-execute-bench \
  bash scripts/sweep_bench.sh --output results.csv

# 小规模快速测试
ACCOUNT_COUNT=500 ROUNDS=1 BATCH_USER=500 \
  bash scripts/sweep_bench.sh --k 16 --max-parents 10 --block-time 1000 --max-txn 700 --gas-limit 500000000 --output quick.csv
```

## 参数说明

| 参数 | 范围建议 | 说明 |
|------|---------|------|
| K | 4-64 | GhostDAG 安全参数，K 越大容忍越多并发块 |
| max_parents | 4-K | 每块最多引用的父块数，必须 ≤ K |
| block_time | 200-5000ms | 出块间隔 |
| max_txn | 200-2000 | 每块最大交易数 |
| gas_limit | 1e8-2e9 | 块 gas 上限 |

## 约束

- `max_parents ≤ K`（BlockDAG 构建时断言检查）
- 每次 bench 运行前自动清理 `/Users/manager/dev` 和 `~/.starcoin`
- 单次运行超时 600s（可配置）

## 文件清单

- `simnet/src/scene/sweep.rs` — DAG 模拟核心
- `simnet/src/bin/sweep_dag_params.rs` — Layer 2 CLI
- `sync/starcoin-execute-bench/src/main.rs` — Layer 1（带 override 参数）
- `scripts/sweep_bench.sh` — Layer 3 编排脚本
