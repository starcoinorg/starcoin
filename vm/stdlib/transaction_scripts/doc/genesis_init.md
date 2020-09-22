
<a name="SCRIPT"></a>

# Script `genesis_init.move`

### Table of Contents

-  [Function `genesis_init`](#SCRIPT_genesis_init)



<a name="SCRIPT_genesis_init"></a>

## Function `genesis_init`



<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_genesis_init">genesis_init</a>(merged_script_allow_list: vector&lt;u8&gt;, is_open_module: bool, instruction_schedule: vector&lt;u8&gt;, native_schedule: vector&lt;u8&gt;, reward_delay: u64, uncle_rate_target: u64, epoch_block_count: u64, init_block_time_target: u64, block_difficulty_window: u64, init_reward_per_block: u128, reward_per_uncle_percent: u64, min_block_time_target: u64, max_block_time_target: u64, max_uncles_per_block: u64, pre_mine_amount: u128, parent_hash: vector&lt;u8&gt;, association_auth_key: vector&lt;u8&gt;, genesis_auth_key: vector&lt;u8&gt;, chain_id: u8, consensus_strategy: u8, genesis_timestamp: u64, block_gas_limit: u64, global_memory_per_byte_cost: u64, global_memory_per_byte_write_cost: u64, min_transaction_gas_units: u64, large_transaction_cutoff: u64, instrinsic_gas_per_byte: u64, maximum_number_of_gas_units: u64, min_price_per_gas_unit: u64, max_price_per_gas_unit: u64, max_transaction_size_in_bytes: u64, gas_unit_scaling_factor: u64, default_account_size: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_genesis_init">genesis_init</a>(
                 merged_script_allow_list: vector&lt;u8&gt;,
                 is_open_module: bool,
                 instruction_schedule: vector&lt;u8&gt;,
                 native_schedule: vector&lt;u8&gt;,
                 reward_delay: u64,
                 uncle_rate_target:u64,
                 epoch_block_count: u64,
                 init_block_time_target: u64,
                 block_difficulty_window: u64,
                 init_reward_per_block: u128,
                 reward_per_uncle_percent: u64,
                 min_block_time_target:u64,
                 max_block_time_target: u64,
                 max_uncles_per_block:u64,
                 pre_mine_amount:u128,
                 parent_hash: vector&lt;u8&gt;,
                 association_auth_key: vector&lt;u8&gt;,
                 genesis_auth_key: vector&lt;u8&gt;,
                 chain_id: u8,
                 consensus_strategy: u8,
                 genesis_timestamp: u64,
                 block_gas_limit: u64,
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
                 ) {

        <b>assert</b>(<a href="../../modules/doc/Timestamp.md#0x1_Timestamp_is_genesis">Timestamp::is_genesis</a>(), 1);

        // create genesis account
        <b>let</b> genesis_account = <a href="../../modules/doc/Account.md#0x1_Account_create_genesis_account">Account::create_genesis_account</a>(<a href="../../modules/doc/CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());

        //Init <b>global</b> time
        <a href="../../modules/doc/Timestamp.md#0x1_Timestamp_initialize">Timestamp::initialize</a>(&genesis_account, genesis_timestamp);
        <a href="../../modules/doc/ChainId.md#0x1_ChainId_initialize">ChainId::initialize</a>(&genesis_account, chain_id);
        <a href="../../modules/doc/ConsensusStrategy.md#0x1_ConsensusStrategy_initialize">ConsensusStrategy::initialize</a>(&genesis_account, consensus_strategy);

        <a href="../../modules/doc/Block.md#0x1_Block_initialize">Block::initialize</a>(&genesis_account, parent_hash);

        <a href="../../modules/doc/TransactionPublishOption.md#0x1_TransactionPublishOption_initialize">TransactionPublishOption::initialize</a>(
            &genesis_account,
            merged_script_allow_list,
            is_open_module,
        );

        // init config
        <a href="../../modules/doc/VMConfig.md#0x1_VMConfig_initialize">VMConfig::initialize</a>(&genesis_account, instruction_schedule, native_schedule,
            block_gas_limit,
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
            default_account_size
            );
        <a href="../../modules/doc/Version.md#0x1_Version_initialize">Version::initialize</a>(&genesis_account);

        <a href="../../modules/doc/TransactionTimeout.md#0x1_TransactionTimeout_initialize">TransactionTimeout::initialize</a>(&genesis_account);

        <a href="../../modules/doc/STC.md#0x1_STC_initialize">STC::initialize</a>(&genesis_account);
        <a href="../../modules/doc/DummyToken.md#0x1_DummyToken_initialize">DummyToken::initialize</a>(&genesis_account);
        <a href="../../modules/doc/Account.md#0x1_Account_accept_token">Account::accept_token</a>&lt;<a href="../../modules/doc/STC.md#0x1_STC">STC</a>&gt;(&genesis_account);

        <b>let</b> association = <a href="../../modules/doc/Account.md#0x1_Account_create_genesis_account">Account::create_genesis_account</a>(<a href="../../modules/doc/CoreAddresses.md#0x1_CoreAddresses_ASSOCIATION_ROOT_ADDRESS">CoreAddresses::ASSOCIATION_ROOT_ADDRESS</a>());
        <a href="../../modules/doc/Account.md#0x1_Account_accept_token">Account::accept_token</a>&lt;<a href="../../modules/doc/STC.md#0x1_STC">STC</a>&gt;(&association);

        <b>if</b> (pre_mine_amount &gt; 0) {
            <b>let</b> stc = <a href="../../modules/doc/Token.md#0x1_Token_mint">Token::mint</a>&lt;<a href="../../modules/doc/STC.md#0x1_STC">STC</a>&gt;(&genesis_account, pre_mine_amount);
            <a href="../../modules/doc/Account.md#0x1_Account_deposit_to">Account::deposit_to</a>(&genesis_account, <a href="../../modules/doc/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(&association), stc);
        };

        <a href="../../modules/doc/Consensus.md#0x1_Consensus_initialize">Consensus::initialize</a>(&genesis_account, uncle_rate_target, epoch_block_count, init_block_time_target, block_difficulty_window,
                                init_reward_per_block, reward_per_uncle_percent, min_block_time_target, max_block_time_target, max_uncles_per_block);

        <a href="../../modules/doc/BlockReward.md#0x1_BlockReward_initialize">BlockReward::initialize</a>(&genesis_account, reward_delay);

        <a href="../../modules/doc/TransactionFee.md#0x1_TransactionFee_initialize">TransactionFee::initialize</a>(&genesis_account);
        //Grant stdlib maintainer <b>to</b> association
        <a href="../../modules/doc/PackageTxnManager.md#0x1_PackageTxnManager_grant_maintainer">PackageTxnManager::grant_maintainer</a>(&genesis_account, <a href="../../modules/doc/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(&association));
        //TODO set stdlib upgrade strategy.

        // only dev network set genesis auth key.
        <b>if</b> (!<a href="../../modules/doc/Vector.md#0x1_Vector_is_empty">Vector::is_empty</a>(&genesis_auth_key)){
            <b>let</b> genesis_rotate_key_cap = <a href="../../modules/doc/Account.md#0x1_Account_extract_key_rotation_capability">Account::extract_key_rotation_capability</a>(&genesis_account);
            <a href="../../modules/doc/Account.md#0x1_Account_rotate_authentication_key">Account::rotate_authentication_key</a>(&genesis_rotate_key_cap, genesis_auth_key);
            <a href="../../modules/doc/Account.md#0x1_Account_restore_key_rotation_capability">Account::restore_key_rotation_capability</a>(genesis_rotate_key_cap);
        };

        <b>let</b> assoc_rotate_key_cap = <a href="../../modules/doc/Account.md#0x1_Account_extract_key_rotation_capability">Account::extract_key_rotation_capability</a>(&association);
        <a href="../../modules/doc/Account.md#0x1_Account_rotate_authentication_key">Account::rotate_authentication_key</a>(&assoc_rotate_key_cap, association_auth_key);
        <a href="../../modules/doc/Account.md#0x1_Account_restore_key_rotation_capability">Account::restore_key_rotation_capability</a>(assoc_rotate_key_cap);

        //Start time, <a href="../../modules/doc/Timestamp.md#0x1_Timestamp_is_genesis">Timestamp::is_genesis</a>() will <b>return</b> <b>false</b>. this call should at the end of genesis init.
        <a href="../../modules/doc/Timestamp.md#0x1_Timestamp_set_time_has_started">Timestamp::set_time_has_started</a>(&genesis_account);
        <a href="../../modules/doc/Account.md#0x1_Account_release_genesis_signer">Account::release_genesis_signer</a>(genesis_account);
        <a href="../../modules/doc/Account.md#0x1_Account_release_genesis_signer">Account::release_genesis_signer</a>(association);

}
</code></pre>



</details>
