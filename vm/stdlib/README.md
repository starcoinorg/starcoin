# Starcoin Move standard library

Starcoin Move stdlib include the Move standard library and starcoin Move framework.

Note: The Starcoin Move Framework code is migrate to [starcoin-framework](https://github.com/starcoinorg/starcoin-framework/). This project is provide capability for integration starcoin-framework with starcoin genesis.

## Compilation and Generation

Compile and generate document:

```shell
cargo run 
```

Release a new `N` version, N must bean an uint number, such as v3:

```shell
cargo run -- -v 3 -m StdlibUpgradeScripts -f upgrade_from_v2_to_v3 -a 3185136000000000000u128
```

Get help

```shell
cargo run -- --help
```

## Generate genesis file

The halley network use the latest stdlib, if stdlib changed, should regenerate the halley genesis file.
```shell
cd ../../genesis && cargo run
```

last, commit all modified source files and generated files.

## Layout
The overall structure of the Starcoin stdlib is as follows:

```
├── compiled                                # Generated files and public rust interface to the Diem Framework
│   ├── 1  # stdlib release v1
│   ├── 2  # stdlib release v2
│       ├── 1-2
│             ├── stdlib/*.mv           # The compiled Move bytecode of the changed module from v1 to v2
│             └── stdlib.blob           # Generated package for upgrade from v1 to v2
│       └── stdlib                      # The compiled Move bytecode of the Starcoin stdlib v2 source modules
│   ├── latest                          # the latest stdlib compiled
│       ├── error_descriptions/*.errmap         # Generated error descriptions for use by the Move Explain tool
│       ├── stdlib/*.mv                         # The compiled Move bytecode of the Starcoin stdlib latest source modules
│       ├── docs/*.md                           # Generated documentation for the Starcoin move framework modules
│       └── transaction_scripts/abi             # Generated ABIs for script function transactions, and all new transactions
├── modules                                 # Starcoin stdlib source modules, script modules, and generated documentation
│   ├── *.move
│   └── doc/*.md                      # Generated documentation for the Starcoin stdlib and framework modules
├── src                                     # Compilation and generation of information from Move source files in the Starcoin Move stdlib. Not designed to be used as a Rust library
└── tests
```
