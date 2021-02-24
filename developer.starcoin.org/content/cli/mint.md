---
title: Mint
weight: 4
---

`starcoin_miner` Command line tools are used to remotely connect to starcoin nodes and provide mining capabilities.

<!--more-->

## Usage

`starcoin_miner` [OPTIONS]

OPTIONS:
- -a, --server <server> , Specifies the rpc address of the starcoin node to connect to, defaults to 127.0.0.1:9870
- -n, --thread-num <thread-num>，Number of threads, defaults to 1.

## Run miner client

When the starcoin node is started locally, we can run the following command to start 4 threads connected to the local node for mining.


```shell
starcoin_miner -n 4
```
Upon startup, you can see the following message in the console


```shell
Miner client Total seals found:  3
starcoin-miner-cpu-worker-0 ⠦ [00:00:00] Hash rate:      20 Seals found:  17
starcoin-miner-cpu-worker-1 ⠦ [00:00:00] Hash rate:      21 Seals found:  17
starcoin-miner-cpu-worker-2 ⠤ [00:00:00] Hash rate:      20 Seals found:  16
starcoin-miner-cpu-worker-3 ⠤ [00:00:00] Hash rate:      20 Seals found:  16
2020-10-28T09:09:53.006852+08:00 INFO - Seal found 16718533681172480617

```
The log shows information such as the total number of seals, the Hash rate of each thread, and the latest found seal.
