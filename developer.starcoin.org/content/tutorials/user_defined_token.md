---
title: User Defined Token
weight: 1
---

This is a example for How to define user custom Token on starcoin blockchain.

<!--more-->

First, start a dev network as described in [Run/Join Network](../setup/runnetwork), and get some coins, say `1000000000`.

In this document, I will use `aa5d01819bb5b6c5fece4eef943fde9c`, the default account address of my dev network, to represent the person who issues and send the new token. And I also created another account `353c411064ee39efaf2f3d115c55166a` and transfer some STC to it. The account will be used to receive the token.

# Deploy module and scripts

First, compile my_token module. The compiled bytecode will be output to `MyToken.mv` under the directory specified by `-o` parameter, or a temp dir if emit the `-o` parameter.

```bash
starcoin% dev compile -o examples examples/my_token/MyToken.move
```

Then, unlock the default account and deploy MyToken module.

```bash
starcoin% account unlock
starcoin% dev deploy examples/MyToken.mv
```
# Execute script function

First, use the default account init module.

```bash
starcoin% account execute-function --function 0xaa5d01819bb5b6c5fece4eef943fde9c::MyToken::init --blocking
```

Second, use the default account mint some MyToken.
```bash
starcoin% account execute-function --function 0xaa5d01819bb5b6c5fece4eef943fde9c::MyToken::mint --blocking --arg 1000000u128
```

Third, the second account accept the new Token. An account can accept the Token only if has adopted the Token.
```bash
starcoin% account accept_token -s 0x353c411064ee39efaf2f3d115c55166a 0xaa5d01819bb5b6c5fece4eef943fde9c::MyToken::MyToken --blocking
```

Fourth, the default account transfer 1000 MyToken to the second user.
```bash
starcoin% account transfer -r 0x353c411064ee39efaf2f3d115c55166a -v 1000 -t 0xaa5d01819bb5b6c5fece4eef943fde9c::MyToken::MyToken --blocking
```

Last, show balances of second user.
```bash
starcoin% account show 0x353c411064ee39efaf2f3d115c55166a
+--------------------+------------------------------------------------------------------+
| account.address    | 353c411064ee39efaf2f3d115c55166a                                 |
+--------------------+------------------------------------------------------------------+
| account.is_default | false                                                            |
+--------------------+------------------------------------------------------------------+
| account.public_key | a3a67682bfe3c9a569a7d67421bb0d012e80fe21293581ade2cf524da9a91955 |
+--------------------+------------------------------------------------------------------+
| auth_key_prefix    | 1cee76178673d4f245f6d4da2e8bd22d                                 |
+--------------------+------------------------------------------------------------------+
| balances.MyToken   | 10000                                                            |
+--------------------+------------------------------------------------------------------+
| balances.STC       | 100185885                                                        |
+--------------------+------------------------------------------------------------------+
| sequence_number    | 1                                                                |
```