---
id: keyconcepts
title: Key Concepts
---

This document briefly describes the key concepts of the Starcoin protocol. 

<!--more-->
## Account
An account represents a resource on the Starcoin that can send transactions. An account is a collection of Move resources stored at a particular 16-byte account address. 

### Addresses, authentication keys, and cryptographic keys
A Starcoin account is uniquely identified by its 16-byte account address. Each account stores an authentication key used to authenticate the signer of a transaction. An account’s address is derived from its initial authentication key, but the Diem Payment Network supports rotating the authentication key of an account without changing its address.

To generate an authentication key and account address:

Generate a fresh key-pair (pubkey_A, privkey_A). The Starcoin uses the PureEdDSA scheme over the Ed25519 curve, as defined in RFC 8032.
Derive a 32-byte authentication key auth_key = sha3-256(pubkey | 0x00), where | denotes concatenation. The 0x00 is a 1-byte signature scheme identifier where 0x00 means single-signature.
The account address is the last 16 bytes of auth_key.
Any transaction that creates an account needs both an account address and an auth key, but a transaction that is interacting with an existing account only needs the address.


## Transaction, Block and State

At the heart of the Starcoin protocol are three fundamental concepts —  transaction, block and state. At any point in time, the blockchain has a “state.” The state (or ledger state) represents the current snapshot of data on the chain. Executing a transaction changes the state of the blockchain. And a block contain many transactions, so an ordered batch of blocks can determine the final state. 

### Transaction

Clients of the Starcoin Blockchain submit transactions to request updates to the ledger state. A signed transaction on the blockchain mainly contains:

- **Sender address** — Account address of the sender of the transaction.
- **Sender public key** — The public key that corresponds to the private key used to sign the transaction.
- **Program** — The program is comprised of the following:
  - A Move bytecode transaction script.
  - An optional list of inputs to the script. For a peer-to-peer transaction, the inputs contain the information about the recipient and the amount transferred to the recipient.
  - An optional list of Move bytecode modules to publish.
- **Sequence number** — An unsigned integer that must be equal to the sequence number stored under the sender’s account.
- **Expiration time** — The time after which the transaction ceases to be valid.
- **Signature** — The digital signature of the sender.

### Block
A block contains a batch of the ordered [transactions](#Transaction) mentioned above, , as well as other key data:
- **Parent hash** — The parent block hash, which chains the blocks.
- **Block number** — Block number, parent block number plus one.
- **State root** — Hash of the final state after execution of the block.
- **Transaction accumulator root** — The transaction accumulator root hash after executing this block.
- **Block accumulator root** — The hash after accumulating the IDs of all the previous blocks in order.

### State

The final result of a transaction or block execution is indicated by its status. The ledger state, or global state of the Starcoin Blockchain, is comprised of the state of all accounts in the blockchain. 

## Proof

All of the data in the Starcoin Blockchain is stored in a single-versioned distributed database. Proof is used to determine whether a transaction or block is included in the blockchain.

In a blockchain, the client does not need to trust the entity from which it is receiving data. A client could query for the state of an account, ask whether a specific transaction or a specific block was processed, and so on. As with other Merkle trees, the ledger history can provide an O(log n)-sized proof of a specific transaction object, where _n_ is the total number of transactions processed.

## Full Node

Clients of the Starcoin Blockchain create transactions and submit them to a full node. Then the full node decides the order of transactions according to certain rules. A full node contains the following logical components:

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
