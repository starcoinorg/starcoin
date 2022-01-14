
<a name="0x1_BlockReward"></a>

# Module `0x1::BlockReward`

The module provide block rewarding calculation logic.


-  [Resource `RewardQueue`](#0x1_BlockReward_RewardQueue)
-  [Struct `RewardInfo`](#0x1_BlockReward_RewardInfo)
-  [Struct `BlockRewardEvent`](#0x1_BlockReward_BlockRewardEvent)
-  [Constants](#@Constants_0)
-  [Function `initialize`](#0x1_BlockReward_initialize)
-  [Function `process_block_reward`](#0x1_BlockReward_process_block_reward)
-  [Specification](#@Specification_1)
    -  [Function `initialize`](#@Specification_1_initialize)
    -  [Function `process_block_reward`](#@Specification_1_process_block_reward)


<pre><code><b>use</b> <a href="Account.md#0x1_Account">0x1::Account</a>;
<b>use</b> <a href="CoreAddresses.md#0x1_CoreAddresses">0x1::CoreAddresses</a>;
<b>use</b> <a href="Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="Event.md#0x1_Event">0x1::Event</a>;
<b>use</b> <a href="RewardConfig.md#0x1_RewardConfig">0x1::RewardConfig</a>;
<b>use</b> <a href="STC.md#0x1_STC">0x1::STC</a>;
<b>use</b> <a href="Timestamp.md#0x1_Timestamp">0x1::Timestamp</a>;
<b>use</b> <a href="Token.md#0x1_Token">0x1::Token</a>;
<b>use</b> <a href="Treasury.md#0x1_Treasury">0x1::Treasury</a>;
<b>use</b> <a href="TreasuryWithdrawDaoProposal.md#0x1_TreasuryWithdrawDaoProposal">0x1::TreasuryWithdrawDaoProposal</a>;
<b>use</b> <a href="Vector.md#0x1_Vector">0x1::Vector</a>;
</code></pre>



<a name="0x1_BlockReward_RewardQueue"></a>

## Resource `RewardQueue`

Queue of rewards distributed to miners.


<pre><code><b>struct</b> <a href="BlockReward.md#0x1_BlockReward_RewardQueue">RewardQueue</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>reward_number: u64</code>
</dt>
<dd>
 How many block rewards has been handled.
</dd>
<dt>
<code>infos: vector&lt;<a href="BlockReward.md#0x1_BlockReward_RewardInfo">BlockReward::RewardInfo</a>&gt;</code>
</dt>
<dd>
 informations about the reward distribution.
</dd>
<dt>
<code>reward_events: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="BlockReward.md#0x1_BlockReward_BlockRewardEvent">BlockReward::BlockRewardEvent</a>&gt;</code>
</dt>
<dd>
 event handle used to emit block reward event.
</dd>
</dl>


</details>

<a name="0x1_BlockReward_RewardInfo"></a>

## Struct `RewardInfo`

Reward info of miners.


<pre><code><b>struct</b> <a href="BlockReward.md#0x1_BlockReward_RewardInfo">RewardInfo</a> <b>has</b> store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>number: u64</code>
</dt>
<dd>
 number of the block miner minted.
</dd>
<dt>
<code>reward: u128</code>
</dt>
<dd>
 how many stc rewards.
</dd>
<dt>
<code>miner: <b>address</b></code>
</dt>
<dd>
 miner who mint the block.
</dd>
<dt>
<code>gas_fees: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;<a href="STC.md#0x1_STC_STC">STC::STC</a>&gt;</code>
</dt>
<dd>
 store the gas fee that users consumed.
</dd>
</dl>


</details>

<a name="0x1_BlockReward_BlockRewardEvent"></a>

## Struct `BlockRewardEvent`

block reward event


<pre><code><b>struct</b> <a href="BlockReward.md#0x1_BlockReward_BlockRewardEvent">BlockRewardEvent</a> <b>has</b> drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>block_number: u64</code>
</dt>
<dd>
 block number
</dd>
<dt>
<code>block_reward: u128</code>
</dt>
<dd>
 STC reward.
</dd>
<dt>
<code>gas_fees: u128</code>
</dt>
<dd>
 gas fees in STC.
</dd>
<dt>
<code>miner: <b>address</b></code>
</dt>
<dd>
 block miner
</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_BlockReward_EAUTHOR_ADDRESS_AND_AUTH_KEY_MISMATCH"></a>



<pre><code><b>const</b> <a href="BlockReward.md#0x1_BlockReward_EAUTHOR_ADDRESS_AND_AUTH_KEY_MISMATCH">EAUTHOR_ADDRESS_AND_AUTH_KEY_MISMATCH</a>: u64 = 105;
</code></pre>



<a name="0x1_BlockReward_EAUTHOR_AUTH_KEY_IS_EMPTY"></a>



<pre><code><b>const</b> <a href="BlockReward.md#0x1_BlockReward_EAUTHOR_AUTH_KEY_IS_EMPTY">EAUTHOR_AUTH_KEY_IS_EMPTY</a>: u64 = 101;
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

Initialize the module, should be called in genesis.


<pre><code><b>public</b> <b>fun</b> <a href="BlockReward.md#0x1_BlockReward_initialize">initialize</a>(account: &signer, reward_delay: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="BlockReward.md#0x1_BlockReward_initialize">initialize</a>(account: &signer, reward_delay: u64) {
    <a href="Timestamp.md#0x1_Timestamp_assert_genesis">Timestamp::assert_genesis</a>();
    <a href="CoreAddresses.md#0x1_CoreAddresses_assert_genesis_address">CoreAddresses::assert_genesis_address</a>(account);

    <a href="RewardConfig.md#0x1_RewardConfig_initialize">RewardConfig::initialize</a>(account, reward_delay);
    <b>move_to</b>&lt;<a href="BlockReward.md#0x1_BlockReward_RewardQueue">RewardQueue</a>&gt;(account, <a href="BlockReward.md#0x1_BlockReward_RewardQueue">RewardQueue</a> {
        reward_number: 0,
        infos: <a href="Vector.md#0x1_Vector_empty">Vector::empty</a>(),
        reward_events: <a href="Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="BlockReward.md#0x1_BlockReward_BlockRewardEvent">Self::BlockRewardEvent</a>&gt;(account),
    });
}
</code></pre>



</details>

<a name="0x1_BlockReward_process_block_reward"></a>

## Function `process_block_reward`

Process the given block rewards.


<pre><code><b>public</b> <b>fun</b> <a href="BlockReward.md#0x1_BlockReward_process_block_reward">process_block_reward</a>(account: &signer, current_number: u64, current_reward: u128, current_author: <b>address</b>, _auth_key_vec: vector&lt;u8&gt;, previous_block_gas_fees: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;<a href="STC.md#0x1_STC_STC">STC::STC</a>&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="BlockReward.md#0x1_BlockReward_process_block_reward">process_block_reward</a>(account: &signer, current_number: u64, current_reward: u128,
                                current_author: <b>address</b>, _auth_key_vec: vector&lt;u8&gt;,
                                previous_block_gas_fees: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;) <b>acquires</b> <a href="BlockReward.md#0x1_BlockReward_RewardQueue">RewardQueue</a> {
    <a href="CoreAddresses.md#0x1_CoreAddresses_assert_genesis_address">CoreAddresses::assert_genesis_address</a>(account);
    <b>if</b> (current_number == 0) {
        <a href="Token.md#0x1_Token_destroy_zero">Token::destroy_zero</a>(previous_block_gas_fees);
        <b>return</b>
    };

    <b>let</b> rewards = <b>borrow_global_mut</b>&lt;<a href="BlockReward.md#0x1_BlockReward_RewardQueue">RewardQueue</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
    <b>let</b> len = <a href="Vector.md#0x1_Vector_length">Vector::length</a>(&rewards.infos);
    <b>assert</b>!((current_number == (rewards.reward_number + len + 1)), <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="BlockReward.md#0x1_BlockReward_ECURRENT_NUMBER_IS_WRONG">ECURRENT_NUMBER_IS_WRONG</a>));

    // distribute gas fee <b>to</b> last block reward info.
    // <b>if</b> not last block reward info, the passed in gas fee must be zero.
    <b>if</b> (len == 0) {
        <a href="Token.md#0x1_Token_destroy_zero">Token::destroy_zero</a>(previous_block_gas_fees);
    } <b>else</b> {
        <b>let</b> reward_info = <a href="Vector.md#0x1_Vector_borrow_mut">Vector::borrow_mut</a>(&<b>mut</b> rewards.infos, len - 1);
        <b>assert</b>!(current_number == reward_info.number + 1, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="BlockReward.md#0x1_BlockReward_ECURRENT_NUMBER_IS_WRONG">ECURRENT_NUMBER_IS_WRONG</a>));
        <a href="Token.md#0x1_Token_deposit">Token::deposit</a>(&<b>mut</b> reward_info.gas_fees, previous_block_gas_fees);
    };

    <b>let</b> reward_delay = <a href="RewardConfig.md#0x1_RewardConfig_reward_delay">RewardConfig::reward_delay</a>();
    <b>if</b> (len &gt;= reward_delay) {//pay and remove
        <b>let</b> i = len;
        <b>while</b> (i &gt; 0 && i &gt;= reward_delay) {
            <b>let</b> <a href="BlockReward.md#0x1_BlockReward_RewardInfo">RewardInfo</a> { number: reward_block_number, reward: block_reward, gas_fees, miner } = <a href="Vector.md#0x1_Vector_remove">Vector::remove</a>(&<b>mut</b> rewards.infos, 0);

            <b>let</b> gas_fee_value = <a href="Token.md#0x1_Token_value">Token::value</a>(&gas_fees);
            <b>let</b> total_reward = gas_fees;
            // add block reward <b>to</b> total.
            <b>if</b> (block_reward &gt; 0) {
                // <b>if</b> no <a href="STC.md#0x1_STC">STC</a> in <a href="Treasury.md#0x1_Treasury">Treasury</a>, <a href="BlockReward.md#0x1_BlockReward">BlockReward</a> will been 0.
                <b>let</b> treasury_balance = <a href="Treasury.md#0x1_Treasury_balance">Treasury::balance</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;();
                <b>if</b> (treasury_balance &lt; block_reward) {
                    block_reward = treasury_balance;
                };
                <b>if</b> (block_reward &gt; 0) {
                    <b>let</b> reward = <a href="TreasuryWithdrawDaoProposal.md#0x1_TreasuryWithdrawDaoProposal_withdraw_for_block_reward">TreasuryWithdrawDaoProposal::withdraw_for_block_reward</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;(account, block_reward);
                    <a href="Token.md#0x1_Token_deposit">Token::deposit</a>(&<b>mut</b> total_reward, reward);
                };
            };
            // distribute total.
            <b>if</b> (<a href="Token.md#0x1_Token_value">Token::value</a>(&total_reward) &gt; 0) {
                <a href="Account.md#0x1_Account_deposit">Account::deposit</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;(miner, total_reward);
            } <b>else</b> {
                <a href="Token.md#0x1_Token_destroy_zero">Token::destroy_zero</a>(total_reward);
            };
            // emit reward event.
            <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>&lt;<a href="BlockReward.md#0x1_BlockReward_BlockRewardEvent">BlockRewardEvent</a>&gt;(
                &<b>mut</b> rewards.reward_events,
                <a href="BlockReward.md#0x1_BlockReward_BlockRewardEvent">BlockRewardEvent</a> {
                    block_number: reward_block_number,
                    block_reward: block_reward,
                    gas_fees: gas_fee_value,
                    miner,
                }
            );

            rewards.reward_number = rewards.reward_number + 1;
            i = i - 1;
        }
    };

    <b>if</b> (!<a href="Account.md#0x1_Account_exists_at">Account::exists_at</a>(current_author)) {
        <a href="Account.md#0x1_Account_create_account_with_address">Account::create_account_with_address</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;(current_author);
    };
    <b>let</b> current_info = <a href="BlockReward.md#0x1_BlockReward_RewardInfo">RewardInfo</a> {
        number: current_number,
        reward: current_reward,
        miner: current_author,
        gas_fees: <a href="Token.md#0x1_Token_zero">Token::zero</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;(),
    };
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> rewards.infos, current_info);

}
</code></pre>



</details>

<a name="@Specification_1"></a>

## Specification



<pre><code><b>pragma</b> verify = <b>false</b>;
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


<pre><code><b>public</b> <b>fun</b> <a href="BlockReward.md#0x1_BlockReward_process_block_reward">process_block_reward</a>(account: &signer, current_number: u64, current_reward: u128, current_author: <b>address</b>, _auth_key_vec: vector&lt;u8&gt;, previous_block_gas_fees: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;<a href="STC.md#0x1_STC_STC">STC::STC</a>&gt;)
</code></pre>




<pre><code><b>aborts_if</b> <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) != <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>();
<b>aborts_if</b> current_number == 0 && <a href="Token.md#0x1_Token_value">Token::value</a>(previous_block_gas_fees) != 0;
<b>aborts_if</b> current_number &gt; 0 && !<b>exists</b>&lt;<a href="BlockReward.md#0x1_BlockReward_RewardQueue">RewardQueue</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
<b>aborts_if</b> current_number &gt; 0 && (<b>global</b>&lt;<a href="BlockReward.md#0x1_BlockReward_RewardQueue">RewardQueue</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>()).reward_number + <a href="Vector.md#0x1_Vector_length">Vector::length</a>(<b>global</b>&lt;<a href="BlockReward.md#0x1_BlockReward_RewardQueue">RewardQueue</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>()).infos) + 1) != current_number;
<b>aborts_if</b> current_number &gt; 0 && !<b>exists</b>&lt;<a href="Config.md#0x1_Config_Config">Config::Config</a>&lt;<a href="RewardConfig.md#0x1_RewardConfig_RewardConfig">RewardConfig::RewardConfig</a>&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
<b>let</b> reward_info_length = <a href="Vector.md#0x1_Vector_length">Vector::length</a>(<b>global</b>&lt;<a href="BlockReward.md#0x1_BlockReward_RewardQueue">RewardQueue</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>()).infos);
<b>aborts_if</b> current_number &gt; 0 && reward_info_length == 0 && <a href="Token.md#0x1_Token_value">Token::value</a>(previous_block_gas_fees) != 0;
<b>aborts_if</b> current_number &gt; 0 && reward_info_length != 0 && <a href="Vector.md#0x1_Vector_borrow">Vector::borrow</a>(<b>global</b>&lt;<a href="BlockReward.md#0x1_BlockReward_RewardQueue">RewardQueue</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>()).infos, reward_info_length - 1).number != current_number - 1;
<b>aborts_if</b> current_number &gt; 0 && <a href="Vector.md#0x1_Vector_length">Vector::length</a>(<b>global</b>&lt;<a href="BlockReward.md#0x1_BlockReward_RewardQueue">RewardQueue</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>()).infos) &gt;= <b>global</b>&lt;<a href="Config.md#0x1_Config_Config">Config::Config</a>&lt;<a href="RewardConfig.md#0x1_RewardConfig_RewardConfig">RewardConfig::RewardConfig</a>&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>()).payload.reward_delay
&& (<b>global</b>&lt;<a href="BlockReward.md#0x1_BlockReward_RewardQueue">RewardQueue</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>()).reward_number + 1) != <a href="Vector.md#0x1_Vector_borrow">Vector::borrow</a>(<b>global</b>&lt;<a href="BlockReward.md#0x1_BlockReward_RewardQueue">RewardQueue</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>()).infos, 0).number;
<b>aborts_if</b> current_number &gt; 0 && <a href="Vector.md#0x1_Vector_length">Vector::length</a>(<b>global</b>&lt;<a href="BlockReward.md#0x1_BlockReward_RewardQueue">RewardQueue</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>()).infos) &gt;= <b>global</b>&lt;<a href="Config.md#0x1_Config_Config">Config::Config</a>&lt;<a href="RewardConfig.md#0x1_RewardConfig_RewardConfig">RewardConfig::RewardConfig</a>&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>()).payload.reward_delay
        && (<b>global</b>&lt;<a href="BlockReward.md#0x1_BlockReward_RewardQueue">RewardQueue</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>()).reward_number + 1) &gt; max_u64();
<b>aborts_if</b> current_number &gt; 0 && !<a href="Account.md#0x1_Account_exists_at">Account::exists_at</a>(current_author) ;
<b>pragma</b> verify = <b>false</b>;
</code></pre>
