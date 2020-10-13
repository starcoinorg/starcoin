
<a name="0x1_STC"></a>

# Module `0x1::STC`



-  [Struct <code><a href="STC.md#0x1_STC">STC</a></code>](#0x1_STC_STC)
-  [Resource <code><a href="STC.md#0x1_STC_SharedBurnCapability">SharedBurnCapability</a></code>](#0x1_STC_SharedBurnCapability)
-  [Const <code><a href="STC.md#0x1_STC_PRECISION">PRECISION</a></code>](#0x1_STC_PRECISION)
-  [Function <code>initialize</code>](#0x1_STC_initialize)
-  [Function <code>is_stc</code>](#0x1_STC_is_stc)
-  [Function <code>burn</code>](#0x1_STC_burn)
-  [Function <code>token_address</code>](#0x1_STC_token_address)
-  [Specification](#@Specification_0)
    -  [Function <code>initialize</code>](#@Specification_0_initialize)
    -  [Function <code>is_stc</code>](#@Specification_0_is_stc)
    -  [Function <code>burn</code>](#@Specification_0_burn)
    -  [Function <code>token_address</code>](#@Specification_0_token_address)


<a name="0x1_STC_STC"></a>

## Struct `STC`



<pre><code><b>struct</b> <a href="STC.md#0x1_STC">STC</a>
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



<pre><code><b>resource</b> <b>struct</b> <a href="STC.md#0x1_STC_SharedBurnCapability">SharedBurnCapability</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>cap: <a href="Token.md#0x1_Token_BurnCapability">Token::BurnCapability</a>&lt;<a href="STC.md#0x1_STC_STC">STC::STC</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_STC_PRECISION"></a>

## Const `PRECISION`

precision of STC token.


<pre><code><b>const</b> <a href="STC.md#0x1_STC_PRECISION">PRECISION</a>: u8 = 9;
</code></pre>



<a name="0x1_STC_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="STC.md#0x1_STC_initialize">initialize</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="STC.md#0x1_STC_initialize">initialize</a>(account: &signer) {
    <a href="Token.md#0x1_Token_register_token">Token::register_token</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;(account, <a href="STC.md#0x1_STC_PRECISION">PRECISION</a>);
    <b>let</b> burn_cap = <a href="Token.md#0x1_Token_remove_burn_capability">Token::remove_burn_capability</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;(account);
    move_to(account, <a href="STC.md#0x1_STC_SharedBurnCapability">SharedBurnCapability</a> { cap: burn_cap });
    <a href="Dao.md#0x1_Dao_plugin">Dao::plugin</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;(
        account,
        <a href="Dao.md#0x1_Dao_default_voting_delay">Dao::default_voting_delay</a>(),
        <a href="Dao.md#0x1_Dao_default_voting_period">Dao::default_voting_period</a>(),
        <a href="Dao.md#0x1_Dao_default_voting_quorum_rate">Dao::default_voting_quorum_rate</a>(),
        <a href="Dao.md#0x1_Dao_default_min_action_delay">Dao::default_min_action_delay</a>(),
    );
    <a href="ModifyDaoConfigProposal.md#0x1_ModifyDaoConfigProposal_plugin">ModifyDaoConfigProposal::plugin</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;(account);
    <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_plugin">UpgradeModuleDaoProposal::plugin</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;(account);
    <b>let</b> upgrade_plan_cap = <a href="PackageTxnManager.md#0x1_PackageTxnManager_extract_submit_upgrade_plan_cap">PackageTxnManager::extract_submit_upgrade_plan_cap</a>(account);
    <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_delegate_module_upgrade_capability">UpgradeModuleDaoProposal::delegate_module_upgrade_capability</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;(
        account,
        upgrade_plan_cap,
    );
    // the following configurations are gov-ed by <a href="Dao.md#0x1_Dao">Dao</a>.
    <a href="OnChainConfigDao.md#0x1_OnChainConfigDao_plugin">OnChainConfigDao::plugin</a>&lt;<a href="STC.md#0x1_STC">STC</a>, <a href="TransactionPublishOption.md#0x1_TransactionPublishOption_TransactionPublishOption">TransactionPublishOption::TransactionPublishOption</a>&gt;(account);
    <a href="OnChainConfigDao.md#0x1_OnChainConfigDao_plugin">OnChainConfigDao::plugin</a>&lt;<a href="STC.md#0x1_STC">STC</a>, <a href="VMConfig.md#0x1_VMConfig_VMConfig">VMConfig::VMConfig</a>&gt;(account);
    <a href="OnChainConfigDao.md#0x1_OnChainConfigDao_plugin">OnChainConfigDao::plugin</a>&lt;<a href="STC.md#0x1_STC">STC</a>, <a href="Version.md#0x1_Version_Version">Version::Version</a>&gt;(account);
    <a href="OnChainConfigDao.md#0x1_OnChainConfigDao_plugin">OnChainConfigDao::plugin</a>&lt;<a href="STC.md#0x1_STC">STC</a>, <a href="ConsensusConfig.md#0x1_ConsensusConfig_ConsensusConfig">ConsensusConfig::ConsensusConfig</a>&gt;(account);
}
</code></pre>



</details>

<a name="0x1_STC_is_stc"></a>

## Function `is_stc`

Returns true if <code>TokenType</code> is <code><a href="STC.md#0x1_STC_STC">STC::STC</a></code>


<pre><code><b>public</b> <b>fun</b> <a href="STC.md#0x1_STC_is_stc">is_stc</a>&lt;TokenType&gt;(): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="STC.md#0x1_STC_is_stc">is_stc</a>&lt;TokenType&gt;(): bool {
    <a href="Token.md#0x1_Token_is_same_token">Token::is_same_token</a>&lt;<a href="STC.md#0x1_STC">STC</a>, TokenType&gt;()
}
</code></pre>



</details>

<a name="0x1_STC_burn"></a>

## Function `burn`



<pre><code><b>public</b> <b>fun</b> <a href="STC.md#0x1_STC_burn">burn</a>(token: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;<a href="STC.md#0x1_STC_STC">STC::STC</a>&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="STC.md#0x1_STC_burn">burn</a>(token: <a href="Token.md#0x1_Token">Token</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;) <b>acquires</b> <a href="STC.md#0x1_STC_SharedBurnCapability">SharedBurnCapability</a> {
    <b>let</b> cap = borrow_global&lt;<a href="STC.md#0x1_STC_SharedBurnCapability">SharedBurnCapability</a>&gt;(<a href="STC.md#0x1_STC_token_address">token_address</a>());
    <a href="Token.md#0x1_Token_burn_with_capability">Token::burn_with_capability</a>(&cap.cap, token);
}
</code></pre>



</details>

<a name="0x1_STC_token_address"></a>

## Function `token_address`



<pre><code><b>public</b> <b>fun</b> <a href="STC.md#0x1_STC_token_address">token_address</a>(): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="STC.md#0x1_STC_token_address">token_address</a>(): address {
    <a href="Token.md#0x1_Token_token_address">Token::token_address</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;()
}
</code></pre>



</details>

<a name="@Specification_0"></a>

## Specification



<pre><code>pragma verify;
pragma aborts_if_is_strict;
</code></pre>



<a name="@Specification_0_initialize"></a>

### Function `initialize`


<pre><code><b>public</b> <b>fun</b> <a href="STC.md#0x1_STC_initialize">initialize</a>(account: &signer)
</code></pre>




<pre><code>pragma verify = <b>false</b>;
</code></pre>



<a name="@Specification_0_is_stc"></a>

### Function `is_stc`


<pre><code><b>public</b> <b>fun</b> <a href="STC.md#0x1_STC_is_stc">is_stc</a>&lt;TokenType&gt;(): bool
</code></pre>




<a name="@Specification_0_burn"></a>

### Function `burn`


<pre><code><b>public</b> <b>fun</b> <a href="STC.md#0x1_STC_burn">burn</a>(token: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;<a href="STC.md#0x1_STC_STC">STC::STC</a>&gt;)
</code></pre>




<pre><code>pragma verify = <b>false</b>;
</code></pre>



<a name="@Specification_0_token_address"></a>

### Function `token_address`


<pre><code><b>public</b> <b>fun</b> <a href="STC.md#0x1_STC_token_address">token_address</a>(): address
</code></pre>
