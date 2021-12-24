
<a name="0x1_Genesis"></a>

# Module `0x1::Genesis`

The module for init Genesis


-  [Function `initialize`](#0x1_Genesis_initialize)
-  [Function `initialize_v2`](#0x1_Genesis_initialize_v2)
-  [Function `do_initialize`](#0x1_Genesis_do_initialize)
-  [Function `initialize_for_unit_tests`](#0x1_Genesis_initialize_for_unit_tests)
-  [Specification](#@Specification_0)


<pre><code><b>use</b> <a href="Account.md#0x1_Account">0x1::Account</a>;
<b>use</b> <a href="Block.md#0x1_Block">0x1::Block</a>;
<b>use</b> <a href="BlockReward.md#0x1_BlockReward">0x1::BlockReward</a>;
<b>use</b> <a href="ChainId.md#0x1_ChainId">0x1::ChainId</a>;
<b>use</b> <a href="Collection.md#0x1_Collection">0x1::Collection</a>;
<b>use</b> <a href="Config.md#0x1_Config">0x1::Config</a>;
<b>use</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig">0x1::ConsensusConfig</a>;
<b>use</b> <a href="ConsensusStrategy.md#0x1_ConsensusStrategy">0x1::ConsensusStrategy</a>;
<b>use</b> <a href="CoreAddresses.md#0x1_CoreAddresses">0x1::CoreAddresses</a>;
<b>use</b> <a href="DummyToken.md#0x1_DummyToken">0x1::DummyToken</a>;
<b>use</b> <a href="Epoch.md#0x1_Epoch">0x1::Epoch</a>;
<b>use</b> <a href="GenesisNFT.md#0x1_GenesisNFT">0x1::GenesisNFT</a>;
<b>use</b> <a href="GenesisSignerCapability.md#0x1_GenesisSignerCapability">0x1::GenesisSignerCapability</a>;
<b>use</b> <a href="Option.md#0x1_Option">0x1::Option</a>;
<b>use</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager">0x1::PackageTxnManager</a>;
<b>use</b> <a href="STC.md#0x1_STC">0x1::STC</a>;
<b>use</b> <a href="Oracle.md#0x1_STCUSDOracle">0x1::STCUSDOracle</a>;
<b>use</b> <a href="Signer.md#0x1_Signer">0x1::Signer</a>;
<b>use</b> <a href="StdlibUpgradeScripts.md#0x1_StdlibUpgradeScripts">0x1::StdlibUpgradeScripts</a>;
<b>use</b> <a href="Timestamp.md#0x1_Timestamp">0x1::Timestamp</a>;
<b>use</b> <a href="Token.md#0x1_Token">0x1::Token</a>;
<b>use</b> <a href="TransactionFee.md#0x1_TransactionFee">0x1::TransactionFee</a>;
<b>use</b> <a href="TransactionPublishOption.md#0x1_TransactionPublishOption">0x1::TransactionPublishOption</a>;
<b>use</b> <a href="TransactionTimeoutConfig.md#0x1_TransactionTimeoutConfig">0x1::TransactionTimeoutConfig</a>;
<b>use</b> <a href="Treasury.md#0x1_Treasury">0x1::Treasury</a>;
<b>use</b> <a href="TreasuryWithdrawDaoProposal.md#0x1_TreasuryWithdrawDaoProposal">0x1::TreasuryWithdrawDaoProposal</a>;
<b>use</b> <a href="VMConfig.md#0x1_VMConfig">0x1::VMConfig</a>;
<b>use</b> <a href="Vector.md#0x1_Vector">0x1::Vector</a>;
<b>use</b> <a href="Version.md#0x1_Version">0x1::Version</a>;
</code></pre>



<a name="0x1_Genesis_initialize"></a>

## Function `initialize`



<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="Genesis.md#0x1_Genesis_initialize">initialize</a>(stdlib_version: u64, reward_delay: u64, pre_mine_stc_amount: u128, time_mint_stc_amount: u128, time_mint_stc_period: u64, parent_hash: vector&lt;u8&gt;, association_auth_key: vector&lt;u8&gt;, genesis_auth_key: vector&lt;u8&gt;, chain_id: u8, genesis_timestamp: u64, uncle_rate_target: u64, epoch_block_count: u64, base_block_time_target: u64, base_block_difficulty_window: u64, base_reward_per_block: u128, base_reward_per_uncle_percent: u64, min_block_time_target: u64, max_block_time_target: u64, base_max_uncles_per_block: u64, base_block_gas_limit: u64, strategy: u8, script_allowed: bool, module_publishing_allowed: bool, instruction_schedule: vector&lt;u8&gt;, native_schedule: vector&lt;u8&gt;, global_memory_per_byte_cost: u64, global_memory_per_byte_write_cost: u64, min_transaction_gas_units: u64, large_transaction_cutoff: u64, instrinsic_gas_per_byte: u64, maximum_number_of_gas_units: u64, min_price_per_gas_unit: u64, max_price_per_gas_unit: u64, max_transaction_size_in_bytes: u64, gas_unit_scaling_factor: u64, default_account_size: u64, voting_delay: u64, voting_period: u64, voting_quorum_rate: u8, min_action_delay: u64, transaction_timeout: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="Genesis.md#0x1_Genesis_initialize">initialize</a>(
    stdlib_version: u64,

    // block reward config
    reward_delay: u64,

    pre_mine_stc_amount: u128,
    time_mint_stc_amount: u128,
    time_mint_stc_period: u64,
    parent_hash: vector&lt;u8&gt;,
    association_auth_key: vector&lt;u8&gt;,
    genesis_auth_key: vector&lt;u8&gt;,
    chain_id: u8,
    genesis_timestamp: u64,

    //consensus config
    uncle_rate_target: u64,
    epoch_block_count: u64,
    base_block_time_target: u64,
    base_block_difficulty_window: u64,
    base_reward_per_block: u128,
    base_reward_per_uncle_percent: u64,
    min_block_time_target: u64,
    max_block_time_target: u64,
    base_max_uncles_per_block: u64,
    base_block_gas_limit: u64,
    strategy: u8,

    //vm config
    script_allowed: bool,
    module_publishing_allowed: bool,
    instruction_schedule: vector&lt;u8&gt;,
    native_schedule: vector&lt;u8&gt;,

    //gas constants
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

    // dao config
    voting_delay: u64,
    voting_period: u64,
    voting_quorum_rate: u8,
    min_action_delay: u64,

    // transaction timeout config
    transaction_timeout: u64,
) {
    <b>assert</b>!(<a href="Timestamp.md#0x1_Timestamp_is_genesis">Timestamp::is_genesis</a>(), 1);
    // create genesis account
    <b>let</b> genesis_account = <a href="Account.md#0x1_Account_create_genesis_account">Account::create_genesis_account</a>(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
    //Init <b>global</b> time
    <a href="Timestamp.md#0x1_Timestamp_initialize">Timestamp::initialize</a>(&genesis_account, genesis_timestamp);
    <a href="ChainId.md#0x1_ChainId_initialize">ChainId::initialize</a>(&genesis_account, chain_id);
    <a href="ConsensusStrategy.md#0x1_ConsensusStrategy_initialize">ConsensusStrategy::initialize</a>(&genesis_account, strategy);
    <a href="Block.md#0x1_Block_initialize">Block::initialize</a>(&genesis_account, parent_hash);
    <a href="TransactionPublishOption.md#0x1_TransactionPublishOption_initialize">TransactionPublishOption::initialize</a>(
        &genesis_account,
        script_allowed,
        module_publishing_allowed,
    );
    // init config
    <a href="VMConfig.md#0x1_VMConfig_initialize">VMConfig::initialize</a>(
        &genesis_account,
        instruction_schedule,
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
        default_account_size,
    );
    <a href="TransactionTimeoutConfig.md#0x1_TransactionTimeoutConfig_initialize">TransactionTimeoutConfig::initialize</a>(&genesis_account, transaction_timeout);
    <a href="ConsensusConfig.md#0x1_ConsensusConfig_initialize">ConsensusConfig::initialize</a>(
        &genesis_account,
        uncle_rate_target,
        epoch_block_count,
        base_block_time_target,
        base_block_difficulty_window,
        base_reward_per_block,
        base_reward_per_uncle_percent,
        min_block_time_target,
        max_block_time_target,
        base_max_uncles_per_block,
        base_block_gas_limit,
        strategy,
    );
    <a href="Epoch.md#0x1_Epoch_initialize">Epoch::initialize</a>(&genesis_account);
    <a href="BlockReward.md#0x1_BlockReward_initialize">BlockReward::initialize</a>(&genesis_account, reward_delay);
    <a href="TransactionFee.md#0x1_TransactionFee_initialize">TransactionFee::initialize</a>(&genesis_account);
    <b>let</b> association = <a href="Account.md#0x1_Account_create_genesis_account">Account::create_genesis_account</a>(
        <a href="CoreAddresses.md#0x1_CoreAddresses_ASSOCIATION_ROOT_ADDRESS">CoreAddresses::ASSOCIATION_ROOT_ADDRESS</a>(),
    );
    <a href="Config.md#0x1_Config_publish_new_config">Config::publish_new_config</a>&lt;<a href="Version.md#0x1_Version_Version">Version::Version</a>&gt;(&genesis_account, <a href="Version.md#0x1_Version_new_version">Version::new_version</a>(stdlib_version));
    // stdlib <b>use</b> two phase upgrade strategy.
    <a href="PackageTxnManager.md#0x1_PackageTxnManager_update_module_upgrade_strategy">PackageTxnManager::update_module_upgrade_strategy</a>(
        &genesis_account,
        <a href="PackageTxnManager.md#0x1_PackageTxnManager_get_strategy_two_phase">PackageTxnManager::get_strategy_two_phase</a>(),
        <a href="Option.md#0x1_Option_some">Option::some</a>(0u64),
    );
    // stc should be initialized after genesis_account's <b>module</b> upgrade strategy set.
    {
        <a href="STC.md#0x1_STC_initialize">STC::initialize</a>(&genesis_account, voting_delay, voting_period, voting_quorum_rate, min_action_delay);
        <a href="Account.md#0x1_Account_do_accept_token">Account::do_accept_token</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;(&genesis_account);
        <a href="DummyToken.md#0x1_DummyToken_initialize">DummyToken::initialize</a>(&genesis_account);
        <a href="Account.md#0x1_Account_do_accept_token">Account::do_accept_token</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;(&association);
    };
    <b>if</b> (pre_mine_stc_amount &gt; 0) {
        <b>let</b> stc = <a href="Token.md#0x1_Token_mint">Token::mint</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;(&genesis_account, pre_mine_stc_amount);
        <a href="Account.md#0x1_Account_deposit">Account::deposit</a>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(&association), stc);
    };
    <b>if</b> (time_mint_stc_amount &gt; 0) {
        <b>let</b> cap = <a href="Token.md#0x1_Token_remove_mint_capability">Token::remove_mint_capability</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;(&genesis_account);
        <b>let</b> key = <a href="Token.md#0x1_Token_issue_linear_mint_key">Token::issue_linear_mint_key</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;(&cap, time_mint_stc_amount, time_mint_stc_period);
        <a href="Token.md#0x1_Token_add_mint_capability">Token::add_mint_capability</a>(&genesis_account, cap);
        <a href="Collection.md#0x1_Collection_put">Collection::put</a>(&association, key);
    };
    // only dev network set genesis auth key.
    <b>if</b> (!<a href="Vector.md#0x1_Vector_is_empty">Vector::is_empty</a>(&genesis_auth_key)) {
        <b>let</b> genesis_rotate_key_cap = <a href="Account.md#0x1_Account_extract_key_rotation_capability">Account::extract_key_rotation_capability</a>(&genesis_account);
        <a href="Account.md#0x1_Account_rotate_authentication_key_with_capability">Account::rotate_authentication_key_with_capability</a>(&genesis_rotate_key_cap, genesis_auth_key);
        <a href="Account.md#0x1_Account_restore_key_rotation_capability">Account::restore_key_rotation_capability</a>(genesis_rotate_key_cap);
    };
    <b>let</b> assoc_rotate_key_cap = <a href="Account.md#0x1_Account_extract_key_rotation_capability">Account::extract_key_rotation_capability</a>(&association);
    <a href="Account.md#0x1_Account_rotate_authentication_key_with_capability">Account::rotate_authentication_key_with_capability</a>(&assoc_rotate_key_cap, association_auth_key);
    <a href="Account.md#0x1_Account_restore_key_rotation_capability">Account::restore_key_rotation_capability</a>(assoc_rotate_key_cap);
    //Start time, <a href="Timestamp.md#0x1_Timestamp_is_genesis">Timestamp::is_genesis</a>() will <b>return</b> <b>false</b>. this call should at the end of genesis init.
    <a href="Timestamp.md#0x1_Timestamp_set_time_has_started">Timestamp::set_time_has_started</a>(&genesis_account);
    <a href="Account.md#0x1_Account_release_genesis_signer">Account::release_genesis_signer</a>(genesis_account);
    <a href="Account.md#0x1_Account_release_genesis_signer">Account::release_genesis_signer</a>(association);
}
</code></pre>



</details>

<a name="0x1_Genesis_initialize_v2"></a>

## Function `initialize_v2`



<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="Genesis.md#0x1_Genesis_initialize_v2">initialize_v2</a>(stdlib_version: u64, reward_delay: u64, total_stc_amount: u128, pre_mine_stc_amount: u128, time_mint_stc_amount: u128, time_mint_stc_period: u64, parent_hash: vector&lt;u8&gt;, association_auth_key: vector&lt;u8&gt;, genesis_auth_key: vector&lt;u8&gt;, chain_id: u8, genesis_timestamp: u64, uncle_rate_target: u64, epoch_block_count: u64, base_block_time_target: u64, base_block_difficulty_window: u64, base_reward_per_block: u128, base_reward_per_uncle_percent: u64, min_block_time_target: u64, max_block_time_target: u64, base_max_uncles_per_block: u64, base_block_gas_limit: u64, strategy: u8, script_allowed: bool, module_publishing_allowed: bool, instruction_schedule: vector&lt;u8&gt;, native_schedule: vector&lt;u8&gt;, global_memory_per_byte_cost: u64, global_memory_per_byte_write_cost: u64, min_transaction_gas_units: u64, large_transaction_cutoff: u64, instrinsic_gas_per_byte: u64, maximum_number_of_gas_units: u64, min_price_per_gas_unit: u64, max_price_per_gas_unit: u64, max_transaction_size_in_bytes: u64, gas_unit_scaling_factor: u64, default_account_size: u64, voting_delay: u64, voting_period: u64, voting_quorum_rate: u8, min_action_delay: u64, transaction_timeout: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="Genesis.md#0x1_Genesis_initialize_v2">initialize_v2</a>(
    stdlib_version: u64,

    // block reward and stc config
    reward_delay: u64,
    total_stc_amount: u128,
    pre_mine_stc_amount: u128,
    time_mint_stc_amount: u128,
    time_mint_stc_period: u64,

    parent_hash: vector&lt;u8&gt;,
    association_auth_key: vector&lt;u8&gt;,
    genesis_auth_key: vector&lt;u8&gt;,
    chain_id: u8,
    genesis_timestamp: u64,

    //consensus config
    uncle_rate_target: u64,
    epoch_block_count: u64,
    base_block_time_target: u64,
    base_block_difficulty_window: u64,
    base_reward_per_block: u128,
    base_reward_per_uncle_percent: u64,
    min_block_time_target: u64,
    max_block_time_target: u64,
    base_max_uncles_per_block: u64,
    base_block_gas_limit: u64,
    strategy: u8,

    //vm config
    script_allowed: bool,
    module_publishing_allowed: bool,
    instruction_schedule: vector&lt;u8&gt;,
    native_schedule: vector&lt;u8&gt;,

    //gas constants
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

    // dao config
    voting_delay: u64,
    voting_period: u64,
    voting_quorum_rate: u8,
    min_action_delay: u64,

    // transaction timeout config
    transaction_timeout: u64,
) {
    <a href="Genesis.md#0x1_Genesis_do_initialize">Self::do_initialize</a>(
    stdlib_version,
    reward_delay,
    total_stc_amount,
    pre_mine_stc_amount,
    time_mint_stc_amount,
    time_mint_stc_period,
    parent_hash,
    association_auth_key,
    genesis_auth_key,
    chain_id,
    genesis_timestamp,
    uncle_rate_target,
    epoch_block_count,
    base_block_time_target,
    base_block_difficulty_window,
    base_reward_per_block,
    base_reward_per_uncle_percent,
    min_block_time_target,
    max_block_time_target,
    base_max_uncles_per_block,
    base_block_gas_limit,
    strategy,
    script_allowed,
    module_publishing_allowed,
    instruction_schedule,
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
    default_account_size,
    voting_delay,
    voting_period,
    voting_quorum_rate,
    min_action_delay,
    transaction_timeout,
    );
}
</code></pre>



</details>

<a name="0x1_Genesis_do_initialize"></a>

## Function `do_initialize`



<pre><code><b>fun</b> <a href="Genesis.md#0x1_Genesis_do_initialize">do_initialize</a>(stdlib_version: u64, reward_delay: u64, total_stc_amount: u128, pre_mine_stc_amount: u128, time_mint_stc_amount: u128, time_mint_stc_period: u64, parent_hash: vector&lt;u8&gt;, association_auth_key: vector&lt;u8&gt;, genesis_auth_key: vector&lt;u8&gt;, chain_id: u8, genesis_timestamp: u64, uncle_rate_target: u64, epoch_block_count: u64, base_block_time_target: u64, base_block_difficulty_window: u64, base_reward_per_block: u128, base_reward_per_uncle_percent: u64, min_block_time_target: u64, max_block_time_target: u64, base_max_uncles_per_block: u64, base_block_gas_limit: u64, strategy: u8, script_allowed: bool, module_publishing_allowed: bool, instruction_schedule: vector&lt;u8&gt;, native_schedule: vector&lt;u8&gt;, global_memory_per_byte_cost: u64, global_memory_per_byte_write_cost: u64, min_transaction_gas_units: u64, large_transaction_cutoff: u64, instrinsic_gas_per_byte: u64, maximum_number_of_gas_units: u64, min_price_per_gas_unit: u64, max_price_per_gas_unit: u64, max_transaction_size_in_bytes: u64, gas_unit_scaling_factor: u64, default_account_size: u64, voting_delay: u64, voting_period: u64, voting_quorum_rate: u8, min_action_delay: u64, transaction_timeout: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="Genesis.md#0x1_Genesis_do_initialize">do_initialize</a>(
    stdlib_version: u64,

    // block reward and stc config
    reward_delay: u64,
    total_stc_amount: u128,
    pre_mine_stc_amount: u128,
    time_mint_stc_amount: u128,
    time_mint_stc_period: u64,

    parent_hash: vector&lt;u8&gt;,
    association_auth_key: vector&lt;u8&gt;,
    genesis_auth_key: vector&lt;u8&gt;,
    chain_id: u8,
    genesis_timestamp: u64,

    //consensus config
    uncle_rate_target: u64,
    epoch_block_count: u64,
    base_block_time_target: u64,
    base_block_difficulty_window: u64,
    base_reward_per_block: u128,
    base_reward_per_uncle_percent: u64,
    min_block_time_target: u64,
    max_block_time_target: u64,
    base_max_uncles_per_block: u64,
    base_block_gas_limit: u64,
    strategy: u8,

    //vm config
    script_allowed: bool,
    module_publishing_allowed: bool,
    instruction_schedule: vector&lt;u8&gt;,
    native_schedule: vector&lt;u8&gt;,

    //gas constants
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

    // dao config
    voting_delay: u64,
    voting_period: u64,
    voting_quorum_rate: u8,
    min_action_delay: u64,

    // transaction timeout config
    transaction_timeout: u64,
){
    <a href="Timestamp.md#0x1_Timestamp_assert_genesis">Timestamp::assert_genesis</a>();
    // create genesis account
    <b>let</b> genesis_account = <a href="Account.md#0x1_Account_create_genesis_account">Account::create_genesis_account</a>(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
    //Init <b>global</b> time
    <a href="Timestamp.md#0x1_Timestamp_initialize">Timestamp::initialize</a>(&genesis_account, genesis_timestamp);
    <a href="ChainId.md#0x1_ChainId_initialize">ChainId::initialize</a>(&genesis_account, chain_id);
    <a href="ConsensusStrategy.md#0x1_ConsensusStrategy_initialize">ConsensusStrategy::initialize</a>(&genesis_account, strategy);
    <a href="Block.md#0x1_Block_initialize">Block::initialize</a>(&genesis_account, parent_hash);
    <a href="TransactionPublishOption.md#0x1_TransactionPublishOption_initialize">TransactionPublishOption::initialize</a>(
        &genesis_account,
        script_allowed,
        module_publishing_allowed,
    );
    // init config
    <a href="VMConfig.md#0x1_VMConfig_initialize">VMConfig::initialize</a>(
        &genesis_account,
        instruction_schedule,
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
        default_account_size,
    );
    <a href="TransactionTimeoutConfig.md#0x1_TransactionTimeoutConfig_initialize">TransactionTimeoutConfig::initialize</a>(&genesis_account, transaction_timeout);
    <a href="ConsensusConfig.md#0x1_ConsensusConfig_initialize">ConsensusConfig::initialize</a>(
        &genesis_account,
        uncle_rate_target,
        epoch_block_count,
        base_block_time_target,
        base_block_difficulty_window,
        base_reward_per_block,
        base_reward_per_uncle_percent,
        min_block_time_target,
        max_block_time_target,
        base_max_uncles_per_block,
        base_block_gas_limit,
        strategy,
    );
    <a href="Epoch.md#0x1_Epoch_initialize">Epoch::initialize</a>(&genesis_account);
    <b>let</b> association = <a href="Account.md#0x1_Account_create_genesis_account">Account::create_genesis_account</a>(
        <a href="CoreAddresses.md#0x1_CoreAddresses_ASSOCIATION_ROOT_ADDRESS">CoreAddresses::ASSOCIATION_ROOT_ADDRESS</a>(),
    );
    <a href="Config.md#0x1_Config_publish_new_config">Config::publish_new_config</a>&lt;<a href="Version.md#0x1_Version_Version">Version::Version</a>&gt;(&genesis_account, <a href="Version.md#0x1_Version_new_version">Version::new_version</a>(stdlib_version));
    // stdlib <b>use</b> two phase upgrade strategy.
    <a href="PackageTxnManager.md#0x1_PackageTxnManager_update_module_upgrade_strategy">PackageTxnManager::update_module_upgrade_strategy</a>(
        &genesis_account,
        <a href="PackageTxnManager.md#0x1_PackageTxnManager_get_strategy_two_phase">PackageTxnManager::get_strategy_two_phase</a>(),
        <a href="Option.md#0x1_Option_some">Option::some</a>(0u64),
    );
    <a href="BlockReward.md#0x1_BlockReward_initialize">BlockReward::initialize</a>(&genesis_account, reward_delay);

    // stc should be initialized after genesis_account's <b>module</b> upgrade strategy set and all on chain config init.
    <b>let</b> withdraw_cap = <a href="STC.md#0x1_STC_initialize_v2">STC::initialize_v2</a>(&genesis_account, total_stc_amount, voting_delay, voting_period, voting_quorum_rate, min_action_delay);
    <a href="Account.md#0x1_Account_do_accept_token">Account::do_accept_token</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;(&genesis_account);
    <a href="Account.md#0x1_Account_do_accept_token">Account::do_accept_token</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;(&association);

    <a href="DummyToken.md#0x1_DummyToken_initialize">DummyToken::initialize</a>(&genesis_account);

    <b>if</b> (pre_mine_stc_amount &gt; 0) {
        <b>let</b> stc = <a href="Treasury.md#0x1_Treasury_withdraw_with_capability">Treasury::withdraw_with_capability</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;(&<b>mut</b> withdraw_cap, pre_mine_stc_amount);
        <a href="Account.md#0x1_Account_deposit">Account::deposit</a>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(&association), stc);
    };
    <b>if</b> (time_mint_stc_amount &gt; 0) {
        <b>let</b> liner_withdraw_cap = <a href="Treasury.md#0x1_Treasury_issue_linear_withdraw_capability">Treasury::issue_linear_withdraw_capability</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;(&<b>mut</b> withdraw_cap, time_mint_stc_amount, time_mint_stc_period);
        <a href="Treasury.md#0x1_Treasury_add_linear_withdraw_capability">Treasury::add_linear_withdraw_capability</a>(&association, liner_withdraw_cap);
    };

    // Lock the TreasuryWithdrawCapability <b>to</b> <a href="Dao.md#0x1_Dao">Dao</a>
    <a href="TreasuryWithdrawDaoProposal.md#0x1_TreasuryWithdrawDaoProposal_plugin">TreasuryWithdrawDaoProposal::plugin</a>(&genesis_account, withdraw_cap);

    <a href="TransactionFee.md#0x1_TransactionFee_initialize">TransactionFee::initialize</a>(&genesis_account);

    // only test/dev network set genesis auth key.
    <b>if</b> (!<a href="Vector.md#0x1_Vector_is_empty">Vector::is_empty</a>(&genesis_auth_key)) {
        <b>let</b> genesis_rotate_key_cap = <a href="Account.md#0x1_Account_extract_key_rotation_capability">Account::extract_key_rotation_capability</a>(&genesis_account);
        <a href="Account.md#0x1_Account_rotate_authentication_key_with_capability">Account::rotate_authentication_key_with_capability</a>(&genesis_rotate_key_cap, genesis_auth_key);
        <a href="Account.md#0x1_Account_restore_key_rotation_capability">Account::restore_key_rotation_capability</a>(genesis_rotate_key_cap);
    };

    <b>let</b> assoc_rotate_key_cap = <a href="Account.md#0x1_Account_extract_key_rotation_capability">Account::extract_key_rotation_capability</a>(&association);
    <a href="Account.md#0x1_Account_rotate_authentication_key_with_capability">Account::rotate_authentication_key_with_capability</a>(&assoc_rotate_key_cap, association_auth_key);
    <a href="Account.md#0x1_Account_restore_key_rotation_capability">Account::restore_key_rotation_capability</a>(assoc_rotate_key_cap);

    // v5 -&gt; v6
    {
        <b>let</b> cap = <a href="Account.md#0x1_Account_remove_signer_capability">Account::remove_signer_capability</a>(&genesis_account);
        <a href="GenesisSignerCapability.md#0x1_GenesisSignerCapability_initialize">GenesisSignerCapability::initialize</a>(&genesis_account, cap);
        //register oracle
        <a href="Oracle.md#0x1_STCUSDOracle_register">STCUSDOracle::register</a>(&genesis_account);
        <b>let</b> merkle_root = x"5969f0e8e19f8769276fb638e6060d5c02e40088f5fde70a6778dd69d659ee6d";
        <b>let</b> image = b"ipfs://QmSPcvcXgdtHHiVTAAarzTeubk5X3iWymPAoKBfiRFjPMY";
        <a href="GenesisNFT.md#0x1_GenesisNFT_initialize">GenesisNFT::initialize</a>(&genesis_account, merkle_root, 1639u64, image);
    };
    <a href="StdlibUpgradeScripts.md#0x1_StdlibUpgradeScripts_do_upgrade_from_v6_to_v7_with_language_version">StdlibUpgradeScripts::do_upgrade_from_v6_to_v7_with_language_version</a>(&genesis_account, 4);
    //Start time, <a href="Timestamp.md#0x1_Timestamp_is_genesis">Timestamp::is_genesis</a>() will <b>return</b> <b>false</b>. this call should at the end of genesis init.
    <a href="Timestamp.md#0x1_Timestamp_set_time_has_started">Timestamp::set_time_has_started</a>(&genesis_account);
    <a href="Account.md#0x1_Account_release_genesis_signer">Account::release_genesis_signer</a>(genesis_account);
    <a href="Account.md#0x1_Account_release_genesis_signer">Account::release_genesis_signer</a>(association);
}
</code></pre>



</details>

<a name="0x1_Genesis_initialize_for_unit_tests"></a>

## Function `initialize_for_unit_tests`

Init the genesis for unit tests


<pre><code><b>public</b> <b>fun</b> <a href="Genesis.md#0x1_Genesis_initialize_for_unit_tests">initialize_for_unit_tests</a>()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Genesis.md#0x1_Genesis_initialize_for_unit_tests">initialize_for_unit_tests</a>(){
    <b>let</b> stdlib_version: u64 = 6;
    <b>let</b> reward_delay: u64 = 7;
    <b>let</b> total_stc_amount: u128 = 3185136000000000000u128;
    <b>let</b> pre_mine_stc_amount: u128 = 159256800000000000u128;
    <b>let</b> time_mint_stc_amount: u128 = (85043130u128 * 3u128 + 74213670u128 * 3u128)*1000000000u128;
    <b>let</b> time_mint_stc_period: u64 = 1000000000;

    <b>let</b> parent_hash: vector&lt;u8&gt; = x"0000000000000000000000000000000000000000000000000000000000000000";
    <b>let</b> association_auth_key: vector&lt;u8&gt; = x"0000000000000000000000000000000000000000000000000000000000000000";
    <b>let</b> genesis_auth_key: vector&lt;u8&gt; = x"0000000000000000000000000000000000000000000000000000000000000000";
    <b>let</b> chain_id: u8 = 255;
    <b>let</b> genesis_timestamp: u64 =0;

    //consensus config
    <b>let</b> uncle_rate_target: u64 = 80;
    <b>let</b> epoch_block_count: u64 = 240;
    <b>let</b> base_block_time_target: u64 = 10000;
    <b>let</b> base_block_difficulty_window: u64 = 24;
    <b>let</b> base_reward_per_block: u128 = 1000000000;
    <b>let</b> base_reward_per_uncle_percent: u64 = 10;
    <b>let</b> min_block_time_target: u64 = 1000;
    <b>let</b> max_block_time_target: u64 = 20000;
    <b>let</b> base_max_uncles_per_block: u64 = 2;
    <b>let</b> base_block_gas_limit: u64 = 500000000;
    <b>let</b> strategy: u8 = 0;

    //vm config
    <b>let</b> script_allowed: bool = <b>true</b>;
    <b>let</b> module_publishing_allowed: bool = <b>true</b>;
    //TODO init the gas table.
    <b>let</b> instruction_schedule: vector&lt;u8&gt; = <a href="Vector.md#0x1_Vector_empty">Vector::empty</a>();
    <b>let</b> native_schedule: vector&lt;u8&gt; = <a href="Vector.md#0x1_Vector_empty">Vector::empty</a>();

    //gas constants
    <b>let</b> global_memory_per_byte_cost: u64 = 1;
    <b>let</b> global_memory_per_byte_write_cost: u64 = 1;
    <b>let</b> min_transaction_gas_units: u64 = 1;
    <b>let</b> large_transaction_cutoff: u64 = 1;
    <b>let</b> instrinsic_gas_per_byte: u64 = 1;
    <b>let</b> maximum_number_of_gas_units: u64 = 1;
    <b>let</b> min_price_per_gas_unit: u64 = 1;
    <b>let</b> max_price_per_gas_unit: u64 = 10000;
    <b>let</b> max_transaction_size_in_bytes: u64 = 1024*1024;
    <b>let</b> gas_unit_scaling_factor: u64 = 1;
    <b>let</b> default_account_size: u64 = 600;

    // dao config
    <b>let</b> voting_delay: u64 = 1000;
    <b>let</b> voting_period: u64 =  6000;
    <b>let</b> voting_quorum_rate: u8 = 4;
    <b>let</b> min_action_delay: u64 = 1000;

    // transaction timeout config
    <b>let</b> transaction_timeout: u64 = 10000;

    <a href="Genesis.md#0x1_Genesis_do_initialize">Self::do_initialize</a>(
        stdlib_version,
        reward_delay,
        total_stc_amount,
        pre_mine_stc_amount,
        time_mint_stc_amount,
        time_mint_stc_period,
        parent_hash,
        association_auth_key,
        genesis_auth_key,
        chain_id,
        genesis_timestamp,
        uncle_rate_target,
        epoch_block_count,
        base_block_time_target,
        base_block_difficulty_window,
        base_reward_per_block,
        base_reward_per_uncle_percent,
        min_block_time_target,
        max_block_time_target,
        base_max_uncles_per_block,
        base_block_gas_limit,
        strategy,
        script_allowed,
        module_publishing_allowed,
        instruction_schedule,
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
        default_account_size,
        voting_delay,
        voting_period,
        voting_quorum_rate,
        min_action_delay,
        transaction_timeout,
    );
}
</code></pre>



</details>

<a name="@Specification_0"></a>

## Specification



<pre><code><b>pragma</b> verify = <b>false</b>;
<b>pragma</b> aborts_if_is_partial = <b>false</b>;
<b>pragma</b> aborts_if_is_strict = <b>true</b>;
</code></pre>
