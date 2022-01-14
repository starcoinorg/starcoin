
<a name="0x1_YieldFarmingV2"></a>

# Module `0x1::YieldFarmingV2`



-  [Struct `Exp`](#0x1_YieldFarmingV2_Exp)
-  [Resource `Farming`](#0x1_YieldFarmingV2_Farming)
-  [Resource `FarmingAsset`](#0x1_YieldFarmingV2_FarmingAsset)
-  [Resource `Stake`](#0x1_YieldFarmingV2_Stake)
-  [Resource `ParameterModifyCapability`](#0x1_YieldFarmingV2_ParameterModifyCapability)
-  [Resource `HarvestCapability`](#0x1_YieldFarmingV2_HarvestCapability)
-  [Constants](#@Constants_0)
-  [Function `exp_direct`](#0x1_YieldFarmingV2_exp_direct)
-  [Function `exp_direct_expand`](#0x1_YieldFarmingV2_exp_direct_expand)
-  [Function `mantissa`](#0x1_YieldFarmingV2_mantissa)
-  [Function `add_exp`](#0x1_YieldFarmingV2_add_exp)
-  [Function `exp`](#0x1_YieldFarmingV2_exp)
-  [Function `add_u128`](#0x1_YieldFarmingV2_add_u128)
-  [Function `sub_u128`](#0x1_YieldFarmingV2_sub_u128)
-  [Function `mul_u128`](#0x1_YieldFarmingV2_mul_u128)
-  [Function `div_u128`](#0x1_YieldFarmingV2_div_u128)
-  [Function `truncate`](#0x1_YieldFarmingV2_truncate)
-  [Function `initialize`](#0x1_YieldFarmingV2_initialize)
-  [Function `add_asset`](#0x1_YieldFarmingV2_add_asset)
-  [Function `modify_parameter`](#0x1_YieldFarmingV2_modify_parameter)
-  [Function `stake`](#0x1_YieldFarmingV2_stake)
-  [Function `stake_for_cap`](#0x1_YieldFarmingV2_stake_for_cap)
-  [Function `unstake`](#0x1_YieldFarmingV2_unstake)
-  [Function `unstake_with_cap`](#0x1_YieldFarmingV2_unstake_with_cap)
-  [Function `harvest`](#0x1_YieldFarmingV2_harvest)
-  [Function `harvest_with_cap`](#0x1_YieldFarmingV2_harvest_with_cap)
-  [Function `query_gov_token_amount`](#0x1_YieldFarmingV2_query_gov_token_amount)
-  [Function `query_total_stake`](#0x1_YieldFarmingV2_query_total_stake)
-  [Function `query_stake`](#0x1_YieldFarmingV2_query_stake)
-  [Function `query_info`](#0x1_YieldFarmingV2_query_info)
-  [Function `calculate_harvest_index_with_asset`](#0x1_YieldFarmingV2_calculate_harvest_index_with_asset)
-  [Function `calculate_harvest_index_weight_zero`](#0x1_YieldFarmingV2_calculate_harvest_index_weight_zero)
-  [Function `calculate_harvest_index`](#0x1_YieldFarmingV2_calculate_harvest_index)
-  [Function `calculate_withdraw_amount`](#0x1_YieldFarmingV2_calculate_withdraw_amount)
-  [Function `exists_at`](#0x1_YieldFarmingV2_exists_at)
-  [Function `exists_asset_at`](#0x1_YieldFarmingV2_exists_asset_at)
-  [Function `exists_stake_at_address`](#0x1_YieldFarmingV2_exists_stake_at_address)
-  [Specification](#@Specification_1)


<pre><code><b>use</b> <a href="Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="Math.md#0x1_Math">0x1::Math</a>;
<b>use</b> <a href="Signer.md#0x1_Signer">0x1::Signer</a>;
<b>use</b> <a href="Timestamp.md#0x1_Timestamp">0x1::Timestamp</a>;
<b>use</b> <a href="Token.md#0x1_Token">0x1::Token</a>;
</code></pre>



<a name="0x1_YieldFarmingV2_Exp"></a>

## Struct `Exp`



<pre><code><b>struct</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_Exp">Exp</a> <b>has</b> <b>copy</b>, drop, store
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

<a name="0x1_YieldFarmingV2_Farming"></a>

## Resource `Farming`

The object of yield farming
RewardTokenT meaning token of yield farming


<pre><code><b>struct</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_Farming">Farming</a>&lt;PoolType, RewardTokenT&gt; <b>has</b> store, key
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

<a name="0x1_YieldFarmingV2_FarmingAsset"></a>

## Resource `FarmingAsset`



<pre><code><b>struct</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_FarmingAsset">FarmingAsset</a>&lt;PoolType, AssetT&gt; <b>has</b> store, key
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
<dt>
<code>alive: bool</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_YieldFarmingV2_Stake"></a>

## Resource `Stake`

To store user's asset token


<pre><code><b>struct</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_Stake">Stake</a>&lt;PoolType, AssetT&gt; <b>has</b> store, key
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

<a name="0x1_YieldFarmingV2_ParameterModifyCapability"></a>

## Resource `ParameterModifyCapability`

Capability to modify parameter such as period and release amount


<pre><code><b>struct</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_ParameterModifyCapability">ParameterModifyCapability</a>&lt;PoolType, AssetT&gt; <b>has</b> store, key
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

<a name="0x1_YieldFarmingV2_HarvestCapability"></a>

## Resource `HarvestCapability`

Harvest ability to harvest


<pre><code><b>struct</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_HarvestCapability">HarvestCapability</a>&lt;PoolType, AssetT&gt; <b>has</b> store, key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>trigger: <b>address</b></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_YieldFarmingV2_ERR_EXP_DIVIDE_BY_ZERO"></a>



<pre><code><b>const</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_ERR_EXP_DIVIDE_BY_ZERO">ERR_EXP_DIVIDE_BY_ZERO</a>: u64 = 107;
</code></pre>



<a name="0x1_YieldFarmingV2_ERR_FARMING_BALANCE_EXCEEDED"></a>



<pre><code><b>const</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_ERR_FARMING_BALANCE_EXCEEDED">ERR_FARMING_BALANCE_EXCEEDED</a>: u64 = 108;
</code></pre>



<a name="0x1_YieldFarmingV2_ERR_FARMING_HAVERST_NO_GAIN"></a>



<pre><code><b>const</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_ERR_FARMING_HAVERST_NO_GAIN">ERR_FARMING_HAVERST_NO_GAIN</a>: u64 = 105;
</code></pre>



<a name="0x1_YieldFarmingV2_ERR_FARMING_INIT_REPEATE"></a>



<pre><code><b>const</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_ERR_FARMING_INIT_REPEATE">ERR_FARMING_INIT_REPEATE</a>: u64 = 101;
</code></pre>



<a name="0x1_YieldFarmingV2_ERR_FARMING_NOT_ENOUGH_ASSET"></a>



<pre><code><b>const</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_ERR_FARMING_NOT_ENOUGH_ASSET">ERR_FARMING_NOT_ENOUGH_ASSET</a>: u64 = 109;
</code></pre>



<a name="0x1_YieldFarmingV2_ERR_FARMING_NOT_STILL_FREEZE"></a>



<pre><code><b>const</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_ERR_FARMING_NOT_STILL_FREEZE">ERR_FARMING_NOT_STILL_FREEZE</a>: u64 = 102;
</code></pre>



<a name="0x1_YieldFarmingV2_ERR_FARMING_STAKE_EXISTS"></a>



<pre><code><b>const</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_ERR_FARMING_STAKE_EXISTS">ERR_FARMING_STAKE_EXISTS</a>: u64 = 103;
</code></pre>



<a name="0x1_YieldFarmingV2_ERR_FARMING_STAKE_NOT_EXISTS"></a>



<pre><code><b>const</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_ERR_FARMING_STAKE_NOT_EXISTS">ERR_FARMING_STAKE_NOT_EXISTS</a>: u64 = 104;
</code></pre>



<a name="0x1_YieldFarmingV2_ERR_FARMING_TIMESTAMP_INVALID"></a>



<pre><code><b>const</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_ERR_FARMING_TIMESTAMP_INVALID">ERR_FARMING_TIMESTAMP_INVALID</a>: u64 = 110;
</code></pre>



<a name="0x1_YieldFarmingV2_ERR_FARMING_TOTAL_WEIGHT_IS_ZERO"></a>



<pre><code><b>const</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_ERR_FARMING_TOTAL_WEIGHT_IS_ZERO">ERR_FARMING_TOTAL_WEIGHT_IS_ZERO</a>: u64 = 106;
</code></pre>



<a name="0x1_YieldFarmingV2_EXP_SCALE"></a>



<pre><code><b>const</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_EXP_SCALE">EXP_SCALE</a>: u128 = 1000000000000000000;
</code></pre>



<a name="0x1_YieldFarmingV2_ERR_FARMING_ALIVE_STATE_INVALID"></a>



<pre><code><b>const</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_ERR_FARMING_ALIVE_STATE_INVALID">ERR_FARMING_ALIVE_STATE_INVALID</a>: u64 = 114;
</code></pre>



<a name="0x1_YieldFarmingV2_ERR_FARMING_CALC_LAST_IDX_BIGGER_THAN_NOW"></a>



<pre><code><b>const</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_ERR_FARMING_CALC_LAST_IDX_BIGGER_THAN_NOW">ERR_FARMING_CALC_LAST_IDX_BIGGER_THAN_NOW</a>: u64 = 112;
</code></pre>



<a name="0x1_YieldFarmingV2_ERR_FARMING_NOT_ALIVE"></a>



<pre><code><b>const</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_ERR_FARMING_NOT_ALIVE">ERR_FARMING_NOT_ALIVE</a>: u64 = 113;
</code></pre>



<a name="0x1_YieldFarmingV2_ERR_FARMING_TOKEN_SCALE_OVERFLOW"></a>



<pre><code><b>const</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_ERR_FARMING_TOKEN_SCALE_OVERFLOW">ERR_FARMING_TOKEN_SCALE_OVERFLOW</a>: u64 = 111;
</code></pre>



<a name="0x1_YieldFarmingV2_EXP_MAX_SCALE"></a>



<pre><code><b>const</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_EXP_MAX_SCALE">EXP_MAX_SCALE</a>: u128 = 9;
</code></pre>



<a name="0x1_YieldFarmingV2_exp_direct"></a>

## Function `exp_direct`



<pre><code><b>fun</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_exp_direct">exp_direct</a>(num: u128): <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_Exp">YieldFarmingV2::Exp</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_exp_direct">exp_direct</a>(num: u128): <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_Exp">Exp</a> {
    <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_Exp">Exp</a> {
        mantissa: num
    }
}
</code></pre>



</details>

<a name="0x1_YieldFarmingV2_exp_direct_expand"></a>

## Function `exp_direct_expand`



<pre><code><b>fun</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_exp_direct_expand">exp_direct_expand</a>(num: u128): <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_Exp">YieldFarmingV2::Exp</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_exp_direct_expand">exp_direct_expand</a>(num: u128): <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_Exp">Exp</a> {
    <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_Exp">Exp</a> {
        mantissa: <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_mul_u128">mul_u128</a>(num, <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_EXP_SCALE">EXP_SCALE</a>)
    }
}
</code></pre>



</details>

<a name="0x1_YieldFarmingV2_mantissa"></a>

## Function `mantissa`



<pre><code><b>fun</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_mantissa">mantissa</a>(a: <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_Exp">YieldFarmingV2::Exp</a>): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_mantissa">mantissa</a>(a: <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_Exp">Exp</a>): u128 {
    a.mantissa
}
</code></pre>



</details>

<a name="0x1_YieldFarmingV2_add_exp"></a>

## Function `add_exp`



<pre><code><b>fun</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_add_exp">add_exp</a>(a: <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_Exp">YieldFarmingV2::Exp</a>, b: <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_Exp">YieldFarmingV2::Exp</a>): <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_Exp">YieldFarmingV2::Exp</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_add_exp">add_exp</a>(a: <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_Exp">Exp</a>, b: <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_Exp">Exp</a>): <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_Exp">Exp</a> {
    <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_Exp">Exp</a> {
        mantissa: <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_add_u128">add_u128</a>(a.mantissa, b.mantissa)
    }
}
</code></pre>



</details>

<a name="0x1_YieldFarmingV2_exp"></a>

## Function `exp`



<pre><code><b>fun</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_exp">exp</a>(num: u128, denom: u128): <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_Exp">YieldFarmingV2::Exp</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_exp">exp</a>(num: u128, denom: u128): <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_Exp">Exp</a> {
    // <b>if</b> overflow <b>move</b> will <b>abort</b>
    <b>let</b> scaledNumerator = <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_mul_u128">mul_u128</a>(num, <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_EXP_SCALE">EXP_SCALE</a>);
    <b>let</b> rational = <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_div_u128">div_u128</a>(scaledNumerator, denom);
    <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_Exp">Exp</a> {
        mantissa: rational
    }
}
</code></pre>



</details>

<a name="0x1_YieldFarmingV2_add_u128"></a>

## Function `add_u128`



<pre><code><b>fun</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_add_u128">add_u128</a>(a: u128, b: u128): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_add_u128">add_u128</a>(a: u128, b: u128): u128 {
    a + b
}
</code></pre>



</details>

<a name="0x1_YieldFarmingV2_sub_u128"></a>

## Function `sub_u128`



<pre><code><b>fun</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_sub_u128">sub_u128</a>(a: u128, b: u128): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_sub_u128">sub_u128</a>(a: u128, b: u128): u128 {
    a - b
}
</code></pre>



</details>

<a name="0x1_YieldFarmingV2_mul_u128"></a>

## Function `mul_u128`



<pre><code><b>fun</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_mul_u128">mul_u128</a>(a: u128, b: u128): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_mul_u128">mul_u128</a>(a: u128, b: u128): u128 {
    <b>if</b> (a == 0 || b == 0) {
        <b>return</b> 0
    };
    a * b
}
</code></pre>



</details>

<a name="0x1_YieldFarmingV2_div_u128"></a>

## Function `div_u128`



<pre><code><b>fun</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_div_u128">div_u128</a>(a: u128, b: u128): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_div_u128">div_u128</a>(a: u128, b: u128): u128 {
    <b>if</b> (b == 0) {
        <b>abort</b> <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="YieldFarmingV2.md#0x1_YieldFarmingV2_ERR_EXP_DIVIDE_BY_ZERO">ERR_EXP_DIVIDE_BY_ZERO</a>)
    };
    <b>if</b> (a == 0) {
        <b>return</b> 0
    };
    a / b
}
</code></pre>



</details>

<a name="0x1_YieldFarmingV2_truncate"></a>

## Function `truncate`



<pre><code><b>fun</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_truncate">truncate</a>(exp: <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_Exp">YieldFarmingV2::Exp</a>): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_truncate">truncate</a>(exp: <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_Exp">Exp</a>): u128 {
    <b>return</b> exp.mantissa / <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_EXP_SCALE">EXP_SCALE</a>
}
</code></pre>



</details>

<a name="0x1_YieldFarmingV2_initialize"></a>

## Function `initialize`

Called by token issuer
this will declare a yield farming pool


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_initialize">initialize</a>&lt;PoolType: store, RewardTokenT: store&gt;(signer: &signer, treasury_token: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;RewardTokenT&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_initialize">initialize</a>&lt;
    PoolType: store,
    RewardTokenT: store&gt;(signer: &signer, treasury_token: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;RewardTokenT&gt;) {
    <b>let</b> scaling_factor = <a href="Math.md#0x1_Math_pow">Math::pow</a>(10, (<a href="YieldFarmingV2.md#0x1_YieldFarmingV2_EXP_MAX_SCALE">EXP_MAX_SCALE</a> <b>as</b> u64));
    <b>let</b> token_scale = <a href="Token.md#0x1_Token_scaling_factor">Token::scaling_factor</a>&lt;RewardTokenT&gt;();
    <b>assert</b>!(token_scale &lt;= scaling_factor, <a href="Errors.md#0x1_Errors_limit_exceeded">Errors::limit_exceeded</a>(<a href="YieldFarmingV2.md#0x1_YieldFarmingV2_ERR_FARMING_TOKEN_SCALE_OVERFLOW">ERR_FARMING_TOKEN_SCALE_OVERFLOW</a>));
    <b>assert</b>!(!<a href="YieldFarmingV2.md#0x1_YieldFarmingV2_exists_at">exists_at</a>&lt;PoolType, RewardTokenT&gt;(
        <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer)), <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="YieldFarmingV2.md#0x1_YieldFarmingV2_ERR_FARMING_INIT_REPEATE">ERR_FARMING_INIT_REPEATE</a>));

    <b>move_to</b>(signer, <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_Farming">Farming</a>&lt;PoolType, RewardTokenT&gt; {
        treasury_token,
    });
}
</code></pre>



</details>

<a name="0x1_YieldFarmingV2_add_asset"></a>

## Function `add_asset`

Add asset pools


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_add_asset">add_asset</a>&lt;PoolType: store, AssetT: store&gt;(signer: &signer, release_per_second: u128, delay: u64): <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_ParameterModifyCapability">YieldFarmingV2::ParameterModifyCapability</a>&lt;PoolType, AssetT&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_add_asset">add_asset</a>&lt;PoolType: store, AssetT: store&gt;(
    signer: &signer,
    release_per_second: u128,
    delay: u64): <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_ParameterModifyCapability">ParameterModifyCapability</a>&lt;PoolType, AssetT&gt; {
    <b>assert</b>!(!<a href="YieldFarmingV2.md#0x1_YieldFarmingV2_exists_asset_at">exists_asset_at</a>&lt;PoolType, AssetT&gt;(
        <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer)),
        <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="YieldFarmingV2.md#0x1_YieldFarmingV2_ERR_FARMING_INIT_REPEATE">ERR_FARMING_INIT_REPEATE</a>));

    <b>let</b> now_seconds = <a href="Timestamp.md#0x1_Timestamp_now_seconds">Timestamp::now_seconds</a>();

    <b>move_to</b>(signer, <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_FarmingAsset">FarmingAsset</a>&lt;PoolType, AssetT&gt; {
        asset_total_weight: 0,
        harvest_index: 0,
        last_update_timestamp: now_seconds,
        release_per_second,
        start_time: now_seconds + delay,
        alive: <b>true</b>
    });
    <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_ParameterModifyCapability">ParameterModifyCapability</a>&lt;PoolType, AssetT&gt; {}
}
</code></pre>



</details>

<a name="0x1_YieldFarmingV2_modify_parameter"></a>

## Function `modify_parameter`

Remove asset for make this pool to the state of not alive
Please make sure all user unstaking from this pool


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_modify_parameter">modify_parameter</a>&lt;PoolType: store, RewardTokenT: store, AssetT: store&gt;(_cap: &<a href="YieldFarmingV2.md#0x1_YieldFarmingV2_ParameterModifyCapability">YieldFarmingV2::ParameterModifyCapability</a>&lt;PoolType, AssetT&gt;, broker: <b>address</b>, release_per_second: u128, alive: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_modify_parameter">modify_parameter</a>&lt;PoolType: store, RewardTokenT: store, AssetT: store&gt;(
    _cap: &<a href="YieldFarmingV2.md#0x1_YieldFarmingV2_ParameterModifyCapability">ParameterModifyCapability</a>&lt;PoolType, AssetT&gt;,
    broker: <b>address</b>,
    release_per_second: u128,
    alive: bool) <b>acquires</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_FarmingAsset">FarmingAsset</a> {

    // Not support <b>to</b> shuttingdown alive state.
    <b>assert</b>!(alive, <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="YieldFarmingV2.md#0x1_YieldFarmingV2_ERR_FARMING_ALIVE_STATE_INVALID">ERR_FARMING_ALIVE_STATE_INVALID</a>));

    <b>let</b> farming_asset = <b>borrow_global_mut</b>&lt;<a href="YieldFarmingV2.md#0x1_YieldFarmingV2_FarmingAsset">FarmingAsset</a>&lt;PoolType, AssetT&gt;&gt;(broker);
    // <b>assert</b>!(farming_asset.alive != alive, <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="YieldFarmingV2.md#0x1_YieldFarmingV2_ERR_FARMING_ALIVE_STATE_INVALID">ERR_FARMING_ALIVE_STATE_INVALID</a>));

    <b>let</b> now_seconds = <a href="Timestamp.md#0x1_Timestamp_now_seconds">Timestamp::now_seconds</a>();

    farming_asset.release_per_second = release_per_second;
    farming_asset.last_update_timestamp = now_seconds;

    // <b>if</b> the pool is alive, then <b>update</b> index
    <b>if</b> (farming_asset.alive) {
        farming_asset.harvest_index = <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_calculate_harvest_index_with_asset">calculate_harvest_index_with_asset</a>&lt;PoolType, AssetT&gt;(farming_asset, now_seconds);
    };
    farming_asset.alive = alive;
}
</code></pre>



</details>

<a name="0x1_YieldFarmingV2_stake"></a>

## Function `stake`

Call by stake user, staking amount of asset in order to get yield farming token


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_stake">stake</a>&lt;PoolType: store, RewardTokenT: store, AssetT: store&gt;(signer: &signer, broker: <b>address</b>, asset: AssetT, asset_weight: u128, _cap: &<a href="YieldFarmingV2.md#0x1_YieldFarmingV2_ParameterModifyCapability">YieldFarmingV2::ParameterModifyCapability</a>&lt;PoolType, AssetT&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_stake">stake</a>&lt;PoolType: store, RewardTokenT: store, AssetT: store&gt;(
    signer: &signer,
    broker: <b>address</b>,
    asset: AssetT,
    asset_weight: u128,
    _cap: &<a href="YieldFarmingV2.md#0x1_YieldFarmingV2_ParameterModifyCapability">ParameterModifyCapability</a>&lt;PoolType, AssetT&gt;) <b>acquires</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_FarmingAsset">FarmingAsset</a> {
    <b>let</b> harvest_cap = <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_stake_for_cap">stake_for_cap</a>&lt;
        PoolType,
        RewardTokenT,
        AssetT&gt;(signer, broker, asset, asset_weight, _cap);

    <b>move_to</b>(signer, harvest_cap);
}
</code></pre>



</details>

<a name="0x1_YieldFarmingV2_stake_for_cap"></a>

## Function `stake_for_cap`



<pre><code><b>public</b> <b>fun</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_stake_for_cap">stake_for_cap</a>&lt;PoolType: store, RewardTokenT: store, AssetT: store&gt;(signer: &signer, broker: <b>address</b>, asset: AssetT, asset_weight: u128, _cap: &<a href="YieldFarmingV2.md#0x1_YieldFarmingV2_ParameterModifyCapability">YieldFarmingV2::ParameterModifyCapability</a>&lt;PoolType, AssetT&gt;): <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_HarvestCapability">YieldFarmingV2::HarvestCapability</a>&lt;PoolType, AssetT&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_stake_for_cap">stake_for_cap</a>&lt;PoolType: store, RewardTokenT: store, AssetT: store&gt;(
    signer: &signer,
    broker: <b>address</b>,
    asset: AssetT,
    asset_weight: u128,
    _cap: &<a href="YieldFarmingV2.md#0x1_YieldFarmingV2_ParameterModifyCapability">ParameterModifyCapability</a>&lt;PoolType, AssetT&gt;)
: <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_HarvestCapability">HarvestCapability</a>&lt;PoolType, AssetT&gt; <b>acquires</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_FarmingAsset">FarmingAsset</a> {
    <b>let</b> account = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer);
    <b>assert</b>!(!<a href="YieldFarmingV2.md#0x1_YieldFarmingV2_exists_stake_at_address">exists_stake_at_address</a>&lt;PoolType, AssetT&gt;(account),
        <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="YieldFarmingV2.md#0x1_YieldFarmingV2_ERR_FARMING_STAKE_EXISTS">ERR_FARMING_STAKE_EXISTS</a>));

    <b>let</b> farming_asset = <b>borrow_global_mut</b>&lt;<a href="YieldFarmingV2.md#0x1_YieldFarmingV2_FarmingAsset">FarmingAsset</a>&lt;PoolType, AssetT&gt;&gt;(broker);
    <b>assert</b>!(farming_asset.alive, <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="YieldFarmingV2.md#0x1_YieldFarmingV2_ERR_FARMING_NOT_ALIVE">ERR_FARMING_NOT_ALIVE</a>));

    // Check locking time
    <b>let</b> now_seconds = <a href="Timestamp.md#0x1_Timestamp_now_seconds">Timestamp::now_seconds</a>();
    <b>assert</b>!(farming_asset.start_time &lt;= now_seconds, <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="YieldFarmingV2.md#0x1_YieldFarmingV2_ERR_FARMING_NOT_STILL_FREEZE">ERR_FARMING_NOT_STILL_FREEZE</a>));

    <b>let</b> time_period = now_seconds - farming_asset.last_update_timestamp;

    <b>if</b> (farming_asset.asset_total_weight &lt;= 0) {
        // <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_Stake">Stake</a> <b>as</b> first user
        <b>let</b> gain = farming_asset.release_per_second * (time_period <b>as</b> u128);
        <b>move_to</b>(signer, <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_Stake">Stake</a>&lt;PoolType, AssetT&gt; {
            asset,
            asset_weight,
            last_harvest_index: 0,
            gain,
        });
        farming_asset.harvest_index = 0;
        farming_asset.asset_total_weight = asset_weight;
    } <b>else</b> {
        <b>let</b> new_harvest_index = <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_calculate_harvest_index_with_asset">calculate_harvest_index_with_asset</a>&lt;PoolType, AssetT&gt;(farming_asset, now_seconds);
        <b>move_to</b>(signer, <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_Stake">Stake</a>&lt;PoolType, AssetT&gt; {
            asset,
            asset_weight,
            last_harvest_index: new_harvest_index,
            gain: 0,
        });
        farming_asset.asset_total_weight = farming_asset.asset_total_weight + asset_weight;
        farming_asset.harvest_index = new_harvest_index;
    };
    farming_asset.last_update_timestamp = now_seconds;
    <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_HarvestCapability">HarvestCapability</a>&lt;PoolType, AssetT&gt; { trigger: account }
}
</code></pre>



</details>

<a name="0x1_YieldFarmingV2_unstake"></a>

## Function `unstake`

Unstake asset from farming pool


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_unstake">unstake</a>&lt;PoolType: store, RewardTokenT: store, AssetT: store&gt;(signer: &signer, broker: <b>address</b>): (AssetT, <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;RewardTokenT&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_unstake">unstake</a>&lt;PoolType: store, RewardTokenT: store, AssetT: store&gt;(
    signer: &signer,
    broker: <b>address</b>)
: (AssetT, <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;RewardTokenT&gt;) <b>acquires</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_HarvestCapability">HarvestCapability</a>, <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_Farming">Farming</a>, <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_FarmingAsset">FarmingAsset</a>, <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_Stake">Stake</a> {
    <b>let</b> account = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer);
    <b>let</b> cap = <b>move_from</b>&lt;<a href="YieldFarmingV2.md#0x1_YieldFarmingV2_HarvestCapability">HarvestCapability</a>&lt;PoolType, AssetT&gt;&gt;(account);
    <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_unstake_with_cap">unstake_with_cap</a>(broker, cap)
}
</code></pre>



</details>

<a name="0x1_YieldFarmingV2_unstake_with_cap"></a>

## Function `unstake_with_cap`



<pre><code><b>public</b> <b>fun</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_unstake_with_cap">unstake_with_cap</a>&lt;PoolType: store, RewardTokenT: store, AssetT: store&gt;(broker: <b>address</b>, cap: <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_HarvestCapability">YieldFarmingV2::HarvestCapability</a>&lt;PoolType, AssetT&gt;): (AssetT, <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;RewardTokenT&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_unstake_with_cap">unstake_with_cap</a>&lt;PoolType: store, RewardTokenT: store, AssetT: store&gt;(
    broker: <b>address</b>,
    cap: <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_HarvestCapability">HarvestCapability</a>&lt;PoolType, AssetT&gt;)
: (AssetT, <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;RewardTokenT&gt;) <b>acquires</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_Farming">Farming</a>, <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_FarmingAsset">FarmingAsset</a>, <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_Stake">Stake</a> {
    // Destroy capability
    <b>let</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_HarvestCapability">HarvestCapability</a>&lt;PoolType, AssetT&gt; { trigger } = cap;

    <b>let</b> farming = <b>borrow_global_mut</b>&lt;<a href="YieldFarmingV2.md#0x1_YieldFarmingV2_Farming">Farming</a>&lt;PoolType, RewardTokenT&gt;&gt;(broker);
    <b>let</b> farming_asset = <b>borrow_global_mut</b>&lt;<a href="YieldFarmingV2.md#0x1_YieldFarmingV2_FarmingAsset">FarmingAsset</a>&lt;PoolType, AssetT&gt;&gt;(broker);

    <b>let</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_Stake">Stake</a>&lt;PoolType, AssetT&gt; { last_harvest_index, asset_weight, asset, gain } =
        <b>move_from</b>&lt;<a href="YieldFarmingV2.md#0x1_YieldFarmingV2_Stake">Stake</a>&lt;PoolType, AssetT&gt;&gt;(trigger);

    <b>let</b> now_seconds = <a href="Timestamp.md#0x1_Timestamp_now_seconds">Timestamp::now_seconds</a>();
    <b>let</b> new_harvest_index = <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_calculate_harvest_index_with_asset">calculate_harvest_index_with_asset</a>&lt;PoolType, AssetT&gt;(farming_asset, now_seconds);

    <b>let</b> period_gain = <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_calculate_withdraw_amount">calculate_withdraw_amount</a>(new_harvest_index, last_harvest_index, asset_weight);
    <b>let</b> total_gain = gain + period_gain;
    <b>let</b> withdraw_token = <a href="Token.md#0x1_Token_withdraw">Token::withdraw</a>&lt;RewardTokenT&gt;(&<b>mut</b> farming.treasury_token, total_gain);

    // Dont <b>update</b> harvest index that because the `<a href="YieldFarmingV2.md#0x1_YieldFarmingV2_Stake">Stake</a>` object <b>has</b> droped.
    // <b>let</b> new_index = <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_calculate_harvest_index_with_asset">calculate_harvest_index_with_asset</a>&lt;PoolType, AssetT&gt;(farming_asset, now_seconds);
    <b>assert</b>!(farming_asset.asset_total_weight &gt;= asset_weight, <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="YieldFarmingV2.md#0x1_YieldFarmingV2_ERR_FARMING_NOT_ENOUGH_ASSET">ERR_FARMING_NOT_ENOUGH_ASSET</a>));

    // Update farm asset
    farming_asset.asset_total_weight = farming_asset.asset_total_weight - asset_weight;
    farming_asset.last_update_timestamp = now_seconds;

    <b>if</b> (farming_asset.alive) {
        farming_asset.harvest_index = new_harvest_index;
    };

    (asset, withdraw_token)
}
</code></pre>



</details>

<a name="0x1_YieldFarmingV2_harvest"></a>

## Function `harvest`

Harvest yield farming token from stake


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_harvest">harvest</a>&lt;PoolType: store, RewardTokenT: store, AssetT: store&gt;(signer: &signer, broker: <b>address</b>, amount: u128): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;RewardTokenT&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_harvest">harvest</a>&lt;PoolType: store,
                   RewardTokenT: store,
                   AssetT: store&gt;(
    signer: &signer,
    broker: <b>address</b>,
    amount: u128) : <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;RewardTokenT&gt; <b>acquires</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_HarvestCapability">HarvestCapability</a>, <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_Farming">Farming</a>, <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_FarmingAsset">FarmingAsset</a>, <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_Stake">Stake</a> {
    <b>let</b> account = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer);
    <b>let</b> cap = <b>borrow_global_mut</b>&lt;<a href="YieldFarmingV2.md#0x1_YieldFarmingV2_HarvestCapability">HarvestCapability</a>&lt;PoolType, AssetT&gt;&gt;(account);
    <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_harvest_with_cap">harvest_with_cap</a>(broker, amount, cap)
}
</code></pre>



</details>

<a name="0x1_YieldFarmingV2_harvest_with_cap"></a>

## Function `harvest_with_cap`



<pre><code><b>public</b> <b>fun</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_harvest_with_cap">harvest_with_cap</a>&lt;PoolType: store, RewardTokenT: store, AssetT: store&gt;(broker: <b>address</b>, amount: u128, _cap: &<a href="YieldFarmingV2.md#0x1_YieldFarmingV2_HarvestCapability">YieldFarmingV2::HarvestCapability</a>&lt;PoolType, AssetT&gt;): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;RewardTokenT&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_harvest_with_cap">harvest_with_cap</a>&lt;PoolType: store,
                            RewardTokenT: store,
                            AssetT: store&gt;(
    broker: <b>address</b>,
    amount: u128,
    _cap: &<a href="YieldFarmingV2.md#0x1_YieldFarmingV2_HarvestCapability">HarvestCapability</a>&lt;PoolType, AssetT&gt;): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;RewardTokenT&gt; <b>acquires</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_Farming">Farming</a>, <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_FarmingAsset">FarmingAsset</a>, <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_Stake">Stake</a> {
    <b>let</b> farming = <b>borrow_global_mut</b>&lt;<a href="YieldFarmingV2.md#0x1_YieldFarmingV2_Farming">Farming</a>&lt;PoolType, RewardTokenT&gt;&gt;(broker);
    <b>let</b> farming_asset = <b>borrow_global_mut</b>&lt;<a href="YieldFarmingV2.md#0x1_YieldFarmingV2_FarmingAsset">FarmingAsset</a>&lt;PoolType, AssetT&gt;&gt;(broker);
    <b>let</b> stake = <b>borrow_global_mut</b>&lt;<a href="YieldFarmingV2.md#0x1_YieldFarmingV2_Stake">Stake</a>&lt;PoolType, AssetT&gt;&gt;(_cap.trigger);

    <b>let</b> now_seconds = <a href="Timestamp.md#0x1_Timestamp_now_seconds">Timestamp::now_seconds</a>();
    <b>let</b> new_harvest_index = <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_calculate_harvest_index_with_asset">calculate_harvest_index_with_asset</a>&lt;PoolType, AssetT&gt;(farming_asset, now_seconds);

    <b>let</b> period_gain = <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_calculate_withdraw_amount">calculate_withdraw_amount</a>(
        new_harvest_index,
        stake.last_harvest_index,
        stake.asset_weight
    );

    <b>let</b> total_gain = stake.gain + period_gain;
    //<b>assert</b>!(total_gain &gt; 0, <a href="Errors.md#0x1_Errors_limit_exceeded">Errors::limit_exceeded</a>(<a href="YieldFarmingV2.md#0x1_YieldFarmingV2_ERR_FARMING_HAVERST_NO_GAIN">ERR_FARMING_HAVERST_NO_GAIN</a>));
    <b>assert</b>!(total_gain &gt;= amount, <a href="Errors.md#0x1_Errors_limit_exceeded">Errors::limit_exceeded</a>(<a href="YieldFarmingV2.md#0x1_YieldFarmingV2_ERR_FARMING_BALANCE_EXCEEDED">ERR_FARMING_BALANCE_EXCEEDED</a>));

    <b>let</b> withdraw_amount = <b>if</b> (amount &lt;= 0) {
        total_gain
    } <b>else</b> {
        amount
    };

    <b>let</b> withdraw_token = <a href="Token.md#0x1_Token_withdraw">Token::withdraw</a>&lt;RewardTokenT&gt;(&<b>mut</b> farming.treasury_token, withdraw_amount);
    stake.gain = total_gain - withdraw_amount;
    stake.last_harvest_index = new_harvest_index;

    <b>if</b> (farming_asset.alive) {
        farming_asset.harvest_index = new_harvest_index;
    };
    farming_asset.last_update_timestamp = now_seconds;

    withdraw_token
}
</code></pre>



</details>

<a name="0x1_YieldFarmingV2_query_gov_token_amount"></a>

## Function `query_gov_token_amount`

The user can quering all yield farming amount in any time and scene


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_query_gov_token_amount">query_gov_token_amount</a>&lt;PoolType: store, RewardTokenT: store, AssetT: store&gt;(account: <b>address</b>, broker: <b>address</b>): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_query_gov_token_amount">query_gov_token_amount</a>&lt;PoolType: store,
                                  RewardTokenT: store,
                                  AssetT: store&gt;(account: <b>address</b>, broker: <b>address</b>): u128 <b>acquires</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_FarmingAsset">FarmingAsset</a>, <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_Stake">Stake</a> {
    <b>let</b> farming_asset = <b>borrow_global_mut</b>&lt;<a href="YieldFarmingV2.md#0x1_YieldFarmingV2_FarmingAsset">FarmingAsset</a>&lt;PoolType, AssetT&gt;&gt;(broker);
    <b>let</b> stake = <b>borrow_global_mut</b>&lt;<a href="YieldFarmingV2.md#0x1_YieldFarmingV2_Stake">Stake</a>&lt;PoolType, AssetT&gt;&gt;(account);
    <b>let</b> now_seconds = <a href="Timestamp.md#0x1_Timestamp_now_seconds">Timestamp::now_seconds</a>();

    <b>let</b> new_harvest_index = <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_calculate_harvest_index_with_asset">calculate_harvest_index_with_asset</a>&lt;PoolType, AssetT&gt;(
        farming_asset,
        now_seconds
    );

    <b>let</b> new_gain = <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_calculate_withdraw_amount">calculate_withdraw_amount</a>(
        new_harvest_index,
        stake.last_harvest_index,
        stake.asset_weight
    );
    stake.gain + new_gain
}
</code></pre>



</details>

<a name="0x1_YieldFarmingV2_query_total_stake"></a>

## Function `query_total_stake`

Query total stake count from yield farming resource


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_query_total_stake">query_total_stake</a>&lt;PoolType: store, AssetT: store&gt;(broker: <b>address</b>): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_query_total_stake">query_total_stake</a>&lt;PoolType: store,
                             AssetT: store&gt;(broker: <b>address</b>): u128 <b>acquires</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_FarmingAsset">FarmingAsset</a> {
    <b>let</b> farming_asset = <b>borrow_global_mut</b>&lt;<a href="YieldFarmingV2.md#0x1_YieldFarmingV2_FarmingAsset">FarmingAsset</a>&lt;PoolType, AssetT&gt;&gt;(broker);
    farming_asset.asset_total_weight
}
</code></pre>



</details>

<a name="0x1_YieldFarmingV2_query_stake"></a>

## Function `query_stake`

Query stake weight from user staking objects.


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_query_stake">query_stake</a>&lt;PoolType: store, AssetT: store&gt;(account: <b>address</b>): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_query_stake">query_stake</a>&lt;PoolType: store,
                       AssetT: store&gt;(account: <b>address</b>): u128 <b>acquires</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_Stake">Stake</a> {
    <b>let</b> stake = <b>borrow_global_mut</b>&lt;<a href="YieldFarmingV2.md#0x1_YieldFarmingV2_Stake">Stake</a>&lt;PoolType, AssetT&gt;&gt;(account);
    stake.asset_weight
}
</code></pre>



</details>

<a name="0x1_YieldFarmingV2_query_info"></a>

## Function `query_info`

Queyry pool info from pool type
return value: (alive, release_per_second, asset_total_weight, harvest_index)


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_query_info">query_info</a>&lt;PoolType: store, AssetT: store&gt;(broker: <b>address</b>): (bool, u128, u128, u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_query_info">query_info</a>&lt;PoolType: store, AssetT: store&gt;(broker: <b>address</b>): (bool, u128, u128, u128) <b>acquires</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_FarmingAsset">FarmingAsset</a> {
    <b>let</b> asset = <b>borrow_global_mut</b>&lt;<a href="YieldFarmingV2.md#0x1_YieldFarmingV2_FarmingAsset">FarmingAsset</a>&lt;PoolType, AssetT&gt;&gt;(broker);
    (
        asset.alive,
        asset.release_per_second,
        asset.asset_total_weight,
        asset.harvest_index
    )
}
</code></pre>



</details>

<a name="0x1_YieldFarmingV2_calculate_harvest_index_with_asset"></a>

## Function `calculate_harvest_index_with_asset`

Update farming asset


<pre><code><b>fun</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_calculate_harvest_index_with_asset">calculate_harvest_index_with_asset</a>&lt;PoolType, AssetT&gt;(farming_asset: &<a href="YieldFarmingV2.md#0x1_YieldFarmingV2_FarmingAsset">YieldFarmingV2::FarmingAsset</a>&lt;PoolType, AssetT&gt;, now_seconds: u64): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_calculate_harvest_index_with_asset">calculate_harvest_index_with_asset</a>&lt;PoolType, AssetT&gt;(farming_asset: &<a href="YieldFarmingV2.md#0x1_YieldFarmingV2_FarmingAsset">FarmingAsset</a>&lt;PoolType, AssetT&gt;, now_seconds: u64): u128 {
    // Recalculate harvest index
    <b>if</b> (farming_asset.asset_total_weight &lt;= 0) {
        <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_calculate_harvest_index_weight_zero">calculate_harvest_index_weight_zero</a>(
            farming_asset.harvest_index,
            farming_asset.last_update_timestamp,
            now_seconds,
            farming_asset.release_per_second
        )
    } <b>else</b> {
        <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_calculate_harvest_index">calculate_harvest_index</a>(
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

<a name="0x1_YieldFarmingV2_calculate_harvest_index_weight_zero"></a>

## Function `calculate_harvest_index_weight_zero`

There is calculating from harvest index and global parameters without asset_total_weight


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_calculate_harvest_index_weight_zero">calculate_harvest_index_weight_zero</a>(harvest_index: u128, last_update_timestamp: u64, now_seconds: u64, release_per_second: u128): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_calculate_harvest_index_weight_zero">calculate_harvest_index_weight_zero</a>(harvest_index: u128,
                                               last_update_timestamp: u64,
                                               now_seconds: u64,
                                               release_per_second: u128): u128 {
    <b>assert</b>!(last_update_timestamp &lt;= now_seconds, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="YieldFarmingV2.md#0x1_YieldFarmingV2_ERR_FARMING_TIMESTAMP_INVALID">ERR_FARMING_TIMESTAMP_INVALID</a>));
    <b>let</b> time_period = now_seconds - last_update_timestamp;
    <b>let</b> addtion_index = release_per_second * ((time_period <b>as</b> u128));
    harvest_index + <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_mantissa">mantissa</a>(<a href="YieldFarmingV2.md#0x1_YieldFarmingV2_exp_direct_expand">exp_direct_expand</a>(addtion_index))
}
</code></pre>



</details>

<a name="0x1_YieldFarmingV2_calculate_harvest_index"></a>

## Function `calculate_harvest_index`

There is calculating from harvest index and global parameters


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_calculate_harvest_index">calculate_harvest_index</a>(harvest_index: u128, asset_total_weight: u128, last_update_timestamp: u64, now_seconds: u64, release_per_second: u128): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_calculate_harvest_index">calculate_harvest_index</a>(harvest_index: u128,
                                   asset_total_weight: u128,
                                   last_update_timestamp: u64,
                                   now_seconds: u64,
                                   release_per_second: u128): u128 {
    <b>assert</b>!(asset_total_weight &gt; 0, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="YieldFarmingV2.md#0x1_YieldFarmingV2_ERR_FARMING_TOTAL_WEIGHT_IS_ZERO">ERR_FARMING_TOTAL_WEIGHT_IS_ZERO</a>));
    <b>assert</b>!(last_update_timestamp &lt;= now_seconds, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="YieldFarmingV2.md#0x1_YieldFarmingV2_ERR_FARMING_TIMESTAMP_INVALID">ERR_FARMING_TIMESTAMP_INVALID</a>));

    <b>let</b> time_period = now_seconds - last_update_timestamp;
    <b>let</b> numr = (release_per_second * (time_period <b>as</b> u128));
    <b>let</b> denom = asset_total_weight;
    harvest_index + <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_mantissa">mantissa</a>(<a href="YieldFarmingV2.md#0x1_YieldFarmingV2_exp">exp</a>(numr, denom))
}
</code></pre>



</details>

<a name="0x1_YieldFarmingV2_calculate_withdraw_amount"></a>

## Function `calculate_withdraw_amount`

This function will return a gain index


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_calculate_withdraw_amount">calculate_withdraw_amount</a>(harvest_index: u128, last_harvest_index: u128, asset_weight: u128): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_calculate_withdraw_amount">calculate_withdraw_amount</a>(harvest_index: u128,
                                     last_harvest_index: u128,
                                     asset_weight: u128): u128 {
    <b>assert</b>!(harvest_index &gt;= last_harvest_index, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="YieldFarmingV2.md#0x1_YieldFarmingV2_ERR_FARMING_CALC_LAST_IDX_BIGGER_THAN_NOW">ERR_FARMING_CALC_LAST_IDX_BIGGER_THAN_NOW</a>));
    <b>let</b> amount = asset_weight * (harvest_index - last_harvest_index);
    <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_truncate">truncate</a>(<a href="YieldFarmingV2.md#0x1_YieldFarmingV2_exp_direct">exp_direct</a>(amount))
}
</code></pre>



</details>

<a name="0x1_YieldFarmingV2_exists_at"></a>

## Function `exists_at`

Check the Farming of TokenT is exists.


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_exists_at">exists_at</a>&lt;PoolType: store, RewardTokenT: store&gt;(broker: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_exists_at">exists_at</a>&lt;PoolType: store, RewardTokenT: store&gt;(broker: <b>address</b>): bool {
    <b>exists</b>&lt;<a href="YieldFarmingV2.md#0x1_YieldFarmingV2_Farming">Farming</a>&lt;PoolType, RewardTokenT&gt;&gt;(broker)
}
</code></pre>



</details>

<a name="0x1_YieldFarmingV2_exists_asset_at"></a>

## Function `exists_asset_at`

Check the Farming of AsssetT is exists.


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_exists_asset_at">exists_asset_at</a>&lt;PoolType: store, AssetT: store&gt;(broker: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_exists_asset_at">exists_asset_at</a>&lt;PoolType: store, AssetT: store&gt;(broker: <b>address</b>): bool {
    <b>exists</b>&lt;<a href="YieldFarmingV2.md#0x1_YieldFarmingV2_FarmingAsset">FarmingAsset</a>&lt;PoolType, AssetT&gt;&gt;(broker)
}
</code></pre>



</details>

<a name="0x1_YieldFarmingV2_exists_stake_at_address"></a>

## Function `exists_stake_at_address`

Check stake at address exists.


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_exists_stake_at_address">exists_stake_at_address</a>&lt;PoolType: store, AssetT: store&gt;(account: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarmingV2.md#0x1_YieldFarmingV2_exists_stake_at_address">exists_stake_at_address</a>&lt;PoolType: store, AssetT: store&gt;(account: <b>address</b>): bool {
    <b>exists</b>&lt;<a href="YieldFarmingV2.md#0x1_YieldFarmingV2_Stake">Stake</a>&lt;PoolType, AssetT&gt;&gt;(account)
}
</code></pre>



</details>

<a name="@Specification_1"></a>

## Specification



<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>
