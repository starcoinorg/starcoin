---
title: Account
weight: 1
---

TODO: this document need to be updated.

Account, Addresses, Authentication keys, and Cryptographic keys

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

