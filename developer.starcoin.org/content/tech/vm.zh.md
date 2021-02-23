---
title: 虚拟机 
weight: 5
---

Starcoin 区块链是一个旨在用作金融基础设施的去中心化系统。更改区块链状态的唯一方法是在 Starcoin VM 上执行交易。交易里包含了由Move编写的智能合约。 Move 是Libra Core开发的一种智能合约编程语言， Starcoin 区块链采用 Move 来编写智能合约是因为它在安全性上表现优异。
<!--more-->

## 虚拟机运行时(VM Runtime)

交易在VM runtime上按顺序验证和执行。如果把VM runtime看成一个黑盒，那么它的输入就是交易和区块链状态，它的输出则是交易执行后的区块链状态。这里的状态是指给定区块高度的Starcoin区块链中的存储的数据和代码。Starcoin交易里的智能合约是用Move编写的，所以Starcoin VM实际上是基于MoveVM开发的。为了有效的利用MoveVM的数据和代码缓存，Starcoin VM采用了一个链状态包裹层，将链状态与MoveVM缓存无缝对接。 另外Starcoin VM正在开发基于状态计费的gas机制，来有效提升存储空间的利用率。也在探索通过形式化验证来提升合约的安全性。 

## 标准库(Standard library)

Starcoin区块链中的每次状态更改都是通过执行嵌入在交易中的Move脚本发生的。交易脚本可以调用标准库里的方法来完成各种状态更改。 Starcoin标准库包括：
- 在创世交易中发布的Modules。这些核心系统Modules，除了最基本的功能（例如Account和Coin）外，它还支持用户自定义的Token。具体示例请参阅XX。
- 一组经过授权的交易脚本，用来支持转账等常用交易。

## Interact with

Starcoin组件通过Executor与Starcoin VM交互。Executor调用VM执行一个交易块。Transaction pool组件使用VM的验证功能在无效交易发送给共识组件之前将其丢弃。然后，共识组件使用VM执行交易并将新状态写入状态数据库。

## Folder Structure

```
├── executor                 # executor
├── vm
│   ├── vm_runtime           # vm runtime
│   ├── stdlib               # standard library
│   ├── functional_tests     # functinal tests

```