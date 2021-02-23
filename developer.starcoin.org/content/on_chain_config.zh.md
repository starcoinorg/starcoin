---
id: on_chain_config
title: OnChainConfig
weight: 18
---

OnChainConfig主要实现不用重启链的情况下，可以更新链上的一些配置。在 Starcoin 中，可以通过标准库中的DAO实现来修改链上的参数。

下面主要介绍OnChainConfig的功能以及使用方式。

## OnChainConfig可修改的配置：
- TransactionPublishOption
- VMConfig
- ConsensusConfig
- RewardConfig
- TransactionTimeoutConfig
- DaoConfig
- Version

具体每个配置的详细字段信息，请参考附录表格。

## OnChainConfig修改流程
OnChainConfig的修改流程主要包括以下步骤：

- 确定待修改配置的参数值，发起OnChainConfigDao提案。
- 用户发起投票。
- 提案通过执行修改交易。
- 验证参数是否生效。

Dao具体流程参考[Dao](./dao_proposal.zh.md)。

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

投票的用户需要blance不能为0，可以通过挖矿或者其他账户转账的方式获得。
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

## 附录：OnChainConfig配置列表

| 配置模块  | 可修改字段  | 提案脚本名称 | 验证方法 |
|:------------- |:---------------|:-------------|:-------------|
| TransactionPublishOption     | script_allow_list，module_publishing_allowed | propose_update_txn_publish_option | TransactionPublishOption::is_script_allowed，is_module_allowed |
| VMConfig   |  instruction_schedule<br>native_schedule<br>global_memory_per_byte_cost<br>global_memory_per_byte_write_cost<br>min_transaction_gas_units<br>large_transaction_cutoff<br>instrinsic_gas_per_byte<br>maximum_number_of_gas_units<br>min_price_per_gas_unit<br>max_price_per_gas_unit<br>max_transaction_size_in_bytes<br>gas_unit_scaling_factor<br>default_account_size | propose_update_vm_config | 需执行交易验证，参考: test_modify_on_chain_vm_config_option的单元测试 |
| ConsensusConfig     | uncle_rate_target,<br>    base_block_time_target,    base_reward_per_block,<br> base_reward_per_uncle_percent,<br>    epoch_block_count,<br>    base_block_difficulty_window,<br>    min_block_time_target,<br>    max_block_time_target,<br>    base_max_uncles_per_block,<br>    base_block_gas_limit,<br>    strategy,<br>        | propose_update_consensus_config | ConsensusConfig::get_config |
| RewardConfig     | reward_delay | propose_update_reward_config | RewardConfig::get_reward_config |
| TransactionTimeoutConfig     | duration_seconds | propose_update_txn_timeout_config | TransactionTimeoutConfig::duration_seconds |
| DaoConfig     | voting_delay,<br> voting_period,<br> voting_quorum_rate,<br> min_action_delay,<br>  |  propose_modify_dao_config |  Dao::voting_delay,<br> voting_period,<br> voting_quorum_rate,<br> min_action_delay |
| Version     | major        |           propose_update_version | Version::get |
