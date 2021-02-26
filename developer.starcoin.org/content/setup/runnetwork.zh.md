---
title: 运行/加入网络
weight: 6
---

`starcoin` 命令行工具可以用来启动本地网络或者加入测试网络或者主网。运行本地网络或者加入测试网络可以方便测试用户合约代码。可以使用 dev 命令编译, 发布，执行智能合约。 

<!--more-->

## 使用方法

`starcoin` [FLAGS] [OPTIONS] [SUBCOMMAND]

FLAGS:
- --disable-file-log 禁止文件日志
- --disable-seed 禁止 seed


OPTIONS:
- --seed 指定 seed
- -n,--net 网络名 ,可以是 dev/halley/proxima/main 其中一个，本地测试网络使用 dev。如果想自定义网络请参看[运行自定义网络](./runcustomnetwork)


SUBCOMMAND:
- console background 运行节点，节点启动完成后，启动交互式命令行工具
- help  输出帮助信息


## 运行本地网络

使用如下命令即可启动 dev 节点， dev 节点默认是按需出块，有交易的时候才会出块:

```
starcoin -n dev
```

节点启动成功后，可以在日志中找到:

```shell
Self address is: /ip4/127.0.0.1/tcp/9840/p2p/12D3KooWPePRG6BDdjgtEYmPDxNyJfMWpQ1Rwgefuz9eqksLfxJb
```

接下来设置第二个节点:

```shell
starcoin -n dev --seed /ip4/127.0.0.1/tcp/9840/p2p/12D3KooWPePRG6BDdjgtEYmPDxNyJfMWpQ1Rwgefuz9eqksLfxJb

```

当然你也可以使用自带交互式命令行的方式启动:

```shell
starcoin -n dev console
```

重复上述步骤，你就可以启动一个本地 dev 网络.

## 加入 Halley 网络

**Halley** 是 starcoin 的第一个测试网络，它上面的数据会定时清理。

可以使用如下命令加入 Halley 网络:

```shell
starcoin -n halley
```

"Halley" 这个名字的灵感来自于[哈雷彗星](https://en.wikipedia.org/wiki/Halley%27s_Comet)，正式名为 1P/Halley，是一颗短周期彗星，每隔75-76年从地球上看到一次。

## 加入 Proxima 网络

**Proxima** 是 starcoin 长期运行的一个测试网络，将在 2020 年第三季度发布，正式发布后不会清除数据.

可以使用如下命令加入 Proxima 网络:

```shell
starcoin -n proxima
```

"Proxima" 这个名字的灵感来自于[比邻星](https://en.wikipedia.org/wiki/Proxima_Centauri)，它是一颗小的、低质量的恒星，位于半人马座南部的南半球，距离太阳4.244光年(1.301pc)。

## 加入主网

TODO