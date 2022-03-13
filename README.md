<h1 align="center">
  <a href="https://starcoin.org">
    <img src="https://starcoin.org/img/stc.svg" alt="Logo" width="125" height="125">
  </a>
</h1>

<div align="center">
  Starcoin - a smart contract blockchain network that scales by layering
  <br />
  <br />
  <a href="https://github.com/starcoinorg/starcoin/issues/new?assignees=&labels=bug&template=01_BUG_REPORT.md&title=bug%3A+">Report a Bug</a>
  ·
  <a href="https://github.com/starcoinorg/starcoin/issues/new?assignees=&labels=enhancement&template=02_FEATURE_REQUEST.md&title=feat%3A+">Request a Feature</a>
  .
  <a href="https://github.com/starcoinorg/starcoin/discussions">Ask a Question</a>
<br />
<br />
</div>


[![Build and Test](https://github.com/starcoinorg/starcoin/workflows/Build%20and%20Test/badge.svg)](https://github.com/starcoinorg/starcoin/actions?query=workflow%3A%22Build+and+Test%22+branch%3Amaster)
[![License](https://img.shields.io/badge/license-Apache-green.svg)](LICENSE)
[![codecov](https://codecov.io/gh/starcoinorg/starcoin/branch/master/graph/badge.svg)](https://codecov.io/gh/starcoinorg/starcoin)
[![LoC](https://tokei.rs/b1/github/starcoinorg/starcoin?category=lines)](https://github.com/starcoinorg/starcoin)


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

## Support

Reach out to the maintainer at one of the following places:

- [GitHub discussions](https://github.com/starcoinorg/starcoin/discussions)
- [Starcoin Discord](https://discord.gg/starcoin)

## License

Starcoin is licensed as [Apache 2.0](./LICENSE).
