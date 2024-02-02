```text
 ██████╗████████╗ █████╗ ██████╗  █████╗  █████╗ ██╗███╗  ██╗
██╔════╝╚══██╔══╝██╔══██╗██╔══██╗██╔══██╗██╔══██╗██║████╗ ██║
╚█████╗    ██║   ███████║██████╔╝██║  ╚═╝██║  ██║██║██╔██╗██║
 ╚═══██╗   ██║   ██╔══██║██╔══██╗██║  ██╗██║  ██║██║██║╚████║
██████╔╝   ██║   ██║  ██║██║  ██║╚█████╔╝╚█████╔╝██║██║ ╚███║
╚═════╝    ╚═╝   ╚═╝  ╚═╝╚═╝  ╚═╝ ╚════╝  ╚════╝ ╚═╝╚═╝  ╚══╝
```

Starcoin - a smart contract blockchain network that scales by layering

net proxima using move with table extension feature. If you want to use it, you should compile dev branch. 

[Report a Bug](https://github.com/starcoinorg/starcoin/issues/new?assignees=&labels=bug&template=01_BUG_REPORT.md&title=bug%3A+")
·
[Request a Feature](https://github.com/starcoinorg/starcoin/issues/new?assignees=&labels=enhancement&template=02_FEATURE_REQUEST.md&title=feat%3A+")
.
[Ask a Question](https://github.com/starcoinorg/starcoin-cookbook/issues/new?assignees=&labels=question&template=02_QUESTION.md&title=%5Bquestion%5D")


[![Build and Test](https://github.com/starcoinorg/starcoin/workflows/Build%20and%20Test/badge.svg)](https://github.com/starcoinorg/starcoin/actions?query=workflow%3A%22Build+and+Test%22+branch%3Amaster)
[![License](https://img.shields.io/badge/license-Apache-green.svg)](LICENSE)
[![codecov](https://codecov.io/gh/starcoinorg/starcoin/branch/master/graph/badge.svg)](https://codecov.io/gh/starcoinorg/starcoin)
[![LoC](https://tokei.rs/b1/github/starcoinorg/starcoin?category=lines)](https://github.com/starcoinorg/starcoin)

## Binary file description

The starcoin project comes with several wrappers/executables, `release` indicates whether the binary is included in the release archive.

|         Command         | SRC Directory           | Release | Description                                                                                                                                                                                                                                                                                                  |
|:-----------------------:|-------------------------|---------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
|     **`starcoin`**      | cmd/starcoin            | true    | Our main Starcoin CLI client. It is the entry point into the Starcoin network. We can use it to run a node, or connect to a another node as an interactive console. `starcoin --help` and the [CLI reference](https://starcoinorg.github.io/starcoin-cookbook/docs/reference/cli/) for command line options. |
|        **`mpm`**        | vm/move-package-manager | true    | Move Package Manager(mpm) is a command line tool to develop move projects, like Cargo for Rust, or NPM for NodeJS.                                                                                                                                                                                           |
|    `starcoin_miner`     | cmd/miner_client        | true    | A CPU miner client for starcoin.                                                                                                                                                                                                                                                                             |             
| `starcoin_db_exporter`  | cmd/db-exporter         | true    | A tool for exporting or importing data from or to the starcoin database.                                                                                                                                                                                                                                     |                                                                                                                                                                                                                                                                   
|  `starcoin_generator`   | cmd/generator           | true    | A tool for generate genesis file or mock data.                                                                                                                                                                                                                                                               |
|        `airdrop`        | cmd/airdrop             | false   | A tool for batch transfer Token based on a csv file.                                                                                                                                                                                                                                                         |                                                                                                                                                                                                                                                        |
|   `genesis-nft-miner`   | cmd/genesis-nft-miner   | false   | A tool for claim the GenesisNFT, All address in the file [contrib-contracts/src/genesis-nft-address.json](contrib-contracts/src/genesis-nft-address.json).                                                                                                                                                   |
|    `starcoin-faucet`    | cmd/faucet              | false   | A tool for providing a simple faucet web service                                                                                                                                                                                                                                                             |                                                                                                                                                                                                                                                                                                     
|   `starcoin-indexer`    | cmd/indexer             | false   | A tool for creating index on Elasticsearch for starcoin's block and transaction, etc.                                                                                                                                                                                                                        |
|   `merkle-generator`    | cmd/merkle-generator    | false   | A tool for generating merkle data from a detail csv file of a distribution, for airdrop by merkle tree.                                                                                                                                                                                                      |
|   `resource-exporter`   | cmd/resource-exporter   | false   | A tool for export `resource` from starcoin state database.                                                                                                                                                                                                                                                   |                                                                                                                                                                                                                                                                                                      
|      `tx-factory`       | cmd/tx-factory          | false   | A tool used to generate transactions, generally for testing or benchmark.                                                                                                                                                                                                                                    |
|    `starcoin-replay`    | cmd/replay              | false   | A tool for replay block data from a database to a new database.                                                                                                                                                                                                                                              |
| `starcoin-peer-watcher` | cmd/peer-watcher        | false   | A sample app for join starcoin p2p network and print the discovered peer info.                                                                                                                                                                                                                               |

## Build from source

```shell
cargo build --release 
```

For prerequisites and detailed build instructions please read [Build from source](https://starcoinorg.github.io/starcoin-cookbook/docs/getting-started/install/build) document.

## Install binary

Download binary release from GitHub [releases](https://github.com/starcoinorg/starcoin/releases) page.

Or install by one-line script:

`curl --proto '=https' -O --tlsv1.2 -sSf https://raw.githubusercontent.com/starcoinorg/starcoin/master/scripts/install_starcoin_mpm.sh | sh install_starcoin_mpm.sh v1.11.12
`

## Run dev node:

```shell
starcoin -n dev console
```

More detailed dev instructions please read [Run starcoin dev network](https://developer.starcoin.org/en/setup/runnetwork/) document.

## Join a test network

```shell
starcoin -n barnard console
```

## Join main network

```shell
starcoin -n main console
```

## Connect to remote node

Connect to the main network seed nodes:

```shell
starcoin --connect ws://main.seed.starcoin.org:9870 console
```

>note: Account-related commands cannot be used when connecting remotely

Connect to the main network seed nodes and use a local account database for using Account-related commands

```shell
starcoin --connect ws://main.seed.starcoin.org:9870 --local-account-dir ~/.starcoin/main/account_vaults console
```

More detailed test network info please read [Join starcoin test network](https://developer.starcoin.org/en/setup/runnetwork/).


## Roadmap

See the [open issues](https://github.com/starcoinorg/starcoin/issues) for a list of proposed features (and known issues).

- [Top Feature Requests](https://github.com/starcoinorg/starcoin/issues?q=label%3Aenhancement+is%3Aopen+sort%3Areactions-%2B1-desc) (Add your votes using the 👍 reaction)
- [Top Bugs](https://github.com/starcoinorg/starcoin/issues?q=is%3Aissue+is%3Aopen+label%3Abug+sort%3Areactions-%2B1-desc) (Add your votes using the 👍 reaction)
- [Newest Bugs](https://github.com/starcoinorg/starcoin/issues?q=is%3Aopen+is%3Aissue+label%3Abug)
- [Help Wanted](https://github.com/starcoinorg/starcoin/issues?q=label%3A"help+wanted"+is%3Aissue+is%3Aopen)

## Contributing

First off, thanks for taking the time to contribute! Contributions are what makes the open-source community such an amazing place to learn, inspire, and create. Any contributions you make will benefit everybody else and are **greatly appreciated**.

Please try to create bug reports that are:

- _Reproducible._ Include steps to reproduce the problem.
- _Specific._ Include as much detail as possible: which version, what environment, etc.
- _Unique._ Do not duplicate existing opened issues.
- _Scoped to a Single Bug._ One bug per report.

You can learn more about contributing to the Starcoin project by reading our [Contribution Guide](./CONTRIBUTING.md) and by viewing our [Code of Conduct](./CODE_OF_CONDUCT.md).

### Code Layout

You could find the introduction of each code directory [here](code_layout.md) for helping to understand the organization of codes.

## Support

Reach out to the maintainer at one of the following places:

- [GitHub discussions](https://github.com/starcoinorg/starcoin/discussions)
- [Starcoin Linktree](https://linktr.ee/starcoin)
- [Starcoin&Move Contributor Telegram Group](https://t.me/starcoin_contributor)

## License

Starcoin is licensed as [Apache 2.0](./LICENSE).
