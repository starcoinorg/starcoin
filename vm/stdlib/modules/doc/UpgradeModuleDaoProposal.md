
<a name="0x1_UpgradeModuleDaoProposal"></a>

# Module `0x1::UpgradeModuleDaoProposal`



-  [Resource `UpgradeModuleCapability`](#0x1_UpgradeModuleDaoProposal_UpgradeModuleCapability)
-  [Struct `UpgradeModule`](#0x1_UpgradeModuleDaoProposal_UpgradeModule)
-  [Constants](#@Constants_0)
-  [Function `plugin`](#0x1_UpgradeModuleDaoProposal_plugin)
-  [Function `propose_module_upgrade`](#0x1_UpgradeModuleDaoProposal_propose_module_upgrade)
-  [Function `submit_module_upgrade_plan`](#0x1_UpgradeModuleDaoProposal_submit_module_upgrade_plan)
-  [Specification](#@Specification_1)
    -  [Function `plugin`](#@Specification_1_plugin)
    -  [Function `propose_module_upgrade`](#@Specification_1_propose_module_upgrade)
    -  [Function `submit_module_upgrade_plan`](#@Specification_1_submit_module_upgrade_plan)


<pre><code><b>use</b> <a href="Block.md#0x1_Block">0x1::Block</a>;
<b>use</b> <a href="Dao.md#0x1_Dao">0x1::Dao</a>;
<b>use</b> <a href="Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager">0x1::PackageTxnManager</a>;
<b>use</b> <a href="Signer.md#0x1_Signer">0x1::Signer</a>;
<b>use</b> <a href="Token.md#0x1_Token">0x1::Token</a>;
</code></pre>



<a name="0x1_UpgradeModuleDaoProposal_UpgradeModuleCapability"></a>

## Resource `UpgradeModuleCapability`



<pre><code><b>resource</b> <b>struct</b> <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_UpgradeModuleCapability">UpgradeModuleCapability</a>&lt;TokenT&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>cap: <a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">PackageTxnManager::UpgradePlanCapability</a></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_UpgradeModuleDaoProposal_UpgradeModule"></a>

## Struct `UpgradeModule`



<pre><code><b>struct</b> <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_UpgradeModule">UpgradeModule</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>module_address: address</code>
</dt>
<dd>

</dd>
<dt>
<code>package_hash: vector&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>version: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_UpgradeModuleDaoProposal_ERR_NOT_AUTHORIZED"></a>



<pre><code><b>const</b> <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_ERR_NOT_AUTHORIZED">ERR_NOT_AUTHORIZED</a>: u64 = 401;
</code></pre>



<a name="0x1_UpgradeModuleDaoProposal_ERR_ADDRESS_MISSMATCH"></a>



<pre><code><b>const</b> <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_ERR_ADDRESS_MISSMATCH">ERR_ADDRESS_MISSMATCH</a>: u64 = 402;
</code></pre>



<a name="0x1_UpgradeModuleDaoProposal_ERR_UNABLE_TO_UPGRADE"></a>



<pre><code><b>const</b> <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_ERR_UNABLE_TO_UPGRADE">ERR_UNABLE_TO_UPGRADE</a>: u64 = 400;
</code></pre>



<a name="0x1_UpgradeModuleDaoProposal_plugin"></a>

## Function `plugin`

If this goverment can upgrade module, call this to register capability.


<pre><code><b>public</b> <b>fun</b> <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_plugin">plugin</a>&lt;TokenT&gt;(signer: &signer, cap: <a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">PackageTxnManager::UpgradePlanCapability</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_plugin">plugin</a>&lt;TokenT&gt;(
    signer: &signer,
    cap: <a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">PackageTxnManager::UpgradePlanCapability</a>,
) {
    <b>let</b> token_issuer = <a href="Token.md#0x1_Token_token_address">Token::token_address</a>&lt;TokenT&gt;();
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer) == token_issuer, <a href="Errors.md#0x1_Errors_requires_address">Errors::requires_address</a>(<a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_ERR_NOT_AUTHORIZED">ERR_NOT_AUTHORIZED</a>));
    move_to(signer, <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_UpgradeModuleCapability">UpgradeModuleCapability</a>&lt;TokenT&gt; { cap })
}
</code></pre>



</details>

<a name="0x1_UpgradeModuleDaoProposal_propose_module_upgrade"></a>

## Function `propose_module_upgrade`

propose a module upgrade, called by proposer.


<pre><code><b>public</b> <b>fun</b> <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_propose_module_upgrade">propose_module_upgrade</a>&lt;TokenT: <b>copyable</b>&gt;(signer: &signer, module_address: address, package_hash: vector&lt;u8&gt;, version: u64, exec_delay: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_propose_module_upgrade">propose_module_upgrade</a>&lt;TokenT: <b>copyable</b>&gt;(
    signer: &signer,
    module_address: address,
    package_hash: vector&lt;u8&gt;,
    version: u64,
    exec_delay: u64,
) <b>acquires</b> <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_UpgradeModuleCapability">UpgradeModuleCapability</a> {
    <b>let</b> cap = borrow_global&lt;<a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_UpgradeModuleCapability">UpgradeModuleCapability</a>&lt;TokenT&gt;&gt;(<a href="Token.md#0x1_Token_token_address">Token::token_address</a>&lt;TokenT&gt;());
    <b>let</b> account_address = <a href="PackageTxnManager.md#0x1_PackageTxnManager_account_address">PackageTxnManager::account_address</a>(&cap.cap);
    <b>assert</b>(account_address == module_address, <a href="Errors.md#0x1_Errors_requires_capability">Errors::requires_capability</a>(<a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_ERR_ADDRESS_MISSMATCH">ERR_ADDRESS_MISSMATCH</a>));
    <a href="Dao.md#0x1_Dao_propose">Dao::propose</a>&lt;TokenT, <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_UpgradeModule">UpgradeModule</a>&gt;(
        signer,
        <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_UpgradeModule">UpgradeModule</a> { module_address, package_hash, version },
        exec_delay,
    );
}
</code></pre>



</details>

<a name="0x1_UpgradeModuleDaoProposal_submit_module_upgrade_plan"></a>

## Function `submit_module_upgrade_plan`



<pre><code><b>public</b> <b>fun</b> <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_submit_module_upgrade_plan">submit_module_upgrade_plan</a>&lt;TokenT: <b>copyable</b>&gt;(proposer_address: address, proposal_id: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_submit_module_upgrade_plan">submit_module_upgrade_plan</a>&lt;TokenT: <b>copyable</b>&gt;(
    proposer_address: address,
    proposal_id: u64,
) <b>acquires</b> <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_UpgradeModuleCapability">UpgradeModuleCapability</a> {
    <b>let</b> <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_UpgradeModule">UpgradeModule</a> { module_address, package_hash, version } = <a href="Dao.md#0x1_Dao_extract_proposal_action">Dao::extract_proposal_action</a>&lt;
        TokenT,
        <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_UpgradeModule">UpgradeModule</a>,
    &gt;(proposer_address, proposal_id);
    <b>let</b> cap = borrow_global&lt;<a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_UpgradeModuleCapability">UpgradeModuleCapability</a>&lt;TokenT&gt;&gt;(<a href="Token.md#0x1_Token_token_address">Token::token_address</a>&lt;TokenT&gt;());
    <b>let</b> account_address = <a href="PackageTxnManager.md#0x1_PackageTxnManager_account_address">PackageTxnManager::account_address</a>(&cap.cap);
    <b>assert</b>(account_address == module_address, <a href="Errors.md#0x1_Errors_requires_capability">Errors::requires_capability</a>(<a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_ERR_ADDRESS_MISSMATCH">ERR_ADDRESS_MISSMATCH</a>));
    <a href="PackageTxnManager.md#0x1_PackageTxnManager_submit_upgrade_plan_with_cap">PackageTxnManager::submit_upgrade_plan_with_cap</a>(
        &cap.cap,
        package_hash,
        version,
        <a href="Block.md#0x1_Block_get_current_block_number">Block::get_current_block_number</a>(),
    );
}
</code></pre>



</details>

<a name="@Specification_1"></a>

## Specification



<pre><code><b>pragma</b> verify;
<b>pragma</b> aborts_if_is_strict;
<b>pragma</b> aborts_if_is_partial;
</code></pre>



<a name="@Specification_1_plugin"></a>

### Function `plugin`


<pre><code><b>public</b> <b>fun</b> <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_plugin">plugin</a>&lt;TokenT&gt;(signer: &signer, cap: <a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">PackageTxnManager::UpgradePlanCapability</a>)
</code></pre>




<pre><code><b>pragma</b> aborts_if_is_partial = <b>false</b>;
<a name="0x1_UpgradeModuleDaoProposal_sender$5"></a>
<b>let</b> sender = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer);
<b>aborts_if</b> sender != <a href="Token.md#0x1_Token_SPEC_TOKEN_TEST_ADDRESS">Token::SPEC_TOKEN_TEST_ADDRESS</a>();
<b>aborts_if</b> <b>exists</b>&lt;<a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_UpgradeModuleCapability">UpgradeModuleCapability</a>&lt;TokenT&gt;&gt;(sender);
</code></pre>




<a name="0x1_UpgradeModuleDaoProposal_AbortIfUnableUpgrade"></a>


<pre><code><b>schema</b> <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_AbortIfUnableUpgrade">AbortIfUnableUpgrade</a>&lt;TokenT&gt; {
    module_address: address;
    <a name="0x1_UpgradeModuleDaoProposal_token_issuer$3"></a>
    <b>let</b> token_issuer = <a href="Token.md#0x1_Token_SPEC_TOKEN_TEST_ADDRESS">Token::SPEC_TOKEN_TEST_ADDRESS</a>();
    <b>aborts_if</b> !<b>exists</b>&lt;<a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_UpgradeModuleCapability">UpgradeModuleCapability</a>&lt;TokenT&gt;&gt;(token_issuer);
    <a name="0x1_UpgradeModuleDaoProposal_cap$4"></a>
    <b>let</b> cap = <b>global</b>&lt;<a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_UpgradeModuleCapability">UpgradeModuleCapability</a>&lt;TokenT&gt;&gt;(token_issuer).cap;
    <b>aborts_if</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_account_address">PackageTxnManager::account_address</a>(cap) != module_address;
}
</code></pre>



<a name="@Specification_1_propose_module_upgrade"></a>

### Function `propose_module_upgrade`


<pre><code><b>public</b> <b>fun</b> <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_propose_module_upgrade">propose_module_upgrade</a>&lt;TokenT: <b>copyable</b>&gt;(signer: &signer, module_address: address, package_hash: vector&lt;u8&gt;, version: u64, exec_delay: u64)
</code></pre>




<pre><code><b>pragma</b> aborts_if_is_partial = <b>true</b>;
<b>include</b> <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_AbortIfUnableUpgrade">AbortIfUnableUpgrade</a>&lt;TokenT&gt;;
</code></pre>



<a name="@Specification_1_submit_module_upgrade_plan"></a>

### Function `submit_module_upgrade_plan`


<pre><code><b>public</b> <b>fun</b> <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_submit_module_upgrade_plan">submit_module_upgrade_plan</a>&lt;TokenT: <b>copyable</b>&gt;(proposer_address: address, proposal_id: u64)
</code></pre>




<a name="0x1_UpgradeModuleDaoProposal_expected_states$6"></a>


<pre><code><b>let</b> expected_states = singleton_vector(6);
<b>include</b> <a href="Dao.md#0x1_Dao_CheckProposalStates">Dao::CheckProposalStates</a>&lt;TokenT, <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_UpgradeModule">UpgradeModule</a>&gt;{expected_states};
<a name="0x1_UpgradeModuleDaoProposal_proposal$7"></a>
<b>let</b> proposal = <b>global</b>&lt;<a href="Dao.md#0x1_Dao_Proposal">Dao::Proposal</a>&lt;TokenT, <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_UpgradeModule">UpgradeModule</a>&gt;&gt;(proposer_address);
<b>aborts_if</b> <a href="Option.md#0x1_Option_spec_is_none">Option::spec_is_none</a>(proposal.action);
<a name="0x1_UpgradeModuleDaoProposal_action$8"></a>
<b>let</b> action = proposal.action.vec[0];
<b>include</b> <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_AbortIfUnableUpgrade">AbortIfUnableUpgrade</a>&lt;TokenT&gt;{module_address: action.module_address};
</code></pre>
