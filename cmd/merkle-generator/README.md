## Merkle Distributor for Airdrop

This project contains a move contract which implements a merkle distribution for airdropping in Starcoin,
and also a command tool to generate merkle data from a detail csv file of a distribution.


### Usage

1. Using the tool to generate the merkle root.

```shell
merkle proof generator

USAGE:
    merkle-generator <output> --input <input>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -i, --input <input>    intput csv without header, like rewards.csv

ARGS:
    <output>    merkle output json file, like merkle.json
```

Run on example:

```
cargo run --bin merkle-generator -- -i examples/reward.csv merkle-result.json
```

The generated json file should be same as `examples/merkle-exmaple.json`.

2. Then create a distribution onchain.

``` move
public(script) fun create<T: store>(signer: signer, merkle_root: vector<u8>, token_amounts: u128, leafs: u64);
```

3. User claim

``` move
// claim from myslef.
public(script) fun claim<T: store>(signer: signer, distribution_address: address, index: u64, amount: u128, merkle_proof: vector<vector<u8>>);

// claim for someone else.
public(script) fun claim_for_address<T: store>(distribution_address: address, index: u64, account: address, amount: u128, merkle_proof: vector<vector<u8>>);
```