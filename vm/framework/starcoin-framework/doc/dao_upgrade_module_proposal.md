
<a id="0x1_dao_upgrade_module_proposal"></a>

# Module `0x1::dao_upgrade_module_proposal`

dao_upgrade_module_proposal is a proposal moudle used to upgrade contract codes under a token.


-  [Resource `UpgradeModuleCapability`](#0x1_dao_upgrade_module_proposal_UpgradeModuleCapability)
-  [Struct `UpgradeModuleV2`](#0x1_dao_upgrade_module_proposal_UpgradeModuleV2)
-  [Constants](#@Constants_0)
-  [Function `plugin`](#0x1_dao_upgrade_module_proposal_plugin)
-  [Function `propose_module_upgrade_v2`](#0x1_dao_upgrade_module_proposal_propose_module_upgrade_v2)
-  [Function `submit_module_upgrade_plan`](#0x1_dao_upgrade_module_proposal_submit_module_upgrade_plan)
-  [Specification](#@Specification_1)
    -  [Function `plugin`](#@Specification_1_plugin)
    -  [Function `propose_module_upgrade_v2`](#@Specification_1_propose_module_upgrade_v2)
    -  [Function `submit_module_upgrade_plan`](#@Specification_1_submit_module_upgrade_plan)


<pre><code><b>use</b> <a href="dao.md#0x1_dao">0x1::dao</a>;
<b>use</b> <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug">0x1::debug</a>;
<b>use</b> <a href="../../move-stdlib/doc/error.md#0x1_error">0x1::error</a>;
<b>use</b> <a href="../../move-stdlib/doc/signer.md#0x1_signer">0x1::signer</a>;
<b>use</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation">0x1::stc_transaction_package_validation</a>;
<b>use</b> <a href="stc_util.md#0x1_stc_util">0x1::stc_util</a>;
<b>use</b> <a href="../../move-stdlib/doc/string.md#0x1_string">0x1::string</a>;
</code></pre>



<a id="0x1_dao_upgrade_module_proposal_UpgradeModuleCapability"></a>

## Resource `UpgradeModuleCapability`

A wrapper of <code><a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_UpgradePlanCapability">stc_transaction_package_validation::UpgradePlanCapability</a></code>.


<pre><code><b>struct</b> <a href="dao_upgrade_module_proposal.md#0x1_dao_upgrade_module_proposal_UpgradeModuleCapability">UpgradeModuleCapability</a>&lt;TokenT&gt; <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>cap: <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_UpgradePlanCapability">stc_transaction_package_validation::UpgradePlanCapability</a></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="0x1_dao_upgrade_module_proposal_UpgradeModuleV2"></a>

## Struct `UpgradeModuleV2`



<pre><code><b>struct</b> <a href="dao_upgrade_module_proposal.md#0x1_dao_upgrade_module_proposal_UpgradeModuleV2">UpgradeModuleV2</a> <b>has</b> <b>copy</b>, drop, store
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
<code>package_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;</code>
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

<a id="@Constants_0"></a>

## Constants


<a id="0x1_dao_upgrade_module_proposal_ERR_NOT_AUTHORIZED"></a>



<pre><code><b>const</b> <a href="dao_upgrade_module_proposal.md#0x1_dao_upgrade_module_proposal_ERR_NOT_AUTHORIZED">ERR_NOT_AUTHORIZED</a>: u64 = 401;
</code></pre>



<a id="0x1_dao_upgrade_module_proposal_ERR_ADDRESS_MISSMATCH"></a>



<pre><code><b>const</b> <a href="dao_upgrade_module_proposal.md#0x1_dao_upgrade_module_proposal_ERR_ADDRESS_MISSMATCH">ERR_ADDRESS_MISSMATCH</a>: u64 = 402;
</code></pre>



<a id="0x1_dao_upgrade_module_proposal_ERR_UNABLE_TO_UPGRADE"></a>



<pre><code><b>const</b> <a href="dao_upgrade_module_proposal.md#0x1_dao_upgrade_module_proposal_ERR_UNABLE_TO_UPGRADE">ERR_UNABLE_TO_UPGRADE</a>: u64 = 400;
</code></pre>



<a id="0x1_dao_upgrade_module_proposal_plugin"></a>

## Function `plugin`

If this goverment can upgrade module, call this to register capability.


<pre><code><b>public</b> <b>fun</b> <a href="dao_upgrade_module_proposal.md#0x1_dao_upgrade_module_proposal_plugin">plugin</a>&lt;TokenT&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, cap: <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_UpgradePlanCapability">stc_transaction_package_validation::UpgradePlanCapability</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="dao_upgrade_module_proposal.md#0x1_dao_upgrade_module_proposal_plugin">plugin</a>&lt;TokenT&gt;(
    <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    cap: <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_UpgradePlanCapability">stc_transaction_package_validation::UpgradePlanCapability</a>,
) {
    <b>let</b> token_issuer = <a href="stc_util.md#0x1_stc_util_token_issuer">stc_util::token_issuer</a>&lt;TokenT&gt;();
    <b>assert</b>!(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>) == token_issuer, <a href="../../move-stdlib/doc/error.md#0x1_error_unauthenticated">error::unauthenticated</a>(<a href="dao_upgrade_module_proposal.md#0x1_dao_upgrade_module_proposal_ERR_NOT_AUTHORIZED">ERR_NOT_AUTHORIZED</a>));
    <b>move_to</b>(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, <a href="dao_upgrade_module_proposal.md#0x1_dao_upgrade_module_proposal_UpgradeModuleCapability">UpgradeModuleCapability</a>&lt;TokenT&gt; { cap })
}
</code></pre>



</details>

<a id="0x1_dao_upgrade_module_proposal_propose_module_upgrade_v2"></a>

## Function `propose_module_upgrade_v2`



<pre><code><b>public</b> <b>fun</b> <a href="dao_upgrade_module_proposal.md#0x1_dao_upgrade_module_proposal_propose_module_upgrade_v2">propose_module_upgrade_v2</a>&lt;TokenT&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, module_address: <b>address</b>, package_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, version: u64, exec_delay: u64, enforced: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="dao_upgrade_module_proposal.md#0x1_dao_upgrade_module_proposal_propose_module_upgrade_v2">propose_module_upgrade_v2</a>&lt;TokenT&gt;(
    <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    module_address: <b>address</b>,
    package_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    version: u64,
    exec_delay: u64,
    enforced: bool,
) <b>acquires</b> <a href="dao_upgrade_module_proposal.md#0x1_dao_upgrade_module_proposal_UpgradeModuleCapability">UpgradeModuleCapability</a> {
    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"<a href="dao_upgrade_module_proposal.md#0x1_dao_upgrade_module_proposal_propose_module_upgrade_v2">dao_upgrade_module_proposal::propose_module_upgrade_v2</a> | entered"));
    <b>let</b> cap = <b>borrow_global</b>&lt;<a href="dao_upgrade_module_proposal.md#0x1_dao_upgrade_module_proposal_UpgradeModuleCapability">UpgradeModuleCapability</a>&lt;TokenT&gt;&gt;(<a href="stc_util.md#0x1_stc_util_token_issuer">stc_util::token_issuer</a>&lt;TokenT&gt;());
    <b>let</b> account_address = <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_account_address">stc_transaction_package_validation::account_address</a>(&cap.cap);

    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"<a href="dao_upgrade_module_proposal.md#0x1_dao_upgrade_module_proposal_propose_module_upgrade_v2">dao_upgrade_module_proposal::propose_module_upgrade_v2</a> | cap"));
    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(cap);

    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"<a href="dao_upgrade_module_proposal.md#0x1_dao_upgrade_module_proposal_propose_module_upgrade_v2">dao_upgrade_module_proposal::propose_module_upgrade_v2</a> | account_address"));
    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&account_address);

    <b>assert</b>!(account_address == module_address, <a href="../../move-stdlib/doc/error.md#0x1_error_permission_denied">error::permission_denied</a>(<a href="dao_upgrade_module_proposal.md#0x1_dao_upgrade_module_proposal_ERR_ADDRESS_MISSMATCH">ERR_ADDRESS_MISSMATCH</a>));
    <a href="dao.md#0x1_dao_propose">dao::propose</a>&lt;TokenT, <a href="dao_upgrade_module_proposal.md#0x1_dao_upgrade_module_proposal_UpgradeModuleV2">UpgradeModuleV2</a>&gt;(
        <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
        <a href="dao_upgrade_module_proposal.md#0x1_dao_upgrade_module_proposal_UpgradeModuleV2">UpgradeModuleV2</a> { module_address, package_hash, version, enforced },
        exec_delay,
    );
    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"<a href="dao_upgrade_module_proposal.md#0x1_dao_upgrade_module_proposal_propose_module_upgrade_v2">dao_upgrade_module_proposal::propose_module_upgrade_v2</a> | exited"));
}
</code></pre>



</details>

<a id="0x1_dao_upgrade_module_proposal_submit_module_upgrade_plan"></a>

## Function `submit_module_upgrade_plan`

Once the proposal is agreed, anyone can call this method to generate the upgrading plan.


<pre><code><b>public</b> <b>fun</b> <a href="dao_upgrade_module_proposal.md#0x1_dao_upgrade_module_proposal_submit_module_upgrade_plan">submit_module_upgrade_plan</a>&lt;TokenT&gt;(proposer_address: <b>address</b>, proposal_id: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="dao_upgrade_module_proposal.md#0x1_dao_upgrade_module_proposal_submit_module_upgrade_plan">submit_module_upgrade_plan</a>&lt;TokenT&gt;(
    proposer_address: <b>address</b>,
    proposal_id: u64,
) <b>acquires</b> <a href="dao_upgrade_module_proposal.md#0x1_dao_upgrade_module_proposal_UpgradeModuleCapability">UpgradeModuleCapability</a> {
    <b>let</b> <a href="dao_upgrade_module_proposal.md#0x1_dao_upgrade_module_proposal_UpgradeModuleV2">UpgradeModuleV2</a> {
        module_address, package_hash, version, enforced
    } = <a href="dao.md#0x1_dao_extract_proposal_action">dao::extract_proposal_action</a>&lt;
        TokenT,
        <a href="dao_upgrade_module_proposal.md#0x1_dao_upgrade_module_proposal_UpgradeModuleV2">UpgradeModuleV2</a>,
    &gt;(proposer_address, proposal_id);
    <b>let</b> cap = <b>borrow_global</b>&lt;<a href="dao_upgrade_module_proposal.md#0x1_dao_upgrade_module_proposal_UpgradeModuleCapability">UpgradeModuleCapability</a>&lt;TokenT&gt;&gt;(<a href="stc_util.md#0x1_stc_util_token_issuer">stc_util::token_issuer</a>&lt;TokenT&gt;());
    <b>let</b> account_address = <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_account_address">stc_transaction_package_validation::account_address</a>(&cap.cap);
    <b>assert</b>!(account_address == module_address, <a href="../../move-stdlib/doc/error.md#0x1_error_permission_denied">error::permission_denied</a>(<a href="dao_upgrade_module_proposal.md#0x1_dao_upgrade_module_proposal_ERR_ADDRESS_MISSMATCH">ERR_ADDRESS_MISSMATCH</a>));
    <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_submit_upgrade_plan_with_cap_v2">stc_transaction_package_validation::submit_upgrade_plan_with_cap_v2</a>(
        &cap.cap,
        package_hash,
        version,
        enforced,
    );
}
</code></pre>



</details>

<a id="@Specification_1"></a>

## Specification



<pre><code><b>pragma</b> verify = <b>false</b>;
<b>pragma</b> aborts_if_is_strict;
<b>pragma</b> aborts_if_is_partial;
</code></pre>



<a id="@Specification_1_plugin"></a>

### Function `plugin`


<pre><code><b>public</b> <b>fun</b> <a href="dao_upgrade_module_proposal.md#0x1_dao_upgrade_module_proposal_plugin">plugin</a>&lt;TokenT&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, cap: <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_UpgradePlanCapability">stc_transaction_package_validation::UpgradePlanCapability</a>)
</code></pre>




<pre><code><b>pragma</b> aborts_if_is_partial = <b>false</b>;
<b>let</b> sender = <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>);
<b>aborts_if</b> sender != @0x2;
<b>aborts_if</b> <b>exists</b>&lt;<a href="dao_upgrade_module_proposal.md#0x1_dao_upgrade_module_proposal_UpgradeModuleCapability">UpgradeModuleCapability</a>&lt;TokenT&gt;&gt;(sender);
</code></pre>




<a id="0x1_dao_upgrade_module_proposal_AbortIfUnableUpgrade"></a>


<pre><code><b>schema</b> <a href="dao_upgrade_module_proposal.md#0x1_dao_upgrade_module_proposal_AbortIfUnableUpgrade">AbortIfUnableUpgrade</a>&lt;TokenT&gt; {
    module_address: <b>address</b>;
    <b>let</b> token_issuer = @0x2;
    <b>aborts_if</b> !<b>exists</b>&lt;<a href="dao_upgrade_module_proposal.md#0x1_dao_upgrade_module_proposal_UpgradeModuleCapability">UpgradeModuleCapability</a>&lt;TokenT&gt;&gt;(token_issuer);
    <b>let</b> cap = <b>global</b>&lt;<a href="dao_upgrade_module_proposal.md#0x1_dao_upgrade_module_proposal_UpgradeModuleCapability">UpgradeModuleCapability</a>&lt;TokenT&gt;&gt;(token_issuer).cap;
    <b>aborts_if</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_account_address">stc_transaction_package_validation::account_address</a>(cap) != module_address;
}
</code></pre>



<a id="@Specification_1_propose_module_upgrade_v2"></a>

### Function `propose_module_upgrade_v2`


<pre><code><b>public</b> <b>fun</b> <a href="dao_upgrade_module_proposal.md#0x1_dao_upgrade_module_proposal_propose_module_upgrade_v2">propose_module_upgrade_v2</a>&lt;TokenT&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, module_address: <b>address</b>, package_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, version: u64, exec_delay: u64, enforced: bool)
</code></pre>




<pre><code><b>pragma</b> aborts_if_is_partial = <b>true</b>;
<b>include</b> <a href="dao_upgrade_module_proposal.md#0x1_dao_upgrade_module_proposal_AbortIfUnableUpgrade">AbortIfUnableUpgrade</a>&lt;TokenT&gt;;
</code></pre>



<a id="@Specification_1_submit_module_upgrade_plan"></a>

### Function `submit_module_upgrade_plan`


<pre><code><b>public</b> <b>fun</b> <a href="dao_upgrade_module_proposal.md#0x1_dao_upgrade_module_proposal_submit_module_upgrade_plan">submit_module_upgrade_plan</a>&lt;TokenT&gt;(proposer_address: <b>address</b>, proposal_id: u64)
</code></pre>




<pre><code><b>let</b> expected_states = vec&lt;u8&gt;(6);
<b>include</b> <a href="dao.md#0x1_dao_CheckProposalStates">dao::CheckProposalStates</a>&lt;TokenT, <a href="dao_upgrade_module_proposal.md#0x1_dao_upgrade_module_proposal_UpgradeModuleV2">UpgradeModuleV2</a>&gt; { expected_states };
<b>let</b> proposal = <b>global</b>&lt;<a href="dao.md#0x1_dao_Proposal">dao::Proposal</a>&lt;TokenT, <a href="dao_upgrade_module_proposal.md#0x1_dao_upgrade_module_proposal_UpgradeModuleV2">UpgradeModuleV2</a>&gt;&gt;(proposer_address);
<b>aborts_if</b> <a href="../../move-stdlib/doc/option.md#0x1_option_is_none">option::is_none</a>(proposal.action);
<b>let</b> action = proposal.action.vec[0];
<b>include</b> <a href="dao_upgrade_module_proposal.md#0x1_dao_upgrade_module_proposal_AbortIfUnableUpgrade">AbortIfUnableUpgrade</a>&lt;TokenT&gt; { module_address: action.module_address };
</code></pre>


[move-book]: https://starcoin.dev/move/book/SUMMARY
