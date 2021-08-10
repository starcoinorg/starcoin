
<a name="0x1_StdlibUpgradeScripts"></a>

# Module `0x1::StdlibUpgradeScripts`

The module for StdlibUpgrade init scripts


-  [Function `upgrade_from_v2_to_v3`](#0x1_StdlibUpgradeScripts_upgrade_from_v2_to_v3)
-  [Function `take_linear_withdraw_capability`](#0x1_StdlibUpgradeScripts_take_linear_withdraw_capability)
-  [Function `upgrade_from_v5_to_v6`](#0x1_StdlibUpgradeScripts_upgrade_from_v5_to_v6)
-  [Specification](#@Specification_0)


<pre><code><b>use</b> <a href="Collection.md#0x1_Collection">0x1::Collection</a>;
<b>use</b> <a href="CoreAddresses.md#0x1_CoreAddresses">0x1::CoreAddresses</a>;
<b>use</b> <a href="NFT.md#0x1_NFT">0x1::NFT</a>;
<b>use</b> <a href="Offer.md#0x1_Offer">0x1::Offer</a>;
<b>use</b> <a href="Oracle.md#0x1_Oracle">0x1::Oracle</a>;
<b>use</b> <a href="STC.md#0x1_STC">0x1::STC</a>;
<b>use</b> <a href="Oracle.md#0x1_STCUSDOracle">0x1::STCUSDOracle</a>;
<b>use</b> <a href="Timestamp.md#0x1_Timestamp">0x1::Timestamp</a>;
<b>use</b> <a href="Token.md#0x1_Token">0x1::Token</a>;
<b>use</b> <a href="Treasury.md#0x1_Treasury">0x1::Treasury</a>;
<b>use</b> <a href="TreasuryWithdrawDaoProposal.md#0x1_TreasuryWithdrawDaoProposal">0x1::TreasuryWithdrawDaoProposal</a>;
</code></pre>



<a name="0x1_StdlibUpgradeScripts_upgrade_from_v2_to_v3"></a>

## Function `upgrade_from_v2_to_v3`

Stdlib upgrade script from v2 to v3


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="StdlibUpgradeScripts.md#0x1_StdlibUpgradeScripts_upgrade_from_v2_to_v3">upgrade_from_v2_to_v3</a>(account: signer, total_stc_amount: u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="StdlibUpgradeScripts.md#0x1_StdlibUpgradeScripts_upgrade_from_v2_to_v3">upgrade_from_v2_to_v3</a>(account: signer, total_stc_amount: u128 ) {
    <a href="CoreAddresses.md#0x1_CoreAddresses_assert_genesis_address">CoreAddresses::assert_genesis_address</a>(&account);

    <b>let</b> withdraw_cap = <a href="STC.md#0x1_STC_upgrade_from_v1_to_v2">STC::upgrade_from_v1_to_v2</a>(&account, total_stc_amount);

    <b>let</b> mint_keys = <a href="Collection.md#0x1_Collection_borrow_collection">Collection::borrow_collection</a>&lt;LinearTimeMintKey&lt;<a href="STC.md#0x1_STC">STC</a>&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_ASSOCIATION_ROOT_ADDRESS">CoreAddresses::ASSOCIATION_ROOT_ADDRESS</a>());
    <b>let</b> mint_key = <a href="Collection.md#0x1_Collection_borrow">Collection::borrow</a>(&mint_keys, 0);
    <b>let</b> (total, minted, start_time, period) = <a href="Token.md#0x1_Token_read_linear_time_key">Token::read_linear_time_key</a>(mint_key);
    <a href="Collection.md#0x1_Collection_return_collection">Collection::return_collection</a>(mint_keys);

    <b>let</b> now = <a href="Timestamp.md#0x1_Timestamp_now_seconds">Timestamp::now_seconds</a>();
    <b>let</b> linear_withdraw_cap = <a href="Treasury.md#0x1_Treasury_issue_linear_withdraw_capability">Treasury::issue_linear_withdraw_capability</a>(&<b>mut</b> withdraw_cap, total-minted, period - (now - start_time));
    // Lock the TreasuryWithdrawCapability <b>to</b> <a href="Dao.md#0x1_Dao">Dao</a>
    <a href="TreasuryWithdrawDaoProposal.md#0x1_TreasuryWithdrawDaoProposal_plugin">TreasuryWithdrawDaoProposal::plugin</a>(&account, withdraw_cap);
    // Give a LinearWithdrawCapability <a href="Offer.md#0x1_Offer">Offer</a> <b>to</b> association, association need <b>to</b> take the offer, and destroy <b>old</b> LinearTimeMintKey.
    <a href="Offer.md#0x1_Offer_create">Offer::create</a>(&account, linear_withdraw_cap, <a href="CoreAddresses.md#0x1_CoreAddresses_ASSOCIATION_ROOT_ADDRESS">CoreAddresses::ASSOCIATION_ROOT_ADDRESS</a>(), 0);
}
</code></pre>



</details>

<a name="0x1_StdlibUpgradeScripts_take_linear_withdraw_capability"></a>

## Function `take_linear_withdraw_capability`

association account should call this script after upgrade from v2 to v3.


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="StdlibUpgradeScripts.md#0x1_StdlibUpgradeScripts_take_linear_withdraw_capability">take_linear_withdraw_capability</a>(signer: signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="StdlibUpgradeScripts.md#0x1_StdlibUpgradeScripts_take_linear_withdraw_capability">take_linear_withdraw_capability</a>(signer: signer){
    <b>let</b> offered = <a href="Offer.md#0x1_Offer_redeem">Offer::redeem</a>&lt;LinearWithdrawCapability&lt;<a href="STC.md#0x1_STC">STC</a>&gt;&gt;(&signer, <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
    <a href="Treasury.md#0x1_Treasury_add_linear_withdraw_capability">Treasury::add_linear_withdraw_capability</a>(&signer, offered);
    <b>let</b> mint_key = <a href="Collection.md#0x1_Collection_take">Collection::take</a>&lt;LinearTimeMintKey&lt;<a href="STC.md#0x1_STC">STC</a>&gt;&gt;(&signer);
    <a href="Token.md#0x1_Token_destroy_linear_time_key">Token::destroy_linear_time_key</a>(mint_key);
}
</code></pre>



</details>

<a name="0x1_StdlibUpgradeScripts_upgrade_from_v5_to_v6"></a>

## Function `upgrade_from_v5_to_v6`



<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="StdlibUpgradeScripts.md#0x1_StdlibUpgradeScripts_upgrade_from_v5_to_v6">upgrade_from_v5_to_v6</a>(account: signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="StdlibUpgradeScripts.md#0x1_StdlibUpgradeScripts_upgrade_from_v5_to_v6">upgrade_from_v5_to_v6</a>(account: signer) {
    <a href="CoreAddresses.md#0x1_CoreAddresses_assert_genesis_address">CoreAddresses::assert_genesis_address</a>(&account);
    <a href="Oracle.md#0x1_Oracle_initialize">Oracle::initialize</a>(&account);
    //register oracle
    <a href="Oracle.md#0x1_STCUSDOracle_register">STCUSDOracle::register</a>(&account);
    <a href="NFT.md#0x1_NFT_initialize">NFT::initialize</a>(&account);
    //TODO: init genesisNFT;
    //<a href="GenesisNFT.md#0x1_GenesisNFT_initialize">GenesisNFT::initialize</a>(&account, merkle_root, leafs, image);
}
</code></pre>



</details>

<a name="@Specification_0"></a>

## Specification



<pre><code><b>pragma</b> verify = <b>false</b>;
<b>pragma</b> aborts_if_is_strict = <b>true</b>;
</code></pre>
