---
title: Deploy Move Contract
weight: 2
---

This article guides you on how to compile and deploy a Move contract to the starcoin blockchain.
<!--more-->

Move is a new programming language developed to provide a safe and programmable foundation for the [Diem Blockchain](https://github.com/deim/diem).
Starcoin Blockchain also support Move language to write smart contract.


First start a dev network as described in [Run/Join Network](../setup/runnetwork), and get some coins, say `1000000000`.

Then, let contracting!

1. create a simple module, say: `MyCounter`.

```move
module MyCounter {

     use 0x1::Signer;

     struct Counter has key, store {
        value:u64,
     }
     public fun init(account: &signer){
        move_to(account, Counter{value:0});
     }
     public fun incr(account: &signer) acquires Counter {
        let counter = borrow_global_mut<Counter>(Signer::address_of(account));
        counter.value = counter.value + 1;
     }

     public(script) fun init_counter(account: signer){
        Self::init(&account)
     }

     public(script) fun incr_counter(account: signer)  acquires Counter {
        Self::incr(&account)
     }

}
```

the source file at https://github.com/starcoinorg/starcoin/tree/master/examples/my_counter/module/MyCounter.move

2. compile the module.

In starcoin console, run:

```bash
starcoin% dev compile examples/my_counter/module/MyCounter.move
{
  "ok": [
    "/Users/jolestar/.starcoin/cli/dev/tmp/8a293eeef01a91a6419d54a597001be2/MyCounter.mv"
  ]
}
```

It will compile the module, and output the bytecode to `MyCounter.mv` under the temp directory.

3. unlock your default account.

```bash
starcoin% account unlock
{
  "ok": {
    "address": "0x8c4d3877592931cacbd87eeb65c9e4f8",
    "is_default": true,
    "is_readonly": false,
    "public_key": "0x13ee131f8a84fac1928834e48ab72bc948e69aa578d7f837f4fc9e9a35fcc740",
    "receipt_identifier": "stc1p33xnsa6e9ycu4j7c0m4ktj0ylqrh4hgp"
  }
}
```

4. deploy module

```bash
starcoin% dev deploy /Users/jolestar/.starcoin/cli/dev/tmp/8a293eeef01a91a6419d54a597001be2/MyCounter.mv -b
txn 0xff7ff5e058ec82cf7f1be955bde01400184401707f7fe7977cf762dced8fc8bd submitted.
{
  "ok": {
    "type": "Run",
    "txn_hash": "0xff7ff5e058ec82cf7f1be955bde01400184401707f7fe7977cf762dced8fc8bd",
    "txn_info": {
      "block_hash": "0x28c658f731405e61c85ded5b3bcd7ebfdd70b7cb43c0558f55367693358c5771",
      "block_number": "2",
      "transaction_hash": "0xff7ff5e058ec82cf7f1be955bde01400184401707f7fe7977cf762dced8fc8bd",
      "transaction_index": 1,
      "state_root_hash": "0xd3273bdad6744512541f615d9c45cabb3c80d5c10154821175352c821a269889",
      "event_root_hash": "0x414343554d554c41544f525f504c414345484f4c4445525f4841534800000000",
      "gas_used": "7800",
      "status": "Executed"
    },
    "events": []
  }
}
```

5. call init_counter script function to init resource

```bash
starcoin% account execute-function --function 0x8c4d3877592931cacbd87eeb65c9e4f8::MyCounter::init_counter -b
txn 0x1d5ba1d86746ab423696ba045799cf58f0c34eee8c848e48982fd192b682c96a submitted.
{
  "ok": {
    "type": "Run",
    "txn_hash": "0x1d5ba1d86746ab423696ba045799cf58f0c34eee8c848e48982fd192b682c96a",
    "txn_info": {
      "block_hash": "0x6fe6b85eba249c24723ba267d067d2411d567d0554aa98aa4259fe46d128e816",
      "block_number": "3",
      "transaction_hash": "0x1d5ba1d86746ab423696ba045799cf58f0c34eee8c848e48982fd192b682c96a",
      "transaction_index": 1,
      "state_root_hash": "0x0949cd5daa4219ef860ddb2438956588b51981954f1fb7b5550c28f7269823e9",
      "event_root_hash": "0x414343554d554c41544f525f504c414345484f4c4445525f4841534800000000",
      "gas_used": "11667",
      "status": "Executed"
    },
    "events": []
  }
}

```

6. show resource

```bash
contract get resource 0x8c4d3877592931cacbd87eeb65c9e4f8 0x8c4d3877592931cacbd87eeb65c9e4f8::MyCounter::Counter
{
  "ok": {
    "abilities": 12,
    "type_": "0x8c4d3877592931cacbd87eeb65c9e4f8::MyCounter::Counter",
    "value": [
      [
        "value",
        {
          "U64": "0"
        }
      ]
    ]
  }
}
```

7. call incr_counter to increment counter

```bash
starcoin% account execute-function --function 0x8c4d3877592931cacbd87eeb65c9e4f8::MyCounter::incr_counter -b
txn 0x0985ad70305e13fddb059017460a669fb6c04e47328296f4e49afb8b9c82c3b8 submitted.
{
  "ok": {
    "type": "Run",
    "txn_hash": "0x0985ad70305e13fddb059017460a669fb6c04e47328296f4e49afb8b9c82c3b8",
    "txn_info": {
      "block_hash": "0x8580dcea21db0535b816d392cd9c60567f5b7ef92e9e72d4cbd1106bc4154a7d",
      "block_number": "4",
      "transaction_hash": "0x0985ad70305e13fddb059017460a669fb6c04e47328296f4e49afb8b9c82c3b8",
      "transaction_index": 1,
      "state_root_hash": "0x1a9e1bc831e085a5efcdbe3f50579089bfdcc849c93158d998007fe57cf754c8",
      "event_root_hash": "0x414343554d554c41544f525f504c414345484f4c4445525f4841534800000000",
      "gas_used": "17231",
      "status": "Executed"
    },
    "events": []
  }
}
```

8. show resource again

```bash
starcoin% contract get resource 0x8c4d3877592931cacbd87eeb65c9e4f8 0x8c4d3877592931cacbd87eeb65c9e4f8::MyCounter::Counter
{
  "ok": {
    "abilities": 12,
    "type_": "0x8c4d3877592931cacbd87eeb65c9e4f8::MyCounter::Counter",
    "value": [
      [
        "value",
        {
          "U64": "1"
        }
      ]
    ]
  }
}
```

You can see the counter's value is 1 now.

9. Use another account to init and incr counter again.

Say the new account address is 0x0da41daaa9dbd912647c765025a12e5a

```bash
starcoin% account execute-function -s 0x0da41daaa9dbd912647c765025a12e5a  --function 0x8c4d3877592931cacbd87eeb65c9e4f8::MyCounter::init_counter -b
starcoin% contract get resource 0x0da41daaa9dbd912647c765025a12e5a 0x8c4d3877592931cacbd87eeb65c9e4f8::MyCounter::Counter
starcoin% account execute-function -s 0x0da41daaa9dbd912647c765025a12e5a  --function 0x8c4d3877592931cacbd87eeb65c9e4f8::MyCounter::incr_counter -b
starcoin% contract get resource 0x0da41daaa9dbd912647c765025a12e5a 0x8c4d3877592931cacbd87eeb65c9e4f8::MyCounter::Counter
```