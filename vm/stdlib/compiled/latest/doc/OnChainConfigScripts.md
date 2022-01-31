
<a name="0x1_OnChainConfigScripts"></a>

# Module `0x1::OnChainConfigScripts`



-  [Function `propose_update_consensus_config`](#0x1_OnChainConfigScripts_propose_update_consensus_config)
-  [Function `propose_update_reward_config`](#0x1_OnChainConfigScripts_propose_update_reward_config)
-  [Function `propose_update_txn_publish_option`](#0x1_OnChainConfigScripts_propose_update_txn_publish_option)
-  [Function `propose_update_txn_timeout_config`](#0x1_OnChainConfigScripts_propose_update_txn_timeout_config)
-  [Function `propose_update_vm_config`](#0x1_OnChainConfigScripts_propose_update_vm_config)
-  [Function `propose_update_move_language_version`](#0x1_OnChainConfigScripts_propose_update_move_language_version)
-  [Function `execute_on_chain_config_proposal`](#0x1_OnChainConfigScripts_execute_on_chain_config_proposal)
-  [Function `execute_on_chain_config_proposal_v2`](#0x1_OnChainConfigScripts_execute_on_chain_config_proposal_v2)
-  [Specification](#@Specification_0)
    -  [Function `propose_update_consensus_config`](#@Specification_0_propose_update_consensus_config)
    -  [Function `propose_update_reward_config`](#@Specification_0_propose_update_reward_config)
    -  [Function `propose_update_txn_publish_option`](#@Specification_0_propose_update_txn_publish_option)
    -  [Function `propose_update_txn_timeout_config`](#@Specification_0_propose_update_txn_timeout_config)
    -  [Function `propose_update_vm_config`](#@Specification_0_propose_update_vm_config)
    -  [Function `propose_update_move_language_version`](#@Specification_0_propose_update_move_language_version)
    -  [Function `execute_on_chain_config_proposal`](#@Specification_0_execute_on_chain_config_proposal)
    -  [Function `execute_on_chain_config_proposal_v2`](#@Specification_0_execute_on_chain_config_proposal_v2)


<pre><code><b>use</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig">0x1::ConsensusConfig</a>;
<b>use</b> <a href="LanguageVersion.md#0x1_LanguageVersion">0x1::LanguageVersion</a>;
<b>use</b> <a href="OnChainConfigDao.md#0x1_OnChainConfigDao">0x1::OnChainConfigDao</a>;
<b>use</b> <a href="RewardConfig.md#0x1_RewardConfig">0x1::RewardConfig</a>;
<b>use</b> <a href="STC.md#0x1_STC">0x1::STC</a>;
<b>use</b> <a href="Signer.md#0x1_Signer">0x1::Signer</a>;
<b>use</b> <a href="TransactionPublishOption.md#0x1_TransactionPublishOption">0x1::TransactionPublishOption</a>;
<b>use</b> <a href="TransactionTimeoutConfig.md#0x1_TransactionTimeoutConfig">0x1::TransactionTimeoutConfig</a>;
<b>use</b> <a href="VMConfig.md#0x1_VMConfig">0x1::VMConfig</a>;
</code></pre>



<a name="0x1_OnChainConfigScripts_propose_update_consensus_config"></a>

## Function `propose_update_consensus_config`



<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="OnChainConfigScripts.md#0x1_OnChainConfigScripts_propose_update_consensus_config">propose_update_consensus_config</a>(account: signer, uncle_rate_target: u64, base_block_time_target: u64, base_reward_per_block: u128, base_reward_per_uncle_percent: u64, epoch_block_count: u64, base_block_difficulty_window: u64, min_block_time_target: u64, max_block_time_target: u64, base_max_uncles_per_block: u64, base_block_gas_limit: u64, strategy: u8, exec_delay: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> ( <b>script</b> ) <b>fun</b> <a href="OnChainConfigScripts.md#0x1_OnChainConfigScripts_propose_update_consensus_config">propose_update_consensus_config</a>(account: signer,
                                                      uncle_rate_target: u64,
                                                      base_block_time_target: u64,
                                                      base_reward_per_block: u128,
                                                      base_reward_per_uncle_percent: u64,
                                                      epoch_block_count: u64,
                                                      base_block_difficulty_window: u64,
                                                      min_block_time_target: u64,
                                                      max_block_time_target: u64,
                                                      base_max_uncles_per_block: u64,
                                                      base_block_gas_limit: u64,
                                                      strategy: u8,
                                                      exec_delay: u64) {
    <b>let</b> consensus_config = <a href="ConsensusConfig.md#0x1_ConsensusConfig_new_consensus_config">ConsensusConfig::new_consensus_config</a>(uncle_rate_target,
        base_block_time_target,
        base_reward_per_block,
        base_reward_per_uncle_percent,
        epoch_block_count,
        base_block_difficulty_window,
        min_block_time_target,
        max_block_time_target,
        base_max_uncles_per_block,
        base_block_gas_limit,
        strategy);
    <a href="OnChainConfigDao.md#0x1_OnChainConfigDao_propose_update">OnChainConfigDao::propose_update</a>&lt;<a href="STC.md#0x1_STC_STC">STC::STC</a>, <a href="ConsensusConfig.md#0x1_ConsensusConfig_ConsensusConfig">ConsensusConfig::ConsensusConfig</a>&gt;(&account, consensus_config, exec_delay);
}
</code></pre>



</details>

<a name="0x1_OnChainConfigScripts_propose_update_reward_config"></a>

## Function `propose_update_reward_config`



<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="OnChainConfigScripts.md#0x1_OnChainConfigScripts_propose_update_reward_config">propose_update_reward_config</a>(account: signer, reward_delay: u64, exec_delay: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> ( <b>script</b> ) <b>fun</b> <a href="OnChainConfigScripts.md#0x1_OnChainConfigScripts_propose_update_reward_config">propose_update_reward_config</a>(account: signer,
                                                   reward_delay: u64,
                                                   exec_delay: u64) {
    <b>let</b> reward_config = <a href="RewardConfig.md#0x1_RewardConfig_new_reward_config">RewardConfig::new_reward_config</a>(reward_delay);
    <a href="OnChainConfigDao.md#0x1_OnChainConfigDao_propose_update">OnChainConfigDao::propose_update</a>&lt;<a href="STC.md#0x1_STC_STC">STC::STC</a>, <a href="RewardConfig.md#0x1_RewardConfig_RewardConfig">RewardConfig::RewardConfig</a>&gt;(&account, reward_config, exec_delay);
}
</code></pre>



</details>

<a name="0x1_OnChainConfigScripts_propose_update_txn_publish_option"></a>

## Function `propose_update_txn_publish_option`



<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="OnChainConfigScripts.md#0x1_OnChainConfigScripts_propose_update_txn_publish_option">propose_update_txn_publish_option</a>(account: signer, script_allowed: bool, module_publishing_allowed: bool, exec_delay: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> ( <b>script</b> ) <b>fun</b> <a href="OnChainConfigScripts.md#0x1_OnChainConfigScripts_propose_update_txn_publish_option">propose_update_txn_publish_option</a>(account: signer,
                                                        script_allowed: bool,
                                                        module_publishing_allowed: bool,
                                                        exec_delay: u64) {
    <b>let</b> txn_publish_option = <a href="TransactionPublishOption.md#0x1_TransactionPublishOption_new_transaction_publish_option">TransactionPublishOption::new_transaction_publish_option</a>(script_allowed, module_publishing_allowed);
    <a href="OnChainConfigDao.md#0x1_OnChainConfigDao_propose_update">OnChainConfigDao::propose_update</a>&lt;<a href="STC.md#0x1_STC_STC">STC::STC</a>, <a href="TransactionPublishOption.md#0x1_TransactionPublishOption_TransactionPublishOption">TransactionPublishOption::TransactionPublishOption</a>&gt;(&account, txn_publish_option, exec_delay);
}
</code></pre>



</details>

<a name="0x1_OnChainConfigScripts_propose_update_txn_timeout_config"></a>

## Function `propose_update_txn_timeout_config`



<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="OnChainConfigScripts.md#0x1_OnChainConfigScripts_propose_update_txn_timeout_config">propose_update_txn_timeout_config</a>(account: signer, duration_seconds: u64, exec_delay: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> ( <b>script</b> ) <b>fun</b> <a href="OnChainConfigScripts.md#0x1_OnChainConfigScripts_propose_update_txn_timeout_config">propose_update_txn_timeout_config</a>(account: signer,
                                                        duration_seconds: u64,
                                                        exec_delay: u64) {
    <b>let</b> txn_timeout_config = <a href="TransactionTimeoutConfig.md#0x1_TransactionTimeoutConfig_new_transaction_timeout_config">TransactionTimeoutConfig::new_transaction_timeout_config</a>(duration_seconds);
    <a href="OnChainConfigDao.md#0x1_OnChainConfigDao_propose_update">OnChainConfigDao::propose_update</a>&lt;<a href="STC.md#0x1_STC_STC">STC::STC</a>, <a href="TransactionTimeoutConfig.md#0x1_TransactionTimeoutConfig_TransactionTimeoutConfig">TransactionTimeoutConfig::TransactionTimeoutConfig</a>&gt;(&account, txn_timeout_config, exec_delay);
}
</code></pre>



</details>

<a name="0x1_OnChainConfigScripts_propose_update_vm_config"></a>

## Function `propose_update_vm_config`



<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="OnChainConfigScripts.md#0x1_OnChainConfigScripts_propose_update_vm_config">propose_update_vm_config</a>(account: signer, instruction_schedule: vector&lt;u8&gt;, native_schedule: vector&lt;u8&gt;, global_memory_per_byte_cost: u64, global_memory_per_byte_write_cost: u64, min_transaction_gas_units: u64, large_transaction_cutoff: u64, instrinsic_gas_per_byte: u64, maximum_number_of_gas_units: u64, min_price_per_gas_unit: u64, max_price_per_gas_unit: u64, max_transaction_size_in_bytes: u64, gas_unit_scaling_factor: u64, default_account_size: u64, exec_delay: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> ( <b>script</b> ) <b>fun</b> <a href="OnChainConfigScripts.md#0x1_OnChainConfigScripts_propose_update_vm_config">propose_update_vm_config</a>(account: signer,
                                               instruction_schedule: vector&lt;u8&gt;,
                                               native_schedule: vector&lt;u8&gt;,
                                               global_memory_per_byte_cost: u64,
                                               global_memory_per_byte_write_cost: u64,
                                               min_transaction_gas_units: u64,
                                               large_transaction_cutoff: u64,
                                               instrinsic_gas_per_byte: u64,
                                               maximum_number_of_gas_units: u64,
                                               min_price_per_gas_unit: u64,
                                               max_price_per_gas_unit: u64,
                                               max_transaction_size_in_bytes: u64,
                                               gas_unit_scaling_factor: u64,
                                               default_account_size: u64,
                                               exec_delay: u64, ) {
    <b>let</b> vm_config = <a href="VMConfig.md#0x1_VMConfig_new_vm_config">VMConfig::new_vm_config</a>(instruction_schedule,
        native_schedule,
        global_memory_per_byte_cost,
        global_memory_per_byte_write_cost,
        min_transaction_gas_units,
        large_transaction_cutoff,
        instrinsic_gas_per_byte,
        maximum_number_of_gas_units,
        min_price_per_gas_unit,
        max_price_per_gas_unit,
        max_transaction_size_in_bytes,
        gas_unit_scaling_factor,
        default_account_size);
    <a href="OnChainConfigDao.md#0x1_OnChainConfigDao_propose_update">OnChainConfigDao::propose_update</a>&lt;<a href="STC.md#0x1_STC_STC">STC::STC</a>, <a href="VMConfig.md#0x1_VMConfig_VMConfig">VMConfig::VMConfig</a>&gt;(&account, vm_config, exec_delay);
}
</code></pre>



</details>

<a name="0x1_OnChainConfigScripts_propose_update_move_language_version"></a>

## Function `propose_update_move_language_version`



<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="OnChainConfigScripts.md#0x1_OnChainConfigScripts_propose_update_move_language_version">propose_update_move_language_version</a>(account: signer, new_version: u64, exec_delay: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="OnChainConfigScripts.md#0x1_OnChainConfigScripts_propose_update_move_language_version">propose_update_move_language_version</a>(account: signer, new_version: u64, exec_delay: u64) {
    <b>let</b> lang_version = <a href="LanguageVersion.md#0x1_LanguageVersion_new">LanguageVersion::new</a>(new_version);
    <a href="OnChainConfigDao.md#0x1_OnChainConfigDao_propose_update">OnChainConfigDao::propose_update</a>&lt;<a href="STC.md#0x1_STC_STC">STC::STC</a>, <a href="LanguageVersion.md#0x1_LanguageVersion_LanguageVersion">LanguageVersion::LanguageVersion</a>&gt;(&account, lang_version, exec_delay);
}
</code></pre>



</details>

<a name="0x1_OnChainConfigScripts_execute_on_chain_config_proposal"></a>

## Function `execute_on_chain_config_proposal`



<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="OnChainConfigScripts.md#0x1_OnChainConfigScripts_execute_on_chain_config_proposal">execute_on_chain_config_proposal</a>&lt;ConfigT: <b>copy</b>, drop, store&gt;(account: signer, proposal_id: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> ( <b>script</b> ) <b>fun</b> <a href="OnChainConfigScripts.md#0x1_OnChainConfigScripts_execute_on_chain_config_proposal">execute_on_chain_config_proposal</a>&lt;ConfigT: <b>copy</b> + drop + store&gt;(account: signer, proposal_id: u64) {
    <a href="OnChainConfigDao.md#0x1_OnChainConfigDao_execute">OnChainConfigDao::execute</a>&lt;<a href="STC.md#0x1_STC_STC">STC::STC</a>, ConfigT&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(&account), proposal_id);
}
</code></pre>



</details>

<a name="0x1_OnChainConfigScripts_execute_on_chain_config_proposal_v2"></a>

## Function `execute_on_chain_config_proposal_v2`



<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="OnChainConfigScripts.md#0x1_OnChainConfigScripts_execute_on_chain_config_proposal_v2">execute_on_chain_config_proposal_v2</a>&lt;TokenType: <b>copy</b>, drop, store, ConfigT: <b>copy</b>, drop, store&gt;(proposer_address: <b>address</b>, proposal_id: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="OnChainConfigScripts.md#0x1_OnChainConfigScripts_execute_on_chain_config_proposal_v2">execute_on_chain_config_proposal_v2</a>&lt;TokenType: <b>copy</b> + drop + store, ConfigT: <b>copy</b> + drop + store&gt;(proposer_address: <b>address</b>, proposal_id: u64) {
    <a href="OnChainConfigDao.md#0x1_OnChainConfigDao_execute">OnChainConfigDao::execute</a>&lt;TokenType, ConfigT&gt;(proposer_address, proposal_id);
}
</code></pre>



</details>

<a name="@Specification_0"></a>

## Specification


<a name="@Specification_0_propose_update_consensus_config"></a>

### Function `propose_update_consensus_config`


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="OnChainConfigScripts.md#0x1_OnChainConfigScripts_propose_update_consensus_config">propose_update_consensus_config</a>(account: signer, uncle_rate_target: u64, base_block_time_target: u64, base_reward_per_block: u128, base_reward_per_uncle_percent: u64, epoch_block_count: u64, base_block_difficulty_window: u64, min_block_time_target: u64, max_block_time_target: u64, base_max_uncles_per_block: u64, base_block_gas_limit: u64, strategy: u8, exec_delay: u64)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>



<a name="@Specification_0_propose_update_reward_config"></a>

### Function `propose_update_reward_config`


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="OnChainConfigScripts.md#0x1_OnChainConfigScripts_propose_update_reward_config">propose_update_reward_config</a>(account: signer, reward_delay: u64, exec_delay: u64)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>



<a name="@Specification_0_propose_update_txn_publish_option"></a>

### Function `propose_update_txn_publish_option`


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="OnChainConfigScripts.md#0x1_OnChainConfigScripts_propose_update_txn_publish_option">propose_update_txn_publish_option</a>(account: signer, script_allowed: bool, module_publishing_allowed: bool, exec_delay: u64)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>



<a name="@Specification_0_propose_update_txn_timeout_config"></a>

### Function `propose_update_txn_timeout_config`


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="OnChainConfigScripts.md#0x1_OnChainConfigScripts_propose_update_txn_timeout_config">propose_update_txn_timeout_config</a>(account: signer, duration_seconds: u64, exec_delay: u64)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>



<a name="@Specification_0_propose_update_vm_config"></a>

### Function `propose_update_vm_config`


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="OnChainConfigScripts.md#0x1_OnChainConfigScripts_propose_update_vm_config">propose_update_vm_config</a>(account: signer, instruction_schedule: vector&lt;u8&gt;, native_schedule: vector&lt;u8&gt;, global_memory_per_byte_cost: u64, global_memory_per_byte_write_cost: u64, min_transaction_gas_units: u64, large_transaction_cutoff: u64, instrinsic_gas_per_byte: u64, maximum_number_of_gas_units: u64, min_price_per_gas_unit: u64, max_price_per_gas_unit: u64, max_transaction_size_in_bytes: u64, gas_unit_scaling_factor: u64, default_account_size: u64, exec_delay: u64)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>



<a name="@Specification_0_propose_update_move_language_version"></a>

### Function `propose_update_move_language_version`


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="OnChainConfigScripts.md#0x1_OnChainConfigScripts_propose_update_move_language_version">propose_update_move_language_version</a>(account: signer, new_version: u64, exec_delay: u64)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>



<a name="@Specification_0_execute_on_chain_config_proposal"></a>

### Function `execute_on_chain_config_proposal`


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="OnChainConfigScripts.md#0x1_OnChainConfigScripts_execute_on_chain_config_proposal">execute_on_chain_config_proposal</a>&lt;ConfigT: <b>copy</b>, drop, store&gt;(account: signer, proposal_id: u64)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>



<a name="@Specification_0_execute_on_chain_config_proposal_v2"></a>

### Function `execute_on_chain_config_proposal_v2`


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="OnChainConfigScripts.md#0x1_OnChainConfigScripts_execute_on_chain_config_proposal_v2">execute_on_chain_config_proposal_v2</a>&lt;TokenType: <b>copy</b>, drop, store, ConfigT: <b>copy</b>, drop, store&gt;(proposer_address: <b>address</b>, proposal_id: u64)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>
