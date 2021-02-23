---
weight: 3
title: "Features"
---

Starcoin's main features.

<!--more-->

## POW Consensus

The StarCoin pow consensus is base on a variant of Bitcoin’s Nakamoto Consensus to achieve network consensus. The pow algorithm design imperatives of novelty, simplicity, and security, and it lowers the barrier for hardware development.

## MOVE Programming Language

Move, a safe and flexible programming language for the StarCoin Blockchain. Move is an executable bytecode language used to implement custom transactions and smart contracts. The key feature of Move is the ability to define custom resource types with semantics inspired by linear logic: a resource can never be copied or implicitly discarded, only moved between program storage locations. These safety guarantees are enforced statically by Move’s type system. Despite these special protections, resources are ordinary program values — they can be stored in data structures, passed as arguments to procedures, and so on. First-class resources are a very general concept that programmers can use not only to implement safe digital assets but also to write correct business logic for wrapping assets and enforcing access control policies.

## Accumulator

The Merkle Accumulator is an append-only Merkle tree that the Starcoin Blockchain uses to store the transaction_info hash. Merkle accumulators can provide proofs that a transaction was included in the chain (“proof of inclusion”). They are also called "history trees" in literature.
