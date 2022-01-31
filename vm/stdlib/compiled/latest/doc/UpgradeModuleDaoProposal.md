
<a name="0x1_UpgradeModuleDaoProposal"></a>

# Module `0x1::UpgradeModuleDaoProposal`

UpgradeModuleDaoProposal is a proposal moudle used to upgrade contract codes under a token.


-  [Resource `UpgradeModuleCapability`](#0x1_UpgradeModuleDaoProposal_UpgradeModuleCapability)
-  [Struct `UpgradeModule`](#0x1_UpgradeModuleDaoProposal_UpgradeModule)
-  [Struct `UpgradeModuleV2`](#0x1_UpgradeModuleDaoProposal_UpgradeModuleV2)
-  [Constants](#@Constants_0)
-  [Function `plugin`](#0x1_UpgradeModuleDaoProposal_plugin)
-  [Function `propose_module_upgrade_v2`](#0x1_UpgradeModuleDaoProposal_propose_module_upgrade_v2)
-  [Function `submit_module_upgrade_plan`](#0x1_UpgradeModuleDaoProposal_submit_module_upgrade_plan)
-  [Specification](#@Specification_1)
    -  [Function `plugin`](#@Specification_1_plugin)
    -  [Function `propose_module_upgrade_v2`](#@Specification_1_propose_module_upgrade_v2)
    -  [Function `submit_module_upgrade_plan`](#@Specification_1_submit_module_upgrade_plan)


<pre><code><b>use</b> <a href="Dao.md#0x1_Dao">0x1::Dao</a>;
<b>use</b> <a href="Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager">0x1::PackageTxnManager</a>;
<b>use</b> <a href="Signer.md#0x1_Signer">0x1::Signer</a>;
<b>use</b> <a href="Token.md#0x1_Token">0x1::Token</a>;
</code></pre>



<a name="0x1_UpgradeModuleDaoProposal_UpgradeModuleCapability"></a>

## Resource `UpgradeModuleCapability`

A wrapper of <code><a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">PackageTxnManager::UpgradePlanCapability</a></code>.


<pre><code><b>struct</b> <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_UpgradeModuleCapability">UpgradeModuleCapability</a>&lt;TokenT&gt; <b>has</b> key
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

request of upgrading module contract code.


<pre><code><b>struct</b> <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_UpgradeModule">UpgradeModule</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>module_address: <b>address</b></code>
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

<a name="0x1_UpgradeModuleDaoProposal_UpgradeModuleV2"></a>

## Struct `UpgradeModuleV2`



<pre><code><b>struct</b> <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_UpgradeModuleV2">UpgradeModuleV2</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>module_address: <b>address</b></code>
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
<dt>
<code>enforced: bool</code>
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


<pre><code><b>public</b> <b>fun</b> <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_plugin">plugin</a>&lt;TokenT: store&gt;(signer: &signer, cap: <a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">PackageTxnManager::UpgradePlanCapability</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_plugin">plugin</a>&lt;TokenT: store&gt;(
    signer: &signer,
    cap: <a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">PackageTxnManager::UpgradePlanCapability</a>,
) {
    <b>let</b> token_issuer = <a href="Token.md#0x1_Token_token_address">Token::token_address</a>&lt;TokenT&gt;();
    <b>assert</b>!(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer) == token_issuer, <a href="Errors.md#0x1_Errors_requires_address">Errors::requires_address</a>(<a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_ERR_NOT_AUTHORIZED">ERR_NOT_AUTHORIZED</a>));
    <b>move_to</b>(signer, <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_UpgradeModuleCapability">UpgradeModuleCapability</a>&lt;TokenT&gt; { cap })
}
</code></pre>



</details>

<a name="0x1_UpgradeModuleDaoProposal_propose_module_upgrade_v2"></a>

## Function `propose_module_upgrade_v2`



<pre><code><b>public</b> <b>fun</b> <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_propose_module_upgrade_v2">propose_module_upgrade_v2</a>&lt;TokenT: <b>copy</b>, drop, store&gt;(signer: &signer, module_address: <b>address</b>, package_hash: vector&lt;u8&gt;, version: u64, exec_delay: u64, enforced: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_propose_module_upgrade_v2">propose_module_upgrade_v2</a>&lt;TokenT: <b>copy</b> + drop + store&gt;(
    signer: &signer,
    module_address: <b>address</b>,
    package_hash: vector&lt;u8&gt;,
    version: u64,
    exec_delay: u64,
    enforced: bool,
) <b>acquires</b> <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_UpgradeModuleCapability">UpgradeModuleCapability</a> {
    <b>let</b> cap = <b>borrow_global</b>&lt;<a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_UpgradeModuleCapability">UpgradeModuleCapability</a>&lt;TokenT&gt;&gt;(<a href="Token.md#0x1_Token_token_address">Token::token_address</a>&lt;TokenT&gt;());
    <b>let</b> account_address = <a href="PackageTxnManager.md#0x1_PackageTxnManager_account_address">PackageTxnManager::account_address</a>(&cap.cap);
    <b>assert</b>!(account_address == module_address, <a href="Errors.md#0x1_Errors_requires_capability">Errors::requires_capability</a>(<a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_ERR_ADDRESS_MISSMATCH">ERR_ADDRESS_MISSMATCH</a>));
    <a href="Dao.md#0x1_Dao_propose">Dao::propose</a>&lt;TokenT, <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_UpgradeModuleV2">UpgradeModuleV2</a>&gt;(
        signer,
        <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_UpgradeModuleV2">UpgradeModuleV2</a> { module_address, package_hash, version, enforced },
        exec_delay,
    );
}
</code></pre>



</details>

<a name="0x1_UpgradeModuleDaoProposal_submit_module_upgrade_plan"></a>

## Function `submit_module_upgrade_plan`

Once the proposal is agreed, anyone can call this method to generate the upgrading plan.


<pre><code><b>public</b> <b>fun</b> <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_submit_module_upgrade_plan">submit_module_upgrade_plan</a>&lt;TokenT: <b>copy</b>, drop, store&gt;(proposer_address: <b>address</b>, proposal_id: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_submit_module_upgrade_plan">submit_module_upgrade_plan</a>&lt;TokenT: <b>copy</b> + drop + store&gt;(
    proposer_address: <b>address</b>,
    proposal_id: u64,
) <b>acquires</b> <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_UpgradeModuleCapability">UpgradeModuleCapability</a> {
    <b>let</b> <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_UpgradeModuleV2">UpgradeModuleV2</a> { module_address, package_hash, version, enforced } = <a href="Dao.md#0x1_Dao_extract_proposal_action">Dao::extract_proposal_action</a>&lt;
        TokenT,
        <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_UpgradeModuleV2">UpgradeModuleV2</a>,
    &gt;(proposer_address, proposal_id);
    <b>let</b> cap = <b>borrow_global</b>&lt;<a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_UpgradeModuleCapability">UpgradeModuleCapability</a>&lt;TokenT&gt;&gt;(<a href="Token.md#0x1_Token_token_address">Token::token_address</a>&lt;TokenT&gt;());
    <b>let</b> account_address = <a href="PackageTxnManager.md#0x1_PackageTxnManager_account_address">PackageTxnManager::account_address</a>(&cap.cap);
    <b>assert</b>!(account_address == module_address, <a href="Errors.md#0x1_Errors_requires_capability">Errors::requires_capability</a>(<a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_ERR_ADDRESS_MISSMATCH">ERR_ADDRESS_MISSMATCH</a>));
    <a href="PackageTxnManager.md#0x1_PackageTxnManager_submit_upgrade_plan_with_cap_v2">PackageTxnManager::submit_upgrade_plan_with_cap_v2</a>(
        &cap.cap,
        package_hash,
        version,
        enforced,
    );
}
</code></pre>



</details>

<a name="@Specification_1"></a>

## Specification



<pre><code><b>pragma</b> verify = <b>false</b>;
<b>pragma</b> aborts_if_is_strict;
<b>pragma</b> aborts_if_is_partial;
</code></pre>



<a name="@Specification_1_plugin"></a>

### Function `plugin`


<pre><code><b>public</b> <b>fun</b> <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_plugin">plugin</a>&lt;TokenT: store&gt;(signer: &signer, cap: <a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">PackageTxnManager::UpgradePlanCapability</a>)
</code></pre>




<pre><code><b>pragma</b> aborts_if_is_partial = <b>false</b>;
<b>let</b> sender = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer);
<b>aborts_if</b> sender != <a href="Token.md#0x1_Token_SPEC_TOKEN_TEST_ADDRESS">Token::SPEC_TOKEN_TEST_ADDRESS</a>();
<b>aborts_if</b> <b>exists</b>&lt;<a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_UpgradeModuleCapability">UpgradeModuleCapability</a>&lt;TokenT&gt;&gt;(sender);
</code></pre>




<a name="0x1_UpgradeModuleDaoProposal_AbortIfUnableUpgrade"></a>


<pre><code><b>schema</b> <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_AbortIfUnableUpgrade">AbortIfUnableUpgrade</a>&lt;TokenT&gt; {
    module_address: <b>address</b>;
    <b>let</b> token_issuer = <a href="Token.md#0x1_Token_SPEC_TOKEN_TEST_ADDRESS">Token::SPEC_TOKEN_TEST_ADDRESS</a>();
    <b>aborts_if</b> !<b>exists</b>&lt;<a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_UpgradeModuleCapability">UpgradeModuleCapability</a>&lt;TokenT&gt;&gt;(token_issuer);
    <b>let</b> cap = <b>global</b>&lt;<a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_UpgradeModuleCapability">UpgradeModuleCapability</a>&lt;TokenT&gt;&gt;(token_issuer).cap;
    <b>aborts_if</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_account_address">PackageTxnManager::account_address</a>(cap) != module_address;
}
</code></pre>



<a name="@Specification_1_propose_module_upgrade_v2"></a>

### Function `propose_module_upgrade_v2`


<pre><code><b>public</b> <b>fun</b> <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_propose_module_upgrade_v2">propose_module_upgrade_v2</a>&lt;TokenT: <b>copy</b>, drop, store&gt;(signer: &signer, module_address: <b>address</b>, package_hash: vector&lt;u8&gt;, version: u64, exec_delay: u64, enforced: bool)
</code></pre>




<pre><code><b>pragma</b> aborts_if_is_partial = <b>true</b>;
<b>include</b> <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_AbortIfUnableUpgrade">AbortIfUnableUpgrade</a>&lt;TokenT&gt;;
</code></pre>



<a name="@Specification_1_submit_module_upgrade_plan"></a>

### Function `submit_module_upgrade_plan`


<pre><code><b>public</b> <b>fun</b> <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_submit_module_upgrade_plan">submit_module_upgrade_plan</a>&lt;TokenT: <b>copy</b>, drop, store&gt;(proposer_address: <b>address</b>, proposal_id: u64)
</code></pre>




<pre><code><b>let</b> expected_states = vec&lt;u8&gt;(6);
<b>include</b> <a href="Dao.md#0x1_Dao_CheckProposalStates">Dao::CheckProposalStates</a>&lt;TokenT, <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_UpgradeModule">UpgradeModule</a>&gt;{expected_states};
<b>let</b> proposal = <b>global</b>&lt;<a href="Dao.md#0x1_Dao_Proposal">Dao::Proposal</a>&lt;TokenT, <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_UpgradeModule">UpgradeModule</a>&gt;&gt;(proposer_address);
<b>aborts_if</b> <a href="Option.md#0x1_Option_is_none">Option::is_none</a>(proposal.action);
<b>let</b> action = proposal.action.vec[0];
<b>include</b> <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_AbortIfUnableUpgrade">AbortIfUnableUpgrade</a>&lt;TokenT&gt;{module_address: action.module_address};
</code></pre>
