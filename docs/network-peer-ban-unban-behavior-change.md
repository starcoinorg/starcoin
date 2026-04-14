# network-p2p peer ban/unban 行为变更记录

> 状态：设计记录。用于说明 `refactor-with-jsonrpsee-v2` 上计划收敛的 ban/unban 语义；本文描述目标行为，不表示该行为已全部实现。

## 背景

`network-p2p` 当前同时存在：

- `RPC / NetworkService` 暴露的手工 `ban_peer(peer, bool)` 入口
- `peerset` 里基于 reputation 的 ban 阈值与临时 ban 通知
- `worker` 里额外维护的一条 `unbans` 定时路径

这三条路径没有形成单一真相源，导致“断连了”与“真正被 ban 了”被混用。

## 旧行为

### 1. 手工 ban 只会断连，不会建立 ban 状态

- 入口：`network-p2p/src/service.rs`
- `NetworkWorker::ban_peer(&PeerId)` 当前只调用 `disconnect_peer_id`
- `NetworkWorker::unban_peer(&PeerId)` 当前是空操作
- `ServiceToWorkerMsg::BanPeer(ban, peer_id)` 在 `ban == true` 时只断连，在 `ban == false` 时不做任何事

结果：

- `ban_peer(peer, true)` 不会阻止该 peer 重新入站连接
- 也不会阻止 peerset 后续再次主动 outbound 连接该 peer
- `ban_peer(peer, false)` 不能恢复任何明确的 ban 状态，因为实际上没有状态被建立

### 2. 临时 ban 通知存在，但解封不闭环

- 入口：`peerset/src/lib.rs`
- peerset 在 peer reputation 低于 `BANNED_THRESHOLD` 后会发出 `Message::Banned(peer_id, UNBANNED_AFTER)`
- 该消息会经由 `protocol/generic_proto/behaviour.rs` 转成 `GenericProtoOut::Banned`
- 再经 `network-p2p/src/service.rs` 转成 `BehaviourOut::BannedRequest(peer_id, duration)`

当前 worker 收到 `BannedRequest` 后：

- 会立即断开当前连接
- 会把 `peer_id` 放入本地 `unbans` timer 队列
- 但 timer 到期后只执行 `let _ = peer_id;`

结果：

- `duration` 没有真正驱动任何“解封动作”
- 所谓 temporary ban 的生效与解除，实际上仍然依赖 peerset 自己的 reputation/decay 路径
- worker 里的 `unbans` 路径只是一个不完整的旁路实现

### 3. ban 决策不集中，容易出现分裂语义

当前真正控制“是否允许入站 / 是否继续 outbound”的逻辑仍在 peerset：

- 入站准入：`peerset.incoming(...)` 会拒绝 reputation 低于 `BANNED_THRESHOLD` 的 peer
- outbound 分配：`alloc_slots()` 不会选择 reputation 低于 `BANNED_THRESHOLD` 的 peer

但手工 `ban_peer` 没有更新 peerset 状态，只在 worker 侧断连。

结果：

- worker 认为“ban 过了”
- peerset 却仍然认为该 peer 可以连接
- 整体表现为“ban 之后还能回来”

## 变更目标

这次行为收敛的核心目标是：

1. ban/unban 语义只由 peerset 统一决定
2. 手工 ban 与临时 ban 要成为显式状态，而不是“碰巧断连”
3. `ban_peer(peer, false)` 只解除手工 ban，不重写历史 reputation
4. 临时 ban 的持续时间要真正由 peerset 维护，不再依赖 worker 的假定时器
5. 手工 ban 触发的断连不能再被当成普通网络断连而额外扣 reputation

## 改动方案

### 1. 把 ban 状态收敛到 peerset

在 peerset 内维护两类 ban：

- `manual ban`
  - 来源：RPC / 管理员显式调用 `ban_peer(peer, true)`
  - 解除：`ban_peer(peer, false)`
- `temporary ban`
  - 来源：peer reputation 低于 `BANNED_THRESHOLD` 后进入临时 ban
  - 解除：到达过期时间后由 peerset 自己清理

设计上，是否允许某个 peer 连接，应统一经过一个 peerset 级判定，例如：

- peer 是否处于 `manual ban`
- peer 是否仍处于 `temporary ban`
- peer reputation 是否仍低于 `BANNED_THRESHOLD`

### 2. 手工 ban/unban 改为 peerset action

`ban_peer(peer, true)` 的目标行为：

- 在 peerset 中标记该 peer 为 manual ban
- 对当前活跃连接发出 drop
- 后续入站连接直接拒绝
- 后续 outbound 分配不再选择该 peer

`ban_peer(peer, false)` 的目标行为：

- 仅移除 manual ban 标记
- 不直接把 reputation 重置为 0
- 不清洗历史 reputation
- 后续是否重新连接，交回 peerset 的正常分配逻辑决定

### 3. 去掉 worker 本地 `unbans` 的伪状态

`worker` 当前维护的 `unbans` 队列不是权威 ban 状态源。

目标改动：

- ban 的到期检查回到 peerset
- worker 不再维护“本地解封”语义
- `BehaviourOut::BannedRequest(peer, duration)` 只作为事件/通知使用，真正状态变化由 peerset 完成

### 4. 区分管理性 drop 与普通网络掉线

当前 peerset 在 `dropped(..., Unknown)` 路径里会追加 `DISCONNECT_REPUTATION_CHANGE`。

如果手工 ban 触发的断连也走这条路径，就会出现额外副作用：

- 管理员只想“封禁”
- 系统却又额外把 peer reputation 再扣一层

目标改动：

- hand-admin ban 导致的断连应被单独识别
- 这类断连只更新连接状态，不额外施加普通网络掉线惩罚

## 新行为

### 1. `ban_peer(peer, true)`

新行为应满足：

- 立即断开该 peer 的现有连接
- peerset 明确记录该 peer 处于 manual ban
- 该 peer 的新入站连接会被拒绝
- peerset 不会再主动向该 peer 发起 outbound 连接
- 不依赖 worker 的局部状态维持 ban

### 2. `ban_peer(peer, false)`

新行为应满足：

- 解除 manual ban 标记
- 不把 peer reputation 强制重置到 0
- 不丢失历史 reputation 轨迹
- 解除后是否立刻重连，由 peerset 的正常 slot 分配和 reputation 判定决定

### 3. 协议层触发的 temporary ban

新行为应满足：

- temporary ban 在 peerset 中拥有明确的有效期
- 有效期内，peer 入站被拒绝、outbound 被抑制
- 到期后由 peerset 自动解除
- worker 不再需要维护一套假的 `unbans` timer

### 4. 单一真相源

新行为收敛后：

- “这个 peer 现在是否被 ban” 只由 peerset 判断
- worker 不再单独维护一套 ban 语义
- 断连只是 ban 的一个副作用，而不是 ban 本身

## 不采用的方案

以下方案明确不采用：

### 1. `ban=true` 时直接把 reputation 设为极小值，`ban=false` 再改回 0

原因：

- 会把“管理员操作”与“行为评分”混成一个维度
- `unban` 会抹掉历史 reputation
- 会让 `ban_peer(peer, false)` 产生意外的“洗白”效果

### 2. 继续在 worker 层补 `disconnect_peer_id`

原因：

- 只能得到“暂时断开”
- 无法真正控制后续入站/出站准入
- 仍然会和 peerset 的准入逻辑分裂

## 实现时的验证点

后续实现 peer ban/unban 语义时，至少应覆盖这些回归点：

1. manual ban 后，peer 的入站连接被拒绝
2. manual ban 后，peerset 不再主动 outbound 连接该 peer
3. manual unban 后，peer 可以重新进入正常准入流程
4. temporary ban 在整个 `UNBANNED_AFTER` 窗口内持续生效
5. admin ban 导致的 drop 不会额外污染 reputation

