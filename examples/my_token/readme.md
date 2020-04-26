# Preparations

First, start a dev network as described in [dev_network.md](./dev_network.md), and get some coins, say `1000000000`. 

In this document, I will use `2f1ea8901faef384366c615447e460da`, the default account address of my dev network, to represent the person who issues and send the new token. And I also created another account `ed20d3217eacf4b3408e51020cb8d62b` and transfer some money to it. The account will be used to receive the token.

```bash
starcoin% wallet create -p ""
starcoin% txn transfer -v 10000000 -t ed20d3217eacf4b3408e51020cb8d62b -k 381c9c61f8f896f8167260f389fb6e4438e93bbdbb55f7a1799c0a23fa5ceee7
```
# Deploy module and scripts
1. compile the module and scripts.

In starcoin console, run:

```bash
starcoin% wallet compile -o examples -f examples/my_token/module/my_token.move -s 0x2f1ea8901faef384366c615447e460da
```

It will compile the my_token module, and output the bytecode to `my_token.mv` under the directory specified by `-o` parameter.

Next, to compile the three scripts. Remember to replace the address in `use 0x2f1ea8901faef384366c615447e460da::MyToken;` with your own address before compile.

```bash
starcoin% wallet compile -o examples -f examples/my_token/scripts/issue.move -s 0x2f1ea8901faef384366c615447e460da -d examples/my_token/module/my_token.move
```

The issue.move script used to issue the new token. It depends on my_token module, so pass the dependency through `-d`.

```bash
starcoin% wallet compile -o examples -f examples/my_token/scripts/adopt.move -s 0x2f1ea8901faef384366c615447e460da -d examples/my_token/module/my_token.move
```

```bash
starcoin% wallet compile -o examples -f examples/my_token/scripts/transfer.move -s 0x2f1ea8901faef384366c615447e460da -d examples/my_token/module/my_token.move
```

2. deploy my_token module. 

```bash
starcoin% wallet deploy -f examples/my_token.mv -g 1000000
```
# Execute scripts

First, the default account issues the new Token
```bash
starcoin% wallet execute -a 2f1ea8901faef384366c615447e460da -f examples/issue.mv -g 1000000 --arg 100000
```

Second, the second account adopts the new Token. People can accept the Token only if he/she has adopted the Token.
```bash
starcoin% wallet execute -a ed20d3217eacf4b3408e51020cb8d62b -f examples/adopt.mv -g 1000000
```

Third, the default account transfer 100000 Tokens to the second user.
```bash
starcoin% wallet execute -a 2f1ea8901faef384366c615447e460da -f examples/transfer.mv -g 1000000 --arg 0xed20d3217eacf4b3408e51020cb8d62b --arg b"4273f11ad3b9f9a4fce033d02ae34fe7" --arg 100000
```