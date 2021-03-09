---
title: 通过治理机制修改链上配置
weight: 7
---

关于链上治理的介绍请参看 [去中心化组织治理](../key_concepts/dao_governance/), 这是一个例子，演示如何通过 cli 修改链上配置。

<!--more-->

下面用修改Version的信息来举例说明：

## 修改Version的配置：

1. 把Version的major修改为8，发起投票。

注意：每个配置修改，对应的提案脚本不一样，参考附录的表格。
```shell
#先解锁默认账户，可以通过 account show命令查询。注意：以下用到的账户（0x43019bd9fd7b25c5275867a0f0b17010）都需要替换成自己的。
account unlock 0x43019bd9fd7b25c5275867a0f0b17010 -d 1800
dev execute -s 0x43019bd9fd7b25c5275867a0f0b17010 -b --script propose_update_version --arg 8 0
```
提案发起后，用户需要等待公示期过后才能开始投票。
可以使用如下命令查看提案信息。

``` shell
dev call --module-address 0x1 --module-name Dao --func-name proposal_info -t 0x1::STC::STC -t 0x1::ModifyDaoConfigProposal::DaoConfigUpdate --arg 0x43019bd9fd7b25c5275867a0f0b17010 --arg 0
```

2. 用户投票

投票的用户需要 balance 不能为0，可以通过挖矿或者其他账户转账的方式获得。
``` shell
# 解锁投票用户
account unlock 0x1ddb8ec4850de3a57dede0f82edc5ec3
dev execute -s 0x1ddb8ec4850de3a57dede0f82edc5ec3 -b --script cast_vote -t 0x1::STC::STC -t 0x1::ModifyDaoConfigProposal::DaoConfigUpdate --arg 0x43019bd9fd7b25c5275867a0f0b17010 --arg 0 --arg true --arg 100000u128
```
投票完成，需要等待投票期结束。

3. 执行提案

注意： Dao的提案通过后的执行脚本都一样，需要注意proposal_id参数，当前示例是0
``` shell
#用节点账号发起交易执行提案
dev execute  -b --script execute_on_chain_config_proposal -t 0x1::Version::Version  --arg 0x43019bd9fd7b25c5275867a0f0b17010 --arg 0
```

4. 验证Version参数是否正确

```shell
dev call --module-address 0x1 --module-name Version --func-name get
```
返回结果如下：
```
+-----+
| U64 |
+-----+
| 8   |
+-----+
```
恭喜你，整个修改流程就完成了。

