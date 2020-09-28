
<a name="0x1_UpgradeModuleDaoProposal"></a>

# Module `0x1::UpgradeModuleDaoProposal`



-  [Resource <code><a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_UpgradeModuleCapabilities">UpgradeModuleCapabilities</a></code>](#0x1_UpgradeModuleDaoProposal_UpgradeModuleCapabilities)
-  [Resource <code><a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_WrappedUpgradePlanCapability">WrappedUpgradePlanCapability</a></code>](#0x1_UpgradeModuleDaoProposal_WrappedUpgradePlanCapability)
-  [Struct <code><a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_UpgradeModule">UpgradeModule</a></code>](#0x1_UpgradeModuleDaoProposal_UpgradeModule)
-  [Const <code><a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_ERR_NOT_AUTHORIZED">ERR_NOT_AUTHORIZED</a></code>](#0x1_UpgradeModuleDaoProposal_ERR_NOT_AUTHORIZED)
-  [Const <code><a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_ERR_UNABLE_TO_UPGRADE">ERR_UNABLE_TO_UPGRADE</a></code>](#0x1_UpgradeModuleDaoProposal_ERR_UNABLE_TO_UPGRADE)
-  [Function <code>plugin</code>](#0x1_UpgradeModuleDaoProposal_plugin)
-  [Function <code>delegate_module_upgrade_capability</code>](#0x1_UpgradeModuleDaoProposal_delegate_module_upgrade_capability)
-  [Function <code>able_to_upgrade</code>](#0x1_UpgradeModuleDaoProposal_able_to_upgrade)
-  [Function <code>propose_module_upgrade</code>](#0x1_UpgradeModuleDaoProposal_propose_module_upgrade)
-  [Function <code>submit_module_upgrade_plan</code>](#0x1_UpgradeModuleDaoProposal_submit_module_upgrade_plan)
-  [Function <code>find_module_upgrade_cap</code>](#0x1_UpgradeModuleDaoProposal_find_module_upgrade_cap)


<a name="0x1_UpgradeModuleDaoProposal_UpgradeModuleCapabilities"></a>

## Resource `UpgradeModuleCapabilities`



<pre><code><b>resource</b> <b>struct</b> <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_UpgradeModuleCapabilities">UpgradeModuleCapabilities</a>&lt;TokenT&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>caps: vector&lt;<a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_WrappedUpgradePlanCapability">UpgradeModuleDaoProposal::WrappedUpgradePlanCapability</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_UpgradeModuleDaoProposal_WrappedUpgradePlanCapability"></a>

## Resource `WrappedUpgradePlanCapability`



<pre><code><b>resource</b> <b>struct</b> <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_WrappedUpgradePlanCapability">WrappedUpgradePlanCapability</a>
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
</dl>


</details>

<a name="0x1_UpgradeModuleDaoProposal_ERR_NOT_AUTHORIZED"></a>

## Const `ERR_NOT_AUTHORIZED`



<pre><code><b>const</b> <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_ERR_NOT_AUTHORIZED">ERR_NOT_AUTHORIZED</a>: u64 = 401;
</code></pre>



<a name="0x1_UpgradeModuleDaoProposal_ERR_UNABLE_TO_UPGRADE"></a>

## Const `ERR_UNABLE_TO_UPGRADE`



<pre><code><b>const</b> <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_ERR_UNABLE_TO_UPGRADE">ERR_UNABLE_TO_UPGRADE</a>: u64 = 400;
</code></pre>



<a name="0x1_UpgradeModuleDaoProposal_plugin"></a>

## Function `plugin`



<pre><code><b>public</b> <b>fun</b> <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_plugin">plugin</a>&lt;TokenT&gt;(signer: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_plugin">plugin</a>&lt;TokenT&gt;(signer: &signer) {
    <b>let</b> token_issuer = <a href="Token.md#0x1_Token_token_address">Token::token_address</a>&lt;TokenT&gt;();
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer) == token_issuer, <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_ERR_NOT_AUTHORIZED">ERR_NOT_AUTHORIZED</a>);
    <b>let</b> caps = <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_UpgradeModuleCapabilities">UpgradeModuleCapabilities</a>&lt;TokenT&gt; { caps: <a href="Vector.md#0x1_Vector_empty">Vector::empty</a>() };
    move_to(signer, caps)
}
</code></pre>



</details>

<a name="0x1_UpgradeModuleDaoProposal_delegate_module_upgrade_capability"></a>

## Function `delegate_module_upgrade_capability`

If this govverment can upgrade module, call this to register capability.


<pre><code><b>public</b> <b>fun</b> <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_delegate_module_upgrade_capability">delegate_module_upgrade_capability</a>&lt;TokenT&gt;(signer: &signer, cap: <a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">PackageTxnManager::UpgradePlanCapability</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_delegate_module_upgrade_capability">delegate_module_upgrade_capability</a>&lt;TokenT&gt;(
    signer: &signer,
    cap: <a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">PackageTxnManager::UpgradePlanCapability</a>,
) <b>acquires</b> <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_UpgradeModuleCapabilities">UpgradeModuleCapabilities</a> {
    <b>let</b> token_issuer = <a href="Token.md#0x1_Token_token_address">Token::token_address</a>&lt;TokenT&gt;();
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer) == token_issuer, <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_ERR_NOT_AUTHORIZED">ERR_NOT_AUTHORIZED</a>);
    <b>let</b> caps = borrow_global_mut&lt;<a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_UpgradeModuleCapabilities">UpgradeModuleCapabilities</a>&lt;TokenT&gt;&gt;(token_issuer);
    // TODO: should check duplicate cap?
    // for now, only one cap <b>exists</b> for a <b>module</b> address.
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> caps.caps, <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_WrappedUpgradePlanCapability">WrappedUpgradePlanCapability</a> { cap });
}
</code></pre>



</details>

<a name="0x1_UpgradeModuleDaoProposal_able_to_upgrade"></a>

## Function `able_to_upgrade`

check whether this gov has the ability to upgrade module in <code>moudle_address</code>.


<pre><code><b>public</b> <b>fun</b> <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_able_to_upgrade">able_to_upgrade</a>&lt;TokenT&gt;(module_address: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_able_to_upgrade">able_to_upgrade</a>&lt;TokenT&gt;(module_address: address): bool
<b>acquires</b> <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_UpgradeModuleCapabilities">UpgradeModuleCapabilities</a> {
    <b>let</b> pos = <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_find_module_upgrade_cap">find_module_upgrade_cap</a>&lt;TokenT&gt;(module_address);
    <a href="Option.md#0x1_Option_is_some">Option::is_some</a>(&pos)
}
</code></pre>



</details>

<a name="0x1_UpgradeModuleDaoProposal_propose_module_upgrade"></a>

## Function `propose_module_upgrade`

propose a module upgrade, called by proposer.


<pre><code><b>public</b> <b>fun</b> <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_propose_module_upgrade">propose_module_upgrade</a>&lt;TokenT: <b>copyable</b>&gt;(signer: &signer, module_address: address, package_hash: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_propose_module_upgrade">propose_module_upgrade</a>&lt;TokenT: <b>copyable</b>&gt;(
    signer: &signer,
    module_address: address,
    package_hash: vector&lt;u8&gt;,
) <b>acquires</b> <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_UpgradeModuleCapabilities">UpgradeModuleCapabilities</a> {
    <b>assert</b>(<a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_able_to_upgrade">able_to_upgrade</a>&lt;TokenT&gt;(module_address), <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_ERR_UNABLE_TO_UPGRADE">ERR_UNABLE_TO_UPGRADE</a>);
    <a href="Dao.md#0x1_Dao_propose">Dao::propose</a>&lt;TokenT, <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_UpgradeModule">UpgradeModule</a>&gt;(
        signer,
        <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_UpgradeModule">UpgradeModule</a> { module_address, package_hash },
        <a href="Dao.md#0x1_Dao_min_action_delay">Dao::min_action_delay</a>&lt;TokenT&gt;(),
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
) <b>acquires</b> <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_UpgradeModuleCapabilities">UpgradeModuleCapabilities</a> {
    <b>let</b> <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_UpgradeModule">UpgradeModule</a> { module_address, package_hash } = <a href="Dao.md#0x1_Dao_extract_proposal_action">Dao::extract_proposal_action</a>&lt;
        TokenT,
        <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_UpgradeModule">UpgradeModule</a>,
    &gt;(proposer_address, proposal_id);
    <b>let</b> pos = <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_find_module_upgrade_cap">find_module_upgrade_cap</a>&lt;TokenT&gt;(module_address);
    <b>assert</b>(<a href="Option.md#0x1_Option_is_some">Option::is_some</a>(&pos), 500);
    <b>let</b> pos = <a href="Option.md#0x1_Option_extract">Option::extract</a>(&<b>mut</b> pos);
    <b>let</b> caps = borrow_global&lt;<a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_UpgradeModuleCapabilities">UpgradeModuleCapabilities</a>&lt;TokenT&gt;&gt;(<a href="Token.md#0x1_Token_token_address">Token::token_address</a>&lt;TokenT&gt;());
    <b>let</b> cap = <a href="Vector.md#0x1_Vector_borrow">Vector::borrow</a>(&caps.caps, pos);
    <a href="PackageTxnManager.md#0x1_PackageTxnManager_submit_upgrade_plan_with_cap">PackageTxnManager::submit_upgrade_plan_with_cap</a>(
        &cap.cap,
        package_hash,
        <a href="Block.md#0x1_Block_get_current_block_number">Block::get_current_block_number</a>(),
    );
}
</code></pre>



</details>

<a name="0x1_UpgradeModuleDaoProposal_find_module_upgrade_cap"></a>

## Function `find_module_upgrade_cap`



<pre><code><b>fun</b> <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_find_module_upgrade_cap">find_module_upgrade_cap</a>&lt;TokenT&gt;(module_address: address): <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;u64&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_find_module_upgrade_cap">find_module_upgrade_cap</a>&lt;TokenT&gt;(module_address: address): <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;u64&gt;
<b>acquires</b> <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_UpgradeModuleCapabilities">UpgradeModuleCapabilities</a> {
    <b>let</b> token_issuer = <a href="Token.md#0x1_Token_token_address">Token::token_address</a>&lt;TokenT&gt;();
    <b>let</b> caps = borrow_global&lt;<a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_UpgradeModuleCapabilities">UpgradeModuleCapabilities</a>&lt;TokenT&gt;&gt;(token_issuer);
    <b>let</b> cap_len = <a href="Vector.md#0x1_Vector_length">Vector::length</a>(&caps.caps);
    <b>let</b> i = 0;
    <b>while</b> (i &lt; cap_len){
        <b>let</b> cap = <a href="Vector.md#0x1_Vector_borrow">Vector::borrow</a>(&caps.caps, i);
        <b>let</b> account_address = <a href="PackageTxnManager.md#0x1_PackageTxnManager_account_address">PackageTxnManager::account_address</a>(&cap.cap);
        <b>if</b> (account_address == module_address) {
            <b>return</b> <a href="Option.md#0x1_Option_some">Option::some</a>(i)
        };
        i = i + 1;
    };
    <a href="Option.md#0x1_Option_none">Option::none</a>&lt;u64&gt;()
}
</code></pre>



</details>
