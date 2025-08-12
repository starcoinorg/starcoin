
<a id="0x1_dao_modify_config_proposal"></a>

# Module `0x1::dao_modify_config_proposal`

A proposal module which is used to modify Token's DAO configuration.


-  [Resource `DaoConfigModifyCapability`](#0x1_dao_modify_config_proposal_DaoConfigModifyCapability)
-  [Struct `DaoConfigUpdate`](#0x1_dao_modify_config_proposal_DaoConfigUpdate)
-  [Constants](#@Constants_0)
-  [Function `plugin`](#0x1_dao_modify_config_proposal_plugin)
-  [Function `propose`](#0x1_dao_modify_config_proposal_propose)
-  [Function `execute`](#0x1_dao_modify_config_proposal_execute)
-  [Specification](#@Specification_1)
    -  [Function `plugin`](#@Specification_1_plugin)
    -  [Function `propose`](#@Specification_1_propose)
    -  [Function `execute`](#@Specification_1_execute)


<pre><code><b>use</b> <a href="dao.md#0x1_dao">0x1::dao</a>;
<b>use</b> <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug">0x1::debug</a>;
<b>use</b> <a href="../../move-stdlib/doc/error.md#0x1_error">0x1::error</a>;
<b>use</b> <a href="on_chain_config.md#0x1_on_chain_config">0x1::on_chain_config</a>;
<b>use</b> <a href="../../move-stdlib/doc/signer.md#0x1_signer">0x1::signer</a>;
<b>use</b> <a href="stc_util.md#0x1_stc_util">0x1::stc_util</a>;
<b>use</b> <a href="../../move-stdlib/doc/string.md#0x1_string">0x1::string</a>;
</code></pre>



<a id="0x1_dao_modify_config_proposal_DaoConfigModifyCapability"></a>

## Resource `DaoConfigModifyCapability`

A wrapper of <code><a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapability">on_chain_config::ModifyConfigCapability</a>&lt;<a href="dao.md#0x1_dao_DaoConfig">dao::DaoConfig</a>&lt;TokenT&gt;&gt;</code>.


<pre><code><b>struct</b> <a href="dao_modify_config_proposal.md#0x1_dao_modify_config_proposal_DaoConfigModifyCapability">DaoConfigModifyCapability</a>&lt;TokenT&gt; <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>cap: <a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapability">on_chain_config::ModifyConfigCapability</a>&lt;<a href="dao.md#0x1_dao_DaoConfig">dao::DaoConfig</a>&lt;TokenT&gt;&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="0x1_dao_modify_config_proposal_DaoConfigUpdate"></a>

## Struct `DaoConfigUpdate`

a proposal action to update dao config.
if any field is <code>0</code>, that means the proposal want to update.


<pre><code><b>struct</b> <a href="dao_modify_config_proposal.md#0x1_dao_modify_config_proposal_DaoConfigUpdate">DaoConfigUpdate</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>voting_delay: u64</code>
</dt>
<dd>
 new voting delay setting.
</dd>
<dt>
<code>voting_period: u64</code>
</dt>
<dd>
 new voting period setting.
</dd>
<dt>
<code>voting_quorum_rate: u8</code>
</dt>
<dd>
 new voting quorum rate setting.
</dd>
<dt>
<code>min_action_delay: u64</code>
</dt>
<dd>
 new min action delay setting.
</dd>
</dl>


</details>

<a id="@Constants_0"></a>

## Constants


<a id="0x1_dao_modify_config_proposal_ERR_NOT_AUTHORIZED"></a>



<pre><code><b>const</b> <a href="dao_modify_config_proposal.md#0x1_dao_modify_config_proposal_ERR_NOT_AUTHORIZED">ERR_NOT_AUTHORIZED</a>: u64 = 401;
</code></pre>



<a id="0x1_dao_modify_config_proposal_ERR_QUORUM_RATE_INVALID"></a>



<pre><code><b>const</b> <a href="dao_modify_config_proposal.md#0x1_dao_modify_config_proposal_ERR_QUORUM_RATE_INVALID">ERR_QUORUM_RATE_INVALID</a>: u64 = 402;
</code></pre>



<a id="0x1_dao_modify_config_proposal_plugin"></a>

## Function `plugin`

Plugin method of the module.
Should be called by token issuer.


<pre><code><b>public</b> <b>fun</b> <a href="dao_modify_config_proposal.md#0x1_dao_modify_config_proposal_plugin">plugin</a>&lt;TokenT&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="dao_modify_config_proposal.md#0x1_dao_modify_config_proposal_plugin">plugin</a>&lt;TokenT&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>) {
    <b>let</b> token_issuer = <a href="stc_util.md#0x1_stc_util_token_issuer">stc_util::token_issuer</a>&lt;TokenT&gt;();
    <b>assert</b>!(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>) == token_issuer, <a href="../../move-stdlib/doc/error.md#0x1_error_unauthenticated">error::unauthenticated</a>(<a href="dao_modify_config_proposal.md#0x1_dao_modify_config_proposal_ERR_NOT_AUTHORIZED">ERR_NOT_AUTHORIZED</a>));
    <b>let</b> dao_config_modify_cap = <a href="on_chain_config.md#0x1_on_chain_config_extract_modify_config_capability">on_chain_config::extract_modify_config_capability</a>&lt;
        <a href="dao.md#0x1_dao_DaoConfig">dao::DaoConfig</a>&lt;TokenT&gt;,
    &gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>);
    <b>assert</b>!(
        <a href="on_chain_config.md#0x1_on_chain_config_account_address">on_chain_config::account_address</a>(&dao_config_modify_cap) == token_issuer,
        <a href="../../move-stdlib/doc/error.md#0x1_error_unauthenticated">error::unauthenticated</a>(<a href="dao_modify_config_proposal.md#0x1_dao_modify_config_proposal_ERR_NOT_AUTHORIZED">ERR_NOT_AUTHORIZED</a>)
    );
    <b>let</b> cap = <a href="dao_modify_config_proposal.md#0x1_dao_modify_config_proposal_DaoConfigModifyCapability">DaoConfigModifyCapability</a> { cap: dao_config_modify_cap };
    <b>move_to</b>(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, cap);
}
</code></pre>



</details>

<a id="0x1_dao_modify_config_proposal_propose"></a>

## Function `propose`

Entrypoint for the proposal.


<pre><code><b>public</b> entry <b>fun</b> <a href="dao_modify_config_proposal.md#0x1_dao_modify_config_proposal_propose">propose</a>&lt;TokenT&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, voting_delay: u64, voting_period: u64, voting_quorum_rate: u8, min_action_delay: u64, exec_delay: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="dao_modify_config_proposal.md#0x1_dao_modify_config_proposal_propose">propose</a>&lt;TokenT&gt;(
    <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    voting_delay: u64,
    voting_period: u64,
    voting_quorum_rate: u8,
    min_action_delay: u64,
    exec_delay: u64,
) {
    <b>assert</b>!(voting_quorum_rate &lt;= 100, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="dao_modify_config_proposal.md#0x1_dao_modify_config_proposal_ERR_QUORUM_RATE_INVALID">ERR_QUORUM_RATE_INVALID</a>));
    <b>let</b> action = <a href="dao_modify_config_proposal.md#0x1_dao_modify_config_proposal_DaoConfigUpdate">DaoConfigUpdate</a> {
        voting_delay,
        voting_period,
        voting_quorum_rate,
        min_action_delay,
    };
    <a href="dao.md#0x1_dao_propose">dao::propose</a>&lt;TokenT, <a href="dao_modify_config_proposal.md#0x1_dao_modify_config_proposal_DaoConfigUpdate">DaoConfigUpdate</a>&gt;(&<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, action, exec_delay);
}
</code></pre>



</details>

<a id="0x1_dao_modify_config_proposal_execute"></a>

## Function `execute`

Once the proposal is agreed, anyone can call the method to make the proposal happen.


<pre><code><b>public</b> entry <b>fun</b> <a href="dao_modify_config_proposal.md#0x1_dao_modify_config_proposal_execute">execute</a>&lt;TokenT&gt;(proposer_address: <b>address</b>, proposal_id: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="dao_modify_config_proposal.md#0x1_dao_modify_config_proposal_execute">execute</a>&lt;TokenT&gt;(proposer_address: <b>address</b>, proposal_id: u64) <b>acquires</b> <a href="dao_modify_config_proposal.md#0x1_dao_modify_config_proposal_DaoConfigModifyCapability">DaoConfigModifyCapability</a> {
    <b>let</b> <a href="dao_modify_config_proposal.md#0x1_dao_modify_config_proposal_DaoConfigUpdate">DaoConfigUpdate</a> {
        voting_delay,
        voting_period,
        voting_quorum_rate,
        min_action_delay,
    } = <a href="dao.md#0x1_dao_extract_proposal_action">dao::extract_proposal_action</a>&lt;TokenT, <a href="dao_modify_config_proposal.md#0x1_dao_modify_config_proposal_DaoConfigUpdate">DaoConfigUpdate</a>&gt;(proposer_address, proposal_id);

    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"<a href="dao_modify_config_proposal.md#0x1_dao_modify_config_proposal_execute">dao_modify_config_proposal::execute</a> | entered"));

    <b>let</b> cap = <b>borrow_global_mut</b>&lt;<a href="dao_modify_config_proposal.md#0x1_dao_modify_config_proposal_DaoConfigModifyCapability">DaoConfigModifyCapability</a>&lt;TokenT&gt;&gt;(
        <a href="stc_util.md#0x1_stc_util_token_issuer">stc_util::token_issuer</a>&lt;TokenT&gt;(),
    );
    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(
        &std::string::utf8(
            b"<a href="dao_modify_config_proposal.md#0x1_dao_modify_config_proposal_execute">dao_modify_config_proposal::execute</a> | <b>borrow_global_mut</b>&lt;<a href="dao_modify_config_proposal.md#0x1_dao_modify_config_proposal_DaoConfigModifyCapability">DaoConfigModifyCapability</a>&lt;TokenT&gt;&gt;"
        )
    );

    <a href="dao.md#0x1_dao_modify_dao_config">dao::modify_dao_config</a>(
        &<b>mut</b> cap.cap,
        voting_delay,
        voting_period,
        voting_quorum_rate,
        min_action_delay,
    );

    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"<a href="dao_modify_config_proposal.md#0x1_dao_modify_config_proposal_execute">dao_modify_config_proposal::execute</a> | exited"));
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


<pre><code><b>public</b> <b>fun</b> <a href="dao_modify_config_proposal.md#0x1_dao_modify_config_proposal_plugin">plugin</a>&lt;TokenT&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>)
</code></pre>




<pre><code><b>pragma</b> aborts_if_is_partial = <b>false</b>;
<b>let</b> sender = <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>);
<b>aborts_if</b> sender != @0x2;
<b>include</b> <a href="on_chain_config.md#0x1_on_chain_config_AbortsIfCapNotExist">on_chain_config::AbortsIfCapNotExist</a>&lt;<a href="dao.md#0x1_dao_DaoConfig">dao::DaoConfig</a>&lt;TokenT&gt;&gt; { <b>address</b>: sender };
<b>let</b> config_cap =
    <a href="on_chain_config.md#0x1_on_chain_config_spec_cap">on_chain_config::spec_cap</a>&lt;<a href="dao.md#0x1_dao_DaoConfig">dao::DaoConfig</a>&lt;TokenT&gt;&gt;(sender);
<b>aborts_if</b> <a href="../../move-stdlib/doc/option.md#0x1_option_is_none">option::is_none</a>(config_cap);
<b>aborts_if</b> <a href="../../move-stdlib/doc/option.md#0x1_option_borrow">option::borrow</a>(config_cap).account_address != sender;
<b>aborts_if</b> <b>exists</b>&lt;<a href="dao_modify_config_proposal.md#0x1_dao_modify_config_proposal_DaoConfigModifyCapability">DaoConfigModifyCapability</a>&lt;TokenT&gt;&gt;(sender);
<b>ensures</b> <b>exists</b>&lt;<a href="dao_modify_config_proposal.md#0x1_dao_modify_config_proposal_DaoConfigModifyCapability">DaoConfigModifyCapability</a>&lt;TokenT&gt;&gt;(sender);
</code></pre>



<a id="@Specification_1_propose"></a>

### Function `propose`


<pre><code><b>public</b> entry <b>fun</b> <a href="dao_modify_config_proposal.md#0x1_dao_modify_config_proposal_propose">propose</a>&lt;TokenT&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, voting_delay: u64, voting_period: u64, voting_quorum_rate: u8, min_action_delay: u64, exec_delay: u64)
</code></pre>




<pre><code><b>pragma</b> aborts_if_is_partial = <b>false</b>;
<b>aborts_if</b> voting_quorum_rate &gt; 100;
<b>include</b> <a href="dao.md#0x1_dao_AbortIfDaoConfigNotExist">dao::AbortIfDaoConfigNotExist</a>&lt;TokenT&gt;;
<b>include</b> <a href="dao.md#0x1_dao_AbortIfDaoInfoNotExist">dao::AbortIfDaoInfoNotExist</a>&lt;TokenT&gt;;
<b>aborts_if</b> !<b>exists</b>&lt;<a href="timestamp.md#0x1_timestamp_CurrentTimeMicroseconds">timestamp::CurrentTimeMicroseconds</a>&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>());
<b>aborts_if</b> exec_delay &gt; 0 && exec_delay &lt; <a href="dao.md#0x1_dao_spec_dao_config">dao::spec_dao_config</a>&lt;TokenT&gt;().min_action_delay;
<b>include</b> <a href="dao.md#0x1_dao_CheckQuorumVotes">dao::CheckQuorumVotes</a>&lt;TokenT&gt;;
<b>let</b> sender = <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>);
<b>aborts_if</b> <b>exists</b>&lt;<a href="dao.md#0x1_dao_Proposal">dao::Proposal</a>&lt;TokenT, <a href="dao_modify_config_proposal.md#0x1_dao_modify_config_proposal_DaoConfigUpdate">DaoConfigUpdate</a>&gt;&gt;(sender);
</code></pre>



<a id="@Specification_1_execute"></a>

### Function `execute`


<pre><code><b>public</b> entry <b>fun</b> <a href="dao_modify_config_proposal.md#0x1_dao_modify_config_proposal_execute">execute</a>&lt;TokenT&gt;(proposer_address: <b>address</b>, proposal_id: u64)
</code></pre>




<pre><code><b>pragma</b> aborts_if_is_partial = <b>true</b>;
<b>aborts_if</b> !<b>exists</b>&lt;<a href="dao_modify_config_proposal.md#0x1_dao_modify_config_proposal_DaoConfigModifyCapability">DaoConfigModifyCapability</a>&lt;TokenT&gt;&gt;(@0x2);
</code></pre>


[move-book]: https://starcoin.dev/move/book/SUMMARY
