
<a name="0x1_STC"></a>

# Module `0x1::STC`

STC is the token of Starcoin blockchain.
It uses apis defined in the <code><a href="Token.md#0x1_Token">Token</a></code> module.


-  [Struct `STC`](#0x1_STC_STC)
-  [Resource `SharedBurnCapability`](#0x1_STC_SharedBurnCapability)
-  [Constants](#@Constants_0)
-  [Function `initialize`](#0x1_STC_initialize)
-  [Function `upgrade_from_v1_to_v2`](#0x1_STC_upgrade_from_v1_to_v2)
-  [Function `initialize_v2`](#0x1_STC_initialize_v2)
-  [Function `is_stc`](#0x1_STC_is_stc)
-  [Function `burn`](#0x1_STC_burn)
-  [Function `token_address`](#0x1_STC_token_address)
-  [Specification](#@Specification_1)
    -  [Function `initialize`](#@Specification_1_initialize)
    -  [Function `upgrade_from_v1_to_v2`](#@Specification_1_upgrade_from_v1_to_v2)
    -  [Function `initialize_v2`](#@Specification_1_initialize_v2)
    -  [Function `is_stc`](#@Specification_1_is_stc)
    -  [Function `burn`](#@Specification_1_burn)
    -  [Function `token_address`](#@Specification_1_token_address)


<pre><code><b>use</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig">0x1::ConsensusConfig</a>;
<b>use</b> <a href="CoreAddresses.md#0x1_CoreAddresses">0x1::CoreAddresses</a>;
<b>use</b> <a href="Dao.md#0x1_Dao">0x1::Dao</a>;
<b>use</b> <a href="ModifyDaoConfigProposal.md#0x1_ModifyDaoConfigProposal">0x1::ModifyDaoConfigProposal</a>;
<b>use</b> <a href="OnChainConfigDao.md#0x1_OnChainConfigDao">0x1::OnChainConfigDao</a>;
<b>use</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager">0x1::PackageTxnManager</a>;
<b>use</b> <a href="RewardConfig.md#0x1_RewardConfig">0x1::RewardConfig</a>;
<b>use</b> <a href="Token.md#0x1_Token">0x1::Token</a>;
<b>use</b> <a href="TransactionPublishOption.md#0x1_TransactionPublishOption">0x1::TransactionPublishOption</a>;
<b>use</b> <a href="TransactionTimeoutConfig.md#0x1_TransactionTimeoutConfig">0x1::TransactionTimeoutConfig</a>;
<b>use</b> <a href="Treasury.md#0x1_Treasury">0x1::Treasury</a>;
<b>use</b> <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal">0x1::UpgradeModuleDaoProposal</a>;
<b>use</b> <a href="VMConfig.md#0x1_VMConfig">0x1::VMConfig</a>;
</code></pre>



<a name="0x1_STC_STC"></a>

## Struct `STC`

STC token marker.


<pre><code><b>struct</b> <a href="STC.md#0x1_STC">STC</a> <b>has</b> <b>copy</b>, drop, store
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

Burn capability of STC.


<pre><code><b>struct</b> <a href="STC.md#0x1_STC_SharedBurnCapability">SharedBurnCapability</a> <b>has</b> store, key
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

<a name="@Constants_0"></a>

## Constants


<a name="0x1_STC_PRECISION"></a>

precision of STC token.


<pre><code><b>const</b> <a href="STC.md#0x1_STC_PRECISION">PRECISION</a>: u8 = 9;
</code></pre>



<a name="0x1_STC_initialize"></a>

## Function `initialize`

STC initialization.


<pre><code><b>public</b> <b>fun</b> <a href="STC.md#0x1_STC_initialize">initialize</a>(account: &signer, voting_delay: u64, voting_period: u64, voting_quorum_rate: u8, min_action_delay: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="STC.md#0x1_STC_initialize">initialize</a>(
    account: &signer,
    voting_delay: u64,
    voting_period: u64,
    voting_quorum_rate: u8,
    min_action_delay: u64,
) {
    <a href="Token.md#0x1_Token_register_token">Token::register_token</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;(account, <a href="STC.md#0x1_STC_PRECISION">PRECISION</a>);
    <b>let</b> burn_cap = <a href="Token.md#0x1_Token_remove_burn_capability">Token::remove_burn_capability</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;(account);
    <b>move_to</b>(account, <a href="STC.md#0x1_STC_SharedBurnCapability">SharedBurnCapability</a> { cap: burn_cap });
    <a href="Dao.md#0x1_Dao_plugin">Dao::plugin</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;(
        account,
        voting_delay,
        voting_period,
        voting_quorum_rate,
        min_action_delay,
    );
    <a href="ModifyDaoConfigProposal.md#0x1_ModifyDaoConfigProposal_plugin">ModifyDaoConfigProposal::plugin</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;(account);
    <b>let</b> upgrade_plan_cap = <a href="PackageTxnManager.md#0x1_PackageTxnManager_extract_submit_upgrade_plan_cap">PackageTxnManager::extract_submit_upgrade_plan_cap</a>(account);
    <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_plugin">UpgradeModuleDaoProposal::plugin</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;(
        account,
        upgrade_plan_cap,
    );
    // the following configurations are gov-ed by <a href="Dao.md#0x1_Dao">Dao</a>.
    <a href="OnChainConfigDao.md#0x1_OnChainConfigDao_plugin">OnChainConfigDao::plugin</a>&lt;<a href="STC.md#0x1_STC">STC</a>, <a href="TransactionPublishOption.md#0x1_TransactionPublishOption_TransactionPublishOption">TransactionPublishOption::TransactionPublishOption</a>&gt;(account);
    <a href="OnChainConfigDao.md#0x1_OnChainConfigDao_plugin">OnChainConfigDao::plugin</a>&lt;<a href="STC.md#0x1_STC">STC</a>, <a href="VMConfig.md#0x1_VMConfig_VMConfig">VMConfig::VMConfig</a>&gt;(account);
    <a href="OnChainConfigDao.md#0x1_OnChainConfigDao_plugin">OnChainConfigDao::plugin</a>&lt;<a href="STC.md#0x1_STC">STC</a>, <a href="ConsensusConfig.md#0x1_ConsensusConfig_ConsensusConfig">ConsensusConfig::ConsensusConfig</a>&gt;(account);
    <a href="OnChainConfigDao.md#0x1_OnChainConfigDao_plugin">OnChainConfigDao::plugin</a>&lt;<a href="STC.md#0x1_STC">STC</a>, <a href="RewardConfig.md#0x1_RewardConfig_RewardConfig">RewardConfig::RewardConfig</a>&gt;(account);
    <a href="OnChainConfigDao.md#0x1_OnChainConfigDao_plugin">OnChainConfigDao::plugin</a>&lt;<a href="STC.md#0x1_STC">STC</a>, <a href="TransactionTimeoutConfig.md#0x1_TransactionTimeoutConfig_TransactionTimeoutConfig">TransactionTimeoutConfig::TransactionTimeoutConfig</a>&gt;(account);
}
</code></pre>



</details>

<a name="0x1_STC_upgrade_from_v1_to_v2"></a>

## Function `upgrade_from_v1_to_v2`



<pre><code><b>public</b> <b>fun</b> <a href="STC.md#0x1_STC_upgrade_from_v1_to_v2">upgrade_from_v1_to_v2</a>(account: &signer, total_amount: u128): <a href="Treasury.md#0x1_Treasury_WithdrawCapability">Treasury::WithdrawCapability</a>&lt;<a href="STC.md#0x1_STC_STC">STC::STC</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="STC.md#0x1_STC_upgrade_from_v1_to_v2">upgrade_from_v1_to_v2</a>(account: &signer,total_amount: u128,): <a href="Treasury.md#0x1_Treasury_WithdrawCapability">Treasury::WithdrawCapability</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt; {
    <a href="CoreAddresses.md#0x1_CoreAddresses_assert_genesis_address">CoreAddresses::assert_genesis_address</a>(account);

    // Mint all stc, and destroy mint capability
    <b>let</b> total_stc = <a href="Token.md#0x1_Token_mint">Token::mint</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;(account, total_amount-<a href="Token.md#0x1_Token_market_cap">Token::market_cap</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;());
    <b>let</b> withdraw_cap = <a href="Treasury.md#0x1_Treasury_initialize">Treasury::initialize</a>(account, total_stc);
    <b>let</b> mint_cap = <a href="Token.md#0x1_Token_remove_mint_capability">Token::remove_mint_capability</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;(account);
    <a href="Token.md#0x1_Token_destroy_mint_capability">Token::destroy_mint_capability</a>(mint_cap);
    withdraw_cap
}
</code></pre>



</details>

<a name="0x1_STC_initialize_v2"></a>

## Function `initialize_v2`

STC initialization.


<pre><code><b>public</b> <b>fun</b> <a href="STC.md#0x1_STC_initialize_v2">initialize_v2</a>(account: &signer, total_amount: u128, voting_delay: u64, voting_period: u64, voting_quorum_rate: u8, min_action_delay: u64): <a href="Treasury.md#0x1_Treasury_WithdrawCapability">Treasury::WithdrawCapability</a>&lt;<a href="STC.md#0x1_STC_STC">STC::STC</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="STC.md#0x1_STC_initialize_v2">initialize_v2</a>(
    account: &signer,
    total_amount: u128,
    voting_delay: u64,
    voting_period: u64,
    voting_quorum_rate: u8,
    min_action_delay: u64,
): <a href="Treasury.md#0x1_Treasury_WithdrawCapability">Treasury::WithdrawCapability</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt; {
    <a href="Token.md#0x1_Token_register_token">Token::register_token</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;(account, <a href="STC.md#0x1_STC_PRECISION">PRECISION</a>);

    // Mint all stc, and destroy mint capability

    <b>let</b> total_stc = <a href="Token.md#0x1_Token_mint">Token::mint</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;(account, total_amount);
    <b>let</b> withdraw_cap = <a href="Treasury.md#0x1_Treasury_initialize">Treasury::initialize</a>(account, total_stc);
    <b>let</b> mint_cap = <a href="Token.md#0x1_Token_remove_mint_capability">Token::remove_mint_capability</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;(account);
    <a href="Token.md#0x1_Token_destroy_mint_capability">Token::destroy_mint_capability</a>(mint_cap);

    <b>let</b> burn_cap = <a href="Token.md#0x1_Token_remove_burn_capability">Token::remove_burn_capability</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;(account);
    <b>move_to</b>(account, <a href="STC.md#0x1_STC_SharedBurnCapability">SharedBurnCapability</a> { cap: burn_cap });
    <a href="Dao.md#0x1_Dao_plugin">Dao::plugin</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;(
        account,
        voting_delay,
        voting_period,
        voting_quorum_rate,
        min_action_delay,
    );
    <a href="ModifyDaoConfigProposal.md#0x1_ModifyDaoConfigProposal_plugin">ModifyDaoConfigProposal::plugin</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;(account);
    <b>let</b> upgrade_plan_cap = <a href="PackageTxnManager.md#0x1_PackageTxnManager_extract_submit_upgrade_plan_cap">PackageTxnManager::extract_submit_upgrade_plan_cap</a>(account);
    <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_plugin">UpgradeModuleDaoProposal::plugin</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;(
        account,
        upgrade_plan_cap,
    );
    // the following configurations are gov-ed by <a href="Dao.md#0x1_Dao">Dao</a>.
    <a href="OnChainConfigDao.md#0x1_OnChainConfigDao_plugin">OnChainConfigDao::plugin</a>&lt;<a href="STC.md#0x1_STC">STC</a>, <a href="TransactionPublishOption.md#0x1_TransactionPublishOption_TransactionPublishOption">TransactionPublishOption::TransactionPublishOption</a>&gt;(account);
    <a href="OnChainConfigDao.md#0x1_OnChainConfigDao_plugin">OnChainConfigDao::plugin</a>&lt;<a href="STC.md#0x1_STC">STC</a>, <a href="VMConfig.md#0x1_VMConfig_VMConfig">VMConfig::VMConfig</a>&gt;(account);
    <a href="OnChainConfigDao.md#0x1_OnChainConfigDao_plugin">OnChainConfigDao::plugin</a>&lt;<a href="STC.md#0x1_STC">STC</a>, <a href="ConsensusConfig.md#0x1_ConsensusConfig_ConsensusConfig">ConsensusConfig::ConsensusConfig</a>&gt;(account);
    <a href="OnChainConfigDao.md#0x1_OnChainConfigDao_plugin">OnChainConfigDao::plugin</a>&lt;<a href="STC.md#0x1_STC">STC</a>, <a href="RewardConfig.md#0x1_RewardConfig_RewardConfig">RewardConfig::RewardConfig</a>&gt;(account);
    <a href="OnChainConfigDao.md#0x1_OnChainConfigDao_plugin">OnChainConfigDao::plugin</a>&lt;<a href="STC.md#0x1_STC">STC</a>, <a href="TransactionTimeoutConfig.md#0x1_TransactionTimeoutConfig_TransactionTimeoutConfig">TransactionTimeoutConfig::TransactionTimeoutConfig</a>&gt;(account);
    withdraw_cap
}
</code></pre>



</details>

<a name="0x1_STC_is_stc"></a>

## Function `is_stc`

Returns true if <code>TokenType</code> is <code><a href="STC.md#0x1_STC_STC">STC::STC</a></code>


<pre><code><b>public</b> <b>fun</b> <a href="STC.md#0x1_STC_is_stc">is_stc</a>&lt;TokenType: store&gt;(): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="STC.md#0x1_STC_is_stc">is_stc</a>&lt;TokenType: store&gt;(): bool {
    <a href="Token.md#0x1_Token_is_same_token">Token::is_same_token</a>&lt;<a href="STC.md#0x1_STC">STC</a>, TokenType&gt;()
}
</code></pre>



</details>

<a name="0x1_STC_burn"></a>

## Function `burn`

Burn STC tokens.
It can be called by anyone.


<pre><code><b>public</b> <b>fun</b> <a href="STC.md#0x1_STC_burn">burn</a>(token: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;<a href="STC.md#0x1_STC_STC">STC::STC</a>&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="STC.md#0x1_STC_burn">burn</a>(token: <a href="Token.md#0x1_Token">Token</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;) <b>acquires</b> <a href="STC.md#0x1_STC_SharedBurnCapability">SharedBurnCapability</a> {
    <b>let</b> cap = <b>borrow_global</b>&lt;<a href="STC.md#0x1_STC_SharedBurnCapability">SharedBurnCapability</a>&gt;(<a href="STC.md#0x1_STC_token_address">token_address</a>());
    <a href="Token.md#0x1_Token_burn_with_capability">Token::burn_with_capability</a>(&cap.cap, token);
}
</code></pre>



</details>

<a name="0x1_STC_token_address"></a>

## Function `token_address`

Return STC token address.


<pre><code><b>public</b> <b>fun</b> <a href="STC.md#0x1_STC_token_address">token_address</a>(): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="STC.md#0x1_STC_token_address">token_address</a>(): <b>address</b> {
    <a href="Token.md#0x1_Token_token_address">Token::token_address</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;()
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


<pre><code><b>public</b> <b>fun</b> <a href="STC.md#0x1_STC_initialize">initialize</a>(account: &signer, voting_delay: u64, voting_period: u64, voting_quorum_rate: u8, min_action_delay: u64)
</code></pre>




<pre><code><b>include</b> <a href="Token.md#0x1_Token_RegisterTokenAbortsIf">Token::RegisterTokenAbortsIf</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;{precision: <a href="STC.md#0x1_STC_PRECISION">PRECISION</a>};
</code></pre>



<a name="@Specification_1_upgrade_from_v1_to_v2"></a>

### Function `upgrade_from_v1_to_v2`


<pre><code><b>public</b> <b>fun</b> <a href="STC.md#0x1_STC_upgrade_from_v1_to_v2">upgrade_from_v1_to_v2</a>(account: &signer, total_amount: u128): <a href="Treasury.md#0x1_Treasury_WithdrawCapability">Treasury::WithdrawCapability</a>&lt;<a href="STC.md#0x1_STC_STC">STC::STC</a>&gt;
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>



<a name="@Specification_1_initialize_v2"></a>

### Function `initialize_v2`


<pre><code><b>public</b> <b>fun</b> <a href="STC.md#0x1_STC_initialize_v2">initialize_v2</a>(account: &signer, total_amount: u128, voting_delay: u64, voting_period: u64, voting_quorum_rate: u8, min_action_delay: u64): <a href="Treasury.md#0x1_Treasury_WithdrawCapability">Treasury::WithdrawCapability</a>&lt;<a href="STC.md#0x1_STC_STC">STC::STC</a>&gt;
</code></pre>




<pre><code><b>include</b> <a href="Token.md#0x1_Token_RegisterTokenAbortsIf">Token::RegisterTokenAbortsIf</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;{precision: <a href="STC.md#0x1_STC_PRECISION">PRECISION</a>};
</code></pre>



<a name="@Specification_1_is_stc"></a>

### Function `is_stc`


<pre><code><b>public</b> <b>fun</b> <a href="STC.md#0x1_STC_is_stc">is_stc</a>&lt;TokenType: store&gt;(): bool
</code></pre>




<a name="@Specification_1_burn"></a>

### Function `burn`


<pre><code><b>public</b> <b>fun</b> <a href="STC.md#0x1_STC_burn">burn</a>(token: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;<a href="STC.md#0x1_STC_STC">STC::STC</a>&gt;)
</code></pre>




<pre><code><b>aborts_if</b> <a href="Token.md#0x1_Token_spec_abstract_total_value">Token::spec_abstract_total_value</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;() - token.value &lt; 0;
<b>aborts_if</b> !<b>exists</b>&lt;<a href="STC.md#0x1_STC_SharedBurnCapability">SharedBurnCapability</a>&gt;(<a href="Token.md#0x1_Token_SPEC_TOKEN_TEST_ADDRESS">Token::SPEC_TOKEN_TEST_ADDRESS</a>());
</code></pre>



<a name="@Specification_1_token_address"></a>

### Function `token_address`


<pre><code><b>public</b> <b>fun</b> <a href="STC.md#0x1_STC_token_address">token_address</a>(): <b>address</b>
</code></pre>
