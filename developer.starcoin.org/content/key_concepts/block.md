---
title: Block
weight: 3
---

A block contains a batch of the ordered [transactions](./transaction/) , as well as other key data:

<!--more-->

- **Parent hash** — The parent block hash, which chains the blocks.
- **Block number** — Block number, parent block number plus one.
- **State root** — Hash of the final state after execution of the block.
- **Transaction accumulator root** — The transaction accumulator root hash after executing this block.
- **Block accumulator root** — The hash after accumulating the IDs of all the previous blocks in order.