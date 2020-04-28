# Preparations

First, start a dev network as described in [dev_network.md](./dev_network.md), and get some coins, say `1000000000`. 

In this document, I will use `2f1ea8901faef384366c615447e460da`, the default account address of my dev network, to represent the person who issues and send the new token. And I also created another account `ed20d3217eacf4b3408e51020cb8d62b` and transfer some money to it. The account will be used to receive the token.

```bash
starcoin% dev get_coin -v 1000000000
starcoin% wallet create -p ""
starcoin% txn transfer -v 10000000 -t ed20d3217eacf4b3408e51020cb8d62b -k 381c9c61f8f896f8167260f389fb6e4438e93bbdbb55f7a1799c0a23fa5ceee7
```
# Deploy module and scripts
 
Remember: before compile, replace the address `0x2f1ea8901faef384366c615447e460da` in *.move files under `examples/my_token/` with the default address of your own wallet.

First, compile my_token module. The compiled bytecode will be output to `my_token.mv` under the directory specified by `-o` parameter.

```bash
starcoin% wallet compile -o examples -f examples/my_token/module/my_token.move -s 2f1ea8901faef384366c615447e460da
```

Then, to compile the three scripts. The scripts depend on my_token module, so pass the dependency through `-d`.

```bash
starcoin% wallet compile -o examples -f examples/my_token/scripts/issue.move -s 2f1ea8901faef384366c615447e460da -d examples/my_token/module/my_token.move
```

```bash
starcoin% wallet compile -o examples -f examples/my_token/scripts/adopt.move -s 2f1ea8901faef384366c615447e460da -d examples/my_token/module/my_token.move
```

```bash
starcoin% wallet compile -o examples -f examples/my_token/scripts/transfer.move -s 2f1ea8901faef384366c615447e460da -d examples/my_token/module/my_token.move
```

Last, deploy my_token module. 

```bash
starcoin% wallet deploy -f examples/my_token.mv -g 1000000
```
# Execute scripts

First, the default account issues the new Token with market cap 100000.
```bash
starcoin% wallet execute -a 2f1ea8901faef384366c615447e460da -f examples/issue.mv -g 1000000 --arg 100000
```

Second, the second account adopts the new Token. An account can accept the Token only if has adopted the Token.
```bash
starcoin% wallet execute -a ed20d3217eacf4b3408e51020cb8d62b -f examples/adopt.mv -g 1000000
```

Third, the default account transfer 100 Tokens to the second user.
```bash
starcoin% wallet execute -a 2f1ea8901faef384366c615447e460da -f examples/transfer.mv -g 1000000 --arg 0xed20d3217eacf4b3408e51020cb8d62b --arg b"4273f11ad3b9f9a4fce033d02ae34fe7" --arg 100
```

Last, show token balance of second user.
```bash
starcoin% wallet show ed20d3217eacf4b3408e51020cb8d62b -t 0x2f1ea8901faef384366c615447e460da::MyToken::T
```