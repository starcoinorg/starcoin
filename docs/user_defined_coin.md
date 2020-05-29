# User defined Coin

This is a example for How to define user custom Coin on starcoin blockchain.

First, start a dev network as described in [dev_network.md](./dev_network.md), and get some coins, say `1000000000`. 

In this document, I will use `aa5d01819bb5b6c5fece4eef943fde9c`, the default account address of my dev network, to represent the person who issues and send the new token. And I also created another account `353c411064ee39efaf2f3d115c55166a` and transfer some money to it. The account will be used to receive the token.

# Deploy module and scripts
 
First, compile my_token module. The compiled bytecode will be output to `my_token.mv` under the directory specified by `-o` parameter, or a temp dir if emit the `-o` parameter.

```bash
starcoin% dev compile -o examples examples/my_token/module/my_token.move 
```

Then, to compile the three scripts. The scripts depend on my_token module, so pass the dependency through `-d`.

```bash
starcoin% dev compile -o examples -f examples/my_token/scripts/init.move -d examples/my_token/module/my_token.move
```

```bash
starcoin% dev compile -o examples -f examples/my_token/scripts/mint.move -d examples/my_token/module/my_token.move
```

Last, unlock the default wallet and deploy my_token module. 

```bash
starcoin% wallet unlock -p
starcoin% dev deploy examples/my_token.mv -g 1000000
```
# Execute scripts

First, use the default account init module.
```bash
starcoin% wallet execute examples/init.mv -g 1000000
```

Second, use the default account mint some MyToken.
```bash
starcoin% wallet execute examples/mint.mv -g 1000000 --arg 1000000
```

Third, the second account accept the new Token. An account can accept the Token only if has adopted the Token.
```bash
starcoin% wallet accept_coin -s 353c411064ee39efaf2f3d115c55166a -g 1000000 0x0::aa5d01819bb5b6c5fece4eef943fde9c::MyToken::T
```

Fourth, the default account transfer 100 MyToken to the second user.
```bash
starcoin% wallet transfer -r 353c411064ee39efaf2f3d115c55166a -v 10000 -c 0xaa5d01819bb5b6c5fece4eef943fde9c::MyToken::T
```

Last, show balances of second user.
```bash
starcoin% wallet show 353c411064ee39efaf2f3d115c55166a
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