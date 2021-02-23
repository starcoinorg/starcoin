---
title: Sync protocols
weight: 1
---

The Starcoin block chain contains a variety of sync modes, the two most common of which are described here **full sync**，**fast sync**.The following describes the sync process and sync protocols involved in each of these two modes.
<!--more-->

## Sync Block

Sync block is used in both full and fast sync modes, and the main process is as follows: 

1. Find common ancestors with best peer using GetBlockHeaders protocol. 
2. Use the GetBlockHeaders protocol to sync the BlockHeader for all missing local blocks. 
3. Use the GetBlockInfos protocol to sync the BlockInfo for all missing local blocks. 
4. Use the GetBlockBodies protocol to sync the BlockBody for all missing local blocks. 
5. Rebuild the block using the corresponding BlockHeader, BlockInfo, BlockBody and add it to BlockChain based on the BlockNumber from small to large. 



## Sync State

Syne state is only used in fast sync mode and the main process is as follows: 

1. Determine the BlockHeader corresponding to the pivot using the GetBlockHeaders protocol with the best peer. 
2. Get all StateNodes of the corresponding state tree recursively using the GetStateNodeByNodeHash protocol according to the state_root in BlockHeader. 
3. Get all AccumulatorNodes of the corresponding transaction accumulator recursively using the GetAccumulatorNodeByNodeHash protocol, according to accumulator_root in BlockHeader. 
4. Get all AccumulatorNodes of the corresponding transaction accumulator recursively using the GetAccumulatorNodeByNodeHash protocol, according to parent_block_accumulator_root in BlockHeader. 



## Sync Protocols

All protocols involved in full synchronization and fast synchronization are as follows: 

### GetBlockHeaders

`[block_id: Hash, max_size: usize, step: usize, reverse: bool]`

Require peer to return a [BlockHeaders] message. Reply must contain a number of block headers, of rising number when `reverse` is `false`, falling when `true`, step blocks apart, beginning at block `block_id` in the canonical chain, and with at most `max_size` items.

### BlockHeaders

`[blockHeader_0, blockHeader_1, ...]`

Reply to [GetBlockHeaders]. The items in the list are BlockHeader in the format described in the main Starcoin specification, previously asked for in a GetBlockHeaders message.

### GetBlockInfos

`[block_id_0, block_id_1, ...]`

Require peer to return a [BlockInfos] message.

### BlockInfos

Reply to [GetBlockInfos]. The items in the list are BlockInfo in the format described in the main Starcoin specification, previously asked for in a GetBlockInfos message.

### GetBlockBodies

`[block_id_0, block_id_1, ...]`

Require peer to return a [BlockBodies] message.

### BlockBodies

`[[transactions_0, block_id_0] , ...]`

Reply to [GetBlockBodies]. The items in the list are BlockBody, previously asked for in a GetBlockBodies message.

### GetStateNodeByNodeHash

`node_hash: Hash`

Require peer to return a [StateNode] message.

### StateNode

Reply to [GetStateNodeByNodeHash]. 

### GetAccumulatorNodeByNodeHash

`node_hash: Hash`

Require peer to return a [AccumulatorNode] message.

### AccumulatorNode

Reply to [GetAccumulatorNodeByNodeHash]. 