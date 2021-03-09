---
title: Full Node
weight: 6
---

Clients of the Starcoin Blockchain create transactions and submit them to a full node. Then the full node decides the order of transactions according to certain rules. A full node contains the following logical components:

<!--more-->

**Txpool**

- Txpool is a buffer that holds the transactions that are “waiting” to be executed.
- When a new transaction is added to a node’s txpool, this node’s txpool shares this transaction with other nodes.

**Consensus**

- The consensus component is responsible for ordering blocks of transactions and agreeing on the results of execution by participating in the consensus protocol with other nodes in the network.

**BlockChain**

- BlockChain maintains the internal state of the chain, providing context for other components to function properly.

**Executor**

- The executor component utilizes the virtual machine (VM) to execute transactions.

**Virtual Machine (VM)**

- Txpool use the VM component to perform validation checks on transactions.
- VM is used to run the program included in a transaction and determine the results.

**Miner**

- Calculate hash by certain rules.

**Storage**

- The storage component is used to persist agreed upon blocks of transactions and their execution results.
