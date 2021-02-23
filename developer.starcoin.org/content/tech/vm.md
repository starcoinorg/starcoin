---
title: VM
weight: 5
---

Starcoin blockchain is a distributed programmable database designed to be used as a financial infrastructure. The only way to change the state of the database is to execute transactions on the Starcoin VM. The transaction can contain smart contract written with Move. Move is a programming language developed by Libra Core. The security design of Move is completely consistent with the design concept of Starcoin, so we use Move to write smart contracts.
<!--more-->

## VM Runtime

Transactions are sequentially verified and executed on the VM runtime. If you think of the VM runtime as a black box, then its input is the transaction and blockchain state, and its output is the blockchain state after the transaction is executed. The state here refers to the data and code stored in the Starcoin blockchain at a given block height. Smart contracts are written in Move, so Starcoin VM is actually developed based on MoveVM. In order to effectively use MoveVM's data and code cache, Starcoin VM uses a chain state wrapper to seamlessly interface the chain state with the MoveVM cache. In addition, Starcoin VM is developing a gas mechanism based on state billing to effectively increase the utilization of storage space. It is also exploring the improvement of smart contract security through formal verification. 

## Standard library

Every state change in the Starcoin blockchain occurs via executing a Move script embedded in a signed transaction. A transaction script invokes procedures of standard library to finish state changes. The Starcoin standard library consists of:
- The modules published in the genesis transaction. Contains the Move code for the core system modules. In addition to the most basic functions such as Account and Coin, it also supports user-defined tokens. For specific examples, please refer to XX.
- The authorized trasaction scripts that can be included in a Starcoin transaction. A transaction with an unauthorized script will be discarded.

## Interact with

Starcoin components interact with VM through executor. Executor uses the VM to execute a block of transactions. The transaction pool component uses verification function of the VM to discard invalid transactions before they are send to consensus. The consensus component then uses VM to execute transactions and write the new state into state DB.

## Folder Structure

```
├── executor                 # executor
├── vm
│   ├── vm_runtime           # vm runtime
│   ├── stdlib               # standard library
│   ├── functional_tests     # functinal tests

```