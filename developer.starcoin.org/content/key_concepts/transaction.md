---
title: Transaction
weight: 2
---

Clients of the Starcoin Blockchain submit transactions to request updates to the ledger state.

<!--more-->

 A signed transaction on the blockchain mainly contains:

- **Sender address** — Account address of the sender of the transaction.
- **Sender public key** — The public key that corresponds to the private key used to sign the transaction.
- **Program** — The program is comprised of the following:
    - A Move bytecode transaction script.
    - An optional list of inputs to the script. For a peer-to-peer transaction, the inputs contain the information about the recipient and the amount transferred to the recipient.
    - An optional list of Move bytecode modules to publish.
- **Sequence number** — An unsigned integer that must be equal to the sequence number stored under the sender’s account.
- **Expiration time** — The time after which the transaction ceases to be valid.
- **Signature** — The digital signature of the sender.

