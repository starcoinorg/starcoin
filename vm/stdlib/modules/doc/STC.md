
<a name="0x1_STC"></a>

# Module `0x1::STC`

### Table of Contents

-  [Struct `STC`](#0x1_STC_STC)
-  [Resource `SharedBurnCapability`](#0x1_STC_SharedBurnCapability)
-  [Const `BASE_SCALING_FACTOR`](#0x1_STC_BASE_SCALING_FACTOR)
-  [Const `FRACTIONAL_PART`](#0x1_STC_FRACTIONAL_PART)
-  [Function `initialize`](#0x1_STC_initialize)
-  [Function `is_stc`](#0x1_STC_is_stc)
-  [Function `burn`](#0x1_STC_burn)
-  [Function `token_address`](#0x1_STC_token_address)
-  [Specification](#0x1_STC_Specification)
    -  [Function `initialize`](#0x1_STC_Specification_initialize)
    -  [Function `is_stc`](#0x1_STC_Specification_is_stc)
    -  [Function `burn`](#0x1_STC_Specification_burn)
    -  [Function `token_address`](#0x1_STC_Specification_token_address)



<a name="0x1_STC_STC"></a>

## Struct `STC`



<pre><code><b>struct</b> <a href="#0x1_STC">STC</a>
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

<a name="0x1_STC_SharedBurnCapability"></a>

## Resource `SharedBurnCapability`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_STC_SharedBurnCapability">SharedBurnCapability</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>cap: <a href="Token.md#0x1_Token_BurnCapability">Token::BurnCapability</a>&lt;<a href="#0x1_STC_STC">STC::STC</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_STC_BASE_SCALING_FACTOR"></a>

## Const `BASE_SCALING_FACTOR`

scaling_factor = 10^6


<pre><code><b>const</b> BASE_SCALING_FACTOR: u128 = 1000000;
</code></pre>



<a name="0x1_STC_FRACTIONAL_PART"></a>

## Const `FRACTIONAL_PART`

fractional_part = 10^3


<pre><code><b>const</b> FRACTIONAL_PART: u128 = 1000;
</code></pre>



<a name="0x1_STC_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_STC_initialize">initialize</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_STC_initialize">initialize</a>(account: &signer) {
    <a href="Token.md#0x1_Token_register_token">Token::register_token</a>&lt;<a href="#0x1_STC">STC</a>&gt;(account, BASE_SCALING_FACTOR, FRACTIONAL_PART);
    <b>let</b> burn_cap = <a href="Token.md#0x1_Token_remove_burn_capability">Token::remove_burn_capability</a>&lt;<a href="#0x1_STC">STC</a>&gt;(account);
    move_to(account, <a href="#0x1_STC_SharedBurnCapability">SharedBurnCapability</a> { cap: burn_cap });
    <a href="Dao.md#0x1_Dao_plugin">Dao::plugin</a>&lt;<a href="#0x1_STC">STC</a>&gt;(account);
    <a href="ModifyDaoConfigProposal.md#0x1_ModifyDaoConfigProposal_plugin">ModifyDaoConfigProposal::plugin</a>&lt;<a href="#0x1_STC">STC</a>&gt;(account);
    <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_plugin">UpgradeModuleDaoProposal::plugin</a>&lt;<a href="#0x1_STC">STC</a>&gt;(account);
    <b>let</b> upgrade_plan_cap = <a href="PackageTxnManager.md#0x1_PackageTxnManager_extract_submit_upgrade_plan_cap">PackageTxnManager::extract_submit_upgrade_plan_cap</a>(account);
    <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_delegate_module_upgrade_capability">UpgradeModuleDaoProposal::delegate_module_upgrade_capability</a>&lt;<a href="#0x1_STC">STC</a>&gt;(
        account,
        upgrade_plan_cap,
    );
}
</code></pre>



</details>

<a name="0x1_STC_is_stc"></a>

## Function `is_stc`

Returns true if
<code>TokenType</code> is
<code><a href="#0x1_STC_STC">STC::STC</a></code>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_STC_is_stc">is_stc</a>&lt;TokenType&gt;(): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_STC_is_stc">is_stc</a>&lt;TokenType&gt;(): bool {
    <a href="Token.md#0x1_Token_is_same_token">Token::is_same_token</a>&lt;<a href="#0x1_STC">STC</a>, TokenType&gt;()
}
</code></pre>



</details>

<a name="0x1_STC_burn"></a>

## Function `burn`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_STC_burn">burn</a>(token: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;<a href="#0x1_STC_STC">STC::STC</a>&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_STC_burn">burn</a>(token: <a href="Token.md#0x1_Token">Token</a>&lt;<a href="#0x1_STC">STC</a>&gt;) <b>acquires</b> <a href="#0x1_STC_SharedBurnCapability">SharedBurnCapability</a> {
    <b>let</b> cap = borrow_global&lt;<a href="#0x1_STC_SharedBurnCapability">SharedBurnCapability</a>&gt;(<a href="#0x1_STC_token_address">token_address</a>());
    <a href="Token.md#0x1_Token_burn_with_capability">Token::burn_with_capability</a>(&cap.cap, token);
}
</code></pre>



</details>

<a name="0x1_STC_token_address"></a>

## Function `token_address`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_STC_token_address">token_address</a>(): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_STC_token_address">token_address</a>(): address {
    <a href="Token.md#0x1_Token_token_address">Token::token_address</a>&lt;<a href="#0x1_STC">STC</a>&gt;()
}
</code></pre>



</details>

<a name="0x1_STC_Specification"></a>

## Specification



<pre><code>pragma verify;
pragma aborts_if_is_strict;
</code></pre>



<a name="0x1_STC_Specification_initialize"></a>

### Function `initialize`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_STC_initialize">initialize</a>(account: &signer)
</code></pre>




<pre><code>pragma verify = <b>false</b>;
</code></pre>



<a name="0x1_STC_Specification_is_stc"></a>

### Function `is_stc`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_STC_is_stc">is_stc</a>&lt;TokenType&gt;(): bool
</code></pre>




<a name="0x1_STC_Specification_burn"></a>

### Function `burn`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_STC_burn">burn</a>(token: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;<a href="#0x1_STC_STC">STC::STC</a>&gt;)
</code></pre>




<pre><code>pragma verify = <b>false</b>;
</code></pre>



<a name="0x1_STC_Specification_token_address"></a>

### Function `token_address`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_STC_token_address">token_address</a>(): address
</code></pre>
