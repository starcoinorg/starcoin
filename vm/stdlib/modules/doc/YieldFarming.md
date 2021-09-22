
<a name="0x1_YieldFarming"></a>

# Module `0x1::YieldFarming`



-  [Resource `Farming`](#0x1_YieldFarming_Farming)
-  [Resource `FarmingAsset`](#0x1_YieldFarming_FarmingAsset)
-  [Resource `ParameterModifyCapability`](#0x1_YieldFarming_ParameterModifyCapability)
-  [Resource `Stake`](#0x1_YieldFarming_Stake)
-  [Struct `Exp`](#0x1_YieldFarming_Exp)
-  [Constants](#@Constants_0)
-  [Function `exp`](#0x1_YieldFarming_exp)
-  [Function `mul_u128`](#0x1_YieldFarming_mul_u128)
-  [Function `div_u128`](#0x1_YieldFarming_div_u128)
-  [Function `truncate`](#0x1_YieldFarming_truncate)
-  [Function `initialize`](#0x1_YieldFarming_initialize)
-  [Function `initialize_asset`](#0x1_YieldFarming_initialize_asset)
-  [Function `modify_parameter`](#0x1_YieldFarming_modify_parameter)
-  [Function `stake`](#0x1_YieldFarming_stake)
-  [Function `unstake`](#0x1_YieldFarming_unstake)
-  [Function `harvest`](#0x1_YieldFarming_harvest)
-  [Function `query_gov_token_amount`](#0x1_YieldFarming_query_gov_token_amount)
-  [Function `query_total_stake`](#0x1_YieldFarming_query_total_stake)
-  [Function `query_stake`](#0x1_YieldFarming_query_stake)
-  [Function `calculate_harvest_index_with_asset`](#0x1_YieldFarming_calculate_harvest_index_with_asset)
-  [Function `calculate_harvest_index_weight_zero`](#0x1_YieldFarming_calculate_harvest_index_weight_zero)
-  [Function `calculate_harvest_index`](#0x1_YieldFarming_calculate_harvest_index)
-  [Function `calculate_withdraw_amount`](#0x1_YieldFarming_calculate_withdraw_amount)
-  [Function `exists_at`](#0x1_YieldFarming_exists_at)
-  [Function `exists_asset_at`](#0x1_YieldFarming_exists_asset_at)
-  [Function `exists_stake_at_address`](#0x1_YieldFarming_exists_stake_at_address)
-  [Specification](#@Specification_1)


<pre><code><b>use</b> <a href="Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="Signer.md#0x1_Signer">0x1::Signer</a>;
<b>use</b> <a href="Timestamp.md#0x1_Timestamp">0x1::Timestamp</a>;
<b>use</b> <a href="Token.md#0x1_Token">0x1::Token</a>;
</code></pre>



<a name="0x1_YieldFarming_Farming"></a>

## Resource `Farming`

The object of yield farming
RewardTokenT meaning token of yield farming


<pre><code><b>struct</b> <a href="YieldFarming.md#0x1_YieldFarming_Farming">Farming</a>&lt;PoolType, RewardTokenT&gt; has store, key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>treasury_token: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;RewardTokenT&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_YieldFarming_FarmingAsset"></a>

## Resource `FarmingAsset`



<pre><code><b>struct</b> <a href="YieldFarming.md#0x1_YieldFarming_FarmingAsset">FarmingAsset</a>&lt;PoolType, AssetT&gt; has store, key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>asset_total_weight: u128</code>
</dt>
<dd>

</dd>
<dt>
<code>harvest_index: u128</code>
</dt>
<dd>

</dd>
<dt>
<code>last_update_timestamp: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>release_per_second: u128</code>
</dt>
<dd>

</dd>
<dt>
<code>start_time: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_YieldFarming_ParameterModifyCapability"></a>

## Resource `ParameterModifyCapability`

Capability to modify parameter such as period and release amount


<pre><code><b>struct</b> <a href="YieldFarming.md#0x1_YieldFarming_ParameterModifyCapability">ParameterModifyCapability</a>&lt;PoolType, AssetT&gt; has store, key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>dummy_field: bool</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_YieldFarming_Stake"></a>

## Resource `Stake`

To store user's asset token


<pre><code><b>struct</b> <a href="YieldFarming.md#0x1_YieldFarming_Stake">Stake</a>&lt;PoolType, AssetT&gt; has store, key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>asset: AssetT</code>
</dt>
<dd>

</dd>
<dt>
<code>asset_weight: u128</code>
</dt>
<dd>

</dd>
<dt>
<code>last_harvest_index: u128</code>
</dt>
<dd>

</dd>
<dt>
<code>gain: u128</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_YieldFarming_Exp"></a>

## Struct `Exp`



<pre><code><b>struct</b> <a href="YieldFarming.md#0x1_YieldFarming_Exp">Exp</a> has <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>mantissa: u128</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_YieldFarming_ERR_EXP_DIVIDE_BY_ZERO"></a>



<pre><code><b>const</b> <a href="YieldFarming.md#0x1_YieldFarming_ERR_EXP_DIVIDE_BY_ZERO">ERR_EXP_DIVIDE_BY_ZERO</a>: u64 = 107;
</code></pre>



<a name="0x1_YieldFarming_ERR_FARMING_BALANCE_EXCEEDED"></a>



<pre><code><b>const</b> <a href="YieldFarming.md#0x1_YieldFarming_ERR_FARMING_BALANCE_EXCEEDED">ERR_FARMING_BALANCE_EXCEEDED</a>: u64 = 108;
</code></pre>



<a name="0x1_YieldFarming_ERR_FARMING_HAVERST_NO_GAIN"></a>



<pre><code><b>const</b> <a href="YieldFarming.md#0x1_YieldFarming_ERR_FARMING_HAVERST_NO_GAIN">ERR_FARMING_HAVERST_NO_GAIN</a>: u64 = 105;
</code></pre>



<a name="0x1_YieldFarming_ERR_FARMING_INIT_REPEATE"></a>



<pre><code><b>const</b> <a href="YieldFarming.md#0x1_YieldFarming_ERR_FARMING_INIT_REPEATE">ERR_FARMING_INIT_REPEATE</a>: u64 = 101;
</code></pre>



<a name="0x1_YieldFarming_ERR_FARMING_NOT_ENOUGH_ASSET"></a>



<pre><code><b>const</b> <a href="YieldFarming.md#0x1_YieldFarming_ERR_FARMING_NOT_ENOUGH_ASSET">ERR_FARMING_NOT_ENOUGH_ASSET</a>: u64 = 109;
</code></pre>



<a name="0x1_YieldFarming_ERR_FARMING_NOT_STILL_FREEZE"></a>



<pre><code><b>const</b> <a href="YieldFarming.md#0x1_YieldFarming_ERR_FARMING_NOT_STILL_FREEZE">ERR_FARMING_NOT_STILL_FREEZE</a>: u64 = 102;
</code></pre>



<a name="0x1_YieldFarming_ERR_FARMING_STAKE_EXISTS"></a>



<pre><code><b>const</b> <a href="YieldFarming.md#0x1_YieldFarming_ERR_FARMING_STAKE_EXISTS">ERR_FARMING_STAKE_EXISTS</a>: u64 = 103;
</code></pre>



<a name="0x1_YieldFarming_ERR_FARMING_STAKE_NOT_EXISTS"></a>



<pre><code><b>const</b> <a href="YieldFarming.md#0x1_YieldFarming_ERR_FARMING_STAKE_NOT_EXISTS">ERR_FARMING_STAKE_NOT_EXISTS</a>: u64 = 104;
</code></pre>



<a name="0x1_YieldFarming_ERR_FARMING_TIMESTAMP_INVALID"></a>



<pre><code><b>const</b> <a href="YieldFarming.md#0x1_YieldFarming_ERR_FARMING_TIMESTAMP_INVALID">ERR_FARMING_TIMESTAMP_INVALID</a>: u64 = 110;
</code></pre>



<a name="0x1_YieldFarming_ERR_FARMING_TOTAL_WEIGHT_IS_ZERO"></a>



<pre><code><b>const</b> <a href="YieldFarming.md#0x1_YieldFarming_ERR_FARMING_TOTAL_WEIGHT_IS_ZERO">ERR_FARMING_TOTAL_WEIGHT_IS_ZERO</a>: u64 = 106;
</code></pre>



<a name="0x1_YieldFarming_EXP_SCALE"></a>



<pre><code><b>const</b> <a href="YieldFarming.md#0x1_YieldFarming_EXP_SCALE">EXP_SCALE</a>: u128 = 1000000000000000000;
</code></pre>



<a name="0x1_YieldFarming_exp"></a>

## Function `exp`



<pre><code><b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_exp">exp</a>(num: u128, denom: u128): <a href="YieldFarming.md#0x1_YieldFarming_Exp">YieldFarming::Exp</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_exp">exp</a>(num: u128, denom: u128): <a href="YieldFarming.md#0x1_YieldFarming_Exp">Exp</a> {
    // <b>if</b> overflow <b>move</b> will <b>abort</b>
    <b>let</b> scaledNumerator = <a href="YieldFarming.md#0x1_YieldFarming_mul_u128">mul_u128</a>(num, <a href="YieldFarming.md#0x1_YieldFarming_EXP_SCALE">EXP_SCALE</a>);
    <b>let</b> rational = <a href="YieldFarming.md#0x1_YieldFarming_div_u128">div_u128</a>(scaledNumerator, denom);
    <a href="YieldFarming.md#0x1_YieldFarming_Exp">Exp</a> {
        mantissa: rational
    }
}
</code></pre>



</details>

<a name="0x1_YieldFarming_mul_u128"></a>

## Function `mul_u128`



<pre><code><b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_mul_u128">mul_u128</a>(a: u128, b: u128): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_mul_u128">mul_u128</a>(a: u128, b: u128): u128 {
    <b>if</b> (a == 0 || b == 0) {
        <b>return</b> 0
    };

    a * b
}
</code></pre>



</details>

<a name="0x1_YieldFarming_div_u128"></a>

## Function `div_u128`



<pre><code><b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_div_u128">div_u128</a>(a: u128, b: u128): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_div_u128">div_u128</a>(a: u128, b: u128): u128 {
    <b>if</b> ( b == 0) {
        <b>abort</b> <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="YieldFarming.md#0x1_YieldFarming_ERR_EXP_DIVIDE_BY_ZERO">ERR_EXP_DIVIDE_BY_ZERO</a>)
    };
    <b>if</b> (a == 0) {
        <b>return</b> 0
    };
    a / b
}
</code></pre>



</details>

<a name="0x1_YieldFarming_truncate"></a>

## Function `truncate`



<pre><code><b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_truncate">truncate</a>(exp: <a href="YieldFarming.md#0x1_YieldFarming_Exp">YieldFarming::Exp</a>): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_truncate">truncate</a>(exp: <a href="YieldFarming.md#0x1_YieldFarming_Exp">Exp</a>): u128 {
    <b>return</b> exp.mantissa / <a href="YieldFarming.md#0x1_YieldFarming_EXP_SCALE">EXP_SCALE</a>
}
</code></pre>



</details>

<a name="0x1_YieldFarming_initialize"></a>

## Function `initialize`

Called by token issuer
this will declare a yield farming pool


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_initialize">initialize</a>&lt;PoolType: store, RewardTokenT: store&gt;(account: &signer, treasury_token: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;RewardTokenT&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_initialize">initialize</a>&lt;
    PoolType: store,
    RewardTokenT: store&gt;(account: &signer, treasury_token: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;RewardTokenT&gt;) {
    <b>assert</b>(!<a href="YieldFarming.md#0x1_YieldFarming_exists_at">exists_at</a>&lt;PoolType, RewardTokenT&gt;(
        <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)),
        <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="YieldFarming.md#0x1_YieldFarming_ERR_FARMING_INIT_REPEATE">ERR_FARMING_INIT_REPEATE</a>));

    move_to(account, <a href="YieldFarming.md#0x1_YieldFarming_Farming">Farming</a>&lt;PoolType, RewardTokenT&gt; {
        treasury_token,
    });
}
</code></pre>



</details>

<a name="0x1_YieldFarming_initialize_asset"></a>

## Function `initialize_asset`



<pre><code><b>public</b> <b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_initialize_asset">initialize_asset</a>&lt;PoolType: store, AssetT: store&gt;(account: &signer, release_per_second: u128, delay: u64): <a href="YieldFarming.md#0x1_YieldFarming_ParameterModifyCapability">YieldFarming::ParameterModifyCapability</a>&lt;PoolType, AssetT&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_initialize_asset">initialize_asset</a>&lt;PoolType: store, AssetT: store&gt;(
    account: &signer,
    release_per_second: u128,
    delay: u64): <a href="YieldFarming.md#0x1_YieldFarming_ParameterModifyCapability">ParameterModifyCapability</a>&lt;PoolType, AssetT&gt; {

    <b>assert</b>(!<a href="YieldFarming.md#0x1_YieldFarming_exists_asset_at">exists_asset_at</a>&lt;PoolType, AssetT&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)),
        <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="YieldFarming.md#0x1_YieldFarming_ERR_FARMING_INIT_REPEATE">ERR_FARMING_INIT_REPEATE</a>));

    <b>let</b> now_seconds = <a href="Timestamp.md#0x1_Timestamp_now_seconds">Timestamp::now_seconds</a>();

    move_to(account, <a href="YieldFarming.md#0x1_YieldFarming_FarmingAsset">FarmingAsset</a>&lt;PoolType, AssetT&gt; {
        asset_total_weight: 0,
        harvest_index: 0,
        last_update_timestamp: now_seconds,
        release_per_second,
        start_time: now_seconds + delay,
    });
    <a href="YieldFarming.md#0x1_YieldFarming_ParameterModifyCapability">ParameterModifyCapability</a>&lt;PoolType, AssetT&gt; {}
}
</code></pre>



</details>

<a name="0x1_YieldFarming_modify_parameter"></a>

## Function `modify_parameter`



<pre><code><b>public</b> <b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_modify_parameter">modify_parameter</a>&lt;PoolType: store, RewardTokenT: store, AssetT: store&gt;(_cap: &<a href="YieldFarming.md#0x1_YieldFarming_ParameterModifyCapability">YieldFarming::ParameterModifyCapability</a>&lt;PoolType, AssetT&gt;, broker: address, release_per_second: u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_modify_parameter">modify_parameter</a>&lt;PoolType: store, RewardTokenT: store, AssetT: store&gt;(
    _cap: &<a href="YieldFarming.md#0x1_YieldFarming_ParameterModifyCapability">ParameterModifyCapability</a>&lt;PoolType, AssetT&gt;,
    broker: address,
    release_per_second: u128) <b>acquires</b> <a href="YieldFarming.md#0x1_YieldFarming_FarmingAsset">FarmingAsset</a> {
    <b>let</b> farming_asset = borrow_global_mut&lt;<a href="YieldFarming.md#0x1_YieldFarming_FarmingAsset">FarmingAsset</a>&lt;PoolType, AssetT&gt;&gt;(broker);
    <b>let</b> now_seconds = <a href="Timestamp.md#0x1_Timestamp_now_seconds">Timestamp::now_seconds</a>();

    <b>let</b> new_index = <a href="YieldFarming.md#0x1_YieldFarming_calculate_harvest_index_with_asset">calculate_harvest_index_with_asset</a>&lt;PoolType, AssetT&gt;(farming_asset, now_seconds);

    farming_asset.release_per_second = release_per_second;
    farming_asset.harvest_index = new_index;
    farming_asset.last_update_timestamp = now_seconds;
}
</code></pre>



</details>

<a name="0x1_YieldFarming_stake"></a>

## Function `stake`

Call by stake user, staking amount of asset in order to get yield farming token


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_stake">stake</a>&lt;PoolType: store, RewardTokenT: store, AssetT: store&gt;(account: &signer, broker: address, asset: AssetT, asset_weight: u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_stake">stake</a>&lt;PoolType: store, RewardTokenT: store, AssetT: store&gt;(
    account: &signer,
    broker: address,
    asset: AssetT,
    asset_weight: u128) <b>acquires</b> <a href="YieldFarming.md#0x1_YieldFarming_FarmingAsset">FarmingAsset</a> {

    // <a href="Debug.md#0x1_Debug_print">Debug::print</a>(account);
    <b>let</b> account_address = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>assert</b>(!<a href="YieldFarming.md#0x1_YieldFarming_exists_stake_at_address">exists_stake_at_address</a>&lt;PoolType, AssetT&gt;(account_address),
        <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="YieldFarming.md#0x1_YieldFarming_ERR_FARMING_STAKE_EXISTS">ERR_FARMING_STAKE_EXISTS</a>));

    <b>let</b> farming_asset = borrow_global_mut&lt;<a href="YieldFarming.md#0x1_YieldFarming_FarmingAsset">FarmingAsset</a>&lt;PoolType, AssetT&gt;&gt;(broker);
    <b>let</b> now_seconds = <a href="Timestamp.md#0x1_Timestamp_now_seconds">Timestamp::now_seconds</a>();

    // Check locking time
    <b>assert</b>(farming_asset.start_time &lt;= now_seconds, <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="YieldFarming.md#0x1_YieldFarming_ERR_FARMING_NOT_STILL_FREEZE">ERR_FARMING_NOT_STILL_FREEZE</a>));

    <b>let</b> time_period = now_seconds - farming_asset.last_update_timestamp;

    <b>if</b> (farming_asset.asset_total_weight &lt;= 0) { // <a href="YieldFarming.md#0x1_YieldFarming_Stake">Stake</a> <b>as</b> first user
        <b>let</b> gain = farming_asset.release_per_second * (time_period <b>as</b> u128);
        move_to(account, <a href="YieldFarming.md#0x1_YieldFarming_Stake">Stake</a>&lt;PoolType, AssetT&gt;{
            asset,
            asset_weight,
            last_harvest_index: 0,
            gain,
        });
        farming_asset.harvest_index = 0;
        farming_asset.asset_total_weight = asset_weight;
    } <b>else</b> {
        <b>let</b> new_harvest_index = <a href="YieldFarming.md#0x1_YieldFarming_calculate_harvest_index_with_asset">calculate_harvest_index_with_asset</a>&lt;PoolType, AssetT&gt;(farming_asset, now_seconds);
        move_to(account, <a href="YieldFarming.md#0x1_YieldFarming_Stake">Stake</a>&lt;PoolType, AssetT&gt;{
            asset,
            asset_weight,
            last_harvest_index: new_harvest_index,
            gain: 0,
        });
        farming_asset.asset_total_weight = farming_asset.asset_total_weight + asset_weight;
        farming_asset.harvest_index = new_harvest_index;
    };
    farming_asset.last_update_timestamp = now_seconds;
}
</code></pre>



</details>

<a name="0x1_YieldFarming_unstake"></a>

## Function `unstake`

Unstake asset from farming pool


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_unstake">unstake</a>&lt;PoolType: store, RewardTokenT: store, AssetT: store&gt;(account: &signer, broker: address): (AssetT, <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;RewardTokenT&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_unstake">unstake</a>&lt;PoolType: store, RewardTokenT: store, AssetT: store&gt;(account: &signer, broker: address)
    : (AssetT, <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;RewardTokenT&gt;) <b>acquires</b> <a href="YieldFarming.md#0x1_YieldFarming_Farming">Farming</a>, <a href="YieldFarming.md#0x1_YieldFarming_FarmingAsset">FarmingAsset</a>, <a href="YieldFarming.md#0x1_YieldFarming_Stake">Stake</a> {
    <b>let</b> farming = borrow_global_mut&lt;<a href="YieldFarming.md#0x1_YieldFarming_Farming">Farming</a>&lt;PoolType, RewardTokenT&gt;&gt;(broker);
    <b>let</b> farming_asset = borrow_global_mut&lt;<a href="YieldFarming.md#0x1_YieldFarming_FarmingAsset">FarmingAsset</a>&lt;PoolType, AssetT&gt;&gt;(broker);

    <b>let</b> <a href="YieldFarming.md#0x1_YieldFarming_Stake">Stake</a>&lt;PoolType, AssetT&gt; {last_harvest_index, asset_weight, asset, gain} =
        move_from&lt;<a href="YieldFarming.md#0x1_YieldFarming_Stake">Stake</a>&lt;PoolType, AssetT&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));

    <b>let</b> now_seconds = <a href="Timestamp.md#0x1_Timestamp_now_seconds">Timestamp::now_seconds</a>();
    <b>let</b> new_harvest_index = <a href="YieldFarming.md#0x1_YieldFarming_calculate_harvest_index_with_asset">calculate_harvest_index_with_asset</a>&lt;PoolType, AssetT&gt;(farming_asset, now_seconds);

    <b>let</b> period_gain = <a href="YieldFarming.md#0x1_YieldFarming_calculate_withdraw_amount">calculate_withdraw_amount</a>(new_harvest_index, last_harvest_index, asset_weight);
    <b>let</b> total_gain = gain + period_gain;
    <b>let</b> withdraw_token = <a href="Token.md#0x1_Token_withdraw">Token::withdraw</a>&lt;RewardTokenT&gt;(&<b>mut</b> farming.treasury_token, total_gain);

    // Dont <b>update</b> harvest index that because the `<a href="YieldFarming.md#0x1_YieldFarming_Stake">Stake</a>` object has droped.
    // <b>let</b> new_index = <a href="YieldFarming.md#0x1_YieldFarming_calculate_harvest_index_with_asset">calculate_harvest_index_with_asset</a>&lt;PoolType, AssetT&gt;(farming_asset, now_seconds);
    <b>assert</b>(farming_asset.asset_total_weight &gt;= asset_weight, <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="YieldFarming.md#0x1_YieldFarming_ERR_FARMING_NOT_ENOUGH_ASSET">ERR_FARMING_NOT_ENOUGH_ASSET</a>));

    // Update farm asset
    farming_asset.asset_total_weight = farming_asset.asset_total_weight - asset_weight;
    farming_asset.harvest_index = new_harvest_index;
    farming_asset.last_update_timestamp = now_seconds;

    (asset, withdraw_token)
}
</code></pre>



</details>

<a name="0x1_YieldFarming_harvest"></a>

## Function `harvest`

Harvest yield farming token from stake


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_harvest">harvest</a>&lt;PoolType: store, RewardTokenT: store, AssetT: store&gt;(account: &signer, broker: address, amount: u128): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;RewardTokenT&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_harvest">harvest</a>&lt;PoolType: store,
                   RewardTokenT: store,
                   AssetT: store&gt;(
    account: &signer,
    broker: address,
    amount: u128) : <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;RewardTokenT&gt; <b>acquires</b> <a href="YieldFarming.md#0x1_YieldFarming_Farming">Farming</a>, <a href="YieldFarming.md#0x1_YieldFarming_FarmingAsset">FarmingAsset</a>, <a href="YieldFarming.md#0x1_YieldFarming_Stake">Stake</a> {

    <b>let</b> farming = borrow_global_mut&lt;<a href="YieldFarming.md#0x1_YieldFarming_Farming">Farming</a>&lt;PoolType, RewardTokenT&gt;&gt;(broker);
    <b>let</b> farming_asset = borrow_global_mut&lt;<a href="YieldFarming.md#0x1_YieldFarming_FarmingAsset">FarmingAsset</a>&lt;PoolType, AssetT&gt;&gt;(broker);
    <b>let</b> stake = borrow_global_mut&lt;<a href="YieldFarming.md#0x1_YieldFarming_Stake">Stake</a>&lt;PoolType, AssetT&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));

    <b>let</b> now_seconds = <a href="Timestamp.md#0x1_Timestamp_now_seconds">Timestamp::now_seconds</a>();
    <b>let</b> new_harvest_index = <a href="YieldFarming.md#0x1_YieldFarming_calculate_harvest_index_with_asset">calculate_harvest_index_with_asset</a>&lt;PoolType, AssetT&gt;(farming_asset, now_seconds);

    <b>let</b> period_gain = <a href="YieldFarming.md#0x1_YieldFarming_calculate_withdraw_amount">calculate_withdraw_amount</a>(
        new_harvest_index,
        stake.last_harvest_index,
        stake.asset_weight
    );

    <b>let</b> total_gain = stake.gain + period_gain;
    //<b>assert</b>(total_gain &gt; 0, <a href="Errors.md#0x1_Errors_limit_exceeded">Errors::limit_exceeded</a>(<a href="YieldFarming.md#0x1_YieldFarming_ERR_FARMING_HAVERST_NO_GAIN">ERR_FARMING_HAVERST_NO_GAIN</a>));
    <b>assert</b>(total_gain &gt;= amount, <a href="Errors.md#0x1_Errors_limit_exceeded">Errors::limit_exceeded</a>(<a href="YieldFarming.md#0x1_YieldFarming_ERR_FARMING_BALANCE_EXCEEDED">ERR_FARMING_BALANCE_EXCEEDED</a>));

    <b>let</b> withdraw_amount = <b>if</b> (amount &lt;= 0) {
        total_gain
    } <b>else</b> {
        amount
    };

    <b>let</b> withdraw_token = <a href="Token.md#0x1_Token_withdraw">Token::withdraw</a>&lt;RewardTokenT&gt;(&<b>mut</b> farming.treasury_token, withdraw_amount);
    stake.gain = total_gain - withdraw_amount;
    stake.last_harvest_index = new_harvest_index;

    farming_asset.harvest_index = new_harvest_index;
    farming_asset.last_update_timestamp = now_seconds;

    withdraw_token
}
</code></pre>



</details>

<a name="0x1_YieldFarming_query_gov_token_amount"></a>

## Function `query_gov_token_amount`

The user can quering all yield farming amount in any time and scene


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_query_gov_token_amount">query_gov_token_amount</a>&lt;PoolType: store, RewardTokenT: store, AssetT: store&gt;(account: &signer, broker: address): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_query_gov_token_amount">query_gov_token_amount</a>&lt;PoolType: store,
                                  RewardTokenT: store,
                                  AssetT: store&gt;(account: &signer, broker: address): u128 <b>acquires</b> <a href="YieldFarming.md#0x1_YieldFarming_FarmingAsset">FarmingAsset</a>, <a href="YieldFarming.md#0x1_YieldFarming_Stake">Stake</a> {
    <b>let</b> farming_asset = borrow_global_mut&lt;<a href="YieldFarming.md#0x1_YieldFarming_FarmingAsset">FarmingAsset</a>&lt;PoolType, AssetT&gt;&gt;(broker);
    <b>let</b> stake = borrow_global_mut&lt;<a href="YieldFarming.md#0x1_YieldFarming_Stake">Stake</a>&lt;PoolType, AssetT&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
    <b>let</b> now_seconds = <a href="Timestamp.md#0x1_Timestamp_now_seconds">Timestamp::now_seconds</a>();

    <b>let</b> new_harvest_index = <a href="YieldFarming.md#0x1_YieldFarming_calculate_harvest_index_with_asset">calculate_harvest_index_with_asset</a>&lt;PoolType, AssetT&gt;(
        farming_asset,
        now_seconds
    );

    <b>let</b> new_gain = <a href="YieldFarming.md#0x1_YieldFarming_calculate_withdraw_amount">calculate_withdraw_amount</a>(
        new_harvest_index,
        stake.last_harvest_index,
        stake.asset_weight
    );

    stake.gain = stake.gain + new_gain;
    stake.last_harvest_index = new_harvest_index;

    farming_asset.harvest_index = new_harvest_index;
    farming_asset.last_update_timestamp = now_seconds;

    stake.gain
}
</code></pre>



</details>

<a name="0x1_YieldFarming_query_total_stake"></a>

## Function `query_total_stake`

Query total stake count from yield farming resource


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_query_total_stake">query_total_stake</a>&lt;PoolType: store, AssetT: store&gt;(broker: address): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_query_total_stake">query_total_stake</a>&lt;PoolType: store,
                             AssetT: store&gt;(broker: address): u128 <b>acquires</b> <a href="YieldFarming.md#0x1_YieldFarming_FarmingAsset">FarmingAsset</a> {
    <b>let</b> farming_asset = borrow_global_mut&lt;<a href="YieldFarming.md#0x1_YieldFarming_FarmingAsset">FarmingAsset</a>&lt;PoolType, AssetT&gt;&gt;(broker);
    farming_asset.asset_total_weight
}
</code></pre>



</details>

<a name="0x1_YieldFarming_query_stake"></a>

## Function `query_stake`

Query stake weight from user staking objects.


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_query_stake">query_stake</a>&lt;PoolType: store, AssetT: store&gt;(account: &signer): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_query_stake">query_stake</a>&lt;PoolType: store,
                       AssetT: store&gt;(account: &signer): u128 <b>acquires</b> <a href="YieldFarming.md#0x1_YieldFarming_Stake">Stake</a> {
    <b>let</b> stake = borrow_global_mut&lt;<a href="YieldFarming.md#0x1_YieldFarming_Stake">Stake</a>&lt;PoolType, AssetT&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
    stake.asset_weight
}
</code></pre>



</details>

<a name="0x1_YieldFarming_calculate_harvest_index_with_asset"></a>

## Function `calculate_harvest_index_with_asset`

Update farming asset


<pre><code><b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_calculate_harvest_index_with_asset">calculate_harvest_index_with_asset</a>&lt;PoolType, AssetT&gt;(farming_asset: &<a href="YieldFarming.md#0x1_YieldFarming_FarmingAsset">YieldFarming::FarmingAsset</a>&lt;PoolType, AssetT&gt;, now_seconds: u64): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_calculate_harvest_index_with_asset">calculate_harvest_index_with_asset</a>&lt;PoolType, AssetT&gt;(farming_asset: &<a href="YieldFarming.md#0x1_YieldFarming_FarmingAsset">FarmingAsset</a>&lt;PoolType, AssetT&gt;, now_seconds: u64) : u128 {
    // Recalculate harvest index
    <b>if</b> (farming_asset.asset_total_weight &lt;= 0) {
        <a href="YieldFarming.md#0x1_YieldFarming_calculate_harvest_index_weight_zero">calculate_harvest_index_weight_zero</a>(
            farming_asset.harvest_index,
            farming_asset.last_update_timestamp,
            now_seconds,
            farming_asset.release_per_second
        )
    } <b>else</b> {
        <a href="YieldFarming.md#0x1_YieldFarming_calculate_harvest_index">calculate_harvest_index</a>(
            farming_asset.harvest_index,
            farming_asset.asset_total_weight,
            farming_asset.last_update_timestamp,
            now_seconds,
            farming_asset.release_per_second
        )
    }
}
</code></pre>



</details>

<a name="0x1_YieldFarming_calculate_harvest_index_weight_zero"></a>

## Function `calculate_harvest_index_weight_zero`

There is calculating from harvest index and global parameters without asset_total_weight


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_calculate_harvest_index_weight_zero">calculate_harvest_index_weight_zero</a>(harvest_index: u128, last_update_timestamp: u64, now_seconds: u64, release_per_second: u128): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_calculate_harvest_index_weight_zero">calculate_harvest_index_weight_zero</a>(harvest_index: u128,
                                               last_update_timestamp: u64,
                                               now_seconds: u64,
                                               release_per_second: u128): u128 {
    <b>assert</b>(last_update_timestamp &lt;= now_seconds, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="YieldFarming.md#0x1_YieldFarming_ERR_FARMING_TIMESTAMP_INVALID">ERR_FARMING_TIMESTAMP_INVALID</a>));
    <b>let</b> time_period = now_seconds - last_update_timestamp;
    harvest_index + (release_per_second * ((time_period <b>as</b> u128)))
}
</code></pre>



</details>

<a name="0x1_YieldFarming_calculate_harvest_index"></a>

## Function `calculate_harvest_index`

There is calculating from harvest index and global parameters


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_calculate_harvest_index">calculate_harvest_index</a>(harvest_index: u128, asset_total_weight: u128, last_update_timestamp: u64, now_seconds: u64, release_per_second: u128): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_calculate_harvest_index">calculate_harvest_index</a>(harvest_index: u128,
                                   asset_total_weight: u128,
                                   last_update_timestamp: u64,
                                   now_seconds: u64,
                                   release_per_second: u128): u128 {
    <b>assert</b>(asset_total_weight &gt; 0, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="YieldFarming.md#0x1_YieldFarming_ERR_FARMING_TOTAL_WEIGHT_IS_ZERO">ERR_FARMING_TOTAL_WEIGHT_IS_ZERO</a>));
    <b>assert</b>(last_update_timestamp &lt;= now_seconds, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="YieldFarming.md#0x1_YieldFarming_ERR_FARMING_TIMESTAMP_INVALID">ERR_FARMING_TIMESTAMP_INVALID</a>));

    <b>let</b> time_period = now_seconds - last_update_timestamp;
    <b>let</b> numr = (release_per_second * (time_period <b>as</b> u128));
    <b>let</b> denom = asset_total_weight;
    <b>let</b> added_index = <a href="YieldFarming.md#0x1_YieldFarming_truncate">truncate</a>(<a href="YieldFarming.md#0x1_YieldFarming_exp">exp</a>(numr, denom));
    harvest_index + added_index
}
</code></pre>



</details>

<a name="0x1_YieldFarming_calculate_withdraw_amount"></a>

## Function `calculate_withdraw_amount`

This function will return a gain index


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_calculate_withdraw_amount">calculate_withdraw_amount</a>(harvest_index: u128, last_harvest_index: u128, asset_weight: u128): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_calculate_withdraw_amount">calculate_withdraw_amount</a>(harvest_index: u128,
                                     last_harvest_index: u128,
                                     asset_weight: u128): u128 {
    asset_weight * (harvest_index - last_harvest_index)
}
</code></pre>



</details>

<a name="0x1_YieldFarming_exists_at"></a>

## Function `exists_at`

Check the Farming of TokenT is exists.


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_exists_at">exists_at</a>&lt;PoolType: store, RewardTokenT: store&gt;(broker: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_exists_at">exists_at</a>&lt;PoolType: store, RewardTokenT: store&gt;(broker: address): bool {
    <b>exists</b>&lt;<a href="YieldFarming.md#0x1_YieldFarming_Farming">Farming</a>&lt;PoolType, RewardTokenT&gt;&gt;(broker)
}
</code></pre>



</details>

<a name="0x1_YieldFarming_exists_asset_at"></a>

## Function `exists_asset_at`

Check the Farming of AsssetT is exists.


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_exists_asset_at">exists_asset_at</a>&lt;PoolType: store, AssetT: store&gt;(broker: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_exists_asset_at">exists_asset_at</a>&lt;PoolType: store, AssetT: store&gt;(broker: address): bool {
    <b>exists</b>&lt;<a href="YieldFarming.md#0x1_YieldFarming_FarmingAsset">FarmingAsset</a>&lt;PoolType, AssetT&gt;&gt;(broker)
}
</code></pre>



</details>

<a name="0x1_YieldFarming_exists_stake_at_address"></a>

## Function `exists_stake_at_address`

Check stake at address exists.


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_exists_stake_at_address">exists_stake_at_address</a>&lt;PoolType: store, AssetT: store&gt;(account: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_exists_stake_at_address">exists_stake_at_address</a>&lt;PoolType: store, AssetT: store&gt;(account: address): bool {
    <b>exists</b>&lt;<a href="YieldFarming.md#0x1_YieldFarming_Stake">Stake</a>&lt;PoolType, AssetT&gt;&gt;(account)
}
</code></pre>



</details>

<a name="@Specification_1"></a>

## Specification



<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>
