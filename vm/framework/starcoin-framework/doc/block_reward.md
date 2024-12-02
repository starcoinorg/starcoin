
<a id="0x1_block_reward"></a>

# Module `0x1::block_reward`

The module provide block rewarding calculation logic.


-  [Resource `RewardQueue`](#0x1_block_reward_RewardQueue)
-  [Struct `RewardInfo`](#0x1_block_reward_RewardInfo)
-  [Struct `BlockRewardEvent`](#0x1_block_reward_BlockRewardEvent)
-  [Constants](#@Constants_0)
-  [Function `initialize`](#0x1_block_reward_initialize)
-  [Function `process_block_reward`](#0x1_block_reward_process_block_reward)
-  [Specification](#@Specification_1)
    -  [Function `initialize`](#@Specification_1_initialize)
    -  [Function `process_block_reward`](#@Specification_1_process_block_reward)


<pre><code><b>use</b> <a href="account.md#0x1_account">0x1::account</a>;
<b>use</b> <a href="block_reward_config.md#0x1_block_reward_config">0x1::block_reward_config</a>;
<b>use</b> <a href="coin.md#0x1_coin">0x1::coin</a>;
<b>use</b> <a href="create_signer.md#0x1_create_signer">0x1::create_signer</a>;
<b>use</b> <a href="dao_treasury_withdraw_proposal.md#0x1_dao_treasury_withdraw_proposal">0x1::dao_treasury_withdraw_proposal</a>;
<b>use</b> <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug">0x1::debug</a>;
<b>use</b> <a href="../../move-stdlib/doc/error.md#0x1_error">0x1::error</a>;
<b>use</b> <a href="event.md#0x1_event">0x1::event</a>;
<b>use</b> <a href="starcoin_coin.md#0x1_starcoin_coin">0x1::starcoin_coin</a>;
<b>use</b> <a href="../../move-stdlib/doc/string.md#0x1_string">0x1::string</a>;
<b>use</b> <a href="system_addresses.md#0x1_system_addresses">0x1::system_addresses</a>;
<b>use</b> <a href="treasury.md#0x1_treasury">0x1::treasury</a>;
<b>use</b> <a href="../../move-stdlib/doc/vector.md#0x1_vector">0x1::vector</a>;
</code></pre>



<a id="0x1_block_reward_RewardQueue"></a>

## Resource `RewardQueue`

Queue of rewards distributed to miners.


<pre><code><b>struct</b> <a href="block_reward.md#0x1_block_reward_RewardQueue">RewardQueue</a> <b>has</b> key
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
<code>infos: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="block_reward.md#0x1_block_reward_RewardInfo">block_reward::RewardInfo</a>&gt;</code>
</dt>
<dd>
 informations about the reward distribution.
</dd>
<dt>
<code>reward_events: <a href="event.md#0x1_event_EventHandle">event::EventHandle</a>&lt;<a href="block_reward.md#0x1_block_reward_BlockRewardEvent">block_reward::BlockRewardEvent</a>&gt;</code>
</dt>
<dd>
 event handle used to emit block reward event.
</dd>
</dl>


</details>

<a id="0x1_block_reward_RewardInfo"></a>

## Struct `RewardInfo`

Reward info of miners.


<pre><code><b>struct</b> <a href="block_reward.md#0x1_block_reward_RewardInfo">RewardInfo</a> <b>has</b> store
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
<code>gas_fees: <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;<a href="starcoin_coin.md#0x1_starcoin_coin_STC">starcoin_coin::STC</a>&gt;</code>
</dt>
<dd>
 store the gas fee that users consumed.
</dd>
</dl>


</details>

<a id="0x1_block_reward_BlockRewardEvent"></a>

## Struct `BlockRewardEvent`

block reward event


<pre><code><b>struct</b> <a href="block_reward.md#0x1_block_reward_BlockRewardEvent">BlockRewardEvent</a> <b>has</b> drop, store
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
<code><a href="block_reward.md#0x1_block_reward">block_reward</a>: u128</code>
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

<a id="@Constants_0"></a>

## Constants


<a id="0x1_block_reward_EAUTHOR_ADDRESS_AND_AUTH_KEY_MISMATCH"></a>



<pre><code><b>const</b> <a href="block_reward.md#0x1_block_reward_EAUTHOR_ADDRESS_AND_AUTH_KEY_MISMATCH">EAUTHOR_ADDRESS_AND_AUTH_KEY_MISMATCH</a>: u64 = 105;
</code></pre>



<a id="0x1_block_reward_EAUTHOR_AUTH_KEY_IS_EMPTY"></a>



<pre><code><b>const</b> <a href="block_reward.md#0x1_block_reward_EAUTHOR_AUTH_KEY_IS_EMPTY">EAUTHOR_AUTH_KEY_IS_EMPTY</a>: u64 = 101;
</code></pre>



<a id="0x1_block_reward_ECURRENT_NUMBER_IS_WRONG"></a>



<pre><code><b>const</b> <a href="block_reward.md#0x1_block_reward_ECURRENT_NUMBER_IS_WRONG">ECURRENT_NUMBER_IS_WRONG</a>: u64 = 102;
</code></pre>



<a id="0x1_block_reward_EMINER_EXIST"></a>



<pre><code><b>const</b> <a href="block_reward.md#0x1_block_reward_EMINER_EXIST">EMINER_EXIST</a>: u64 = 104;
</code></pre>



<a id="0x1_block_reward_EREWARD_NUMBER_IS_WRONG"></a>



<pre><code><b>const</b> <a href="block_reward.md#0x1_block_reward_EREWARD_NUMBER_IS_WRONG">EREWARD_NUMBER_IS_WRONG</a>: u64 = 103;
</code></pre>



<a id="0x1_block_reward_initialize"></a>

## Function `initialize`

Initialize the module, should be called in genesis.


<pre><code><b>public</b> <b>fun</b> <a href="block_reward.md#0x1_block_reward_initialize">initialize</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, reward_delay: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="block_reward.md#0x1_block_reward_initialize">initialize</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, reward_delay: u64) {
    // Timestamp::assert_genesis();
    <a href="system_addresses.md#0x1_system_addresses_assert_starcoin_framework">system_addresses::assert_starcoin_framework</a>(<a href="account.md#0x1_account">account</a>);

    <a href="block_reward_config.md#0x1_block_reward_config_initialize">block_reward_config::initialize</a>(<a href="account.md#0x1_account">account</a>, reward_delay);
    <b>move_to</b>&lt;<a href="block_reward.md#0x1_block_reward_RewardQueue">RewardQueue</a>&gt;(<a href="account.md#0x1_account">account</a>, <a href="block_reward.md#0x1_block_reward_RewardQueue">RewardQueue</a> {
        reward_number: 0,
        infos: <a href="../../move-stdlib/doc/vector.md#0x1_vector_empty">vector::empty</a>(),
        reward_events: <a href="account.md#0x1_account_new_event_handle">account::new_event_handle</a>&lt;<a href="block_reward.md#0x1_block_reward_BlockRewardEvent">Self::BlockRewardEvent</a>&gt;(<a href="account.md#0x1_account">account</a>),
    });
}
</code></pre>



</details>

<a id="0x1_block_reward_process_block_reward"></a>

## Function `process_block_reward`

Process the given block rewards.


<pre><code><b>public</b> <b>fun</b> <a href="block_reward.md#0x1_block_reward_process_block_reward">process_block_reward</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, current_number: u64, current_reward: u128, current_author: <b>address</b>, _auth_key_vec: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, previous_block_gas_fees: <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;<a href="starcoin_coin.md#0x1_starcoin_coin_STC">starcoin_coin::STC</a>&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="block_reward.md#0x1_block_reward_process_block_reward">process_block_reward</a>(
    <a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    current_number: u64,
    current_reward: u128,
    current_author: <b>address</b>,
    _auth_key_vec: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    previous_block_gas_fees: <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;STC&gt;
) <b>acquires</b> <a href="block_reward.md#0x1_block_reward_RewardQueue">RewardQueue</a> {
    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"<a href="block_reward.md#0x1_block_reward_process_block_reward">block_reward::process_block_reward</a> | Entered"));

    <a href="system_addresses.md#0x1_system_addresses_assert_starcoin_framework">system_addresses::assert_starcoin_framework</a>(<a href="account.md#0x1_account">account</a>);

    <b>if</b> (current_number == 0) {
        <a href="coin.md#0x1_coin_destroy_zero">coin::destroy_zero</a>(previous_block_gas_fees);
        <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"<a href="block_reward.md#0x1_block_reward_process_block_reward">block_reward::process_block_reward</a> | Exited, current_number is 0"));
        <b>return</b>
    };

    <b>let</b> rewards = <b>borrow_global_mut</b>&lt;<a href="block_reward.md#0x1_block_reward_RewardQueue">RewardQueue</a>&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>());
    <b>let</b> len = <a href="../../move-stdlib/doc/vector.md#0x1_vector_length">vector::length</a>(&rewards.infos);

    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"<a href="block_reward.md#0x1_block_reward_process_block_reward">block_reward::process_block_reward</a> | rewards info len: "));
    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&len);

    <b>assert</b>!(
        (current_number == (rewards.reward_number + len + 1)),
        <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="block_reward.md#0x1_block_reward_ECURRENT_NUMBER_IS_WRONG">ECURRENT_NUMBER_IS_WRONG</a>)
    );

    // distribute gas fee <b>to</b> last block reward info.
    // <b>if</b> not last block reward info, the passed in gas fee must be zero.
    <b>if</b> (len == 0) {
        <a href="coin.md#0x1_coin_destroy_zero">coin::destroy_zero</a>(previous_block_gas_fees);
    } <b>else</b> {
        <b>let</b> reward_info = <a href="../../move-stdlib/doc/vector.md#0x1_vector_borrow_mut">vector::borrow_mut</a>(&<b>mut</b> rewards.infos, len - 1);
        <b>assert</b>!(current_number == reward_info.number + 1, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="block_reward.md#0x1_block_reward_ECURRENT_NUMBER_IS_WRONG">ECURRENT_NUMBER_IS_WRONG</a>));
        <a href="coin.md#0x1_coin_merge">coin::merge</a>(&<b>mut</b> reward_info.gas_fees, previous_block_gas_fees);
    };

    <b>let</b> reward_delay = <a href="block_reward_config.md#0x1_block_reward_config_reward_delay">block_reward_config::reward_delay</a>();
    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"<a href="block_reward.md#0x1_block_reward_process_block_reward">block_reward::process_block_reward</a> | rewards delay: "));
    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&reward_delay);
    <b>if</b> (len &gt;= reward_delay) {
        //pay and remove
        <b>let</b> i = len;
        <b>while</b> (i &gt; 0 && i &gt;= reward_delay) {
            <b>let</b> <a href="block_reward.md#0x1_block_reward_RewardInfo">RewardInfo</a> {
                number: reward_block_number,
                reward: <a href="block_reward.md#0x1_block_reward">block_reward</a>,
                gas_fees,
                miner
            } = <a href="../../move-stdlib/doc/vector.md#0x1_vector_remove">vector::remove</a>(
                &<b>mut</b> rewards.infos,
                0
            );

            <b>let</b> gas_fee_value = (<a href="coin.md#0x1_coin_value">coin::value</a>(&gas_fees) <b>as</b> u128);
            <b>let</b> total_reward = gas_fees;
            <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"<a href="block_reward.md#0x1_block_reward_process_block_reward">block_reward::process_block_reward</a> | total_reward: "));
            <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&<a href="coin.md#0x1_coin_value">coin::value</a>(&total_reward));

            // add block reward <b>to</b> total.
            <b>if</b> (<a href="block_reward.md#0x1_block_reward">block_reward</a> &gt; 0) {
                // <b>if</b> no STC in Treasury, BlockReward will been 0.
                <b>let</b> treasury_balance = <a href="treasury.md#0x1_treasury_balance">treasury::balance</a>&lt;STC&gt;();
                <b>if</b> (treasury_balance &lt; <a href="block_reward.md#0x1_block_reward">block_reward</a>) {
                    <a href="block_reward.md#0x1_block_reward">block_reward</a> = treasury_balance;
                };
                <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"<a href="block_reward.md#0x1_block_reward_process_block_reward">block_reward::process_block_reward</a> | treasury_balance: "));
                <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&treasury_balance);
                <b>if</b> (<a href="block_reward.md#0x1_block_reward">block_reward</a> &gt; 0) {
                    <b>let</b> reward = <a href="dao_treasury_withdraw_proposal.md#0x1_dao_treasury_withdraw_proposal_withdraw_for_block_reward">dao_treasury_withdraw_proposal::withdraw_for_block_reward</a>&lt;STC&gt;(<a href="account.md#0x1_account">account</a>, <a href="block_reward.md#0x1_block_reward">block_reward</a>);
                    <a href="coin.md#0x1_coin_merge">coin::merge</a>(&<b>mut</b> total_reward, reward);
                };
            };

            // distribute total.
            <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"<a href="block_reward.md#0x1_block_reward_process_block_reward">block_reward::process_block_reward</a> | distribute total reward: "));
            <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&<a href="coin.md#0x1_coin_value">coin::value</a>(&total_reward));
            <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&miner);

            <b>if</b> (<a href="coin.md#0x1_coin_value">coin::value</a>(&total_reward) &gt; 0) {
                <a href="coin.md#0x1_coin_deposit">coin::deposit</a>&lt;STC&gt;(miner, total_reward);
            } <b>else</b> {
                <a href="coin.md#0x1_coin_destroy_zero">coin::destroy_zero</a>(total_reward);
            };

            <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"<a href="block_reward.md#0x1_block_reward_process_block_reward">block_reward::process_block_reward</a> | before emit reward <a href="event.md#0x1_event">event</a>"));

            // emit reward <a href="event.md#0x1_event">event</a>.
            <a href="event.md#0x1_event_emit_event">event::emit_event</a>&lt;<a href="block_reward.md#0x1_block_reward_BlockRewardEvent">BlockRewardEvent</a>&gt;(
                &<b>mut</b> rewards.reward_events,
                <a href="block_reward.md#0x1_block_reward_BlockRewardEvent">BlockRewardEvent</a> {
                    block_number: reward_block_number,
                    <a href="block_reward.md#0x1_block_reward">block_reward</a>,
                    gas_fees: gas_fee_value,
                    miner,
                }
            );

            <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"<a href="block_reward.md#0x1_block_reward_process_block_reward">block_reward::process_block_reward</a> | after emit reward <a href="event.md#0x1_event">event</a>"));

            rewards.reward_number = rewards.reward_number + 1;
            i = i - 1;
        }
    };

    <a href="account.md#0x1_account_create_account_if_does_not_exist">account::create_account_if_does_not_exist</a>(current_author);
    <b>if</b> (!<a href="coin.md#0x1_coin_is_account_registered">coin::is_account_registered</a>&lt;STC&gt;(current_author)) {
        <a href="coin.md#0x1_coin_register">coin::register</a>&lt;STC&gt;(&<a href="create_signer.md#0x1_create_signer_create_signer">create_signer::create_signer</a>(current_author));
    };

    <b>let</b> current_info = <a href="block_reward.md#0x1_block_reward_RewardInfo">RewardInfo</a> {
        number: current_number,
        reward: current_reward,
        miner: current_author,
        gas_fees: <a href="coin.md#0x1_coin_zero">coin::zero</a>&lt;STC&gt;(),
    };
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> rewards.infos, current_info);

    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"<a href="block_reward.md#0x1_block_reward_process_block_reward">block_reward::process_block_reward</a> | Exited"));
}
</code></pre>



</details>

<a id="@Specification_1"></a>

## Specification



<pre><code><b>pragma</b> verify = <b>false</b>;
<b>pragma</b> aborts_if_is_strict = <b>true</b>;
</code></pre>



<a id="@Specification_1_initialize"></a>

### Function `initialize`


<pre><code><b>public</b> <b>fun</b> <a href="block_reward.md#0x1_block_reward_initialize">initialize</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, reward_delay: u64)
</code></pre>




<pre><code><b>aborts_if</b> <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>) != <a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>();
<b>include</b> <a href="on_chain_config.md#0x1_on_chain_config_PublishNewConfigAbortsIf">on_chain_config::PublishNewConfigAbortsIf</a>&lt;<a href="block_reward_config.md#0x1_block_reward_config_RewardConfig">block_reward_config::RewardConfig</a>&gt;;
<b>include</b> <a href="on_chain_config.md#0x1_on_chain_config_PublishNewConfigEnsures">on_chain_config::PublishNewConfigEnsures</a>&lt;<a href="block_reward_config.md#0x1_block_reward_config_RewardConfig">block_reward_config::RewardConfig</a>&gt;;
<b>aborts_if</b> <b>exists</b>&lt;<a href="block_reward.md#0x1_block_reward_RewardQueue">RewardQueue</a>&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>());
<b>ensures</b> <b>exists</b>&lt;<a href="block_reward.md#0x1_block_reward_RewardQueue">RewardQueue</a>&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>());
</code></pre>



<a id="@Specification_1_process_block_reward"></a>

### Function `process_block_reward`


<pre><code><b>public</b> <b>fun</b> <a href="block_reward.md#0x1_block_reward_process_block_reward">process_block_reward</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, current_number: u64, current_reward: u128, current_author: <b>address</b>, _auth_key_vec: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, previous_block_gas_fees: <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;<a href="starcoin_coin.md#0x1_starcoin_coin_STC">starcoin_coin::STC</a>&gt;)
</code></pre>




<pre><code><b>aborts_if</b> <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>) != <a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>();
<b>aborts_if</b> current_number == 0 && <a href="coin.md#0x1_coin_value">coin::value</a>(previous_block_gas_fees) != 0;
<b>aborts_if</b> current_number &gt; 0 && !<b>exists</b>&lt;<a href="block_reward.md#0x1_block_reward_RewardQueue">RewardQueue</a>&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>());
<b>aborts_if</b> current_number &gt; 0 && (<b>global</b>&lt;<a href="block_reward.md#0x1_block_reward_RewardQueue">RewardQueue</a>&gt;(
    <a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>()
).reward_number + <a href="../../move-stdlib/doc/vector.md#0x1_vector_length">vector::length</a>(
    <b>global</b>&lt;<a href="block_reward.md#0x1_block_reward_RewardQueue">RewardQueue</a>&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>()).infos
) + 1) != current_number;
<b>aborts_if</b> current_number &gt; 0 && !<b>exists</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_Config">on_chain_config::Config</a>&lt;<a href="block_reward_config.md#0x1_block_reward_config_RewardConfig">block_reward_config::RewardConfig</a>&gt;&gt;(
    <a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>()
);
<b>let</b> reward_info_length = <a href="../../move-stdlib/doc/vector.md#0x1_vector_length">vector::length</a>(<b>global</b>&lt;<a href="block_reward.md#0x1_block_reward_RewardQueue">RewardQueue</a>&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>()).infos);
<b>aborts_if</b> current_number &gt; 0 && reward_info_length == 0 && <a href="coin.md#0x1_coin_value">coin::value</a>(previous_block_gas_fees) != 0;
<b>aborts_if</b> current_number &gt; 0 && reward_info_length != 0 && <a href="../../move-stdlib/doc/vector.md#0x1_vector_borrow">vector::borrow</a>(
    <b>global</b>&lt;<a href="block_reward.md#0x1_block_reward_RewardQueue">RewardQueue</a>&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>()).infos,
    reward_info_length - 1
).number != current_number - 1;
<b>aborts_if</b> current_number &gt; 0 && <a href="../../move-stdlib/doc/vector.md#0x1_vector_length">vector::length</a>(
    <b>global</b>&lt;<a href="block_reward.md#0x1_block_reward_RewardQueue">RewardQueue</a>&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>()).infos
) &gt;= <b>global</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_Config">on_chain_config::Config</a>&lt;<a href="block_reward_config.md#0x1_block_reward_config_RewardConfig">block_reward_config::RewardConfig</a>&gt;&gt;(
    <a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>()
).payload.reward_delay
    && (<b>global</b>&lt;<a href="block_reward.md#0x1_block_reward_RewardQueue">RewardQueue</a>&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>()).reward_number + 1) != <a href="../../move-stdlib/doc/vector.md#0x1_vector_borrow">vector::borrow</a>(
    <b>global</b>&lt;<a href="block_reward.md#0x1_block_reward_RewardQueue">RewardQueue</a>&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>()).infos,
    0
).number;
<b>aborts_if</b> current_number &gt; 0 && <a href="../../move-stdlib/doc/vector.md#0x1_vector_length">vector::length</a>(
    <b>global</b>&lt;<a href="block_reward.md#0x1_block_reward_RewardQueue">RewardQueue</a>&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>()).infos
) &gt;= <b>global</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_Config">on_chain_config::Config</a>&lt;<a href="block_reward_config.md#0x1_block_reward_config_RewardConfig">block_reward_config::RewardConfig</a>&gt;&gt;(
    <a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>()
).payload.reward_delay
    && (<b>global</b>&lt;<a href="block_reward.md#0x1_block_reward_RewardQueue">RewardQueue</a>&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>()).reward_number + 1) &gt; max_u64();
<b>aborts_if</b> current_number &gt; 0 && !<a href="account.md#0x1_account_exists_at">account::exists_at</a>(current_author) ;
<b>pragma</b> verify = <b>false</b>;
</code></pre>


[move-book]: https://starcoin.dev/move/book/SUMMARY
