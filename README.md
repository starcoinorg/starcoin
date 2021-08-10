# Starcoin

[![Build and Test](https://github.com/starcoinorg/starcoin/workflows/Build%20and%20Test/badge.svg)](https://github.com/starcoinorg/starcoin/actions?query=workflow%3A%22Build+and+Test%22+branch%3Amaster)
[![License](https://img.shields.io/badge/license-Apache-green.svg)](LICENSE)
[![codecov](https://codecov.io/gh/starcoinorg/starcoin/branch/master/graph/badge.svg)](https://codecov.io/gh/starcoinorg/starcoin)

A Layered Cryptocurrency and Decentralized Blockchain System.

## Build from source

```shell
cargo build --release 
```

For prerequisites and detailed build instructions please read [Build from source](https://developer.starcoin.org/en/setup/build/) document.

## Install binary

Download binary release from github [releases](https://github.com/starcoinorg/starcoin/releases) page.


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

Connect main network seed nodes:

```shell
starcoin --connect ws://main.seed.starcoin.org:9870 console
```

>note: Account-related commands cannot be used when connecting remotely

More detailed test network info please read [Join starcoin test network](https://developer.starcoin.org/en/setup/runnetwork/).

## Contribution
Thank you for considering to help out with the source code! Feel free to submit an issue or pull request.
Starcoin Move stdlib contribution document at [Starcoin Move standard library and framework](vm/stdlib/README.md).

## License

Starcoin is licensed as [Apache 2.0](./LICENSE).
