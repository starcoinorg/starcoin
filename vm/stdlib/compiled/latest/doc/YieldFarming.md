
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
<b>use</b> <a href="Token.md#0x1_Token">0x1::Token</a>;
</code></pre>



<a name="0x1_YieldFarming_Farming"></a>

## Resource `Farming`

The object of yield farming
RewardTokenT meaning token of yield farming


<pre><code><b>struct</b> <a href="YieldFarming.md#0x1_YieldFarming_Farming">Farming</a>&lt;PoolType, RewardTokenT&gt; <b>has</b> store, key
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



<pre><code><b>struct</b> <a href="YieldFarming.md#0x1_YieldFarming_FarmingAsset">FarmingAsset</a>&lt;PoolType, AssetT&gt; <b>has</b> store, key
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


<pre><code><b>struct</b> <a href="YieldFarming.md#0x1_YieldFarming_ParameterModifyCapability">ParameterModifyCapability</a>&lt;PoolType, AssetT&gt; <b>has</b> store, key
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


<pre><code><b>struct</b> <a href="YieldFarming.md#0x1_YieldFarming_Stake">Stake</a>&lt;PoolType, AssetT&gt; <b>has</b> store, key
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



<pre><code><b>struct</b> <a href="YieldFarming.md#0x1_YieldFarming_Exp">Exp</a> <b>has</b> <b>copy</b>, drop, store
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


<a name="0x1_YieldFarming_EDEPRECATED_FUNCTION"></a>



<pre><code><b>const</b> <a href="YieldFarming.md#0x1_YieldFarming_EDEPRECATED_FUNCTION">EDEPRECATED_FUNCTION</a>: u64 = 19;
</code></pre>



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


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_initialize">initialize</a>&lt;PoolType: store, RewardTokenT: store&gt;(_account: &signer, _treasury_token: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;RewardTokenT&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_initialize">initialize</a>&lt;
    PoolType: store,
    RewardTokenT: store&gt;(_account: &signer,
                         _treasury_token: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;RewardTokenT&gt;) {
    <b>abort</b> <a href="Errors.md#0x1_Errors_deprecated">Errors::deprecated</a>(<a href="YieldFarming.md#0x1_YieldFarming_EDEPRECATED_FUNCTION">EDEPRECATED_FUNCTION</a>)
}
</code></pre>



</details>

<a name="0x1_YieldFarming_initialize_asset"></a>

## Function `initialize_asset`



<pre><code><b>public</b> <b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_initialize_asset">initialize_asset</a>&lt;PoolType: store, AssetT: store&gt;(_account: &signer, _release_per_second: u128, _delay: u64): <a href="YieldFarming.md#0x1_YieldFarming_ParameterModifyCapability">YieldFarming::ParameterModifyCapability</a>&lt;PoolType, AssetT&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_initialize_asset">initialize_asset</a>&lt;PoolType: store, AssetT: store&gt;(
    _account: &signer,
    _release_per_second: u128,
    _delay: u64): <a href="YieldFarming.md#0x1_YieldFarming_ParameterModifyCapability">ParameterModifyCapability</a>&lt;PoolType, AssetT&gt; {
    <b>abort</b> <a href="Errors.md#0x1_Errors_deprecated">Errors::deprecated</a>(<a href="YieldFarming.md#0x1_YieldFarming_EDEPRECATED_FUNCTION">EDEPRECATED_FUNCTION</a>)
}
</code></pre>



</details>

<a name="0x1_YieldFarming_modify_parameter"></a>

## Function `modify_parameter`



<pre><code><b>public</b> <b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_modify_parameter">modify_parameter</a>&lt;PoolType: store, RewardTokenT: store, AssetT: store&gt;(_cap: &<a href="YieldFarming.md#0x1_YieldFarming_ParameterModifyCapability">YieldFarming::ParameterModifyCapability</a>&lt;PoolType, AssetT&gt;, _broker: <b>address</b>, _release_per_second: u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_modify_parameter">modify_parameter</a>&lt;PoolType: store, RewardTokenT: store, AssetT: store&gt;(
    _cap: &<a href="YieldFarming.md#0x1_YieldFarming_ParameterModifyCapability">ParameterModifyCapability</a>&lt;PoolType, AssetT&gt;,
    _broker: <b>address</b>,
    _release_per_second: u128) {
    <b>abort</b> <a href="Errors.md#0x1_Errors_deprecated">Errors::deprecated</a>(<a href="YieldFarming.md#0x1_YieldFarming_EDEPRECATED_FUNCTION">EDEPRECATED_FUNCTION</a>)
}
</code></pre>



</details>

<a name="0x1_YieldFarming_stake"></a>

## Function `stake`

Call by stake user, staking amount of asset in order to get yield farming token


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_stake">stake</a>&lt;PoolType: store, RewardTokenT: store, AssetT: store&gt;(_account: &signer, _broker: <b>address</b>, _asset: AssetT, _asset_weight: u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_stake">stake</a>&lt;PoolType: store, RewardTokenT: store, AssetT: store&gt;(
    _account: &signer,
    _broker: <b>address</b>,
    _asset: AssetT,
    _asset_weight: u128) {
    <b>abort</b> <a href="Errors.md#0x1_Errors_deprecated">Errors::deprecated</a>(<a href="YieldFarming.md#0x1_YieldFarming_EDEPRECATED_FUNCTION">EDEPRECATED_FUNCTION</a>)
}
</code></pre>



</details>

<a name="0x1_YieldFarming_unstake"></a>

## Function `unstake`

Unstake asset from farming pool


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_unstake">unstake</a>&lt;PoolType: store, RewardTokenT: store, AssetT: store&gt;(_account: &signer, _broker: <b>address</b>): (AssetT, <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;RewardTokenT&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_unstake">unstake</a>&lt;PoolType: store, RewardTokenT: store, AssetT: store&gt;(_account: &signer, _broker: <b>address</b>)
: (AssetT, <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;RewardTokenT&gt;) {
    <b>abort</b> <a href="Errors.md#0x1_Errors_deprecated">Errors::deprecated</a>(<a href="YieldFarming.md#0x1_YieldFarming_EDEPRECATED_FUNCTION">EDEPRECATED_FUNCTION</a>)
}
</code></pre>



</details>

<a name="0x1_YieldFarming_harvest"></a>

## Function `harvest`

Harvest yield farming token from stake


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_harvest">harvest</a>&lt;PoolType: store, RewardTokenT: store, AssetT: store&gt;(_account: &signer, _broker: <b>address</b>, _amount: u128): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;RewardTokenT&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_harvest">harvest</a>&lt;PoolType: store,
                   RewardTokenT: store,
                   AssetT: store&gt;(
    _account: &signer,
    _broker: <b>address</b>,
    _amount: u128): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;RewardTokenT&gt; {
    <b>abort</b> <a href="Errors.md#0x1_Errors_deprecated">Errors::deprecated</a>(<a href="YieldFarming.md#0x1_YieldFarming_EDEPRECATED_FUNCTION">EDEPRECATED_FUNCTION</a>)
}
</code></pre>



</details>

<a name="0x1_YieldFarming_query_gov_token_amount"></a>

## Function `query_gov_token_amount`

The user can quering all yield farming amount in any time and scene


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_query_gov_token_amount">query_gov_token_amount</a>&lt;PoolType: store, RewardTokenT: store, AssetT: store&gt;(_account: &signer, _broker: <b>address</b>): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_query_gov_token_amount">query_gov_token_amount</a>&lt;PoolType: store,
                                  RewardTokenT: store,
                                  AssetT: store&gt;(_account: &signer, _broker: <b>address</b>): u128 {
    0
}
</code></pre>



</details>

<a name="0x1_YieldFarming_query_total_stake"></a>

## Function `query_total_stake`

Query total stake count from yield farming resource


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_query_total_stake">query_total_stake</a>&lt;PoolType: store, AssetT: store&gt;(_broker: <b>address</b>): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_query_total_stake">query_total_stake</a>&lt;PoolType: store,
                             AssetT: store&gt;(_broker: <b>address</b>): u128 {
    0
}
</code></pre>



</details>

<a name="0x1_YieldFarming_query_stake"></a>

## Function `query_stake`

Query stake weight from user staking objects.


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_query_stake">query_stake</a>&lt;PoolType: store, AssetT: store&gt;(_account: &signer): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_query_stake">query_stake</a>&lt;PoolType: store,
                       AssetT: store&gt;(_account: &signer): u128 {
    0
}
</code></pre>



</details>

<a name="0x1_YieldFarming_calculate_harvest_index_with_asset"></a>

## Function `calculate_harvest_index_with_asset`

Update farming asset


<pre><code><b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_calculate_harvest_index_with_asset">calculate_harvest_index_with_asset</a>&lt;PoolType, AssetT&gt;(_farming_asset: &<a href="YieldFarming.md#0x1_YieldFarming_FarmingAsset">YieldFarming::FarmingAsset</a>&lt;PoolType, AssetT&gt;, _now_seconds: u64): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_calculate_harvest_index_with_asset">calculate_harvest_index_with_asset</a>&lt;PoolType, AssetT&gt;(_farming_asset: &<a href="YieldFarming.md#0x1_YieldFarming_FarmingAsset">FarmingAsset</a>&lt;PoolType, AssetT&gt;, _now_seconds: u64): u128 {
    0
}
</code></pre>



</details>

<a name="0x1_YieldFarming_calculate_harvest_index_weight_zero"></a>

## Function `calculate_harvest_index_weight_zero`

There is calculating from harvest index and global parameters without asset_total_weight


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_calculate_harvest_index_weight_zero">calculate_harvest_index_weight_zero</a>(_harvest_index: u128, _last_update_timestamp: u64, _now_seconds: u64, _release_per_second: u128): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_calculate_harvest_index_weight_zero">calculate_harvest_index_weight_zero</a>(_harvest_index: u128,
                                               _last_update_timestamp: u64,
                                               _now_seconds: u64,
                                               _release_per_second: u128): u128 {
    0
}
</code></pre>



</details>

<a name="0x1_YieldFarming_calculate_harvest_index"></a>

## Function `calculate_harvest_index`

There is calculating from harvest index and global parameters


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_calculate_harvest_index">calculate_harvest_index</a>(_harvest_index: u128, _asset_total_weight: u128, _last_update_timestamp: u64, _now_seconds: u64, _release_per_second: u128): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_calculate_harvest_index">calculate_harvest_index</a>(_harvest_index: u128,
                                   _asset_total_weight: u128,
                                   _last_update_timestamp: u64,
                                   _now_seconds: u64,
                                   _release_per_second: u128): u128 {
    0
}
</code></pre>



</details>

<a name="0x1_YieldFarming_calculate_withdraw_amount"></a>

## Function `calculate_withdraw_amount`

This function will return a gain index


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_calculate_withdraw_amount">calculate_withdraw_amount</a>(_harvest_index: u128, _last_harvest_index: u128, _asset_weight: u128): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_calculate_withdraw_amount">calculate_withdraw_amount</a>(_harvest_index: u128,
                                     _last_harvest_index: u128,
                                     _asset_weight: u128): u128 {
    0
}
</code></pre>



</details>

<a name="0x1_YieldFarming_exists_at"></a>

## Function `exists_at`

Check the Farming of TokenT is exists.


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_exists_at">exists_at</a>&lt;PoolType: store, RewardTokenT: store&gt;(broker: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_exists_at">exists_at</a>&lt;PoolType: store, RewardTokenT: store&gt;(broker: <b>address</b>): bool {
    <b>exists</b>&lt;<a href="YieldFarming.md#0x1_YieldFarming_Farming">Farming</a>&lt;PoolType, RewardTokenT&gt;&gt;(broker)
}
</code></pre>



</details>

<a name="0x1_YieldFarming_exists_asset_at"></a>

## Function `exists_asset_at`

Check the Farming of AsssetT is exists.


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_exists_asset_at">exists_asset_at</a>&lt;PoolType: store, AssetT: store&gt;(broker: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_exists_asset_at">exists_asset_at</a>&lt;PoolType: store, AssetT: store&gt;(broker: <b>address</b>): bool {
    <b>exists</b>&lt;<a href="YieldFarming.md#0x1_YieldFarming_FarmingAsset">FarmingAsset</a>&lt;PoolType, AssetT&gt;&gt;(broker)
}
</code></pre>



</details>

<a name="0x1_YieldFarming_exists_stake_at_address"></a>

## Function `exists_stake_at_address`

Check stake at address exists.


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_exists_stake_at_address">exists_stake_at_address</a>&lt;PoolType: store, AssetT: store&gt;(account: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="YieldFarming.md#0x1_YieldFarming_exists_stake_at_address">exists_stake_at_address</a>&lt;PoolType: store, AssetT: store&gt;(account: <b>address</b>): bool {
    <b>exists</b>&lt;<a href="YieldFarming.md#0x1_YieldFarming_Stake">Stake</a>&lt;PoolType, AssetT&gt;&gt;(account)
}
</code></pre>



</details>

<a name="@Specification_1"></a>

## Specification



<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>
