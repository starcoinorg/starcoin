---
title: 链上配置
weight: 8
---

链上配置(OnChainConfig)主要实现不用更新节点的情况下，通过链上交易更新链上的一些配置。在 Starcoin 中，可以通过标准库中的DAO实现来修改链上的参数。

<!--more-->

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

Dao具体流程参考[Dao](./dao_governance)。


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
