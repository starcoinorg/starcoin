---
title: 通过治理机制修改 DAO 的设置
weight: 6
---

关于链上治理的介绍请参看 [去中心化组织治理](../key_concepts/dao_governance/), 这是一个例子，演示如何通过 cli 进行链上治理。

<!--more-->

DAO 本身也拥有几个链上参数，包括：

- voting_delay: 提案公示期。
- voting_period: 投票期。
- quorum_vote: 投票率。
- min_action_delay: 提案执行的最小公示期。

这些参数也可以通过 DAO 本身去投票修改。

DEV 环境下，STC 的 DAO 治理参数默认是：

```
voting_delay: 60000,       // 1min
voting_period: 60 * 60 * 1000, // 1h
voting_quorum_rate: 4,  // 4%
min_action_delay: 60 * 60 * 1000, // 1h
```

以下通过 cli 命令，演示如何投票更改 STC 治理参数 中的提案公示期为 `60 * 60 * 1000 = 1h`， 以展示 *提案-投票-执行* 这系列流程。


注：以下假设你使用的是 DEV 环境的节点，并且节点默认账号是  `0x3ce9c3beeb95b555f5e3f2ac297afbf1`。
命令中出现 `0x3ce9c3beeb95b555f5e3f2ac297afbf1` 的地方需要换成你的节点的默认账号。

1. 提交修改 DaoConfig 的提案（具体参数可以参考 stdlib 中 ModifyDaoConfigProposal module 的文档说明）：

``` bash
# 解锁节点账号，用节点账号发起提案
account unlock 0x3ce9c3beeb95b555f5e3f2ac297afbf1
account execute-function -s 0x3ce9c3beeb95b555f5e3f2ac297afbf1 --function 0x1::ModifyDaoConfigProposal::propose -t 0x1::STC::STC --arg 60000 600000 4u8 1000 0
```
注：dao参数不能等于0，否则该参数不会被更新。

提案发起后，用户需要等待公示期过后才能开始投票。

可以使用如下命令查看提案信息。

``` bash
dev call --function 0x1::Dao::proposal_info -t 0x1::STC::STC -t 0x1::ModifyDaoConfigProposal::DaoConfigUpdate --arg 0x3ce9c3beeb95b555f5e3f2ac297afbf1
```

返回结果包含了五个值，依次是：proposal_id，投票开始时间，投票结束时间，赞成票数，反对票数。

``` json
{
  "ok": [
    {
      "U64": "0"
    },
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
account execute-function -s 0x0000000000000000000000000a550c18 --function 0x1::DaoVoteScripts::cast_vote -t 0x1::STC::STC -t 0x1::ModifyDaoConfigProposal::DaoConfigUpdate --arg 0x3ce9c3beeb95b555f5e3f2ac297afbf1 --arg 0 --arg true --arg 490000000000000000u128
```

再次查看提案信息。

``` bash
dev call --function 0x1::Dao::proposal_info -t 0x1::STC::STC -t 0x1::ModifyDaoConfigProposal::DaoConfigUpdate --arg 0x3ce9c3beeb95b555f5e3f2ac297afbf1
```

返回：

``` json
{
  "ok": [
    {
      "U64": "0"
    },
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
dev call --function 0x1::Dao::proposal_state -t 0x1::STC::STC -t 0x1::ModifyDaoConfigProposal::DaoConfigUpdate --arg 0x0000000000000000000000000a550c18 --arg 0
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
account execute-function -s 0x3ce9c3beeb95b555f5e3f2ac297afbf1 --function 0x1::Dao::queue_proposal_action -t 0x1::STC::STC -t 0x1::ModifyDaoConfigProposal::DaoConfigUpdate --arg 0x3ce9c3beeb95b555f5e3f2ac297afbf1 --arg 0
```

4. 取回质押的 Token

投票期过后，用户可以把自己质押的 token 取回，
使用如下脚本提交交易：
``` bash
# 取回基金会账号质押的 token
account execute-function -s 0x0000000000000000000000000a550c18 --function 0x1::DaoVoteScripts::unstake_vote -t 0x1::STC::STC -t 0x1::ModifyDaoConfigProposal::DaoConfigUpdate --arg 0x3ce9c3beeb95b555f5e3f2ac297afbf1 --arg 0
```

5. 执行提案

待执行的提案在执行公示期过后，可以由任何人发起交易触发执行。命令如下：

``` bash
# # 用节点账号发起交易执行提案
account execute-function -s 0x3ce9c3beeb95b555f5e3f2ac297afbf1 --function 0x1::ModifyDaoConfigProposal::execute -t 0x1::STC::STC --arg 0x3ce9c3beeb95b555f5e3f2ac297afbf1 --arg 0
```

6. 确认执行结果

最后，我们还需要确认参数被成功修改掉。

```bash
starcoin% dev call --function 0x1::Dao::voting_delay -t 0x1::STC::STC
{
  "ok": [
    {
      "type": "U64",
      "value": 3600
    }
  ]
}
```
7.清理掉完成的提案
```bash
account execute-function -s 0x3ce9c3beeb95b555f5e3f2ac297afbf1 --function 0x1::Dao::destroy_terminated_proposal -t 0x1::STC::STC -t 0x1::ModifyDaoConfigProposal::DaoConfigUpdate --arg 0x3ce9c3beeb95b555f5e3f2ac297afbf1 0
```
以上是一个去中心化治理的案例流程，它没有展示出 DAO 模块的所有功能。
更多请探索 Starcoin 标准库的官方文档。