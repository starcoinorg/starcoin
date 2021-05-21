---
title: First Transaction
weight: 3
---

This article guides you on how to execute your first transaction on the starcoin blockchain.
Before that, I recommend you read tech-related articles to get some idea of the basic concepts of starcoin.
<!--more-->

## Prerequisite

Let's say you've run up a starcoin dev node locally.

## A few steps to submit a transaction

- Start the CLI console and connect to the starcoin node，detail document at [Use starcoin console](./console).
- Create two accounts: Alice,Bob，detail step see [Account manager](./account_manager).
- Mint money into Alice's account.
- Submit transfer transaction: Alice send money to Bob.

### Create an account

After connecting to the node, let's first create two accounts. Here we assume that both accounts have been created successfully, 
Alice is the default account with the address 0x988acf6d210701242af03cbb13780745 and Bob's address is 0x1179ec968815ded9c59775274446ad4c , 
receipt_identifier is stc1pz9u7e95gzh0dn3vhw5n5g34dfn8pp69czzy46e8n60lpe53s0nmpz70vj6ypthkeckth2f6yg6k5cslzx6t .

### Use Faucet to top up your account

 In dev environment, faucet can be used to mint accounts. faucet only exists in dev and test net to make it easier for developers developing and testing dapps.

 Let's do it!.

 ``` bash
starcoin% dev get_coin -v 100000000000
```

`dev get_coin` will mint some coins the default account, and if the account does not exist on the chain, it will creates the account first and then transfers a specified (with `-v`) number of coins to the account.
The output of the command is the transaction data  issued by the FAUCET account (address `0000000000000000000000000A550C18`).

Wait a few seconds and then check your account information again.

```bash
starcoin% account show 0x988acf6d210701242af03cbb13780745
+--------------------+------------------------------------------------------------------------------------------+
| account.address    | 0x988acf6d210701242af03cbb13780745                                                       |
+--------------------+------------------------------------------------------------------------------------------+
| account.is_default | true                                                                                     |
+--------------------+------------------------------------------------------------------------------------------+
| account.public_key | 0xd574c33580942a124b377c0fa64c0d1c021c405893ac99b1cf77a44dc530e4b2                       |
+--------------------+------------------------------------------------------------------------------------------+
| auth_key           | 0x6d9ca71670371e406e6e7821c4560f31988acf6d210701242af03cbb13780745                       |
+--------------------+------------------------------------------------------------------------------------------+
| receipt_identifier | stc1pnz9v7mfpquqjg2hs8ja3x7q8g4keefckwqm3usrwdeuzr3zkpuce3zk0d5sswqfy9tcrewcn0qr528sv4vw |
+--------------------+------------------------------------------------------------------------------------------+
| sequence_number    | 0                                                                                        |
+--------------------+------------------------------------------------------------------------------------------+
| balances.STC       | 100000000000                                                                             |
+--------------------+------------------------------------------------------------------------------------------+
```

Now, `balances` and `sequence_number` is filled.



### Submit Transaction

First you need to unlock Alice's account and authorize node to sign the transaction using Alice's private key.

```` bash
account unlock -p my-pass 1d8133a0c1a07366de459fb08d28d2a6
````

where `-p my-pass` is the password that was needed when creating the account, if the default account's init password is empty.

Once the account is unlocked, execute the following command.

```bash
starcoin% account transfer -s 0x988acf6d210701242af03cbb13780745 -r stc1pz9u7e95gzh0dn3vhw5n5g34dfn8pp69czzy46e8n60lpe53s0nmpz70vj6ypthkeckth2f6yg6k5cslzx6t -v 10000 -b
```

- `-s 0x988acf6d210701242af03cbb13780745`: is Alice's account address.
- `-r stc1pz9u7e95gzh0dn3vhw5n5g34dfn8pp69czzy46e8n60lpe53s0nmpz70vj6ypthkeckth2f6yg6k5cslzx6t`: is Bob's receipt_identifier.

> If, Bob's account does not yet exist on the chain, the transfer transaction will automatically create Bob's account on the chain.


At this point, the transaction has been submitted to the chain.
You still need to wait a few seconds (in the dev environment, maybe longer in test env) to let the transaction included the chain.
Then check Bob's account information again:.


``` bash
starcoin% account show 0x1179ec968815ded9c59775274446ad4c
+----------------------------+------------------------------------------------------------------------------------------+
| account.address            | 0x1179ec968815ded9c59775274446ad4c                                                       |
+----------------------------+------------------------------------------------------------------------------------------+
| account.is_default         | false                                                                                    |
+----------------------------+------------------------------------------------------------------------------------------+
| account.public_key         | 0xfacd584290ee7baea7fe8e22d13332633babca46e77c0ca941b6b5c6266523cb                       |
+----------------------------+------------------------------------------------------------------------------------------+
| account.receipt_identifier | stc1pz9u7e95gzh0dn3vhw5n5g34dfn8pp69czzy46e8n60lpe53s0nmpz70vj6ypthkeckth2f6yg6k5cslzx6t |
+----------------------------+------------------------------------------------------------------------------------------+
| auth_key                   | 0xce10e8b810895d64f3d3fe1cd2307cf61179ec968815ded9c59775274446ad4c                       |
+----------------------------+------------------------------------------------------------------------------------------+
| sequence_number            | 0                                                                                        |
+----------------------------+------------------------------------------------------------------------------------------+
| balances.STC               | 10000                                                                                    |
+----------------------------+------------------------------------------------------------------------------------------+
```

Bob has the money now!

