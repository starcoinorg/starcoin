---
id: dao
title: 去中心化治理
---

Token 的去中心化治理已经成为区块链必不可少的一部分。
在 Starcoin 中，可以用 Move 语言方便的实现 DAO 功能。
标准库中内置了一个 DAO 实现，Starcoin 本身也可以通过该模块对各种链上参数进行投票治理。

本节主要介绍该模块的功能以及使用方式。

## DAO 功能

一个基本的 DAO 治理流程大概包括：
- 发起人发起提案。
- 用户投票。
- 提案通过以及执行。

目前以太坊上已经有一套功能完备的实现，
但 Starcoin 的 Token 模型以及 Move 的编程模型和 ERC20/Solidity 有很大区别，无法直接复用现有的代码体系。
因此，我们设计和实现了适合 Starcoin 的 DAO 治理。

![dao](/images/dao.jpg)

Starcoin 的 DAO 实现与以太坊的 DAO 实现一个最大的区别是：Starcoin 中，每种类型的提案都有一个单独的合约模块去控制，由该模块去实现提案的发起和提案的执行。

这是因为，以太坊中，智能合约可以通过动态分发去调用其他合约接口，因此可以做到一个合约去发起所有类别的提案，只需要在合约内部动态调用即可。但 Move 是一个函数静态分发调用的模型（这里不多叙述，感兴趣的读者可以阅读 Move 相关的文档），所有的代码调用都必须在编译时确定，做不到动态分发。因此产生了前述的区别。

提案的投票则由 DAO 模块统一负责，DAO 模块对提案做了抽象（实现上，提案是 DAO 模块的一个范型参数），用 `proposal_id` 去标识某个提案，至于提案是什么内容，它不关心，交给用户自己去判断。
投票时，用户通过 DAPP 去获取某个提案的具体内容，然后直接调用 DAO 模块的接口投票，赞成或者反对。

这样可以做到，不同提案可以实现自己的提案逻辑，但又可以共享 DAO 模块的投票功能。

标准库默认提供了以下几种提案：
- ModifyDaoConfigPorposal: 更改 DAO 投票参数的提案。
- OnChainConfigDao: 更改链上参数的提案。
- UpgradeModuleDaoProposal: 升级合约代码的提案。

在发行自己的 Token 时，如有类似需求，用户可以直接接入标准库中的提案，如果有其他更复杂的需求，也可以编写更多的自定义的提案。

### 用户投票

用户投票时，需要质押自己的 Token，票数和 Token 数成正比，即：一币一票。
在投票期，用户可以多次投票，也可以撤回投票，甚至可以倒戈到对方（由赞成变反对，由反对变赞成）。
投票期过后，用户可以立即提回自己质押的 Token。

### 提案通过和执行

投票期过后，如果投票率通过，并且赞成人数多余反对人数，那提案就通过了。
此时，任何人都可以发送交易将提案标识为 **待执行** 状态，放入到队列中，等待执行。
当执行公示期过后，任何人可以发送交易去执行该提案。
提案执行后，提案发起人才可以删除自己的提案，释放提案占用的链上空间。

提案的一个完整生命周期如下：

![proposal lifetime](/images/proposal_lifetime.jpg)

## 案例 - 修改 DAO 参数

DAO 本身也拥有几个链上参数，包括：
- voting_delay: 提案公示期。
- voting_period: 投票期。
- quorum_vote: 投票率。
- min_action_delay: 提案执行的最小公示期。

这些参数也可以通过 DAO 本身去投票修改。

DEV 环境下，STC 的 DAO 治理参数默认是：

```
voting_delay: 60,       // 1min
voting_period: 60 * 60, // 1h
voting_quorum_rate: 4,  // 4%
min_action_delay: 60 * 60, // 1h
```

以下通过 cli 命令，演示如何投票更改 STC 治理参数 中的提案公示期为 `60 * 60 = 1h`，
以展示 *提案-投票-执行* 这系列流程。


注：以下假设你使用的是 DEV 环境的节点，并且节点默认账号是  `0x3ce9c3beeb95b555f5e3f2ac297afbf1`。
命令中出现 `0x3ce9c3beeb95b555f5e3f2ac297afbf1` 的地方需要换成你的节点的默认账号。

1. 提交修改 DaoConfig 的提案（具体参数可以参考 stdlib 中关于该脚本的文档说明）：

``` bash
# 解锁节点账号，用节点账号发起提案
dev unlock 0x3ce9c3beeb95b555f5e3f2ac297afbf1
dev execute -b --script propose_modify_dao_config  -t 0x1::STC::STC --arg 3600 0 0u8 0 0
```

提案发起后，用户需要等待公示期过后才能开始投票。

可以使用如下命令查看提案信息。

``` bash
dev call --module-address 0x1 --module-name Dao --func-name proposal_info -t 0x1::STC::STC -t 0x1::ModifyDaoConfigProposal::DaoConfigUpdate --arg 0x3ce9c3beeb95b555f5e3f2ac297afbf1 --arg 0
```

返回结果包含了四个值，依次是：投票开始时间，投票结束时间，赞成票数，反对票数。

``` json
{
  "ok": [
    {
      "type": "U64",
      "value": 1602596122
    },
    {
      "type": "U64",
      "value": 1602599722
    },
    {
      "type": "U128",
      "value": 0
    },
    {
      "type": "U128",
      "value": 0
    }
  ]
```

2. 用户投票

DEV 链启动后，会默认给基金会账号 mint 一笔 stc，而节点账号还没有 stc，
所以需要使用基金会账号来投票，让提案通过。（DEV 环境下，用户可以直接使用基金会账号）

``` bash
# 解锁基金会账号，用基金会账号对提案投票
account unlock 0000000000000000000000000a550c18
dev execute -s 0x0000000000000000000000000a550c18 -b --script cast_vote -t 0x1::STC::STC -t 0x1::ModifyDaoConfigProposal::DaoConfigUpdate --arg 0x3ce9c3beeb95b555f5e3f2ac297afbf1 --arg 0 --arg true --arg 490000000000000000u128
```

再次查看提案信息。

``` bash
dev call --module-address 0x1 --module-name Dao --func-name proposal_info -t 0x1::STC::STC -t 0x1::ModifyDaoConfigProposal::DaoConfigUpdate --arg 0x3ce9c3beeb95b555f5e3f2ac297afbf1 --arg 0
```

返回：

``` json
{
  "ok": [
    {
      "type": "U64",
      "value": 1602596122
    },
    {
      "type": "U64",
      "value": 1602599722
    },
    {
      "type": "U128",
      "value": 490000000000000000
    },
    {
      "type": "U128",
      "value": 0
    }
  ]
}
```

投完票，然后等待投票期结束。

3. 提案通过

投票期结束后，如果提案是通过状态，那就可以将其放入待执行队列，进入执行公示期。

可以通过以下命令查看提案状态：

``` bash
dev call --module-address 0x1 --module-name Dao --func-name proposal_state -t 0x1::STC::STC -t 0x1::ModifyDaoConfigProposal::DaoConfigUpdate --arg 0x3ce9c3beeb95b555f5e3f2ac297afbf1 --arg 0
```

返回：（如果返回结果是 4，说明提案通过，其他提案状态可以参考标准库文档）

``` json
{
  "ok": [
    {
      "type": "U8",
      "value": 4
    }
  ]
}
```


放入待执行对列：

``` bash
# 用节点账号将通过后的提案入队列
dev execute -b  --script queue_proposal_action -t 0x1::STC::STC -t 0x1::ModifyDaoConfigProposal::DaoConfigUpdate --arg 0x3ce9c3beeb95b555f5e3f2ac297afbf1 --arg 0
```


4. 取回质押的 Token

投票期过后，用户可以把自己质押的 token 取回，
使用如下脚本提交交易：
``` bash
# 取回基金会账号质押的 token
dev execute -b -s 0000000000000000000000000a550c18  --script unstake_vote -t 0x1::STC::STC -t 0x1::ModifyDaoConfigProposal::DaoConfigUpdate --arg 0x3ce9c3beeb95b555f5e3f2ac297afbf1 --arg 0
```

5. 执行提案

待执行的提案在执行公示期过后，可以由任何人发起交易触发执行。命令如下：

``` bash
# # 用节点账号发起交易执行提案
dev execute  -b --script execute_modify_dao_config_proposal -t 0x1::STC::STC  --arg 0x3ce9c3beeb95b555f5e3f2ac297afbf1 --arg 0
```

5. 确认执行结果

最后，我们还需与奥确认参数被成功修改掉。

```bash
starcoin% dev call --module-address 0x1 --module-name Dao --func-name voting_delay -t 0x1::STC::STC
{
  "ok": [
    {
      "type": "U64",
      "value": 3600
    }
  ]
}
```

以上是一个去中心化治理的案例流程，它没有展示出 DAO 模块的所有功能。
更多请探索 Starcoin 标准库的官方文档。