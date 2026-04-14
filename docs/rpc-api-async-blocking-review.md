# RPC API async/sync 审查记录

更新日期: 2026-03-03

## 背景

本记录用于回答以下问题:

1. `rpc/api` 的 jsonrpsee 桥接层是否存在明显问题。
2. 各 API 是否存在“用 async 包裹同步阻塞逻辑”的风险。

## 结论摘要

1. `rpc/api` 桥接层整体健康: 主要是 `Api::method(...).await` + 错误映射，不承载重逻辑。
2. `AccountApi` 链路无明显“RPC runtime 上直接阻塞”问题: 底层账户同步操作主要在独立 actor 线程执行。
3. `TxPoolApi` 是当前最明确的同步热点: `TxPoolSyncService` 多处同步调用后用 `ready/ok` 返回 future。
4. `Chain/State/Contract` 在 async 路径里包含较多同步存储读取/解码/ABI 解析，存在性能风险，但不属于签名层面的错误。
5. `DebugApi::sleep` 是显式阻塞行为(仅 dev/test 场景)，属于设计选择。
6. `PubSub` 的较重解码在 `PubSubService` actor 内执行，和 RPC handler 隔离。

## 模块级审查

### 1) 桥接层(`rpc/api`)

- 状态: 基本正常。
- 说明: `*ApiRpcServer` 实现主要做参数转发和 `map_jsonrpc_err`，未发现额外同步重活。
- 参考:
  - `rpc/api/src/account/mod.rs`
  - `rpc/api/src/chain/mod.rs`
  - `rpc/api/src/state/mod.rs`
  - `rpc/api/src/txpool/mod.rs`

### 2) Account API

- 状态: 主链路可接受。
- 说明:
  - RPC 层 `await AccountApi`。
  - `AccountApi` 实现调用 `AccountAsyncService`。
  - `AccountAsyncService` 通过 `ServiceRef::send` 向 actor 发消息，底层同步账户管理逻辑主要在 service actor 线程执行。
- 关注点(非本次迁移新引入):
  - `account/api/src/service.rs` 中对响应类型不匹配仍有 `panic!` 分支，建议后续改为结构化错误返回。

### 3) TxPool API

- 状态: 高优先级风险点。
- 说明:
  - `TxPoolRpcImpl` 大量直接调用 `TxPoolSyncService` 同步方法，然后通过 `futures::future::ready/ok` 包装结果。
  - 这会在处理请求的线程上执行同步逻辑，吞吐/尾延迟可能受影响。
- 参考:
  - `rpc/server/src/module/txpool_rpc.rs`

### 4) Chain / State / Contract API

- 状态: 中优先级性能风险点。
- 说明:
  - async 方法体中包含同步存储访问、状态构建、ABI/Move 解码等逻辑。
  - 功能正确，但在高并发或大数据请求下可能造成执行线程占用时间偏长。
- 参考:
  - `rpc/server/src/module/chain_rpc.rs`
  - `rpc/server/src/module/state_rpc.rs`
  - `rpc/server/src/module/contract_rpc.rs`

### 5) Node / NetworkManager / SyncManager / NodeManager / Miner

- 状态: 主路径可接受。
- 说明:
  - 多数走异步 service/actor 调用，未发现明显“async 包同步重活”的集中问题。
- 参考:
  - `rpc/server/src/module/node_rpc.rs`
  - `rpc/server/src/module/network_manager_rpc.rs`
  - `rpc/server/src/module/sync_manager_rpc.rs`
  - `rpc/server/src/module/node_manager_rpc.rs`
  - `rpc/server/src/module/miner_rpc.rs`

### 6) Debug API

- 状态: 已知阻塞行为。
- 说明:
  - `sleep` 直接调用时间服务阻塞，属于 debug/dev 用途，应保留场景限制。
- 参考:
  - `rpc/server/src/module/debug_rpc.rs`

### 7) PubSub

- 状态: 可接受。
- 说明:
  - 订阅解析与事件解码逻辑较重，但主要在 `PubSubService` actor 上执行，与 RPC handler 线程隔离。
- 参考:
  - `rpc/server/src/module/pubsub.rs`

## 建议优先级

1. P0: 优先治理 `TxPoolRpcImpl` 同步调用热点。
2. P1: 对 `Chain/State/Contract` 的重解码/重查询路径补充压测，必要时考虑隔离执行策略。
3. P2: 清理 `AccountAsyncService` 中 `panic!` 响应分支，统一改为可观测错误返回。

## 备注

本次记录聚焦“接口分层与阻塞风险”，不等同于完整性能评估报告。是否实际触发瓶颈，仍需结合压测数据确认。
