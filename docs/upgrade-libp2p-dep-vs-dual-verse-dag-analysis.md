# upgrade-libp2p-dep 分支变更分析（历史参考，对照 dual-verse-dag）

> 说明：这份分析文档原本用于 `upgrade-libp2p-dep` 分支评审，当前保留在 `refactor-with-jsonrpsee-v2` PR 中仅作为历史背景材料；它不描述本 PR 的实际变更范围。

## 对比范围

- 基线分支：`dual-verse-dag`
- 目标分支：`upgrade-libp2p-dep`
- merge-base：`06042c4d364d4929527464e3ce8c6248cbe56e4c`
- 对比区间：`06042c4..66d7360`
- 提交数：20
- 变更规模：83 files changed, `+5797 / -4407`

## 主要变更总结

1. `libp2p` 升级到 `0.56`，并完成 `network-p2p` 侧适配。
   - 入口依赖：`Cargo.toml`
   - 核心适配：`network-p2p/src/request_responses.rs`

2. 发现与连接稳定性修复（Kademlia/Discovery）。
   - Kademlia server mode 不再在生产路径无条件开启，仅测试可强制。
   - 关键位置：
     - `network-p2p/src/discovery.rs:103`
     - `network-p2p/src/discovery.rs:218`
     - `network-p2p/src/discovery.rs:1011`

3. generic proto 重连路径增强。
   - 缓存 `NewExternalAddrOfPeer` 的地址，在 outbound 无地址时兜底拨号。
   - 关键位置：
     - `network-p2p/src/protocol/generic_proto/behaviour.rs:109`
     - `network-p2p/src/protocol/generic_proto/behaviour.rs:1161`
     - `network-p2p/src/protocol/generic_proto/behaviour.rs:1628`

4. ping 错误处理改为非 panic。
   - `PingEvent` 的错误分支仅记录日志，避免崩溃。
   - 关键位置：`network-p2p/src/peer_info.rs:450`

5. sync 路径增强（可靠性/可恢复性）。
   - 新增 watchdog 机制，支持 stall 检测并触发 cancel + restart。
   - 引入 cancel flag 贯穿 sync task / parallel executor。
   - 并行执行支持 `execute_timeout_ms`，超时转为错误态并中止。
   - `get_block_ids` 支持 timeout 驱动的自适应缩容与稳定后扩容。
   - 关键位置：
     - `sync/src/sync.rs:735`
     - `config/src/sync_config.rs:50`
     - `types/src/sync_status.rs:71`
     - `sync/src/parallel/executor.rs:246`
     - `sync/src/tasks/block_sync_task.rs:554`
     - `sync/src/verified_rpc_client.rs:126`

6. DAG 颜色查询重构。
   - `BlockDAG` 抽出 `get_block_color` 与 `BlockColorError`。
   - `chain_service` 改为调用该 API，按错误类型决定返回 `None` 还是上抛错误。
   - 关键位置：
     - `flexidag/src/blockdag.rs:65`
     - `flexidag/src/blockdag.rs:997`
     - `chain/service/src/chain_service.rs:592`

7. RPC/CLI `network_manager.get_address` 类型回归为 `Vec<Multiaddr>`。
   - 关键位置：
     - `rpc/api/src/network_manager.rs:23`
     - `cmd/starcoin/src/node/network/get_address_cmd.rs:26`

## 分支中已覆盖的测试（已有）

1. `sync watchdog` 基础逻辑测试。
   - `sync/tests/sync_watchdog_test.rs`

2. `VerifiedRpcClient.get_block_ids` 自适应策略测试（缩容/下限/扩容）。
   - `sync/tests/verified_rpc_client_test.rs`

3. 并行执行 timeout 与 cancel 路径测试。
   - `sync/src/parallel/tests.rs`
   - `sync/src/tasks/tests.rs:428`（cancel by flag）

4. DAG block color 关键拓扑测试。
   - `flexidag/tests/tests.rs:1207`
   - `flexidag/tests/tests.rs:1237`
   - `flexidag/tests/tests.rs:1270`

## 建议补充或更新的测试用例

### P0（优先补齐）

1. `network-p2p`：`peer_info` ping error 回归测试。
   - 目标：覆盖 `PingEvent::Err` 分支，确保不 panic。
   - 位置：`network-p2p/src/peer_info.rs`

2. `network-p2p`：generic proto 地址兜底拨号测试。
   - 目标：覆盖 `NewExternalAddrOfPeer -> handle_pending_outbound_connection` 路径。
   - 位置：
     - `network-p2p/src/protocol/generic_proto/behaviour.rs`
     - `network-p2p/src/protocol/generic_proto/tests.rs`

3. `sync`：`get_block_ids` 非 timeout 错误不触发缩容测试。
   - 目标：确保只对 timeout 调整 batch size。
   - 位置：`sync/tests/verified_rpc_client_test.rs`

4. `sync`：`BlockCollector.fetch_blocks` 去重与本地优先命中测试。
   - 目标：验证重复 block id 去重、local/dag_store 命中优先、远端请求最小化。
   - 位置：`sync/src/tasks/block_sync_task.rs`

5. `sync service`：watchdog 集成测试。
   - 目标：模拟 stalled sync，验证 `cancel + restart` 触发以及状态回到 `Prepare`。
   - 位置：
     - `sync/src/sync.rs`
     - `types/src/sync_status.rs`

6. `block connector`：`PeerNewBlock` 分支覆盖。
   - 目标：覆盖 `AlreadyExecuted` 与 `TryLater`，确认 `TryLater` 会触发 sync 检查。
   - 位置：`sync/src/block_connector/execute_service.rs`

7. `chain service`：`BlockColorError` 映射行为测试。
   - 目标：校验“可识别颜色错误返回 None，其他错误继续上抛”。
   - 位置：`chain/service/src/chain_service.rs`

8. `rpc/cmd`：`network_manager.get_address` 类型契约回归测试。
   - 目标：防止 `String` 与 `Multiaddr` 类型在 API/Client/CMD 层再次漂移。
   - 位置：
     - `rpc/api/src/network_manager.rs`
     - `rpc/server/src/module/network_manager_rpc.rs`
     - `rpc/client/src/lib.rs`
     - `cmd/starcoin/src/node/network/get_address_cmd.rs`

### P1（可选增强）

1. `discovery` 的 server mode 行为测试。
   - 目标：确认生产默认不强制 server mode；测试路径可显式强制。
   - 位置：`network-p2p/src/discovery.rs`

2. `sync` watchdog 配置边界值测试。
   - 目标：`watchdog_interval_secs/watchdog_stall_secs/execute_timeout_ms` 的 0 值和默认回退逻辑。
   - 位置：`config/src/sync_config.rs`

3. `chain_get_block_txn_infos_in_seq` 稳定性测试扩展。
   - 目标：在慢路径下验证重试等待行为，不依赖固定 sleep。
   - 位置：`rpc/client/tests/chain_get_block_txn_infos_in_seq_test.rs`

## 备注

- 本文档基于代码与提交差异分析生成，未执行完整测试矩阵。
- 如需落地执行，建议先按 P0 清单补齐回归，再跑 `network-p2p`、`sync`、`flexidag`、`rpc/client` 的定向测试集。
