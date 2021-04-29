
<a name="0x1_StdlibUpgradeScripts"></a>

# Module `0x1::StdlibUpgradeScripts`

The module for StdlibUpgrade init scripts


-  [Function `upgrade_from_v2_to_v3`](#0x1_StdlibUpgradeScripts_upgrade_from_v2_to_v3)


<pre><code><b>use</b> <a href="CoreAddresses.md#0x1_CoreAddresses">0x1::CoreAddresses</a>;
<b>use</b> <a href="STC.md#0x1_STC">0x1::STC</a>;
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


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="StdlibUpgradeScripts.md#0x1_StdlibUpgradeScripts_upgrade_from_v2_to_v3">upgrade_from_v2_to_v3</a>(account: signer, total_stc_amount: u128) {
    <a href="CoreAddresses.md#0x1_CoreAddresses_assert_genesis_address">CoreAddresses::assert_genesis_address</a>(&account);
    <b>let</b> withdraw_cap = <a href="STC.md#0x1_STC_upgrade_from_v1_to_v2">STC::upgrade_from_v1_to_v2</a>(&account, total_stc_amount);
    // Lock the TreasuryWithdrawCapability <b>to</b> <a href="Dao.md#0x1_Dao">Dao</a>
    <a href="TreasuryWithdrawDaoProposal.md#0x1_TreasuryWithdrawDaoProposal_plugin">TreasuryWithdrawDaoProposal::plugin</a>(&account, withdraw_cap);
}
</code></pre>



</details>
