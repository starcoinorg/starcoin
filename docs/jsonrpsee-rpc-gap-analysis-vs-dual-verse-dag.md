# RPC Server/Client/API 变动对比与功能缺口反思（基线：dual-verse-dag）

## 范围与基线

- 对比分支：`dual-verse-dag...HEAD`
- 关注目录：`rpc/server`、`rpc/client`、`rpc/api`，以及强相关的 `config`、`node/src/rpc_service_factory.rs`

## 当前状态（已更新）

相对 `dual-verse-dag`，`jsonrpc -> jsonrpsee` 迁移在本分支的 RPC 主链路已基本打通：

1. 服务端已恢复 `IPC + HTTP + WS` 启动链路（jsonrpsee backend）。
2. PubSub 已完成 jsonrpsee 版接线，`starcoin_subscribe/starcoin_unsubscribe` 可注册。
3. `ApiRegistry` 对“配置请求但未注册”的 API 增加 warning，避免静默失效。
4. 客户端 `jsonrpsee-ws-client` 已启用 TLS feature，具备 `wss://` 客户端能力。
5. `tcp` 端点在当前实现中不再支持；配置到 `tcp` 时会明确打印不支持告警。
6. `RpcService` 启动流程已改为通过专用 runtime 投递异步建链，规避运行时嵌套 `block_on` 导致的 WS/IPC 启动阻塞问题。
7. `client_server_test` 已补充串行锁与订阅稳定性处理，`reconnect/reconnect_subscribe/multi_client` 在本地可稳定通过。
8. 服务端已补齐 `HTTPS + WSS` TLS listener，证书由 `--rpc-tls-cert-path/--rpc-tls-key-path` 配置；`client_server_test` 已新增端到端 TLS 集成测试覆盖。

## 能力对照表

| 能力 | `dual-verse-dag` | 当前分支 |
| --- | --- | --- |
| IPC server | 支持 | 支持 |
| HTTP server | 支持 | 支持 |
| WS server | 支持 | 支持 |
| TCP server | 支持 | 不支持，配置时仅告警 |
| HTTPS server | 仓库内未见内建支持 | 支持 |
| WSS server | 仓库内未见内建支持 | 支持 |
| PubSub | 支持，旧 `jsonrpc_pubsub` | 支持，已迁到 `jsonrpsee` |
| WS 重连回归测试 | 有基础覆盖 | 有，且已稳定化 |
| WS 订阅重连测试 | 有基础覆盖 | 有，且已稳定化 |
| HTTPS/WSS 集成测试 | 未见 | 已新增 |
| TLS 证书配置 | 未见 `rpc tls cert/key` 配置 | `--rpc-tls-cert-path` / `--rpc-tls-key-path` |
| TLS 热更新 | 未见 | 不支持 |
| SNI/多证书 | 未见 | 不支持 |
| mTLS | 未见 | 不支持 |

结论：

1. `HTTP/WS/IPC/PubSub` 主能力已经恢复。
2. `TLS` 能力不是“与升级前相同”，而是“当前分支更强”，因为现在明确支持 `HTTPS/WSS`。
3. `TCP` 不是等价迁移，这是当前相对 `dual-verse-dag` 最明显的能力缺口。

## Server 对比

### dual-verse-dag（基线）

- 基于旧 jsonrpc 体系，包含 `ipc/http/tcp/ws` 及对应生命周期控制。
- PubSub 为旧 `jsonrpc_pubsub` 实现。

### 当前分支（refactor-with-jsonrpsee）

- `RpcService` 持有并管理 `ipc/http/ws` 的 `ServerHandle`。
- 生命周期中会按配置启动 `ipc/http/ws`，并在关闭时统一 stop。
- `tcp` 不再进入启动链路，转为显式告警（避免“配置可写但行为不确定”）。
- 当配置 TLS 证书与私钥时，`http/ws` 会分别以 `https/wss` 形式启动，并通过 rustls 终止 TLS。

## PubSub 对比

### dual-verse-dag（基线）

- 旧 jsonrpc pubsub 路径，方法名与订阅模型为历史实现。

### 当前分支（refactor-with-jsonrpsee）

- 已替换为 jsonrpsee subscription 注册方式。
- 已在 server module 导出并由 rpc service 注册。
- 已在 node 侧恢复 `PubSubService` factory 注入。
- 保留了对历史参数形态的兼容解析（降低老客户端断裂风险）。

## Client 对比

### dual-verse-dag（基线）

- 旧 jsonrpc 客户端栈。

### 当前分支（refactor-with-jsonrpsee）

- `ws://` 与订阅路径使用 jsonrpsee 客户端。
- `wss://` 依赖 TLS feature；当前已在 `rpc/client/Cargo.toml` 显式启用。
- 已新增基于自签证书的 `https/wss` 集成测试，验证服务端 TLS listener 与客户端握手链路。

## 仍需关注的缺口

1. **TCP 传输协议**：当前后端不支持；若业务仍要求 TCP，需要单独设计/实现（非 jsonrpsee 现成能力）。
2. **TLS 证书运维策略**：当前支持静态证书/私钥文件加载，但未提供热更新、SNI、多证书或客户端证书校验能力。
3. **端到端测试稳定性**：当前关键回归用例已在本地通过，仍需在 CI 多平台（Linux/macOS）持续观察长稳性。

## 建议的后续工作

1. 增加可在 CI 稳定运行的 RPC 集成测试（独立端口策略 + 串行化，覆盖 HTTP/WS/PubSub）。
2. 在配置文档中明确 `tcp` 的现状（不支持/废弃），避免误配。
3. 若产品继续增强 TLS 能力，补充证书热更新、多证书/SNI、mTLS 等运维特性。
