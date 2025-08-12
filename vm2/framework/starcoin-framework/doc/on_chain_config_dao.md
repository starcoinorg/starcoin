
<a id="0x1_on_chain_config_dao"></a>

# Module `0x1::on_chain_config_dao`

OnChainConfigDao is a DAO proposal for modify onchain configuration.


-  [Resource `WrappedConfigModifyCapability`](#0x1_on_chain_config_dao_WrappedConfigModifyCapability)
-  [Struct `OnChainConfigUpdate`](#0x1_on_chain_config_dao_OnChainConfigUpdate)
-  [Constants](#@Constants_0)
-  [Function `plugin`](#0x1_on_chain_config_dao_plugin)
-  [Function `propose_update`](#0x1_on_chain_config_dao_propose_update)
-  [Function `execute`](#0x1_on_chain_config_dao_execute)
-  [Specification](#@Specification_1)
    -  [Function `plugin`](#@Specification_1_plugin)
    -  [Function `propose_update`](#@Specification_1_propose_update)
    -  [Function `execute`](#@Specification_1_execute)


<pre><code><b>use</b> <a href="dao.md#0x1_dao">0x1::dao</a>;
<b>use</b> <a href="../../move-stdlib/doc/error.md#0x1_error">0x1::error</a>;
<b>use</b> <a href="on_chain_config.md#0x1_on_chain_config">0x1::on_chain_config</a>;
<b>use</b> <a href="../../move-stdlib/doc/signer.md#0x1_signer">0x1::signer</a>;
<b>use</b> <a href="stc_util.md#0x1_stc_util">0x1::stc_util</a>;
</code></pre>



<a id="0x1_on_chain_config_dao_WrappedConfigModifyCapability"></a>

## Resource `WrappedConfigModifyCapability`

A wrapper of <code>Config::ModifyConfigCapability&lt;ConfigT&gt;</code>.


<pre><code><b>struct</b> <a href="on_chain_config_dao.md#0x1_on_chain_config_dao_WrappedConfigModifyCapability">WrappedConfigModifyCapability</a>&lt;TokenT, ConfigT: <b>copy</b>, drop, store&gt; <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>cap: <a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapability">on_chain_config::ModifyConfigCapability</a>&lt;ConfigT&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="0x1_on_chain_config_dao_OnChainConfigUpdate"></a>

## Struct `OnChainConfigUpdate`

request of updating configuration.


<pre><code><b>struct</b> <a href="on_chain_config_dao.md#0x1_on_chain_config_dao_OnChainConfigUpdate">OnChainConfigUpdate</a>&lt;ConfigT: <b>copy</b>, drop, store&gt; <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>value: ConfigT</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="@Constants_0"></a>

## Constants


<a id="0x1_on_chain_config_dao_ERR_NOT_AUTHORIZED"></a>



<pre><code><b>const</b> <a href="on_chain_config_dao.md#0x1_on_chain_config_dao_ERR_NOT_AUTHORIZED">ERR_NOT_AUTHORIZED</a>: u64 = 401;
</code></pre>



<a id="0x1_on_chain_config_dao_plugin"></a>

## Function `plugin`

Plugin method of the module.
Should be called by token issuer.


<pre><code><b>public</b> <b>fun</b> <a href="on_chain_config_dao.md#0x1_on_chain_config_dao_plugin">plugin</a>&lt;TokenT, ConfigT: <b>copy</b>, drop, store&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="on_chain_config_dao.md#0x1_on_chain_config_dao_plugin">plugin</a>&lt;TokenT, ConfigT: <b>copy</b> + drop + store&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>) {
    <b>let</b> token_issuer = <a href="stc_util.md#0x1_stc_util_token_issuer">stc_util::token_issuer</a>&lt;TokenT&gt;(); // coin::token_address&lt;TokenT&gt;();
    <b>assert</b>!(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>) == token_issuer, <a href="../../move-stdlib/doc/error.md#0x1_error_not_found">error::not_found</a>(<a href="on_chain_config_dao.md#0x1_on_chain_config_dao_ERR_NOT_AUTHORIZED">ERR_NOT_AUTHORIZED</a>));
    <b>let</b> config_modify_cap = <a href="on_chain_config.md#0x1_on_chain_config_extract_modify_config_capability">on_chain_config::extract_modify_config_capability</a>&lt;ConfigT&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>);
    <b>let</b> cap = <a href="on_chain_config_dao.md#0x1_on_chain_config_dao_WrappedConfigModifyCapability">WrappedConfigModifyCapability</a>&lt;TokenT, ConfigT&gt; { cap: config_modify_cap };
    <b>move_to</b>(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, cap);
}
</code></pre>



</details>

<a id="0x1_on_chain_config_dao_propose_update"></a>

## Function `propose_update`

issue a proposal to update config of ConfigT goved by TokenT


<pre><code><b>public</b> <b>fun</b> <a href="on_chain_config_dao.md#0x1_on_chain_config_dao_propose_update">propose_update</a>&lt;TokenT, ConfigT: <b>copy</b>, drop, store&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, new_config: ConfigT, exec_delay: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="on_chain_config_dao.md#0x1_on_chain_config_dao_propose_update">propose_update</a>&lt;TokenT, ConfigT: <b>copy</b> + drop + store&gt;(
    <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    new_config: ConfigT,
    exec_delay: u64,
) {
    <a href="dao.md#0x1_dao_propose">dao::propose</a>&lt;TokenT, <a href="on_chain_config_dao.md#0x1_on_chain_config_dao_OnChainConfigUpdate">OnChainConfigUpdate</a>&lt;ConfigT&gt;&gt;(
        <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
        <a href="on_chain_config_dao.md#0x1_on_chain_config_dao_OnChainConfigUpdate">OnChainConfigUpdate</a> { value: new_config },
        exec_delay,
    );
}
</code></pre>



</details>

<a id="0x1_on_chain_config_dao_execute"></a>

## Function `execute`

Once the proposal is agreed, anyone can call the method to make the proposal happen.
Caller need to make sure that the proposal of <code>proposal_id</code> under <code>proposal_address</code> is
the kind of this proposal module.


<pre><code><b>public</b> <b>fun</b> <a href="on_chain_config_dao.md#0x1_on_chain_config_dao_execute">execute</a>&lt;TokenT, ConfigT: <b>copy</b>, drop, store&gt;(proposer_address: <b>address</b>, proposal_id: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="on_chain_config_dao.md#0x1_on_chain_config_dao_execute">execute</a>&lt;TokenT, ConfigT: <b>copy</b> + drop + store&gt;(
    proposer_address: <b>address</b>,
    proposal_id: u64,
) <b>acquires</b> <a href="on_chain_config_dao.md#0x1_on_chain_config_dao_WrappedConfigModifyCapability">WrappedConfigModifyCapability</a> {
    <b>let</b> <a href="on_chain_config_dao.md#0x1_on_chain_config_dao_OnChainConfigUpdate">OnChainConfigUpdate</a> { value } = <a href="dao.md#0x1_dao_extract_proposal_action">dao::extract_proposal_action</a>&lt;
        TokenT,
        <a href="on_chain_config_dao.md#0x1_on_chain_config_dao_OnChainConfigUpdate">OnChainConfigUpdate</a>&lt;ConfigT&gt;,
    &gt;(proposer_address, proposal_id);
    <b>let</b> cap = <b>borrow_global_mut</b>&lt;<a href="on_chain_config_dao.md#0x1_on_chain_config_dao_WrappedConfigModifyCapability">WrappedConfigModifyCapability</a>&lt;TokenT, ConfigT&gt;&gt;(
        <a href="stc_util.md#0x1_stc_util_token_issuer">stc_util::token_issuer</a>&lt;TokenT&gt;(),
    );
    <a href="on_chain_config.md#0x1_on_chain_config_set_with_capability">on_chain_config::set_with_capability</a>(&<b>mut</b> cap.cap, value);
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


<pre><code><b>public</b> <b>fun</b> <a href="on_chain_config_dao.md#0x1_on_chain_config_dao_plugin">plugin</a>&lt;TokenT, ConfigT: <b>copy</b>, drop, store&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>)
</code></pre>




<pre><code><b>pragma</b> aborts_if_is_partial = <b>false</b>;
<b>let</b> sender = <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>);
<b>aborts_if</b> sender != @0x2;
<b>include</b> <a href="on_chain_config.md#0x1_on_chain_config_AbortsIfCapNotExist">on_chain_config::AbortsIfCapNotExist</a>&lt;ConfigT&gt; { <b>address</b>: sender };
<b>aborts_if</b> <b>exists</b>&lt;<a href="on_chain_config_dao.md#0x1_on_chain_config_dao_WrappedConfigModifyCapability">WrappedConfigModifyCapability</a>&lt;TokenT, ConfigT&gt;&gt;(sender);
<b>ensures</b> <b>exists</b>&lt;<a href="on_chain_config_dao.md#0x1_on_chain_config_dao_WrappedConfigModifyCapability">WrappedConfigModifyCapability</a>&lt;TokenT, ConfigT&gt;&gt;(sender);
</code></pre>



<a id="@Specification_1_propose_update"></a>

### Function `propose_update`


<pre><code><b>public</b> <b>fun</b> <a href="on_chain_config_dao.md#0x1_on_chain_config_dao_propose_update">propose_update</a>&lt;TokenT, ConfigT: <b>copy</b>, drop, store&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, new_config: ConfigT, exec_delay: u64)
</code></pre>




<pre><code><b>pragma</b> aborts_if_is_partial = <b>false</b>;
<b>include</b> <a href="dao.md#0x1_dao_AbortIfDaoConfigNotExist">dao::AbortIfDaoConfigNotExist</a>&lt;TokenT&gt;;
<b>include</b> <a href="dao.md#0x1_dao_AbortIfDaoInfoNotExist">dao::AbortIfDaoInfoNotExist</a>&lt;TokenT&gt;;
<b>aborts_if</b> !<b>exists</b>&lt;<a href="timestamp.md#0x1_timestamp_CurrentTimeMicroseconds">timestamp::CurrentTimeMicroseconds</a>&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>());
<b>aborts_if</b> exec_delay &gt; 0 && exec_delay &lt; <a href="dao.md#0x1_dao_spec_dao_config">dao::spec_dao_config</a>&lt;TokenT&gt;().min_action_delay;
<b>include</b> <a href="dao.md#0x1_dao_CheckQuorumVotes">dao::CheckQuorumVotes</a>&lt;TokenT&gt;;
<b>let</b> sender = <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>);
<b>aborts_if</b> <b>exists</b>&lt;<a href="dao.md#0x1_dao_Proposal">dao::Proposal</a>&lt;TokenT, <a href="on_chain_config_dao.md#0x1_on_chain_config_dao_OnChainConfigUpdate">OnChainConfigUpdate</a>&lt;ConfigT&gt;&gt;&gt;(sender);
</code></pre>



<a id="@Specification_1_execute"></a>

### Function `execute`


<pre><code><b>public</b> <b>fun</b> <a href="on_chain_config_dao.md#0x1_on_chain_config_dao_execute">execute</a>&lt;TokenT, ConfigT: <b>copy</b>, drop, store&gt;(proposer_address: <b>address</b>, proposal_id: u64)
</code></pre>




<pre><code><b>pragma</b> aborts_if_is_partial = <b>true</b>;
<b>let</b> expected_states = vec&lt;u8&gt;(6);
<b>include</b> <a href="dao.md#0x1_dao_CheckProposalStates">dao::CheckProposalStates</a>&lt;TokenT, <a href="on_chain_config_dao.md#0x1_on_chain_config_dao_OnChainConfigUpdate">OnChainConfigUpdate</a>&lt;ConfigT&gt;&gt; { expected_states };
<b>aborts_if</b> !<b>exists</b>&lt;<a href="on_chain_config_dao.md#0x1_on_chain_config_dao_WrappedConfigModifyCapability">WrappedConfigModifyCapability</a>&lt;TokenT, ConfigT&gt;&gt;(@0x2);
</code></pre>


[move-book]: https://starcoin.dev/move/book/SUMMARY
