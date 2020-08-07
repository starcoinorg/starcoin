
<a name="0x1_BlockReward"></a>

# Module `0x1::BlockReward`

### Table of Contents

-  [Resource `RewardQueue`](#0x1_BlockReward_RewardQueue)
-  [Struct `RewardInfo`](#0x1_BlockReward_RewardInfo)
-  [Function `AUTH_KEY_PREFIX_IS_NOT_EMPTY`](#0x1_BlockReward_AUTH_KEY_PREFIX_IS_NOT_EMPTY)
-  [Function `CURRENT_NUMBER_IS_WRONG`](#0x1_BlockReward_CURRENT_NUMBER_IS_WRONG)
-  [Function `LEN_OF_REWARD_INFO_IS_WRONG`](#0x1_BlockReward_LEN_OF_REWARD_INFO_IS_WRONG)
-  [Function `REWARD_NUMBER_IS_WRONG`](#0x1_BlockReward_REWARD_NUMBER_IS_WRONG)
-  [Function `MINER_EXIST`](#0x1_BlockReward_MINER_EXIST)
-  [Function `initialize`](#0x1_BlockReward_initialize)
-  [Function `process_block_reward`](#0x1_BlockReward_process_block_reward)



<a name="0x1_BlockReward_RewardQueue"></a>

## Resource `RewardQueue`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_BlockReward_RewardQueue">RewardQueue</a>
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

<a name="0x1_BlockReward_AUTH_KEY_PREFIX_IS_NOT_EMPTY"></a>

## Function `AUTH_KEY_PREFIX_IS_NOT_EMPTY`



<pre><code><b>fun</b> <a href="#0x1_BlockReward_AUTH_KEY_PREFIX_IS_NOT_EMPTY">AUTH_KEY_PREFIX_IS_NOT_EMPTY</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_BlockReward_AUTH_KEY_PREFIX_IS_NOT_EMPTY">AUTH_KEY_PREFIX_IS_NOT_EMPTY</a>(): u64 { <a href="ErrorCode.md#0x1_ErrorCode_ECODE_BASE">ErrorCode::ECODE_BASE</a>() + 1}
</code></pre>



</details>

<a name="0x1_BlockReward_CURRENT_NUMBER_IS_WRONG"></a>

## Function `CURRENT_NUMBER_IS_WRONG`



<pre><code><b>fun</b> <a href="#0x1_BlockReward_CURRENT_NUMBER_IS_WRONG">CURRENT_NUMBER_IS_WRONG</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_BlockReward_CURRENT_NUMBER_IS_WRONG">CURRENT_NUMBER_IS_WRONG</a>(): u64 { <a href="ErrorCode.md#0x1_ErrorCode_ECODE_BASE">ErrorCode::ECODE_BASE</a>() + 2}
</code></pre>



</details>

<a name="0x1_BlockReward_LEN_OF_REWARD_INFO_IS_WRONG"></a>

## Function `LEN_OF_REWARD_INFO_IS_WRONG`



<pre><code><b>fun</b> <a href="#0x1_BlockReward_LEN_OF_REWARD_INFO_IS_WRONG">LEN_OF_REWARD_INFO_IS_WRONG</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_BlockReward_LEN_OF_REWARD_INFO_IS_WRONG">LEN_OF_REWARD_INFO_IS_WRONG</a>(): u64 { <a href="ErrorCode.md#0x1_ErrorCode_ECODE_BASE">ErrorCode::ECODE_BASE</a>() + 3}
</code></pre>



</details>

<a name="0x1_BlockReward_REWARD_NUMBER_IS_WRONG"></a>

## Function `REWARD_NUMBER_IS_WRONG`



<pre><code><b>fun</b> <a href="#0x1_BlockReward_REWARD_NUMBER_IS_WRONG">REWARD_NUMBER_IS_WRONG</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_BlockReward_REWARD_NUMBER_IS_WRONG">REWARD_NUMBER_IS_WRONG</a>(): u64 { <a href="ErrorCode.md#0x1_ErrorCode_ECODE_BASE">ErrorCode::ECODE_BASE</a>() + 4}
</code></pre>



</details>

<a name="0x1_BlockReward_MINER_EXIST"></a>

## Function `MINER_EXIST`



<pre><code><b>fun</b> <a href="#0x1_BlockReward_MINER_EXIST">MINER_EXIST</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_BlockReward_MINER_EXIST">MINER_EXIST</a>(): u64 { <a href="ErrorCode.md#0x1_ErrorCode_ECODE_BASE">ErrorCode::ECODE_BASE</a>() + 5}
</code></pre>



</details>

<a name="0x1_BlockReward_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_BlockReward_initialize">initialize</a>(account: &signer, reward_delay: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_BlockReward_initialize">initialize</a>(account: &signer, reward_delay: u64) {
    <b>assert</b>(<a href="Timestamp.md#0x1_Timestamp_is_genesis">Timestamp::is_genesis</a>(), <a href="ErrorCode.md#0x1_ErrorCode_ENOT_GENESIS">ErrorCode::ENOT_GENESIS</a>());
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>(), <a href="ErrorCode.md#0x1_ErrorCode_ENOT_GENESIS_ACCOUNT">ErrorCode::ENOT_GENESIS_ACCOUNT</a>());
    <b>assert</b>(reward_delay &gt; 0, <a href="ErrorCode.md#0x1_ErrorCode_EINVALID_ARGUMENT">ErrorCode::EINVALID_ARGUMENT</a>());
    move_to&lt;<a href="#0x1_BlockReward_RewardQueue">RewardQueue</a>&gt;(account, <a href="#0x1_BlockReward_RewardQueue">RewardQueue</a> {
        reward_number: 0,
        reward_delay: reward_delay,
        infos: <a href="Vector.md#0x1_Vector_empty">Vector::empty</a>(),
    });
}
</code></pre>



</details>

<a name="0x1_BlockReward_process_block_reward"></a>

## Function `process_block_reward`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_BlockReward_process_block_reward">process_block_reward</a>(account: &signer, current_number: u64, current_reward: u128, current_author: address, auth_key_prefix: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_BlockReward_process_block_reward">process_block_reward</a>(account: &signer, current_number: u64, current_reward: u128,
    current_author: address, auth_key_prefix: vector&lt;u8&gt;) <b>acquires</b> <a href="#0x1_BlockReward_RewardQueue">RewardQueue</a> {
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>(), <a href="ErrorCode.md#0x1_ErrorCode_ENOT_GENESIS_ACCOUNT">ErrorCode::ENOT_GENESIS_ACCOUNT</a>());

    <b>if</b> (current_number &gt; 0) {
        <b>let</b> rewards = borrow_global_mut&lt;<a href="#0x1_BlockReward_RewardQueue">RewardQueue</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
        <b>let</b> len = <a href="Vector.md#0x1_Vector_length">Vector::length</a>(&rewards.infos);
        <b>assert</b>((current_number == (rewards.reward_number + len + 1)), <a href="#0x1_BlockReward_CURRENT_NUMBER_IS_WRONG">CURRENT_NUMBER_IS_WRONG</a>());
        <b>assert</b>(len &lt;= rewards.reward_delay, <a href="#0x1_BlockReward_LEN_OF_REWARD_INFO_IS_WRONG">LEN_OF_REWARD_INFO_IS_WRONG</a>());

        <b>if</b> (len == rewards.reward_delay) {//pay and remove
            <b>let</b> reward_number = *&rewards.reward_number + 1;
            <b>let</b> first_info = *<a href="Vector.md#0x1_Vector_borrow">Vector::borrow</a>(&rewards.infos, 0);
            <b>assert</b>((reward_number == first_info.number), <a href="#0x1_BlockReward_REWARD_NUMBER_IS_WRONG">REWARD_NUMBER_IS_WRONG</a>());

            rewards.reward_number = reward_number;
            <b>if</b> (first_info.reward &gt; 0) {
                <b>assert</b>(<a href="Account.md#0x1_Account_exists_at">Account::exists_at</a>(first_info.miner), <a href="#0x1_BlockReward_MINER_EXIST">MINER_EXIST</a>());
                <b>let</b> reward = <a href="Token.md#0x1_Token_mint">Token::mint</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;(account, first_info.reward);
                <a href="Account.md#0x1_Account_deposit_to">Account::deposit_to</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;(account, first_info.miner, reward);
            };
            <a href="Vector.md#0x1_Vector_remove">Vector::remove</a>(&<b>mut</b> rewards.infos, 0);
        };

        <b>if</b> (!<a href="Account.md#0x1_Account_exists_at">Account::exists_at</a>(current_author)) {
            <b>assert</b>(!<a href="Vector.md#0x1_Vector_is_empty">Vector::is_empty</a>(&auth_key_prefix), <a href="#0x1_BlockReward_AUTH_KEY_PREFIX_IS_NOT_EMPTY">AUTH_KEY_PREFIX_IS_NOT_EMPTY</a>());
            <a href="Account.md#0x1_Account_create_account">Account::create_account</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;(current_author, auth_key_prefix);
        };
        <b>let</b> current_info = <a href="#0x1_BlockReward_RewardInfo">RewardInfo</a> {
            number: current_number,
            reward: current_reward,
            miner: current_author,
        };
        <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> rewards.infos, current_info);
    };
}
</code></pre>



</details>
