Code Layout Overview
===

Overview of codes.

All codes directories are listed here. (in alphabetical order)

| directory                                    | description                                                                                             | details                                                       |
|----------------------------------------------|---------------------------------------------------------------------------------------------------------|---------------------------------------------------------------|
| [abi](abi)                                   | Move (language for smart contracts) ABI                                                                 |                                                               |
| [account](account)                           | implementation of accounts                                                                              |                                                               |
| [benchmarks](benchmarks)                     | using cargo bench to measure code performance                                                           | [README](benchmarks/README.md)                                |
| [block-relayer](block-relayer)               | blocks transaction service                                                                              |                                                               |
| [chain](chain)                               | core of Starcoin chain                                                                                  |                                                               |
| [cmd](cmd)                                   | command-line tools source codes                                                                         | [binary file description](README.md#binary-file-description)  |
| [commons](commons)                           | independent tool crate                                                                                  | [README](commons/README.md)                                   |
| [config](config)                             | default configs of Starcoin chain and cmd, and methods to build config structs                          |                                                               |
| [consensus](consensus)                       | implementation of blockchain consensus                                                                  |                                                               |
| [contrib-contracts](contrib-contracts)       | contribution move libraries, which are not suitable to be in Starcoin stdlib                            | [README](contrib-contracts/README.md)                         |
| [dataformat-generator](dataformat-generator) | generates [onchain_events.yml](stc/onchain_events.yml) & [starcoin_types.yml](etc/starcoin_types.yml)   |                                                               |
| [docker](docker)                             | docker config for Starcoin                                                                              |                                                               |
| [etc](etc)                                   | [onchain_events.yml](stc/onchain_events.yml) & [starcoin_types.yml](etc/starcoin_types.yml)             |                                                               |
| [examples](examples)                         | [User Guide of Move Package Manager(mpm)](https://github.com/starcoinorg/guide-to-move-package-manager) |                                                               |
| [executor](executor)                         | executor for Starcoin                                                                                   |                                                               |
| [genesis](genesis)                           | lib and tool to generate and load genesis                                                               | [README](genesis/README.md)                                   |
| [kube](kube)                                 | deploy Starcoin on K8S                                                                                  | [README](kube/README.md)                                      |
| [miner](miner)                               | miner service                                                                                           |                                                               |
| [network](network)                           | kit of network                                                                                          |                                                               |
| [network-p2p](network-p2p)                   | kit of P2P network                                                                                      |                                                               |
| [network-rpc](network-rpc)                   | kit of RPC network                                                                                      |                                                               |
| [node](node)                                 | local node                                                                                              |                                                               |
| [rpc](rpc)                                   | kit of RPC                                                                                              |                                                               |
| [scripts](scripts)                           | scripts helping for building, testing etc.,                                                             |                                                               |
| [state](state)                               | maintain the chain state                                                                                |                                                               |
| [storage](storage)                           | backend storage of chain                                                                                |                                                               |
| [stratum](stratum)                           | stratum mining                                                                                          | [Stratum Mining Protocol](stratum/stratum_mining_protocol.md) |
| [sync](sync)                                 | sync from network                                                                                       |                                                               |
| [test-helper](test-helper)                   | test helper functions                                                                                   |                                                               |
| [testsuite](testsuite)                       | codes & scripts for setting up test                                                                     |                                                               |
| [txpool](txpool)                             | handler of txn                                                                                          |                                                               |
| [types](types)                               | user-defined types for Starcoin                                                                         |                                                               |
| [vm](vm)                                     | virtual machine built upon Starcoin                                                                     |                                                               |
