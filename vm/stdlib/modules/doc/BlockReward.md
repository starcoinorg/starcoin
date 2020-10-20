
<a name="0x1_BlockReward"></a>

# Module `0x1::BlockReward`



-  [Resource `RewardQueue`](#0x1_BlockReward_RewardQueue)
-  [Struct `RewardInfo`](#0x1_BlockReward_RewardInfo)
-  [Constants](#@Constants_0)
-  [Function `initialize`](#0x1_BlockReward_initialize)
-  [Function `process_block_reward`](#0x1_BlockReward_process_block_reward)
-  [Specification](#@Specification_1)
    -  [Function `initialize`](#@Specification_1_initialize)
    -  [Function `process_block_reward`](#@Specification_1_process_block_reward)


<pre><code><b>use</b> <a href="Account.md#0x1_Account">0x1::Account</a>;
<b>use</b> <a href="CoreAddresses.md#0x1_CoreAddresses">0x1::CoreAddresses</a>;
<b>use</b> <a href="Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="RewardConfig.md#0x1_RewardConfig">0x1::RewardConfig</a>;
<b>use</b> <a href="STC.md#0x1_STC">0x1::STC</a>;
<b>use</b> <a href="Timestamp.md#0x1_Timestamp">0x1::Timestamp</a>;
<b>use</b> <a href="Token.md#0x1_Token">0x1::Token</a>;
<b>use</b> <a href="Vector.md#0x1_Vector">0x1::Vector</a>;
</code></pre>



<a name="0x1_BlockReward_RewardQueue"></a>

## Resource `RewardQueue`



<pre><code><b>resource</b> <b>struct</b> <a href="BlockReward.md#0x1_BlockReward_RewardQueue">RewardQueue</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>reward_number: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>infos: vector&lt;<a href="BlockReward.md#0x1_BlockReward_RewardInfo">BlockReward::RewardInfo</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_BlockReward_RewardInfo"></a>

## Struct `RewardInfo`



<pre><code><b>struct</b> <a href="BlockReward.md#0x1_BlockReward_RewardInfo">RewardInfo</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>number: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>reward: u128</code>
</dt>
<dd>

</dd>
<dt>
<code>miner: address</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_BlockReward_EAUTHOR_PUBLIC_KEY_IS_NOT_EMPTY"></a>



<pre><code><b>const</b> <a href="BlockReward.md#0x1_BlockReward_EAUTHOR_PUBLIC_KEY_IS_NOT_EMPTY">EAUTHOR_PUBLIC_KEY_IS_NOT_EMPTY</a>: u64 = 101;
</code></pre>



<a name="0x1_BlockReward_ECURRENT_NUMBER_IS_WRONG"></a>



<pre><code><b>const</b> <a href="BlockReward.md#0x1_BlockReward_ECURRENT_NUMBER_IS_WRONG">ECURRENT_NUMBER_IS_WRONG</a>: u64 = 102;
</code></pre>



<a name="0x1_BlockReward_EMINER_EXIST"></a>



<pre><code><b>const</b> <a href="BlockReward.md#0x1_BlockReward_EMINER_EXIST">EMINER_EXIST</a>: u64 = 104;
</code></pre>



<a name="0x1_BlockReward_EREWARD_NUMBER_IS_WRONG"></a>



<pre><code><b>const</b> <a href="BlockReward.md#0x1_BlockReward_EREWARD_NUMBER_IS_WRONG">EREWARD_NUMBER_IS_WRONG</a>: u64 = 103;
</code></pre>



<a name="0x1_BlockReward_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="BlockReward.md#0x1_BlockReward_initialize">initialize</a>(account: &signer, reward_delay: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="BlockReward.md#0x1_BlockReward_initialize">initialize</a>(account: &signer, reward_delay: u64) {
    <a href="Timestamp.md#0x1_Timestamp_assert_genesis">Timestamp::assert_genesis</a>();
    <a href="CoreAddresses.md#0x1_CoreAddresses_assert_genesis_address">CoreAddresses::assert_genesis_address</a>(account);

    <a href="RewardConfig.md#0x1_RewardConfig_initialize">RewardConfig::initialize</a>(account, reward_delay);
    move_to&lt;<a href="BlockReward.md#0x1_BlockReward_RewardQueue">RewardQueue</a>&gt;(account, <a href="BlockReward.md#0x1_BlockReward_RewardQueue">RewardQueue</a> {
        reward_number: 0,
        infos: <a href="Vector.md#0x1_Vector_empty">Vector::empty</a>(),
    });
}
</code></pre>



</details>

<a name="0x1_BlockReward_process_block_reward"></a>

## Function `process_block_reward`



<pre><code><b>public</b> <b>fun</b> <a href="BlockReward.md#0x1_BlockReward_process_block_reward">process_block_reward</a>(account: &signer, current_number: u64, current_reward: u128, current_author: address, public_key_vec: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="BlockReward.md#0x1_BlockReward_process_block_reward">process_block_reward</a>(account: &signer, current_number: u64, current_reward: u128,
                                current_author: address, public_key_vec: vector&lt;u8&gt;) <b>acquires</b> <a href="BlockReward.md#0x1_BlockReward_RewardQueue">RewardQueue</a> {
    <a href="CoreAddresses.md#0x1_CoreAddresses_assert_genesis_address">CoreAddresses::assert_genesis_address</a>(account);

    <b>if</b> (current_number &gt; 0) {
        <b>let</b> rewards = borrow_global_mut&lt;<a href="BlockReward.md#0x1_BlockReward_RewardQueue">RewardQueue</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
        <b>let</b> len = <a href="Vector.md#0x1_Vector_length">Vector::length</a>(&rewards.infos);
        <b>assert</b>((current_number == (rewards.reward_number + len + 1)), <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="BlockReward.md#0x1_BlockReward_ECURRENT_NUMBER_IS_WRONG">ECURRENT_NUMBER_IS_WRONG</a>));

        <b>let</b> reward_delay = <a href="RewardConfig.md#0x1_RewardConfig_reward_delay">RewardConfig::reward_delay</a>();
        <b>if</b> (len &gt;= reward_delay) {//pay and remove
            <b>let</b> i = len;
            <b>while</b> (i &gt; 0 && i &gt;= reward_delay) {
                <b>let</b> reward_number = rewards.reward_number + 1;
                <b>let</b> first_info = *<a href="Vector.md#0x1_Vector_borrow">Vector::borrow</a>(&rewards.infos, 0);
                <b>assert</b>((reward_number == first_info.number), <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="BlockReward.md#0x1_BlockReward_EREWARD_NUMBER_IS_WRONG">EREWARD_NUMBER_IS_WRONG</a>));

                rewards.reward_number = reward_number;
                <b>if</b> (first_info.reward &gt; 0) {
                    <b>assert</b>(<a href="Account.md#0x1_Account_exists_at">Account::exists_at</a>(first_info.miner), <a href="Errors.md#0x1_Errors_requires_address">Errors::requires_address</a>(<a href="BlockReward.md#0x1_BlockReward_EMINER_EXIST">EMINER_EXIST</a>));
                    <b>let</b> reward = <a href="Token.md#0x1_Token_mint">Token::mint</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;(account, first_info.reward);
                    <a href="Account.md#0x1_Account_deposit">Account::deposit</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;(first_info.miner, reward);
                };
                <a href="Vector.md#0x1_Vector_remove">Vector::remove</a>(&<b>mut</b> rewards.infos, 0);
                i = i - 1;
            }
        };

        <b>if</b> (!<a href="Account.md#0x1_Account_exists_at">Account::exists_at</a>(current_author)) {
            //create account from <b>public</b> key
            <b>assert</b>(!<a href="Vector.md#0x1_Vector_is_empty">Vector::is_empty</a>(&public_key_vec), <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="BlockReward.md#0x1_BlockReward_EAUTHOR_PUBLIC_KEY_IS_NOT_EMPTY">EAUTHOR_PUBLIC_KEY_IS_NOT_EMPTY</a>));
            <a href="Account.md#0x1_Account_create_account">Account::create_account</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;(current_author, public_key_vec);
        };
        <b>let</b> current_info = <a href="BlockReward.md#0x1_BlockReward_RewardInfo">RewardInfo</a> {
            number: current_number,
            reward: current_reward,
            miner: current_author,
        };
        <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> rewards.infos, current_info);
    };
}
</code></pre>



</details>

<a name="@Specification_1"></a>

## Specification



<pre><code><b>pragma</b> verify = <b>true</b>;
<b>pragma</b> aborts_if_is_strict = <b>true</b>;
</code></pre>



<a name="@Specification_1_initialize"></a>

### Function `initialize`


<pre><code><b>public</b> <b>fun</b> <a href="BlockReward.md#0x1_BlockReward_initialize">initialize</a>(account: &signer, reward_delay: u64)
</code></pre>




<pre><code><b>aborts_if</b> !<a href="Timestamp.md#0x1_Timestamp_is_genesis">Timestamp::is_genesis</a>();
<b>aborts_if</b> <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) != <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>();
<b>include</b> <a href="Config.md#0x1_Config_PublishNewConfigAbortsIf">Config::PublishNewConfigAbortsIf</a>&lt;<a href="RewardConfig.md#0x1_RewardConfig_RewardConfig">RewardConfig::RewardConfig</a>&gt;;
<b>include</b> <a href="Config.md#0x1_Config_PublishNewConfigEnsures">Config::PublishNewConfigEnsures</a>&lt;<a href="RewardConfig.md#0x1_RewardConfig_RewardConfig">RewardConfig::RewardConfig</a>&gt;;
<b>aborts_if</b> <b>exists</b>&lt;<a href="BlockReward.md#0x1_BlockReward_RewardQueue">RewardQueue</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
<b>ensures</b> <b>exists</b>&lt;<a href="BlockReward.md#0x1_BlockReward_RewardQueue">RewardQueue</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
</code></pre>



<a name="@Specification_1_process_block_reward"></a>

### Function `process_block_reward`


<pre><code><b>public</b> <b>fun</b> <a href="BlockReward.md#0x1_BlockReward_process_block_reward">process_block_reward</a>(account: &signer, current_number: u64, current_reward: u128, current_author: address, public_key_vec: vector&lt;u8&gt;)
</code></pre>




<pre><code><b>aborts_if</b> <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) != <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>();
<b>aborts_if</b> current_number &gt; 0 && !<b>exists</b>&lt;<a href="BlockReward.md#0x1_BlockReward_RewardQueue">RewardQueue</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
<b>aborts_if</b> current_number &gt; 0 && (<b>global</b>&lt;<a href="BlockReward.md#0x1_BlockReward_RewardQueue">RewardQueue</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>()).reward_number + <a href="Vector.md#0x1_Vector_length">Vector::length</a>(<b>global</b>&lt;<a href="BlockReward.md#0x1_BlockReward_RewardQueue">RewardQueue</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>()).infos) + 1) != current_number;
<b>aborts_if</b> current_number &gt; 0 && !<b>exists</b>&lt;<a href="Config.md#0x1_Config_Config">Config::Config</a>&lt;<a href="RewardConfig.md#0x1_RewardConfig_RewardConfig">RewardConfig::RewardConfig</a>&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
<b>aborts_if</b> current_number &gt; 0 && <a href="Vector.md#0x1_Vector_length">Vector::length</a>(<b>global</b>&lt;<a href="BlockReward.md#0x1_BlockReward_RewardQueue">RewardQueue</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>()).infos) &gt;= <b>global</b>&lt;<a href="Config.md#0x1_Config_Config">Config::Config</a>&lt;<a href="RewardConfig.md#0x1_RewardConfig_RewardConfig">RewardConfig::RewardConfig</a>&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>()).payload.reward_delay
&& (<b>global</b>&lt;<a href="BlockReward.md#0x1_BlockReward_RewardQueue">RewardQueue</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>()).reward_number + 1) != <a href="Vector.md#0x1_Vector_borrow">Vector::borrow</a>(<b>global</b>&lt;<a href="BlockReward.md#0x1_BlockReward_RewardQueue">RewardQueue</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>()).infos, 0).number;
<b>aborts_if</b> current_number &gt; 0 && <a href="Vector.md#0x1_Vector_length">Vector::length</a>(<b>global</b>&lt;<a href="BlockReward.md#0x1_BlockReward_RewardQueue">RewardQueue</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>()).infos) &gt;= <b>global</b>&lt;<a href="Config.md#0x1_Config_Config">Config::Config</a>&lt;<a href="RewardConfig.md#0x1_RewardConfig_RewardConfig">RewardConfig::RewardConfig</a>&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>()).payload.reward_delay
        && (<b>global</b>&lt;<a href="BlockReward.md#0x1_BlockReward_RewardQueue">RewardQueue</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>()).reward_number + 1) &gt; max_u64();
<b>aborts_if</b> current_number &gt; 0 && !<a href="Account.md#0x1_Account_exists_at">Account::exists_at</a>(current_author) && <a href="Vector.md#0x1_Vector_is_empty">Vector::is_empty</a>(public_key_vec);
<b>aborts_if</b> current_number &gt; 0 && !<a href="Account.md#0x1_Account_exists_at">Account::exists_at</a>(current_author) && len(<a href="Authenticator.md#0x1_Authenticator_spec_ed25519_authentication_key">Authenticator::spec_ed25519_authentication_key</a>(public_key_vec)) != 32;
<b>aborts_if</b> current_number &gt; 0 && !<a href="Account.md#0x1_Account_exists_at">Account::exists_at</a>(current_author) && <a href="Authenticator.md#0x1_Authenticator_spec_derived_address">Authenticator::spec_derived_address</a>(<a href="Authenticator.md#0x1_Authenticator_spec_ed25519_authentication_key">Authenticator::spec_ed25519_authentication_key</a>(public_key_vec)) != current_author;
<b>pragma</b> verify = <b>false</b>;
</code></pre>
