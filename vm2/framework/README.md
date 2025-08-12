---
id: Starcoin-framework
title: Starcoin Framework
custom_edit_url: https://github.com/starcoin-labs/starcoin-core/edit/main/Starcoin-move/Starcoin-framework/README.md
---

## The Starcoin Framework

The Starcoin Framework defines the standard actions that can be performed on-chain
both by the Starcoin VM---through the various prologue/epilogue functions---and by
users of the blockchain---through the allowed set of transactions. This
directory contains different directories that hold the source Move
modules and transaction scripts, along with a framework for generation of
documentation, ABIs, and error information from the Move source
files. See the [Layout](#layout) section for a more detailed overview of the structure.

## Documentation

Each of the main components of the Starcoin Framework and contributing guidelines are documented separately. See them by version below:

* *Starcoin tokens* - [main](https://github.com/starcoin-labs/starcoin-core/blob/main/starcoin-move/framework/starcoin-token/doc/overview.md), [testnet](https://github.com/starcoin-labs/starcoin-core/blob/testnet/starcoin-move/framework/starcoin-token/doc/overview.md), [devnet](https://github.com/starcoin-labs/starcoin-core/blob/devnet/starcoin-move/framework/starcoin-token/doc/overview.md)
* *Starcoin framework* - [main](https://github.com/starcoin-labs/starcoin-core/blob/main/starcoin-move/framework/starcoin-framework/doc/overview.md), [testnet](https://github.com/starcoin-labs/starcoin-core/blob/testnet/starcoin-move/framework/starcoin-framework/doc/overview.md), [devnet](https://github.com/starcoin-labs/starcoin-core/blob/devnet/starcoin-move/framework/starcoin-framework/doc/overview.md)
* *Starcoin stdlib* - [main](https://github.com/starcoin-labs/starcoin-core/blob/main/starcoin-move/framework/starcoin-stdlib/doc/overview.md), [testnet](https://github.com/starcoin-labs/starcoin-core/blob/testnet/starcoin-move/framework/starcoin-stdlib/doc/overview.md), [devnet](https://github.com/starcoin-labs/starcoin-core/blob/devnet/starcoin-move/framework/starcoin-stdlib/doc/overview.md)
* *Move stdlib* - [main](https://github.com/starcoin-labs/starcoin-core/blob/main/starcoin-move/framework/move-stdlib/doc/overview.md), [testnet](https://github.com/starcoin-labs/starcoin-core/blob/testnet/starcoin-move/framework/move-stdlib/doc/overview.md), [devnet](https://github.com/starcoin-labs/starcoin-core/blob/devnet/starcoin-move/framework/move-stdlib/doc/overview.md)

Follow our [contributing guidelines](CONTRIBUTING.md) and basic coding standards for the Starcoin Framework.

## Compilation and Generation

The documents above were created by the Move documentation generator for Starcoin. It is available as part of the Starcoin CLI. To see its options, run:
```shell
starcoin move document --help
```

The documentation process is also integrated into the framework building process and will be automatically triggered like other derived artifacts, via `cached-packages` or explicit release building.

## Running Move tests

To test our Move code while developing the Starcoin Framework, run `cargo test` inside this directory:

```
cargo test
```

(Alternatively, run `cargo test -p starcoin-framework` from anywhere.)

To skip the Move prover tests, run:

```
cargo test -- --skip prover
```

To filter and run only the tests in specific packages (e.g., `starcoin_stdlib`), run:

```
cargo test -- starcoin_stdlib --skip prover
```

(See tests in `tests/move_unit_test.rs` to determine which filter to use; e.g., to run the tests in `starcoin_framework` you must filter by `move_framework`.)

Sometimes, Rust runs out of stack memory in dev build mode.  You can address this by either:
1. Adjusting the stack size

```
export RUST_MIN_STACK=4297152
```

2. Compiling in release mode

```
cargo test --release -- --skip prover
```

## Layout
The overall structure of the Starcoin Framework is as follows:

```
├── starcoin-framework                                 # Sources, testing and generated documentation for Starcoin framework component
├── starcoin-token                                 # Sources, testing and generated documentation for Starcoin token component
├── starcoin-stdlib                                 # Sources, testing and generated documentation for Starcoin stdlib component
├── move-stdlib                                 # Sources, testing and generated documentation for Move stdlib component
├── cached-packages                                 # Tooling to generate SDK from mvoe sources.
├── src                                     # Compilation and generation of information from Move source files in the Starcoin Framework. Not designed to be used as a Rust library
├── releases                                    # Move release bundles
└── tests
```
