### Airdrop in Starcoin

Airdrop STC to starcoin users.

```shell
./target/debug/airdrop -i amount.csv
```

amount.csv should be in format: `address,auth_key,amount`. for example:

```text
0x00009cc5c3d56231df33dd74cc4780f2,809828f0150fc07e15c6d27db607a56b00009cc5c3d56231df33dd74cc4780f2,1000
0x0026241a238ede23f1d9e18421c0185d,b6a80ebbf065eb1e8964b83f2c8fad040026241a238ede23f1d9e18421c0185d,1000
0x00342e46b92e499108f2e5fbd3e44227,65f53b52ab2b12b683678cb83fa03bd200342e46b92e499108f2e5fbd3e44227,999
0x0034914ea62e78ac86ddf99f9240c7c5,232f6bd40fc5efb59f27ff2003323b150034914ea62e78ac86ddf99f9240c7c5,99
```