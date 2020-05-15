# Deploy Move contract

Move is a new programming language developed to provide a safe and programmable foundation for the [Libra Blockchain](https://github.com/libra/libra).
Starcoin Blockchain also support Move language to write smart contract.


First start a dev network as described in [dev_network.md](./dev_network.md), and get some coins, say `1000000000`.

Then, let contracting!

1. create a move script, say: `peer_to_peer.move`, to transfer some tokens to other address.


```move
script {
use 0x0::LibraAccount;

fun main<Token>(payee: address, auth_key_prefix: vector<u8>, amount: u64) {
    LibraAccount::pay_from_sender<Token>(payee, auth_key_prefix, amount)
}
}
```

and save it in the starcoin project dir.

2. compile the script.

In starcoin console, run:

```bash
starcoin% dev compile -o build/ -f peer_to_peer.move -s 759e96a81c7f0c828cd3bf1cc84239cb
build/peer_to_peer.mv
```

It will compile the script, and output the bytecode to `peer_to_peer.mv` under the directory passed in `-o build/`.

3. unlock your account. 

```bash
starcoin% wallet unlock 759e96a81c7f0c828cd3bf1cc84239cb
account 759e96a81c7f0c828cd3bf1cc84239cb unlocked for 300s
```

4. create other account.

```bash
starcoin% wallet create -p my-pass
+------------+------------------------------------------------------------------+
| address    | 1d8133a0c1a07366de459fb08d28d2a6                                 |
+------------+------------------------------------------------------------------+
| is_default | false                                                            |
+------------+------------------------------------------------------------------+
| public_key | 7add08c841d0f99f1f90ea2632c72aee483fab882e0d8d6d6defed2f1987345d |
+------------+------------------------------------------------------------------+
starcoin% wallet show 1d8133a0c1a07366de459fb08d28d2a6
+--------------------+------------------------------------------------------------------+
| account.address    | 1d8133a0c1a07366de459fb08d28d2a6                                 |
+--------------------+------------------------------------------------------------------+
| account.is_default | false                                                            |
+--------------------+------------------------------------------------------------------+
| account.public_key | 7add08c841d0f99f1f90ea2632c72aee483fab882e0d8d6d6defed2f1987345d |
+--------------------+------------------------------------------------------------------+
| auth_key_prefix    | 7bc6066656bb248755686d2ab78aef14                                 |
+--------------------+------------------------------------------------------------------+
| balance            |                                                                  |
+--------------------+------------------------------------------------------------------+
| sequence_number    |                                                                  |
+--------------------+------------------------------------------------------------------+
```

5. execute the script.(will build a transaction and submit to chain).

```bash
starcoin% dev execute -a 759e96a81c7f0c828cd3bf1cc84239cb -f build/peer_to_peer.mv -g 1000000 -t 0x0::Starcoin::T --arg 0x1d8133a0c1a07366de459fb08d28d2a6 --arg b"7bc6066656bb248755686d2ab78aef14" --arg 100000
621f7f59fec2a3dc5bd9ee34897278b03cc4f804e0fe531e745276e579779c0d
```

`-a 759e96a81c7f0c828cd3bf1cc84239cb` means the user address who wants to transfer tokens. (You need to replace the address with yours.)
`-t 0x0::Starcoin::T` means the token type you want to transfer, here `0x0::Starcoin::T` is the token of starcoin.
First argument `--arg 0x1d8133a0c1a07366de459fb08d28d2a6` means the receiver address which will receive the tokens.(You need to replace the address with yours.)
Second argument `--arg b"7bc6066656bb248755686d2ab78aef14"` is the auth_key_prefix of receiver address. (You can get it by command `wallet show [your-address]`).
Last argument is the amount of tokens you want to transfer.

It will return the txn id, which you can see the status of it in a block explorer.

Wait some times, and the txn will be included on chain. Then:

```
starcoin% wallet show 1d8133a0c1a07366de459fb08d28d2a6
+--------------------+------------------------------------------------------------------+
| account.address    | 1d8133a0c1a07366de459fb08d28d2a6                                 |
+--------------------+------------------------------------------------------------------+
| account.is_default | false                                                            |
+--------------------+------------------------------------------------------------------+
| account.public_key | 7add08c841d0f99f1f90ea2632c72aee483fab882e0d8d6d6defed2f1987345d |
+--------------------+------------------------------------------------------------------+
| auth_key_prefix    | 7bc6066656bb248755686d2ab78aef14                                 |
+--------------------+------------------------------------------------------------------+
| balance            | 100000                                                           |
+--------------------+------------------------------------------------------------------+
| sequence_number    | 0                                                                |
+--------------------+------------------------------------------------------------------+
```

I got the tokens!