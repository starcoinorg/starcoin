---
title: 挖矿
weight: 4
---

`starcoin_miner` 命令行工具用于远程连接到 starcoin 节点，并提供挖矿功能。

<!--more-->

## 使用方法

`starcoin_miner` [OPTIONS]

OPTIONS:
- -a, --server <server> , 指定要连接到的 starcoin node 的 rpc 地址，默认值为 127.0.0.1:9870
- -n, --thread-num <thread-num>，线程数，默认为 1。

## 连接到节点进行挖矿
当本地启动了 starcoin node 时，我们可以运行如下命令，启动4个线程连接到本地节点进行挖矿。


```shell
starcoin_miner -n 4
```
启动后可以看到 console 中有如下信息:


```shell
Miner client Total seals found:  3
starcoin-miner-cpu-worker-0 ⠦ [00:00:00] Hash rate:      20 Seals found:  17
starcoin-miner-cpu-worker-1 ⠦ [00:00:00] Hash rate:      21 Seals found:  17
starcoin-miner-cpu-worker-2 ⠤ [00:00:00] Hash rate:      20 Seals found:  16
starcoin-miner-cpu-worker-3 ⠤ [00:00:00] Hash rate:      20 Seals found:  16
2020-10-28T09:09:53.006852+08:00 INFO - Seal found 16718533681172480617

```

日志中可以看到挖到的 seals 总数，每个线程的算力，以及新计算得出的 seal 等信息。
