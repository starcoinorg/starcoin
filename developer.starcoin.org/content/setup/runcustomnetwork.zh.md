---
title: 运行自定义的区块链网络
weight: 7
---

`starcoin` 支持运行一个用户自定义区块链网络，方便用户搭建私有链进行测试或者二次开发。

<!--more-->

## 使用方法

`starcoin` -n chain_name:chain_id --genesis-config genesis_config_name_or_path

运行自定义区块链网络的时候，-n,--net 参数的值由三个参数通过 `:` 符号拼接而成。

* chain_name: 例如 `my_chain`, 这个名字会用作数据文件夹的名称。
* chain_id: u8 类型的数字，比如 `123`。
* genesis_config_name_or_path: 可以是一个内置的区块链网络的名称，表示复用改网络配置，比如 `halley`, 也可以是一个 genesis 配置文件的地址。后面会介绍如何生成 genesis 配置文件。


## 生成 genesis 配置文件

```
starcoin_generator -n my_chain:123 --genesis-config halley genesis_config
```

该命令将以 `halley` 配置文件为模板， 生成一个名为 genesis_config.json 配置文件在 ~/.starcoin/$chain_name 目录下。然后用任何编辑器修改 ~/.starcoin/$chain_name/genesis_config.json 文件中的参数。
如果不想生成在默认的 ~/.starcoin 目录下，也可以通过 -d 参数指定目录。

## 生成 genesis 区块

```
starcoin_generator -n my_chain:123 genesis
```

该命令将根据前面生成的 genesis 配置文件生成 genesis 区块。默认的 genesis 配置文件即是 ~/.starcoin/$chain_name/genesis_config.json。当然，也可以将 genesis_config.json 文件放置在其他位置，然后通过绝对地址指定，比如 

```
starcoin_generator -n my_chain:123 --genesis-config /data/conf/my_chain/genesis_config.json genesis
```

## 运行自定义网络节点

使用如下命令即可启动自定义网络节点:

```
starcoin -n my_chain:123 console 
```

然后再启动其他节点，通过 --seed 指定第一个节点为 seed，即可组成一个网络。需要注意的是，多个节点必须使用同一套 genesis 配置文件来生成 genesis 区块，才能组成一个网络。