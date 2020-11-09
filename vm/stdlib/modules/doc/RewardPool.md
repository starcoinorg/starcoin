
<a name="0x1_RewardPool"></a>

# Module `0x1::RewardPool`



-  [Resource `T`](#0x1_RewardPool_T)
-  [Resource `Stakings`](#0x1_RewardPool_Stakings)
-  [Struct `Staking`](#0x1_RewardPool_Staking)
-  [Function `initialize`](#0x1_RewardPool_initialize)
-  [Function `notify_rewards`](#0x1_RewardPool_notify_rewards)
-  [Function `total_staked_shares`](#0x1_RewardPool_total_staked_shares)
-  [Function `duration`](#0x1_RewardPool_duration)
-  [Function `period_finish`](#0x1_RewardPool_period_finish)
-  [Function `reward_rate`](#0x1_RewardPool_reward_rate)
-  [Function `reward_per_token`](#0x1_RewardPool_reward_per_token)
-  [Function `enter_pool`](#0x1_RewardPool_enter_pool)
-  [Function `stake`](#0x1_RewardPool_stake)
-  [Function `unstake`](#0x1_RewardPool_unstake)
-  [Function `withdraw_rewards`](#0x1_RewardPool_withdraw_rewards)
-  [Function `exit`](#0x1_RewardPool_exit)
-  [Function `earned`](#0x1_RewardPool_earned)
-  [Function `staked_share`](#0x1_RewardPool_staked_share)
-  [Function `_stake`](#0x1_RewardPool__stake)
-  [Function `_unstake`](#0x1_RewardPool__unstake)
-  [Function `_withdraw_rewards`](#0x1_RewardPool__withdraw_rewards)
-  [Function `_exit`](#0x1_RewardPool__exit)
-  [Function `locate_staking`](#0x1_RewardPool_locate_staking)
-  [Function `_update_reward`](#0x1_RewardPool__update_reward)
-  [Function `_earned`](#0x1_RewardPool__earned)
-  [Function `_reward_per_token`](#0x1_RewardPool__reward_per_token)
-  [Function `_last_time_reward_applicable`](#0x1_RewardPool__last_time_reward_applicable)


<pre><code><b>use</b> <a href="Block.md#0x1_Block">0x1::Block</a>;
<b>use</b> <a href="Option.md#0x1_Option">0x1::Option</a>;
<b>use</b> <a href="Signer.md#0x1_Signer">0x1::Signer</a>;
<b>use</b> <a href="Token.md#0x1_Token">0x1::Token</a>;
<b>use</b> <a href="Vector.md#0x1_Vector">0x1::Vector</a>;
</code></pre>



<a name="0x1_RewardPool_T"></a>

## Resource `T`

Pool data


<pre><code><b>resource</b> <b>struct</b> <a href="RewardPool.md#0x1_RewardPool_T">T</a>&lt;StakeToken, RewardToken&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>remaining_rewards: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;RewardToken&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>duration: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>period_finish: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>reward_rate: u128</code>
</dt>
<dd>

</dd>
<dt>
<code>last_update_time: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>reward_per_token_stored: u128</code>
</dt>
<dd>

</dd>
<dt>
<code>stakes: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;StakeToken&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_RewardPool_Stakings"></a>

## Resource `Stakings`



<pre><code><b>resource</b> <b>struct</b> <a href="RewardPool.md#0x1_RewardPool_Stakings">Stakings</a>&lt;StakeToken, RewardToken&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>staked: vector&lt;<a href="RewardPool.md#0x1_RewardPool_Staking">RewardPool::Staking</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_RewardPool_Staking"></a>

## Struct `Staking`



<pre><code><b>struct</b> <a href="RewardPool.md#0x1_RewardPool_Staking">Staking</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>pool_address: address</code>
</dt>
<dd>

</dd>
<dt>
<code>stake: u128</code>
</dt>
<dd>

</dd>
<dt>
<code>reward_per_token_paid: u128</code>
</dt>
<dd>

</dd>
<dt>
<code>rewards: u128</code>
</dt>
<dd>
 record current rewards to be withdrawed by user.
</dd>
</dl>


</details>

<a name="0x1_RewardPool_initialize"></a>

## Function `initialize`

Called by any one who want to create a reward pool.


<pre><code><b>public</b> <b>fun</b> <a href="RewardPool.md#0x1_RewardPool_initialize">initialize</a>&lt;StakeToken, RewardToken&gt;(signer: &signer, rewards: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;RewardToken&gt;, duration: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="RewardPool.md#0x1_RewardPool_initialize">initialize</a>&lt;StakeToken, RewardToken&gt;(
    signer: &signer,
    rewards: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;RewardToken&gt;,
    duration: u64,
) <b>acquires</b> <a href="RewardPool.md#0x1_RewardPool_T">T</a> {
    // init the pool
    <b>let</b> pool = <a href="RewardPool.md#0x1_RewardPool_T">T</a>&lt;StakeToken, RewardToken&gt; {
        remaining_rewards: <a href="Token.md#0x1_Token_zero">Token::zero</a>&lt;RewardToken&gt;(),
        duration,
        period_finish: 0,
        reward_rate: 0,
        last_update_time: 0,
        reward_per_token_stored: 0,
        stakes: <a href="Token.md#0x1_Token_zero">Token::zero</a>(),
    };
    move_to(signer, pool);
    // dispatch the reward <b>to</b> pool
    <a href="RewardPool.md#0x1_RewardPool_notify_rewards">notify_rewards</a>&lt;StakeToken, RewardToken&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer), rewards);
}
</code></pre>



</details>

<a name="0x1_RewardPool_notify_rewards"></a>

## Function `notify_rewards`



<pre><code><b>fun</b> <a href="RewardPool.md#0x1_RewardPool_notify_rewards">notify_rewards</a>&lt;StakeToken, RewardToken&gt;(pool_address: address, rewards: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;RewardToken&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="RewardPool.md#0x1_RewardPool_notify_rewards">notify_rewards</a>&lt;StakeToken, RewardToken&gt;(
    pool_address: address,
    rewards: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;RewardToken&gt;,
) <b>acquires</b> <a href="RewardPool.md#0x1_RewardPool_T">T</a> {
    <b>let</b> pool = borrow_global_mut&lt;<a href="RewardPool.md#0x1_RewardPool_T">T</a>&lt;StakeToken, RewardToken&gt;&gt;(pool_address);
    <b>let</b> reward_share = <a href="Token.md#0x1_Token_value">Token::value</a>&lt;RewardToken&gt;(&rewards);
    <b>let</b> block_number = <a href="Block.md#0x1_Block_get_current_block_number">Block::get_current_block_number</a>();
    <b>let</b> new_reward_rate = <b>if</b> (block_number &gt;= pool.period_finish) {
        reward_share / (pool.duration <b>as</b> u128)
    } <b>else</b> {
        <b>let</b> leftover = (pool.period_finish - block_number <b>as</b> u128) * pool.reward_rate;
        reward_share + leftover / (pool.duration <b>as</b> u128)
    };
    pool.reward_rate = new_reward_rate;
    pool.last_update_time = block_number;
    pool.period_finish = block_number + pool.duration;
    <a href="Token.md#0x1_Token_deposit">Token::deposit</a>(&<b>mut</b> pool.remaining_rewards, rewards);
}
</code></pre>



</details>

<a name="0x1_RewardPool_total_staked_shares"></a>

## Function `total_staked_shares`



<pre><code><b>public</b> <b>fun</b> <a href="RewardPool.md#0x1_RewardPool_total_staked_shares">total_staked_shares</a>&lt;StakeToken, RewardToken&gt;(pool: address): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="RewardPool.md#0x1_RewardPool_total_staked_shares">total_staked_shares</a>&lt;StakeToken, RewardToken&gt;(pool: address): u128 <b>acquires</b> <a href="RewardPool.md#0x1_RewardPool_T">T</a> {
    <b>let</b> pool = borrow_global&lt;<a href="RewardPool.md#0x1_RewardPool_T">T</a>&lt;StakeToken, RewardToken&gt;&gt;(pool);
    <a href="Token.md#0x1_Token_value">Token::value</a>(&pool.stakes)
}
</code></pre>



</details>

<a name="0x1_RewardPool_duration"></a>

## Function `duration`



<pre><code><b>public</b> <b>fun</b> <a href="RewardPool.md#0x1_RewardPool_duration">duration</a>&lt;StakeToken, RewardToken&gt;(pool: address): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="RewardPool.md#0x1_RewardPool_duration">duration</a>&lt;StakeToken, RewardToken&gt;(pool: address): u64 <b>acquires</b> <a href="RewardPool.md#0x1_RewardPool_T">T</a> {
    <b>let</b> pool = borrow_global&lt;<a href="RewardPool.md#0x1_RewardPool_T">T</a>&lt;StakeToken, RewardToken&gt;&gt;(pool);
    pool.duration
}
</code></pre>



</details>

<a name="0x1_RewardPool_period_finish"></a>

## Function `period_finish`



<pre><code><b>public</b> <b>fun</b> <a href="RewardPool.md#0x1_RewardPool_period_finish">period_finish</a>&lt;StakeToken, RewardToken&gt;(pool: address): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="RewardPool.md#0x1_RewardPool_period_finish">period_finish</a>&lt;StakeToken, RewardToken&gt;(pool: address): u64 <b>acquires</b> <a href="RewardPool.md#0x1_RewardPool_T">T</a> {
    <b>let</b> pool = borrow_global&lt;<a href="RewardPool.md#0x1_RewardPool_T">T</a>&lt;StakeToken, RewardToken&gt;&gt;(pool);
    pool.period_finish
}
</code></pre>



</details>

<a name="0x1_RewardPool_reward_rate"></a>

## Function `reward_rate`



<pre><code><b>public</b> <b>fun</b> <a href="RewardPool.md#0x1_RewardPool_reward_rate">reward_rate</a>&lt;StakeToken, RewardToken&gt;(pool: address): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="RewardPool.md#0x1_RewardPool_reward_rate">reward_rate</a>&lt;StakeToken, RewardToken&gt;(pool: address): u128 <b>acquires</b> <a href="RewardPool.md#0x1_RewardPool_T">T</a> {
    <b>let</b> pool = borrow_global&lt;<a href="RewardPool.md#0x1_RewardPool_T">T</a>&lt;StakeToken, RewardToken&gt;&gt;(pool);
    pool.reward_rate
}
</code></pre>



</details>

<a name="0x1_RewardPool_reward_per_token"></a>

## Function `reward_per_token`



<pre><code><b>public</b> <b>fun</b> <a href="RewardPool.md#0x1_RewardPool_reward_per_token">reward_per_token</a>&lt;StakeToken, RewardToken&gt;(pool: address): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="RewardPool.md#0x1_RewardPool_reward_per_token">reward_per_token</a>&lt;StakeToken, RewardToken&gt;(pool: address): u128 <b>acquires</b> <a href="RewardPool.md#0x1_RewardPool_T">T</a> {
    <b>let</b> pool = borrow_global&lt;<a href="RewardPool.md#0x1_RewardPool_T">T</a>&lt;StakeToken, RewardToken&gt;&gt;(pool);
    <a href="RewardPool.md#0x1_RewardPool__reward_per_token">_reward_per_token</a>(pool)
}
</code></pre>



</details>

<a name="0x1_RewardPool_enter_pool"></a>

## Function `enter_pool`

accept this kind of pool.


<pre><code><b>public</b> <b>fun</b> <a href="RewardPool.md#0x1_RewardPool_enter_pool">enter_pool</a>&lt;StakeToken, RewardToken&gt;(signer: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="RewardPool.md#0x1_RewardPool_enter_pool">enter_pool</a>&lt;StakeToken, RewardToken&gt;(signer: &signer) {
    move_to(signer, <a href="RewardPool.md#0x1_RewardPool_Stakings">Stakings</a>&lt;StakeToken, RewardToken&gt; { staked: <a href="Vector.md#0x1_Vector_empty">Vector::empty</a>() });
}
</code></pre>



</details>

<a name="0x1_RewardPool_stake"></a>

## Function `stake`

Stake some token into rewardpool to earn reward token


<pre><code><b>public</b> <b>fun</b> <a href="RewardPool.md#0x1_RewardPool_stake">stake</a>&lt;StakeToken, RewardToken&gt;(signer: &signer, reward_pool: address, to_stake: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;StakeToken&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="RewardPool.md#0x1_RewardPool_stake">stake</a>&lt;StakeToken, RewardToken&gt;(
    signer: &signer,
    reward_pool: address,
    to_stake: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;StakeToken&gt;,
) <b>acquires</b> <a href="RewardPool.md#0x1_RewardPool_T">T</a>, <a href="RewardPool.md#0x1_RewardPool_Stakings">Stakings</a> {
    <b>assert</b>(<a href="Token.md#0x1_Token_value">Token::value</a>(&to_stake) &gt; 0, 1000);
    <b>if</b> (!<b>exists</b>&lt;<a href="RewardPool.md#0x1_RewardPool_Stakings">Stakings</a>&lt;StakeToken, RewardToken&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer))) {
        <a href="RewardPool.md#0x1_RewardPool_enter_pool">enter_pool</a>&lt;StakeToken, RewardToken&gt;(signer);
    };
    <b>let</b> user_staked = borrow_global_mut&lt;<a href="RewardPool.md#0x1_RewardPool_Stakings">Stakings</a>&lt;StakeToken, RewardToken&gt;&gt;(
        <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer),
    );
    <b>let</b> location = <a href="RewardPool.md#0x1_RewardPool_locate_staking">locate_staking</a>(&user_staked.staked, reward_pool);
    // add a <a href="RewardPool.md#0x1_RewardPool_Staking">Staking</a> record <b>if</b> it's first stake into the this pool.
    <b>let</b> idx;
    <b>if</b> (<a href="Option.md#0x1_Option_is_none">Option::is_none</a>(&location)) {
        <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(
            &<b>mut</b> user_staked.staked,
            <a href="RewardPool.md#0x1_RewardPool_Staking">Staking</a> {
                pool_address: reward_pool,
                stake: 0,
                reward_per_token_paid: 0,
                rewards: 0,
            },
        );
        idx = <a href="Vector.md#0x1_Vector_length">Vector::length</a>(&user_staked.staked) - 1;
    } <b>else</b> {
        idx = <a href="Option.md#0x1_Option_extract">Option::extract</a>(&<b>mut</b> location);
    };
    <b>let</b> staking = <a href="Vector.md#0x1_Vector_borrow_mut">Vector::borrow_mut</a>(&<b>mut</b> user_staked.staked, idx);
    <b>let</b> pool = borrow_global_mut&lt;<a href="RewardPool.md#0x1_RewardPool_T">T</a>&lt;StakeToken, RewardToken&gt;&gt;(reward_pool);
    // <b>update</b> reward first
    <a href="RewardPool.md#0x1_RewardPool__update_reward">_update_reward</a>(pool, staking);
    <a href="RewardPool.md#0x1_RewardPool__stake">_stake</a>(pool, staking, to_stake);
}
</code></pre>



</details>

<a name="0x1_RewardPool_unstake"></a>

## Function `unstake`



<pre><code><b>public</b> <b>fun</b> <a href="RewardPool.md#0x1_RewardPool_unstake">unstake</a>&lt;StakeToken, RewardToken&gt;(signer: &signer, reward_pool: address, share: u128): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;StakeToken&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="RewardPool.md#0x1_RewardPool_unstake">unstake</a>&lt;StakeToken, RewardToken&gt;(
    signer: &signer,
    reward_pool: address,
    share: u128,
): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;StakeToken&gt; <b>acquires</b> <a href="RewardPool.md#0x1_RewardPool_Stakings">Stakings</a>, <a href="RewardPool.md#0x1_RewardPool_T">T</a> {
    <b>let</b> user_staked = borrow_global_mut&lt;<a href="RewardPool.md#0x1_RewardPool_Stakings">Stakings</a>&lt;StakeToken, RewardToken&gt;&gt;(
        <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer),
    );
    <b>let</b> location = {
        <b>let</b> location = <a href="RewardPool.md#0x1_RewardPool_locate_staking">locate_staking</a>(&user_staked.staked, reward_pool);
        <b>assert</b>(<a href="Option.md#0x1_Option_is_some">Option::is_some</a>(&location), 400);
        <a href="Option.md#0x1_Option_extract">Option::extract</a>(&<b>mut</b> location)
    };
    <b>let</b> staking = <a href="Vector.md#0x1_Vector_borrow_mut">Vector::borrow_mut</a>(&<b>mut</b> user_staked.staked, location);
    <b>let</b> pool = borrow_global_mut&lt;<a href="RewardPool.md#0x1_RewardPool_T">T</a>&lt;StakeToken, RewardToken&gt;&gt;(reward_pool);
    <a href="RewardPool.md#0x1_RewardPool__update_reward">_update_reward</a>(pool, staking);
    <a href="RewardPool.md#0x1_RewardPool__unstake">_unstake</a>(pool, staking, share)
}
</code></pre>



</details>

<a name="0x1_RewardPool_withdraw_rewards"></a>

## Function `withdraw_rewards`

WithddrawRewards withdraw all earned token.


<pre><code><b>public</b> <b>fun</b> <a href="RewardPool.md#0x1_RewardPool_withdraw_rewards">withdraw_rewards</a>&lt;StakeToken, RewardToken&gt;(signer: &signer, reward_pool: address): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;RewardToken&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="RewardPool.md#0x1_RewardPool_withdraw_rewards">withdraw_rewards</a>&lt;StakeToken, RewardToken&gt;(
    signer: &signer,
    reward_pool: address,
): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;RewardToken&gt; <b>acquires</b> <a href="RewardPool.md#0x1_RewardPool_Stakings">Stakings</a>, <a href="RewardPool.md#0x1_RewardPool_T">T</a> {
    <b>let</b> user_staked = borrow_global_mut&lt;<a href="RewardPool.md#0x1_RewardPool_Stakings">Stakings</a>&lt;StakeToken, RewardToken&gt;&gt;(
        <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer),
    );
    <b>let</b> location = {
        <b>let</b> location = <a href="RewardPool.md#0x1_RewardPool_locate_staking">locate_staking</a>(&user_staked.staked, reward_pool);
        <b>assert</b>(<a href="Option.md#0x1_Option_is_some">Option::is_some</a>(&location), 400);
        <a href="Option.md#0x1_Option_extract">Option::extract</a>(&<b>mut</b> location)
    };
    <b>let</b> staking = <a href="Vector.md#0x1_Vector_borrow_mut">Vector::borrow_mut</a>(&<b>mut</b> user_staked.staked, location);
    <b>let</b> pool = borrow_global_mut&lt;<a href="RewardPool.md#0x1_RewardPool_T">T</a>&lt;StakeToken, RewardToken&gt;&gt;(reward_pool);
    <a href="RewardPool.md#0x1_RewardPool__update_reward">_update_reward</a>(pool, staking);
    <a href="RewardPool.md#0x1_RewardPool__withdraw_rewards">_withdraw_rewards</a>(pool, staking)
}
</code></pre>



</details>

<a name="0x1_RewardPool_exit"></a>

## Function `exit`



<pre><code><b>public</b> <b>fun</b> <a href="RewardPool.md#0x1_RewardPool_exit">exit</a>&lt;StakeToken, RewardToken&gt;(signer: &signer, reward_pool: address): (<a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;StakeToken&gt;, <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;RewardToken&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="RewardPool.md#0x1_RewardPool_exit">exit</a>&lt;StakeToken, RewardToken&gt;(
    signer: &signer,
    reward_pool: address,
): (<a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;StakeToken&gt;, <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;RewardToken&gt;) <b>acquires</b> <a href="RewardPool.md#0x1_RewardPool_Stakings">Stakings</a>, <a href="RewardPool.md#0x1_RewardPool_T">T</a> {
    <b>let</b> user_staked = borrow_global_mut&lt;<a href="RewardPool.md#0x1_RewardPool_Stakings">Stakings</a>&lt;StakeToken, RewardToken&gt;&gt;(
        <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer),
    );
    <b>let</b> location = {
        <b>let</b> location = <a href="RewardPool.md#0x1_RewardPool_locate_staking">locate_staking</a>(&user_staked.staked, reward_pool);
        <b>assert</b>(<a href="Option.md#0x1_Option_is_some">Option::is_some</a>(&location), 400);
        <a href="Option.md#0x1_Option_extract">Option::extract</a>(&<b>mut</b> location)
    };
    <b>let</b> staking = <a href="Vector.md#0x1_Vector_swap_remove">Vector::swap_remove</a>(&<b>mut</b> user_staked.staked, location);
    <b>let</b> pool = borrow_global_mut&lt;<a href="RewardPool.md#0x1_RewardPool_T">T</a>&lt;StakeToken, RewardToken&gt;&gt;(reward_pool);
    <a href="RewardPool.md#0x1_RewardPool__update_reward">_update_reward</a>(pool, &<b>mut</b> staking);
    <b>let</b> (s, r) = <a href="RewardPool.md#0x1_RewardPool__exit">_exit</a>(pool, &<b>mut</b> staking);
    (s, r)
}
</code></pre>



</details>

<a name="0x1_RewardPool_earned"></a>

## Function `earned`

Calculate reward earned.


<pre><code><b>public</b> <b>fun</b> <a href="RewardPool.md#0x1_RewardPool_earned">earned</a>&lt;StakeToken, RewardToken&gt;(account: address, reward_pool: address): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="RewardPool.md#0x1_RewardPool_earned">earned</a>&lt;StakeToken, RewardToken&gt;(account: address, reward_pool: address): u128
<b>acquires</b> <a href="RewardPool.md#0x1_RewardPool_T">T</a>, <a href="RewardPool.md#0x1_RewardPool_Stakings">Stakings</a> {
    <b>let</b> user_staked = borrow_global&lt;<a href="RewardPool.md#0x1_RewardPool_Stakings">Stakings</a>&lt;StakeToken, RewardToken&gt;&gt;(account);
    <b>let</b> location = {
        <b>let</b> location = <a href="RewardPool.md#0x1_RewardPool_locate_staking">locate_staking</a>(&user_staked.staked, reward_pool);
        <b>assert</b>(<a href="Option.md#0x1_Option_is_some">Option::is_some</a>(&location), 400);
        <a href="Option.md#0x1_Option_extract">Option::extract</a>(&<b>mut</b> location)
    };
    <b>let</b> staking = <a href="Vector.md#0x1_Vector_borrow">Vector::borrow</a>(&user_staked.staked, location);
    <b>let</b> pool = borrow_global&lt;<a href="RewardPool.md#0x1_RewardPool_T">T</a>&lt;StakeToken, RewardToken&gt;&gt;(reward_pool);
    <a href="RewardPool.md#0x1_RewardPool__earned">_earned</a>&lt;StakeToken, RewardToken&gt;(staking, pool)
}
</code></pre>



</details>

<a name="0x1_RewardPool_staked_share"></a>

## Function `staked_share`



<pre><code><b>public</b> <b>fun</b> <a href="RewardPool.md#0x1_RewardPool_staked_share">staked_share</a>&lt;StakeToken, RewardToken&gt;(account: address, reward_pool: address): <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;u128&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="RewardPool.md#0x1_RewardPool_staked_share">staked_share</a>&lt;StakeToken, RewardToken&gt;(
    account: address,
    reward_pool: address,
): <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;u128&gt; <b>acquires</b> <a href="RewardPool.md#0x1_RewardPool_Stakings">Stakings</a> {
    <b>let</b> staked = borrow_global&lt;<a href="RewardPool.md#0x1_RewardPool_Stakings">Stakings</a>&lt;StakeToken, RewardToken&gt;&gt;(account);
    <b>let</b> location = <a href="RewardPool.md#0x1_RewardPool_locate_staking">locate_staking</a>(&staked.staked, reward_pool);
    <b>if</b> (<a href="Option.md#0x1_Option_is_some">Option::is_some</a>(&location)) {
        <b>let</b> staking = <a href="Vector.md#0x1_Vector_borrow">Vector::borrow</a>(&staked.staked, <a href="Option.md#0x1_Option_extract">Option::extract</a>(&<b>mut</b> location));
        <a href="Option.md#0x1_Option_some">Option::some</a>(staking.stake)
    } <b>else</b> {
        <a href="Option.md#0x1_Option_none">Option::none</a>()
    }
}
</code></pre>



</details>

<a name="0x1_RewardPool__stake"></a>

## Function `_stake`

internal function of Stake action, caller should update_reward first.


<pre><code><b>fun</b> <a href="RewardPool.md#0x1_RewardPool__stake">_stake</a>&lt;StakeToken, RewardToken&gt;(pool: &<b>mut</b> <a href="RewardPool.md#0x1_RewardPool_T">RewardPool::T</a>&lt;StakeToken, RewardToken&gt;, staking: &<b>mut</b> <a href="RewardPool.md#0x1_RewardPool_Staking">RewardPool::Staking</a>, to_stake: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;StakeToken&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="RewardPool.md#0x1_RewardPool__stake">_stake</a>&lt;StakeToken, RewardToken&gt;(
    pool: &<b>mut</b> <a href="RewardPool.md#0x1_RewardPool_T">T</a>&lt;StakeToken, RewardToken&gt;,
    staking: &<b>mut</b> <a href="RewardPool.md#0x1_RewardPool_Staking">Staking</a>,
    to_stake: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;StakeToken&gt;,
) {
    // <b>update</b> user's stake info and <b>move</b> the staking token <b>to</b> pool.
    staking.stake = staking.stake + <a href="Token.md#0x1_Token_value">Token::value</a>(&to_stake);
    <a href="Token.md#0x1_Token_deposit">Token::deposit</a>(&<b>mut</b> pool.stakes, to_stake);
}
</code></pre>



</details>

<a name="0x1_RewardPool__unstake"></a>

## Function `_unstake`

internal function of Unstake action, caller should update_reward first.


<pre><code><b>fun</b> <a href="RewardPool.md#0x1_RewardPool__unstake">_unstake</a>&lt;StakeToken, RewardToken&gt;(pool: &<b>mut</b> <a href="RewardPool.md#0x1_RewardPool_T">RewardPool::T</a>&lt;StakeToken, RewardToken&gt;, staking: &<b>mut</b> <a href="RewardPool.md#0x1_RewardPool_Staking">RewardPool::Staking</a>, share: u128): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;StakeToken&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="RewardPool.md#0x1_RewardPool__unstake">_unstake</a>&lt;StakeToken, RewardToken&gt;(
    pool: &<b>mut</b> <a href="RewardPool.md#0x1_RewardPool_T">T</a>&lt;StakeToken, RewardToken&gt;,
    staking: &<b>mut</b> <a href="RewardPool.md#0x1_RewardPool_Staking">Staking</a>,
    share: u128,
): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;StakeToken&gt; {
    staking.stake = staking.stake - share;
    <a href="Token.md#0x1_Token_withdraw">Token::withdraw</a>(&<b>mut</b> pool.stakes, share)
}
</code></pre>



</details>

<a name="0x1_RewardPool__withdraw_rewards"></a>

## Function `_withdraw_rewards`

internal function of WithdrawReward action, caller should update_reward first.


<pre><code><b>fun</b> <a href="RewardPool.md#0x1_RewardPool__withdraw_rewards">_withdraw_rewards</a>&lt;StakeToken, RewardToken&gt;(pool: &<b>mut</b> <a href="RewardPool.md#0x1_RewardPool_T">RewardPool::T</a>&lt;StakeToken, RewardToken&gt;, staking: &<b>mut</b> <a href="RewardPool.md#0x1_RewardPool_Staking">RewardPool::Staking</a>): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;RewardToken&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="RewardPool.md#0x1_RewardPool__withdraw_rewards">_withdraw_rewards</a>&lt;StakeToken, RewardToken&gt;(
    pool: &<b>mut</b> <a href="RewardPool.md#0x1_RewardPool_T">T</a>&lt;StakeToken, RewardToken&gt;,
    staking: &<b>mut</b> <a href="RewardPool.md#0x1_RewardPool_Staking">Staking</a>,
): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;RewardToken&gt; {
    <b>let</b> my_share = staking.rewards;
    <b>if</b> (my_share == 0) {
        <b>return</b> <a href="Token.md#0x1_Token_zero">Token::zero</a>()
    };
    staking.rewards = 0;
    <a href="Token.md#0x1_Token_withdraw">Token::withdraw</a>(&<b>mut</b> pool.remaining_rewards, my_share)
}
</code></pre>



</details>

<a name="0x1_RewardPool__exit"></a>

## Function `_exit`



<pre><code><b>fun</b> <a href="RewardPool.md#0x1_RewardPool__exit">_exit</a>&lt;StakeToken, RewardToken&gt;(pool: &<b>mut</b> <a href="RewardPool.md#0x1_RewardPool_T">RewardPool::T</a>&lt;StakeToken, RewardToken&gt;, staking: &<b>mut</b> <a href="RewardPool.md#0x1_RewardPool_Staking">RewardPool::Staking</a>): (<a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;StakeToken&gt;, <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;RewardToken&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="RewardPool.md#0x1_RewardPool__exit">_exit</a>&lt;StakeToken, RewardToken&gt;(
    pool: &<b>mut</b> <a href="RewardPool.md#0x1_RewardPool_T">T</a>&lt;StakeToken, RewardToken&gt;,
    staking: &<b>mut</b> <a href="RewardPool.md#0x1_RewardPool_Staking">Staking</a>,
): (<a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;StakeToken&gt;, <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;RewardToken&gt;) {
    <b>let</b> rewards = <a href="RewardPool.md#0x1_RewardPool__withdraw_rewards">_withdraw_rewards</a>(pool, staking);
    <b>let</b> all_stakes = staking.stake;
    <b>let</b> staked_tokens = <a href="RewardPool.md#0x1_RewardPool__unstake">_unstake</a>(pool, staking, all_stakes);
    (staked_tokens, rewards)
}
</code></pre>



</details>

<a name="0x1_RewardPool_locate_staking"></a>

## Function `locate_staking`



<pre><code><b>fun</b> <a href="RewardPool.md#0x1_RewardPool_locate_staking">locate_staking</a>(staked: &vector&lt;<a href="RewardPool.md#0x1_RewardPool_Staking">RewardPool::Staking</a>&gt;, pool_address: address): <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;u64&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="RewardPool.md#0x1_RewardPool_locate_staking">locate_staking</a>(staked: &vector&lt;<a href="RewardPool.md#0x1_RewardPool_Staking">Staking</a>&gt;, pool_address: address): <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;u64&gt; {
    <b>let</b> stake_len = <a href="Vector.md#0x1_Vector_length">Vector::length</a>(staked);
    <b>let</b> i = 0;
    <b>while</b> (i &lt; stake_len){
        <b>let</b> staking = <a href="Vector.md#0x1_Vector_borrow">Vector::borrow</a>(staked, i);
        <b>if</b> (staking.pool_address == pool_address) {
            <b>return</b> <a href="Option.md#0x1_Option_some">Option::some</a>(i)
        };
        i = i + 1;
    };
    <a href="Option.md#0x1_Option_none">Option::none</a>()
}
</code></pre>



</details>

<a name="0x1_RewardPool__update_reward"></a>

## Function `_update_reward`



<pre><code><b>fun</b> <a href="RewardPool.md#0x1_RewardPool__update_reward">_update_reward</a>&lt;StakeToken, RewardToken&gt;(reward_pool: &<b>mut</b> <a href="RewardPool.md#0x1_RewardPool_T">RewardPool::T</a>&lt;StakeToken, RewardToken&gt;, user_stake: &<b>mut</b> <a href="RewardPool.md#0x1_RewardPool_Staking">RewardPool::Staking</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="RewardPool.md#0x1_RewardPool__update_reward">_update_reward</a>&lt;StakeToken, RewardToken&gt;(
    reward_pool: &<b>mut</b> <a href="RewardPool.md#0x1_RewardPool_T">T</a>&lt;StakeToken, RewardToken&gt;,
    user_stake: &<b>mut</b> <a href="RewardPool.md#0x1_RewardPool_Staking">Staking</a>,
) {
    // <b>assert</b>(user_stake.pool_address == reward_pool, 400);
    // <b>update</b> reward pool first.
    <b>let</b> reward_per_token = <a href="RewardPool.md#0x1_RewardPool__reward_per_token">_reward_per_token</a>&lt;StakeToken, RewardToken&gt;(reward_pool);
    <b>let</b> last_time_reward_applicable = <a href="RewardPool.md#0x1_RewardPool__last_time_reward_applicable">_last_time_reward_applicable</a>&lt;StakeToken, RewardToken&gt;(
        reward_pool,
    );
    reward_pool.reward_per_token_stored = reward_per_token;
    reward_pool.last_update_time = last_time_reward_applicable;
    // then <b>update</b> user reward info.
    <b>let</b> earned = <a href="RewardPool.md#0x1_RewardPool__earned">_earned</a>&lt;StakeToken, RewardToken&gt;(user_stake, reward_pool);
    user_stake.rewards = earned;
    user_stake.reward_per_token_paid = reward_per_token;
}
</code></pre>



</details>

<a name="0x1_RewardPool__earned"></a>

## Function `_earned`



<pre><code><b>fun</b> <a href="RewardPool.md#0x1_RewardPool__earned">_earned</a>&lt;StakeToken, RewardToken&gt;(user_stake: &<a href="RewardPool.md#0x1_RewardPool_Staking">RewardPool::Staking</a>, reward_pool: &<a href="RewardPool.md#0x1_RewardPool_T">RewardPool::T</a>&lt;StakeToken, RewardToken&gt;): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="RewardPool.md#0x1_RewardPool__earned">_earned</a>&lt;StakeToken, RewardToken&gt;(
    user_stake: &<a href="RewardPool.md#0x1_RewardPool_Staking">Staking</a>,
    reward_pool: &<a href="RewardPool.md#0x1_RewardPool_T">T</a>&lt;StakeToken, RewardToken&gt;,
): u128 {
    <b>let</b> reward_per_token = <a href="RewardPool.md#0x1_RewardPool__reward_per_token">_reward_per_token</a>&lt;StakeToken, RewardToken&gt;(reward_pool);
    user_stake.rewards +
        user_stake.stake * (reward_per_token - user_stake.reward_per_token_paid)
}
</code></pre>



</details>

<a name="0x1_RewardPool__reward_per_token"></a>

## Function `_reward_per_token`



<pre><code><b>fun</b> <a href="RewardPool.md#0x1_RewardPool__reward_per_token">_reward_per_token</a>&lt;StakeToken, RewardToken&gt;(pool: &<a href="RewardPool.md#0x1_RewardPool_T">RewardPool::T</a>&lt;StakeToken, RewardToken&gt;): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="RewardPool.md#0x1_RewardPool__reward_per_token">_reward_per_token</a>&lt;StakeToken, RewardToken&gt;(pool: &<a href="RewardPool.md#0x1_RewardPool_T">T</a>&lt;StakeToken, RewardToken&gt;): u128 {
    <b>let</b> total_staked = <a href="Token.md#0x1_Token_value">Token::value</a>(&pool.stakes);
    <b>if</b> (total_staked == 0) {
        pool.reward_per_token_stored
    } <b>else</b> {
        <b>let</b> duration_from_last_time = <a href="RewardPool.md#0x1_RewardPool__last_time_reward_applicable">_last_time_reward_applicable</a>&lt;StakeToken, RewardToken&gt;(
            pool,
        ) - pool.last_update_time;
        <b>let</b> reword_per_token_from_last = (duration_from_last_time <b>as</b> u128) * pool.reward_rate /
            total_staked;
        pool.reward_per_token_stored + reword_per_token_from_last
    }
}
</code></pre>



</details>

<a name="0x1_RewardPool__last_time_reward_applicable"></a>

## Function `_last_time_reward_applicable`



<pre><code><b>public</b> <b>fun</b> <a href="RewardPool.md#0x1_RewardPool__last_time_reward_applicable">_last_time_reward_applicable</a>&lt;StakeToken, RewardToken&gt;(pool: &<a href="RewardPool.md#0x1_RewardPool_T">RewardPool::T</a>&lt;StakeToken, RewardToken&gt;): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="RewardPool.md#0x1_RewardPool__last_time_reward_applicable">_last_time_reward_applicable</a>&lt;StakeToken, RewardToken&gt;(
    pool: &<a href="RewardPool.md#0x1_RewardPool_T">T</a>&lt;StakeToken, RewardToken&gt;,
): u64 {
    <b>let</b> block_number = <a href="Block.md#0x1_Block_get_current_block_number">Block::get_current_block_number</a>();
    <b>if</b> (block_number &gt; pool.period_finish) {
        pool.period_finish
    } <b>else</b> {
        block_number
    }
}
</code></pre>



</details>
