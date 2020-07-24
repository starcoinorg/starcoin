
<a name="0x1_BlockReward"></a>

# Module `0x1::BlockReward`

### Table of Contents

-  [Resource `BlockReward`](#0x1_BlockReward_BlockReward)
-  [Resource `RewardQueue`](#0x1_BlockReward_RewardQueue)
-  [Struct `RewardInfo`](#0x1_BlockReward_RewardInfo)
-  [Function `initialize`](#0x1_BlockReward_initialize)
-  [Function `withdraw`](#0x1_BlockReward_withdraw)
-  [Function `process_block_reward`](#0x1_BlockReward_process_block_reward)



<a name="0x1_BlockReward_BlockReward"></a>

## Resource `BlockReward`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_BlockReward">BlockReward</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>balance: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;<a href="STC.md#0x1_STC_STC">STC::STC</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_BlockReward_RewardQueue"></a>

## Resource `RewardQueue`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_BlockReward_RewardQueue">RewardQueue</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>reward_height: u64</code>
</dt>
<dd>

</dd>
<dt>

<code>reward_delay: u64</code>
</dt>
<dd>

</dd>
<dt>

<code>infos: vector&lt;<a href="#0x1_BlockReward_RewardInfo">BlockReward::RewardInfo</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_BlockReward_RewardInfo"></a>

## Struct `RewardInfo`



<pre><code><b>struct</b> <a href="#0x1_BlockReward_RewardInfo">RewardInfo</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>height: u64</code>
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

<a name="0x1_BlockReward_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_BlockReward_initialize">initialize</a>(account: &signer, reward_balance: u128, reward_delay: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_BlockReward_initialize">initialize</a>(account: &signer, reward_balance: u128, reward_delay: u64) {
    <b>assert</b>(<a href="Timestamp.md#0x1_Timestamp_is_genesis">Timestamp::is_genesis</a>(), 1);
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ACCOUNT">CoreAddresses::GENESIS_ACCOUNT</a>(), 1);
    <b>assert</b>(reward_delay &gt; 0, 4);
    move_to&lt;<a href="#0x1_BlockReward_RewardQueue">RewardQueue</a>&gt;(account, <a href="#0x1_BlockReward_RewardQueue">RewardQueue</a> {
        reward_height: 0,
        reward_delay: reward_delay,
        infos: <a href="Vector.md#0x1_Vector_empty">Vector::empty</a>(),
    });
    <b>let</b> reward_token = <a href="Token.md#0x1_Token_mint">Token::mint</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;(account,  reward_balance);
    move_to&lt;<a href="#0x1_BlockReward">BlockReward</a>&gt;(account, <a href="#0x1_BlockReward">BlockReward</a> {
        balance: reward_token,
    });
}
</code></pre>



</details>

<a name="0x1_BlockReward_withdraw"></a>

## Function `withdraw`



<pre><code><b>fun</b> <a href="#0x1_BlockReward_withdraw">withdraw</a>(amount: u128): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;<a href="STC.md#0x1_STC_STC">STC::STC</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_BlockReward_withdraw">withdraw</a>(amount: u128): <a href="Token.md#0x1_Token">Token</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt; <b>acquires</b> <a href="#0x1_BlockReward">BlockReward</a> {
    <b>let</b> block_reward = borrow_global_mut&lt;<a href="#0x1_BlockReward">BlockReward</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ACCOUNT">CoreAddresses::GENESIS_ACCOUNT</a>());
    <b>let</b> real_amount = <b>if</b> (<a href="Token.md#0x1_Token_value">Token::value</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;(&block_reward.balance) &lt; amount) {
        <a href="Token.md#0x1_Token_value">Token::value</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;(&block_reward.balance)
    } <b>else</b> {
        amount
    };
    <a href="Token.md#0x1_Token_withdraw">Token::withdraw</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;(&<b>mut</b> block_reward.balance, real_amount)
}
</code></pre>



</details>

<a name="0x1_BlockReward_process_block_reward"></a>

## Function `process_block_reward`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_BlockReward_process_block_reward">process_block_reward</a>(account: &signer, current_height: u64, current_reward: u128, current_author: address, auth_key_prefix: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_BlockReward_process_block_reward">process_block_reward</a>(account: &signer, current_height: u64, current_reward: u128,
    current_author: address, auth_key_prefix: vector&lt;u8&gt;) <b>acquires</b> <a href="#0x1_BlockReward_RewardQueue">RewardQueue</a>, <a href="#0x1_BlockReward">BlockReward</a> {
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ACCOUNT">CoreAddresses::GENESIS_ACCOUNT</a>(), 1);

    <b>if</b> (current_height &gt; 0) {
        <b>let</b> rewards = borrow_global_mut&lt;<a href="#0x1_BlockReward_RewardQueue">RewardQueue</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ACCOUNT">CoreAddresses::GENESIS_ACCOUNT</a>());
        <b>let</b> len = <a href="Vector.md#0x1_Vector_length">Vector::length</a>(&rewards.infos);
        <b>assert</b>((current_height == (rewards.reward_height + len + 1)), 6002);
        <b>assert</b>(len &lt;= rewards.reward_delay, 6003);

        <b>if</b> (len == rewards.reward_delay) {//pay and remove
            <b>let</b> reward_height = *&rewards.reward_height + 1;
            <b>let</b> first_info = *<a href="Vector.md#0x1_Vector_borrow">Vector::borrow</a>(&rewards.infos, 0);
            <b>assert</b>((reward_height == first_info.height), 6005);

            rewards.reward_height = reward_height;
            <b>if</b> (first_info.reward &gt; 0) {
                <b>assert</b>(<a href="Account.md#0x1_Account_exists_at">Account::exists_at</a>(first_info.miner), 6006);
                <b>let</b> reward = <a href="#0x1_BlockReward_withdraw">Self::withdraw</a>(first_info.reward);
                <a href="Account.md#0x1_Account_deposit">Account::deposit</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;(account, first_info.miner, reward);
            };
            <a href="Vector.md#0x1_Vector_remove">Vector::remove</a>(&<b>mut</b> rewards.infos, 0);
        };

        <b>if</b> (!<a href="Account.md#0x1_Account_exists_at">Account::exists_at</a>(current_author)) {
            <b>assert</b>(!<a href="Vector.md#0x1_Vector_is_empty">Vector::is_empty</a>(&auth_key_prefix), 6007);
            <a href="Account.md#0x1_Account_create_account">Account::create_account</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;(current_author, auth_key_prefix);
        };
        <b>let</b> current_info = <a href="#0x1_BlockReward_RewardInfo">RewardInfo</a> {
            height: current_height,
            reward: current_reward,
            miner: current_author,
        };
        <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> rewards.infos, current_info);
    };
}
</code></pre>



</details>
