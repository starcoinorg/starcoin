# RPC TCP listener 恢复说明

> 状态：已实现。本文记录 `refactor-with-jsonrpsee-v2` 上 TCP RPC listener 的缺口、修复方式和当前行为边界，避免后续再次退化为补丁式处理。

## 背景

Starcoin 在旧 JSON-RPC 栈下同时提供：

- HTTP RPC
- WebSocket RPC
- IPC RPC
- raw TCP RPC

在切换到 `jsonrpsee` 之后，HTTP / WS / IPC 已完成迁移，但 TCP listener 被移除，而配置面仍然保留：

- `rpc.tcp.port`
- `rpc.tcp.apis`
- 默认 `tcp://0.0.0.0:9860`

这导致“配置还在，但实现不在”。

## 旧问题

### 1. TCP 配置面保留，但 listener 不再启动

`rpc/server/src/service.rs` 在迁移后的中间态里不再调用 `start_tcp()`，只启动：

- `start_http()`
- `start_ws()`
- `start_ipc()`

结果是：

- 用户仍然可以配置 `rpc.tcp`
- 节点日志仍然会打印 TCP endpoint
- 但进程不会真正绑定 TCP RPC 端口

### 2. fail-fast 只是缓解 silent regression，不是修复

为避免“配置了 TCP 但悄悄不生效”，中间态引入了一条 fail-fast 逻辑：

- 如果检测到显式配置了 TCP endpoint
- 启动阶段直接报错退出

这比 silent regression 更安全，但它只解决了“别悄悄失效”，没有恢复“TCP endpoint 正常工作”。

### 3. 不能把 `tcp://` 简单重解释为 HTTP

`jsonrpsee` 自带的 server backend 是基于 TCP listener 的 HTTP / WS 传输。

但 Starcoin 这里的 `tcp://` 历史语义不是：

- `http://...`
- 也不是 `ws://...`

而是 raw JSON-RPC over stream TCP。

如果把 `tcp://` 配置直接重绑到 HTTP server，会带来两个问题：

- 旧配置语义被悄悄改变
- transport 行为会和历史 TCP 客户端预期脱节

因此，这次修复不采用“把 HTTP 伪装成 TCP”的方案。

## 解决方法

### 1. 新增独立的 raw TCP transport crate

新增 `commons/rpc/tcp`，把 TCP transport 相关职责收敛进去：

- `TcpListener` 接入与连接生命周期
- raw JSON frame 读取与写回
- `jsonrpsee` method dispatch
- 每连接 `Extensions` 注入

这样 TCP 恢复不是在 `rpc/server` 里继续堆 transport 细节，而是形成一个独立 transport 层。

### 2. 复用现有 stream framing 语义，而不是重造一套协议

新的 TCP transport 复用了现有 `StreamCodec`：

- 输入侧使用 `stream_incoming()`
- 支持 raw 拼接的 JSON 请求流
- 也兼容 newline-delimited 输入

这让 TCP 与 IPC 共用同一套 stream framing contract，而不是各自维护一份近似实现。

### 3. 保留连接级 metadata，而不是只恢复“能监听”

旧 TCP RPC 会把 `peer_addr.ip()` 注入到请求 metadata。

迁移后的速率限制和审计逻辑依赖 `Metadata.user`，因此这次恢复不仅要“把端口绑起来”，还要恢复这条语义。

当前做法是：

- `commons/rpc/tcp` 只接受一个连接级 `Extensions` builder
- `rpc/server` 在 TCP 连接建立时，把 `peer_addr.ip()` 写入 `Metadata.user`

这样 transport 层保持通用，业务元数据仍由 `rpc/server` 决定。

### 4. `rpc/server` 恢复真实 `start_tcp()`

`rpc/server/src/service.rs` 现在重新：

- 按 `rpc.tcp.apis()` 组装 methods
- 绑定配置的 TCP 地址
- 启动 `starcoin-rpc-tcp` server
- 在关闭流程里显式 stop TCP handle

这意味着 TCP endpoint 再次回到和 HTTP / WS / IPC 对等的启动路径，而不是特殊 case。

## 新行为

### 1. 显式配置 TCP endpoint 会真正启动 listener

现在如果配置了 `rpc.tcp`，节点会：

- 真正绑定 TCP RPC 地址
- 启动 raw TCP JSON-RPC server
- 不再因为“当前 backend 不支持”而 fail-fast

### 2. `tcp://` 语义仍然是 raw JSON-RPC/TCP，不是 HTTP

当前实现明确保持：

- `tcp://` 不是 HTTP endpoint
- 不提供 HTTP request parsing
- 不提供 HTTP health path
- 不走 HTTP metadata middleware

这条语义在实现和文档里都应被视为稳定 contract。

### 3. TCP 上的 `user` 只信任 socket peer，不信任 forwarded headers

HTTP 侧是否信任 `X-Forwarded-For` / `X-Real-IP` 取决于独立配置。

TCP 侧没有这层概念。当前行为是：

- `Metadata.user = peer_addr.ip().to_string()`
- 不读取任何 forwarded IP header
- 不存在 proxy header trust boundary

这和 HTTP/WS 的元数据路径是明确区分的。

### 4. 当前 framing contract

新 TCP transport 使用共享的 `StreamCodec`，因此当前 contract 是：

- 输入：兼容 raw 拼接 JSON 和 newline-delimited JSON
- 输出：响应采用 newline-delimited frame

如果未来要兼容其他 framing 形式，应在 transport 层显式扩展，而不是在 `rpc/server` 再堆额外判断。

## 这次修复刻意避免的方案

### 1. 保留 fail-fast，不恢复 listener

原因：

- 只能避免 silent regression
- 不能恢复用户配置的 TCP RPC 能力
- review comment 的本意仍未满足

### 2. 直接把 `tcp://` 绑定到 `jsonrpsee` HTTP server

原因：

- 会改变 `tcp://` 的历史语义
- 会把 TCP transport 和 HTTP middleware 不必要地耦合起来
- 只是“看起来恢复了端口”，并不是真正恢复 raw TCP RPC

### 3. 在 `rpc/server` 里继续堆 transport 细节

原因：

- 会让 service 层重新承担 listener、connection、framing、dispatch 等职责
- 以后修 TCP / IPC 行为时更容易再次出现局部补丁
- 无法形成清晰的 transport 边界

## 验证点

这次修复至少覆盖了以下验证点：

1. `commons/rpc/tcp` 能直接处理 raw TCP JSON-RPC 请求
2. TCP 连接级 metadata 能注入到 `Request.extensions`
3. `rpc/server` 的 TCP metadata 映射会把 peer IP 写入 `Metadata.user`

后续如果 TCP 语义继续扩展，新增行为也应优先补在 transport 层及本文档中，而不是继续在启动逻辑上叠补丁。
