---
title: 同步协议
weight: 1
---

Starcoin 链包含多种同步模式，这里介绍最常用的两种同步模式，分别是**全同步**与**快速同步**。 下面分别介绍这两种模式涉及到的同步流程及同步协议。
<!--more-->


## 区块同步

区块同步会同时在全同步和快速同步两种同步模式中使用，主要流程如下：

1. 使用GetBlockHeaders协议与best peer查找共同祖先
2. 使用GetBlockHeaders协议同步所有本地缺少的区块对应的BlockHeader
3. 使用GetBlockInfos协议同步所有本地缺少的区块对应的BlockInfo
4. 使用GetBlockBodies协议同步所有本地缺少的区块对应的BlockBody
5. 使用对应的BlockHeader、BlockInfo、BlockBody重新构建Block并根据BlockNumber由小到大添加到BlockChain



## 状态同步

状态同步只会在快速同步模式中使用，主要流程如下：

1. 使用GetBlockHeaders协议与best peer确定pivot对应的BlockHeader
2. 根据BlockHeader中的state_root，使用GetStateNodeByNodeHash协议递归获取相应状态树的所有StateNode
3. 根据BlockHeader中的accumulator_root，使用GetAccumulatorNodeByNodeHash协议递归获取相应交易累加器的所有AccumulatorNode
4. 根据BlockHeader中的parent_block_accumulator_root，使用GetAccumulatorNodeByNodeHash协议递归获取相应区块累加器的所有AccumulatorNode



## 同步协议

全同步和快速同步涉及到的所有协议如下：

### GetBlockHeaders

`[block_id: Hash, max_size: usize, step: usize, reverse: bool]`

要求节点响应 [BlockHeaders] 消息。 响应值包含BlockHeader列表， `reverse` 为 `false`表示升序， 为 `true`表示降序， `step` 表示跳过的区块步长，  `block_id` 表示开始的区块哈希，  `max_size` 表示最大值。

### BlockHeaders

`[blockHeader_0, blockHeader_1, ...]`

响应 [GetBlockHeaders]请求。列表数据是Starcoin规范中描述的BlockHeader。

### GetBlockInfos

`[block_id_0, block_id_1, ...]`

要求节点响应 [BlockInfos] 消息。

### BlockInfos

响应 [GetBlockInfos]请求。列表数据是Starcoin规范中描述的BlockInfo。

### GetBlockBodies

`[block_id_0, block_id_1, ...]`

要求节点响应 [BlockBodies] 消息。

### BlockBodies

`[[transactions_0, block_id_0] , ...]`

响应 [GetBlockBodies]请求。列表数据是BlockBody。

### GetStateNodeByNodeHash

`node_hash: Hash`

要求节点响应 [StateNode] 消息。

### StateNode

响应 [GetStateNodeByNodeHash]请求。

### GetAccumulatorNodeByNodeHash

`node_hash: Hash`

要求节点响应 [AccumulatorNode] 消息。

### AccumulatorNode

响应 [GetAccumulatorNodeByNodeHash]请求。