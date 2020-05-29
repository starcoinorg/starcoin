# Deploy Move contract

Move is a new programming language developed to provide a safe and programmable foundation for the [Libra Blockchain](https://github.com/libra/libra).
Starcoin Blockchain also support Move language to write smart contract.


First start a dev network as described in [dev_network.md](./dev_network.md), and get some coins, say `1000000000`.

Then, let contracting!

1. create a simple module, say: `MyCounter`.

```move
module MyCounter {
     use 0x0::Transaction;
     resource struct T {
        value:u64,
     }
     public fun init(){
        move_to_sender(T{value:0});
     }
     public fun incr() acquires T {
        let counter = borrow_global_mut<T>(Transaction::sender());
        counter.value = counter.value + 1;
     }

}
```

the source file at examples/my_counter/module/my_counter.move

2. compile the module.

In starcoin console, run:

```bash
starcoin% dev compile examples/my_counter/module/my_counter.move
/var/folders/by/8jj_3yzx4072w19vb_m934wc0000gn/T/33ee6c17c2cc7327980da96651757650/my_counter.mv
```

It will compile the module, and output the bytecode to `my_counter.mv` under the temp directory.

3. unlock your default account.

```bash
starcoin% wallet unlock -p
account 759e96a81c7f0c828cd3bf1cc84239cb unlocked for 300s
```

4. deploy module

```bash
starcoin% dev deploy /var/folders/by/8jj_3yzx4072w19vb_m934wc0000gn/T/33ee6c17c2cc7327980da96651757650/my_counter.mv
txn 3c957f688c628e7ce2a4e1c238b14505b9cf6068aa3da897f555474ee7b2dd0b submitted.
3c957f688c628e7ce2a4e1c238b14505b9cf6068aa3da897f555474ee7b2dd0b
```
5. write script to call module.
//TODO

6. use state cmd to query account state.
//TODO

Next to read [User defined Coin](./user_defined_coin.md).