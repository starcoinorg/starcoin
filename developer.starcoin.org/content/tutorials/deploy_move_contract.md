---
title: Deploy Move Contract
weight: 2
---

This article guides you on how to compile and deploy a Move contract to the starcoin blockchain.
<!--more-->

Move is a new programming language developed to provide a safe and programmable foundation for the [Libra Blockchain](https://github.com/libra/libra).Starcoin Blockchain also support Move language to write smart contract.


First start a dev network as described in [Run/Join Network](../setup/runnetwork), and get some coins, say `1000000000`.

Then, let contracting!

1. create a simple module, say: `MyCounter`.

```move
module MyCounter {
     use 0x1::Signer;

     resource struct T {
        value:u64,
     }
     public fun init(account: &signer){
        move_to(account, T{value:0});
     }
     public fun incr(account: &signer) acquires T {
        let counter = borrow_global_mut<T>(Signer::address_of(account));
        counter.value = counter.value + 1;
     }

}
```

the source file at https://github.com/starcoinorg/starcoin/tree/master/examples/my_counter/module/MyCounter.move

2. compile the module.

In starcoin console, run:

```bash
starcoin% dev compile examples/my_counter/module/MyCounter.move
/Users/jolestar/.starcoin/cli/dev/tmp/6e0f73c87b0a0c6c4e8d77ca4a3a9c48/MyCounter.mv
```

It will compile the module, and output the bytecode to `MyCounter.mv` under the temp directory.

3. unlock your default account.

```bash
starcoin% account unlock
account 759e96a81c7f0c828cd3bf1cc84239cb unlocked for 300s
```

4. deploy module

```bash
starcoin% dev deploy /Users/jolestar/.starcoin/cli/dev/tmp/6e0f73c87b0a0c6c4e8d77ca4a3a9c48/MyCounter.mv
txn 3c957f688c628e7ce2a4e1c238b14505b9cf6068aa3da897f555474ee7b2dd0b submitted.
3c957f688c628e7ce2a4e1c238b14505b9cf6068aa3da897f555474ee7b2dd0b
```

5. write script to call module.
//TODO

6. use state cmd to query account state.
//TODO
