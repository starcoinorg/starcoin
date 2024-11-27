
<a id="0x1_stc_genesis"></a>

# Module `0x1::stc_genesis`

The module for init Genesis


-  [Function `initialize`](#0x1_stc_genesis_initialize)
-  [Function `initialize_versions`](#0x1_stc_genesis_initialize_versions)
-  [Function `initialize_stc`](#0x1_stc_genesis_initialize_stc)
-  [Function `initialize_stc_governance_allocation`](#0x1_stc_genesis_initialize_stc_governance_allocation)
-  [Function `initialize_for_unit_tests`](#0x1_stc_genesis_initialize_for_unit_tests)
-  [Specification](#@Specification_0)


<pre><code><b>use</b> <a href="account.md#0x1_account">0x1::account</a>;
<b>use</b> <a href="aggregator_factory.md#0x1_aggregator_factory">0x1::aggregator_factory</a>;
<b>use</b> <a href="block_reward.md#0x1_block_reward">0x1::block_reward</a>;
<b>use</b> <a href="block_reward_config.md#0x1_block_reward_config">0x1::block_reward_config</a>;
<b>use</b> <a href="chain_id.md#0x1_chain_id">0x1::chain_id</a>;
<b>use</b> <a href="coin.md#0x1_coin">0x1::coin</a>;
<b>use</b> <a href="consensus_config.md#0x1_consensus_config">0x1::consensus_config</a>;
<b>use</b> <a href="consensus_strategy.md#0x1_consensus_strategy">0x1::consensus_strategy</a>;
<b>use</b> <a href="dao.md#0x1_dao">0x1::dao</a>;
<b>use</b> <a href="dao_modify_config_proposal.md#0x1_dao_modify_config_proposal">0x1::dao_modify_config_proposal</a>;
<b>use</b> <a href="dao_treasury_withdraw_proposal.md#0x1_dao_treasury_withdraw_proposal">0x1::dao_treasury_withdraw_proposal</a>;
<b>use</b> <a href="dao_upgrade_module_proposal.md#0x1_dao_upgrade_module_proposal">0x1::dao_upgrade_module_proposal</a>;
<b>use</b> <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug">0x1::debug</a>;
<b>use</b> <a href="epoch.md#0x1_epoch">0x1::epoch</a>;
<b>use</b> <a href="flexi_dag_config.md#0x1_flexi_dag_config">0x1::flexi_dag_config</a>;
<b>use</b> <a href="on_chain_config.md#0x1_on_chain_config">0x1::on_chain_config</a>;
<b>use</b> <a href="on_chain_config_dao.md#0x1_on_chain_config_dao">0x1::on_chain_config_dao</a>;
<b>use</b> <a href="../../move-stdlib/doc/option.md#0x1_option">0x1::option</a>;
<b>use</b> <a href="oracle_stc_usd.md#0x1_oracle_stc_usd">0x1::oracle_stc_usd</a>;
<b>use</b> <a href="starcoin_coin.md#0x1_starcoin_coin">0x1::starcoin_coin</a>;
<b>use</b> <a href="stc_block.md#0x1_stc_block">0x1::stc_block</a>;
<b>use</b> <a href="stc_language_version.md#0x1_stc_language_version">0x1::stc_language_version</a>;
<b>use</b> <a href="stc_transaction_fee.md#0x1_stc_transaction_fee">0x1::stc_transaction_fee</a>;
<b>use</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation">0x1::stc_transaction_package_validation</a>;
<b>use</b> <a href="stc_transaction_timeout_config.md#0x1_stc_transaction_timeout_config">0x1::stc_transaction_timeout_config</a>;
<b>use</b> <a href="stc_util.md#0x1_stc_util">0x1::stc_util</a>;
<b>use</b> <a href="stc_version.md#0x1_stc_version">0x1::stc_version</a>;
<b>use</b> <a href="../../move-stdlib/doc/string.md#0x1_string">0x1::string</a>;
<b>use</b> <a href="system_addresses.md#0x1_system_addresses">0x1::system_addresses</a>;
<b>use</b> <a href="timestamp.md#0x1_timestamp">0x1::timestamp</a>;
<b>use</b> <a href="stc_transaction_publish_option.md#0x1_transaction_publish_option">0x1::transaction_publish_option</a>;
<b>use</b> <a href="treasury.md#0x1_treasury">0x1::treasury</a>;
<b>use</b> <a href="../../move-stdlib/doc/vector.md#0x1_vector">0x1::vector</a>;
<b>use</b> <a href="vm_config.md#0x1_vm_config">0x1::vm_config</a>;
</code></pre>



<a id="0x1_stc_genesis_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> entry <b>fun</b> <a href="stc_genesis.md#0x1_stc_genesis_initialize">initialize</a>(stdlib_version: u64, reward_delay: u64, total_stc_amount: u128, pre_mine_stc_amount: u128, time_mint_stc_amount: u128, time_mint_stc_period: u64, parent_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, association_auth_key: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, genesis_auth_key: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, <a href="chain_id.md#0x1_chain_id">chain_id</a>: u8, _genesis_timestamp: u64, uncle_rate_target: u64, epoch_block_count: u64, base_block_time_target: u64, base_block_difficulty_window: u64, base_reward_per_block: u128, base_reward_per_uncle_percent: u64, min_block_time_target: u64, max_block_time_target: u64, base_max_uncles_per_block: u64, base_block_gas_limit: u64, strategy: u8, script_allowed: bool, module_publishing_allowed: bool, gas_schedule_blob: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, voting_delay: u64, voting_period: u64, voting_quorum_rate: u8, min_action_delay: u64, transaction_timeout: u64, dag_effective_height: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="stc_genesis.md#0x1_stc_genesis_initialize">initialize</a>(
    stdlib_version: u64,
    // <a href="block.md#0x1_block">block</a> reward and stc config
    reward_delay: u64,
    total_stc_amount: u128,
    pre_mine_stc_amount: u128,
    time_mint_stc_amount: u128,
    time_mint_stc_period: u64,
    parent_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    association_auth_key: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    genesis_auth_key: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    <a href="chain_id.md#0x1_chain_id">chain_id</a>: u8,
    _genesis_timestamp: u64,
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
    gas_schedule_blob: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    // <a href="dao.md#0x1_dao">dao</a> config
    voting_delay: u64,
    voting_period: u64,
    voting_quorum_rate: u8,
    min_action_delay: u64,
    // transaction timeout config
    transaction_timeout: u64,
    dag_effective_height: u64,
) {
    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"<a href="stc_genesis.md#0x1_stc_genesis_initialize">stc_genesis::initialize</a> Entered"));


    // create <a href="genesis.md#0x1_genesis">genesis</a> <a href="account.md#0x1_account">account</a>
    <b>let</b> (starcoin_framework_account, _genesis_signer_cap) =
        <a href="account.md#0x1_account_create_framework_reserved_account">account::create_framework_reserved_account</a>(@starcoin_framework);

    <a href="stc_genesis.md#0x1_stc_genesis_initialize_versions">initialize_versions</a>(&starcoin_framework_account, stdlib_version);

    <a href="aggregator_factory.md#0x1_aggregator_factory_initialize_aggregator_factory">aggregator_factory::initialize_aggregator_factory</a>(&starcoin_framework_account);

    // Init <b>global</b> time
    <a href="timestamp.md#0x1_timestamp_set_time_has_started">timestamp::set_time_has_started</a>(&starcoin_framework_account);

    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"<a href="stc_genesis.md#0x1_stc_genesis_initialize">stc_genesis::initialize</a> | <a href="chain_id.md#0x1_chain_id">chain_id</a>: "));
    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&<a href="chain_id.md#0x1_chain_id">chain_id</a>);
    <a href="chain_id.md#0x1_chain_id_initialize">chain_id::initialize</a>(&starcoin_framework_account, <a href="chain_id.md#0x1_chain_id">chain_id</a>);

    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"<a href="stc_genesis.md#0x1_stc_genesis_initialize">stc_genesis::initialize</a> | <a href="consensus_strategy.md#0x1_consensus_strategy_initialize">consensus_strategy::initialize</a> "));
    <a href="consensus_strategy.md#0x1_consensus_strategy_initialize">consensus_strategy::initialize</a>(&starcoin_framework_account, strategy);
    <a href="stc_block.md#0x1_stc_block_initialize">stc_block::initialize</a>(&starcoin_framework_account, parent_hash);

    <a href="stc_transaction_publish_option.md#0x1_transaction_publish_option_initialize">transaction_publish_option::initialize</a>(
        &starcoin_framework_account,
        script_allowed,
        module_publishing_allowed,
    );

    // init config
    <a href="vm_config.md#0x1_vm_config_initialize">vm_config::initialize</a>(
        &starcoin_framework_account,
        gas_schedule_blob,
    );

    <a href="flexi_dag_config.md#0x1_flexi_dag_config_initialize">flexi_dag_config::initialize</a>(&starcoin_framework_account, dag_effective_height);

    <a href="stc_transaction_timeout_config.md#0x1_stc_transaction_timeout_config_initialize">stc_transaction_timeout_config::initialize</a>(&starcoin_framework_account, transaction_timeout);
    <a href="consensus_config.md#0x1_consensus_config_initialize">consensus_config::initialize</a>(
        &starcoin_framework_account,
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

    <a href="epoch.md#0x1_epoch_initialize">epoch::initialize</a>(&starcoin_framework_account);
    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"<a href="stc_genesis.md#0x1_stc_genesis_initialize">stc_genesis::initialize</a> | <a href="epoch.md#0x1_epoch">epoch</a> initialized "));

    // stdlib <b>use</b> two phase upgrade strategy.
    <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_update_module_upgrade_strategy">stc_transaction_package_validation::update_module_upgrade_strategy</a>(
        &starcoin_framework_account,
        <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_get_strategy_two_phase">stc_transaction_package_validation::get_strategy_two_phase</a>(),
        <a href="../../move-stdlib/doc/option.md#0x1_option_some">option::some</a>(0u64),
    );

    <a href="block_reward.md#0x1_block_reward_initialize">block_reward::initialize</a>(&starcoin_framework_account, reward_delay);
    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"<a href="stc_genesis.md#0x1_stc_genesis_initialize">stc_genesis::initialize</a> | <a href="block_reward.md#0x1_block_reward">block_reward</a> initialized "));

    // Initliaze STC
    <b>let</b> total_supply_coin = <a href="stc_genesis.md#0x1_stc_genesis_initialize_stc">Self::initialize_stc</a>(
        &starcoin_framework_account,
        total_stc_amount,
        voting_delay,
        voting_period,
        voting_quorum_rate,
        min_action_delay
    );

    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"<a href="stc_genesis.md#0x1_stc_genesis_initialize">stc_genesis::initialize</a> | initialize_stc "));

    // Init goverances <a href="account.md#0x1_account">account</a>
    <b>let</b> core_resource_account = <a href="account.md#0x1_account_create_account">account::create_account</a>(@core_resources);
    <a href="coin.md#0x1_coin_register">coin::register</a>&lt;STC&gt;(&core_resource_account);
    <a href="stc_genesis.md#0x1_stc_genesis_initialize_stc_governance_allocation">Self::initialize_stc_governance_allocation</a>(
        &starcoin_framework_account,
        &core_resource_account,
        total_supply_coin,
        pre_mine_stc_amount,
        time_mint_stc_amount,
        time_mint_stc_period,
    );

    <a href="stc_transaction_fee.md#0x1_stc_transaction_fee_initialize">stc_transaction_fee::initialize</a>(&starcoin_framework_account);

    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"<a href="stc_genesis.md#0x1_stc_genesis_initialize">stc_genesis::initialize</a> | <a href="stc_transaction_fee.md#0x1_stc_transaction_fee">stc_transaction_fee</a> initialized "));

    // Only test/dev network set <a href="genesis.md#0x1_genesis">genesis</a> auth key.
    <b>if</b> (!<a href="../../move-stdlib/doc/vector.md#0x1_vector_is_empty">vector::is_empty</a>(&genesis_auth_key) && (<a href="stc_util.md#0x1_stc_util_is_net_dev">stc_util::is_net_dev</a>() || <a href="stc_util.md#0x1_stc_util_is_net_test">stc_util::is_net_test</a>())) {
        <a href="account.md#0x1_account_rotate_authentication_key_internal">account::rotate_authentication_key_internal</a>(&starcoin_framework_account, genesis_auth_key);
    };
    <a href="account.md#0x1_account_rotate_authentication_key_internal">account::rotate_authentication_key_internal</a>(&core_resource_account, association_auth_key);

    // <b>let</b> assoc_rotate_key_cap = Account::extract_key_rotation_capability(&core_resource_account);
    // Account::rotate_authentication_key_with_capability(&assoc_rotate_key_cap, association_auth_key);
    // Account::restore_key_rotation_capability(assoc_rotate_key_cap);
    //
    // // v5 -&gt; v6
    // {
    //     <b>let</b> cap = Account::remove_signer_capability(&genesis_account);
    //     GenesisSignerCapability::initialize(&genesis_account, cap);
    //     //register <a href="oracle.md#0x1_oracle">oracle</a>
    //     STCUSDOracle::register(&genesis_account);
    //     <b>let</b> merkle_root = x"5969f0e8e19f8769276fb638e6060d5c02e40088f5fde70a6778dd69d659ee6d";
    //     <b>let</b> image = b"ipfs://QmSPcvcXgdtHHiVTAAarzTeubk5X3iWymPAoKBfiRFjPMY";
    //     GenesisNFT::initialize(&genesis_account, merkle_root, 1639u64, image);
    // };
    // StdlibUpgradeScripts::do_upgrade_from_v6_to_v7_with_language_version(&genesis_account, 6);
    // StdlibUpgradeScripts::do_upgrade_from_v11_to_v12(&genesis_account);

    // //Start time, Timestamp::is_genesis() will <b>return</b> <b>false</b>. this call should at the end of <a href="genesis.md#0x1_genesis">genesis</a> init.
    // <a href="timestamp.md#0x1_timestamp_set_time_has_started">timestamp::set_time_has_started</a>(&starcoin_framework_account);
    // account::release_genesis_signer(genesis_account);
    // account::release_genesis_signer(association);

    // Register <a href="oracle.md#0x1_oracle">oracle</a>
    <a href="oracle_stc_usd.md#0x1_oracle_stc_usd_register">oracle_stc_usd::register</a>(&starcoin_framework_account);

    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"<a href="stc_genesis.md#0x1_stc_genesis_initialize">stc_genesis::initialize</a> | Exited"));
}
</code></pre>



</details>

<a id="0x1_stc_genesis_initialize_versions"></a>

## Function `initialize_versions`



<pre><code><b>fun</b> <a href="stc_genesis.md#0x1_stc_genesis_initialize_versions">initialize_versions</a>(starcoin_framework_account: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, stdlib_version: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="stc_genesis.md#0x1_stc_genesis_initialize_versions">initialize_versions</a>(starcoin_framework_account: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, stdlib_version: u64) {
    // Version initialization
    <a href="on_chain_config.md#0x1_on_chain_config_publish_new_config">on_chain_config::publish_new_config</a>&lt;<a href="stc_version.md#0x1_stc_version_Version">stc_version::Version</a>&gt;(
        starcoin_framework_account,
        <a href="stc_version.md#0x1_stc_version_new_version">stc_version::new_version</a>(stdlib_version)
    );
    <a href="on_chain_config.md#0x1_on_chain_config_publish_new_config">on_chain_config::publish_new_config</a>&lt;<a href="stc_language_version.md#0x1_stc_language_version_LanguageVersion">stc_language_version::LanguageVersion</a>&gt;(
        starcoin_framework_account,
        <a href="stc_language_version.md#0x1_stc_language_version_new">stc_language_version::new</a>(13),
    );
}
</code></pre>



</details>

<a id="0x1_stc_genesis_initialize_stc"></a>

## Function `initialize_stc`

First we need to initialize the STC token.
Then we can initialize the treasury.
The treasury will mint the total_stc_amount to the treasury.


<pre><code><b>fun</b> <a href="stc_genesis.md#0x1_stc_genesis_initialize_stc">initialize_stc</a>(starcoin_framework: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, total_stc_amount: u128, voting_delay: u64, voting_period: u64, voting_quorum_rate: u8, min_action_delay: u64): <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;<a href="starcoin_coin.md#0x1_starcoin_coin_STC">starcoin_coin::STC</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="stc_genesis.md#0x1_stc_genesis_initialize_stc">initialize_stc</a>(
    starcoin_framework: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    total_stc_amount: u128,
    voting_delay: u64,
    voting_period: u64,
    voting_quorum_rate: u8,
    min_action_delay: u64
): <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;STC&gt; {
    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"initialize_stc | Entered"));

    <b>let</b> (burn_cap, mint_cap) = <a href="starcoin_coin.md#0x1_starcoin_coin_initialize">starcoin_coin::initialize</a>(starcoin_framework);
    <a href="coin.md#0x1_coin_register">coin::register</a>&lt;STC&gt;(starcoin_framework);

    <a href="coin.md#0x1_coin_create_coin_conversion_map">coin::create_coin_conversion_map</a>(starcoin_framework);
    <a href="coin.md#0x1_coin_create_pairing">coin::create_pairing</a>&lt;STC&gt;(starcoin_framework);

    // <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"initialize_stc | <a href="coin.md#0x1_coin_create_coin_conversion_map">coin::create_coin_conversion_map</a>"));

    <b>let</b> total_stc_coin = <a href="coin.md#0x1_coin_mint">coin::mint</a>((total_stc_amount <b>as</b> u64), &mint_cap);

    // Destroy mint capability and burn cap <b>to</b> ensure constant supply for STC
    <a href="coin.md#0x1_coin_destroy_mint_cap">coin::destroy_mint_cap</a>(mint_cap);
    <a href="coin.md#0x1_coin_destroy_burn_cap">coin::destroy_burn_cap</a>(burn_cap);

    <a href="dao.md#0x1_dao_plugin">dao::plugin</a>&lt;STC&gt;(
        starcoin_framework,
        voting_delay,
        voting_period,
        voting_quorum_rate,
        min_action_delay,
    );

    <a href="dao_modify_config_proposal.md#0x1_dao_modify_config_proposal_plugin">dao_modify_config_proposal::plugin</a>&lt;STC&gt;(starcoin_framework);

    <b>let</b> upgrade_plan_cap =
        <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_extract_submit_upgrade_plan_cap">stc_transaction_package_validation::extract_submit_upgrade_plan_cap</a>(starcoin_framework);
    <a href="dao_upgrade_module_proposal.md#0x1_dao_upgrade_module_proposal_plugin">dao_upgrade_module_proposal::plugin</a>&lt;STC&gt;(starcoin_framework, upgrade_plan_cap);

    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"<a href="stc_genesis.md#0x1_stc_genesis_initialize_stc">stc_genesis::initialize_stc</a> | plugin upgrade cap "));

    // the following configurations are gov-ed by Dao.
    <a href="on_chain_config_dao.md#0x1_on_chain_config_dao_plugin">on_chain_config_dao::plugin</a>&lt;STC, <a href="stc_transaction_publish_option.md#0x1_transaction_publish_option_TransactionPublishOption">transaction_publish_option::TransactionPublishOption</a>&gt;(starcoin_framework);
    <a href="on_chain_config_dao.md#0x1_on_chain_config_dao_plugin">on_chain_config_dao::plugin</a>&lt;STC, <a href="vm_config.md#0x1_vm_config_VMConfig">vm_config::VMConfig</a>&gt;(starcoin_framework);
    <a href="on_chain_config_dao.md#0x1_on_chain_config_dao_plugin">on_chain_config_dao::plugin</a>&lt;STC, <a href="consensus_config.md#0x1_consensus_config_ConsensusConfig">consensus_config::ConsensusConfig</a>&gt;(starcoin_framework);
    <a href="on_chain_config_dao.md#0x1_on_chain_config_dao_plugin">on_chain_config_dao::plugin</a>&lt;STC, <a href="block_reward_config.md#0x1_block_reward_config_RewardConfig">block_reward_config::RewardConfig</a>&gt;(starcoin_framework);
    <a href="on_chain_config_dao.md#0x1_on_chain_config_dao_plugin">on_chain_config_dao::plugin</a>&lt;STC, <a href="stc_transaction_timeout_config.md#0x1_stc_transaction_timeout_config_TransactionTimeoutConfig">stc_transaction_timeout_config::TransactionTimeoutConfig</a>&gt;(starcoin_framework);
    <a href="on_chain_config_dao.md#0x1_on_chain_config_dao_plugin">on_chain_config_dao::plugin</a>&lt;STC, <a href="flexi_dag_config.md#0x1_flexi_dag_config_FlexiDagConfig">flexi_dag_config::FlexiDagConfig</a>&gt;(starcoin_framework);

    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"initialize_stc | Exited"));

    total_stc_coin
}
</code></pre>



</details>

<a id="0x1_stc_genesis_initialize_stc_governance_allocation"></a>

## Function `initialize_stc_governance_allocation`

Overall governance allocation strategy:
1. <code>pre_mine_stc_amount</code> of the total supply is allocated to the Association.
2. <code>time_mint_stc_amount</code> of the total supply is allocated to the Association linearly over <code>time_mint_stc_period</code> blocks.


<pre><code><b>fun</b> <a href="stc_genesis.md#0x1_stc_genesis_initialize_stc_governance_allocation">initialize_stc_governance_allocation</a>(starcoin_framework: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, core_resource_account: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, total_supply_stc: <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;<a href="starcoin_coin.md#0x1_starcoin_coin_STC">starcoin_coin::STC</a>&gt;, pre_mine_stc_amount: u128, time_mint_stc_amount: u128, time_mint_stc_period: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="stc_genesis.md#0x1_stc_genesis_initialize_stc_governance_allocation">initialize_stc_governance_allocation</a>(
    starcoin_framework: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    core_resource_account: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    total_supply_stc: <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;STC&gt;,
    pre_mine_stc_amount: u128,
    time_mint_stc_amount: u128,
    time_mint_stc_period: u64,
) {
    <b>let</b> treasury_withdraw_cap = <a href="treasury.md#0x1_treasury_initialize">treasury::initialize</a>(starcoin_framework, total_supply_stc);

    <b>if</b> (pre_mine_stc_amount &gt; 0) {
        <b>let</b> core_resource_address = <a href="system_addresses.md#0x1_system_addresses_get_core_resource_address">system_addresses::get_core_resource_address</a>();
        <b>let</b> stc = <a href="treasury.md#0x1_treasury_withdraw_with_capability">treasury::withdraw_with_capability</a>&lt;STC&gt;(
            &<b>mut</b> treasury_withdraw_cap,
            pre_mine_stc_amount
        );
        <a href="coin.md#0x1_coin_deposit">coin::deposit</a>(core_resource_address, stc);
    };
    <b>if</b> (time_mint_stc_amount &gt; 0) {
        <b>let</b> liner_withdraw_cap = <a href="treasury.md#0x1_treasury_issue_linear_withdraw_capability">treasury::issue_linear_withdraw_capability</a>&lt;STC&gt;(
            &<b>mut</b> treasury_withdraw_cap,
            time_mint_stc_amount,
            time_mint_stc_period
        );
        <a href="treasury.md#0x1_treasury_add_linear_withdraw_capability">treasury::add_linear_withdraw_capability</a>(core_resource_account, liner_withdraw_cap);
    };
    <a href="dao_treasury_withdraw_proposal.md#0x1_dao_treasury_withdraw_proposal_plugin">dao_treasury_withdraw_proposal::plugin</a>&lt;STC&gt;(starcoin_framework, treasury_withdraw_cap);
}
</code></pre>



</details>

<a id="0x1_stc_genesis_initialize_for_unit_tests"></a>

## Function `initialize_for_unit_tests`

Init the genesis for unit tests


<pre><code><b>public</b> <b>fun</b> <a href="stc_genesis.md#0x1_stc_genesis_initialize_for_unit_tests">initialize_for_unit_tests</a>()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_genesis.md#0x1_stc_genesis_initialize_for_unit_tests">initialize_for_unit_tests</a>() {
    <b>let</b> stdlib_version: u64 = 6;
    <b>let</b> reward_delay: u64 = 7;
    <b>let</b> total_stc_amount: u128 = 3185136000000000000u128;
    <b>let</b> pre_mine_stc_amount: u128 = 159256800000000000u128;
    <b>let</b> time_mint_stc_amount: u128 = (85043130u128 * 3u128 + 74213670u128 * 3u128) * 1000000000u128;
    <b>let</b> time_mint_stc_period: u64 = 1000000000;

    <b>let</b> parent_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt; = x"0000000000000000000000000000000000000000000000000000000000000000";
    <b>let</b> association_auth_key: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt; = x"0000000000000000000000000000000000000000000000000000000000000000";
    <b>let</b> genesis_auth_key: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt; = x"0000000000000000000000000000000000000000000000000000000000000000";
    <b>let</b> <a href="chain_id.md#0x1_chain_id">chain_id</a>: u8 = 255;
    <b>let</b> genesis_timestamp: u64 = 0;

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

    // todo: initialize gas_schedule_blob properly
    <b>let</b> gas_schedule_blob: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt; = <a href="../../move-stdlib/doc/vector.md#0x1_vector_empty">vector::empty</a>&lt;u8&gt;();

    // <a href="dao.md#0x1_dao">dao</a> config
    <b>let</b> voting_delay: u64 = 1000;
    <b>let</b> voting_period: u64 = 6000;
    <b>let</b> voting_quorum_rate: u8 = 4;
    <b>let</b> min_action_delay: u64 = 1000;

    // transaction timeout config
    <b>let</b> transaction_timeout: u64 = 10000;

    <a href="stc_genesis.md#0x1_stc_genesis_initialize">Self::initialize</a>(
        stdlib_version,
        reward_delay,
        total_stc_amount,
        pre_mine_stc_amount,
        time_mint_stc_amount,
        time_mint_stc_period,
        parent_hash,
        association_auth_key,
        genesis_auth_key,
        <a href="chain_id.md#0x1_chain_id">chain_id</a>,
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
        gas_schedule_blob,
        voting_delay,
        voting_period,
        voting_quorum_rate,
        min_action_delay,
        transaction_timeout,
        0,
    );
}
</code></pre>



</details>

<a id="@Specification_0"></a>

## Specification



<pre><code><b>pragma</b> verify = <b>false</b>;
<b>pragma</b> aborts_if_is_partial = <b>false</b>;
<b>pragma</b> aborts_if_is_strict = <b>true</b>;
</code></pre>


[move-book]: https://starcoin.dev/move/book/SUMMARY
