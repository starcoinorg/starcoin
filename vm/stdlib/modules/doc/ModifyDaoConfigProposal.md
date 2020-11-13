
<a name="0x1_ModifyDaoConfigProposal"></a>

# Module `0x1::ModifyDaoConfigProposal`



-  [Resource `DaoConfigModifyCapability`](#0x1_ModifyDaoConfigProposal_DaoConfigModifyCapability)
-  [Struct `DaoConfigUpdate`](#0x1_ModifyDaoConfigProposal_DaoConfigUpdate)
-  [Constants](#@Constants_0)
-  [Function `plugin`](#0x1_ModifyDaoConfigProposal_plugin)
-  [Function `propose`](#0x1_ModifyDaoConfigProposal_propose)
-  [Function `execute`](#0x1_ModifyDaoConfigProposal_execute)
-  [Specification](#@Specification_1)
    -  [Function `plugin`](#@Specification_1_plugin)
    -  [Function `propose`](#@Specification_1_propose)
    -  [Function `execute`](#@Specification_1_execute)


<pre><code><b>use</b> <a href="Config.md#0x1_Config">0x1::Config</a>;
<b>use</b> <a href="Dao.md#0x1_Dao">0x1::Dao</a>;
<b>use</b> <a href="Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="Signer.md#0x1_Signer">0x1::Signer</a>;
<b>use</b> <a href="Token.md#0x1_Token">0x1::Token</a>;
</code></pre>



<a name="0x1_ModifyDaoConfigProposal_DaoConfigModifyCapability"></a>

## Resource `DaoConfigModifyCapability`



<pre><code><b>resource</b> <b>struct</b> <a href="ModifyDaoConfigProposal.md#0x1_ModifyDaoConfigProposal_DaoConfigModifyCapability">DaoConfigModifyCapability</a>&lt;TokenT: <b>copyable</b>&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>cap: <a href="Config.md#0x1_Config_ModifyConfigCapability">Config::ModifyConfigCapability</a>&lt;<a href="Dao.md#0x1_Dao_DaoConfig">Dao::DaoConfig</a>&lt;TokenT&gt;&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_ModifyDaoConfigProposal_DaoConfigUpdate"></a>

## Struct `DaoConfigUpdate`

a proposal action to update dao config.
if any field is <code>0</code>, that means the proposal want to update.


<pre><code><b>struct</b> <a href="ModifyDaoConfigProposal.md#0x1_ModifyDaoConfigProposal_DaoConfigUpdate">DaoConfigUpdate</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>voting_delay: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>voting_period: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>voting_quorum_rate: u8</code>
</dt>
<dd>

</dd>
<dt>
<code>min_action_delay: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_ModifyDaoConfigProposal_ERR_NOT_AUTHORIZED"></a>



<pre><code><b>const</b> <a href="ModifyDaoConfigProposal.md#0x1_ModifyDaoConfigProposal_ERR_NOT_AUTHORIZED">ERR_NOT_AUTHORIZED</a>: u64 = 401;
</code></pre>



<a name="0x1_ModifyDaoConfigProposal_ERR_QUORUM_RATE_INVALID"></a>



<pre><code><b>const</b> <a href="ModifyDaoConfigProposal.md#0x1_ModifyDaoConfigProposal_ERR_QUORUM_RATE_INVALID">ERR_QUORUM_RATE_INVALID</a>: u64 = 402;
</code></pre>



<a name="0x1_ModifyDaoConfigProposal_plugin"></a>

## Function `plugin`



<pre><code><b>public</b> <b>fun</b> <a href="ModifyDaoConfigProposal.md#0x1_ModifyDaoConfigProposal_plugin">plugin</a>&lt;TokenT: <b>copyable</b>&gt;(signer: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ModifyDaoConfigProposal.md#0x1_ModifyDaoConfigProposal_plugin">plugin</a>&lt;TokenT: <b>copyable</b>&gt;(signer: &signer) {
    <b>let</b> token_issuer = <a href="Token.md#0x1_Token_token_address">Token::token_address</a>&lt;TokenT&gt;();
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer) == token_issuer, <a href="Errors.md#0x1_Errors_requires_address">Errors::requires_address</a>(<a href="ModifyDaoConfigProposal.md#0x1_ModifyDaoConfigProposal_ERR_NOT_AUTHORIZED">ERR_NOT_AUTHORIZED</a>));
    <b>let</b> dao_config_modify_cap = <a href="Config.md#0x1_Config_extract_modify_config_capability">Config::extract_modify_config_capability</a>&lt;
        <a href="Dao.md#0x1_Dao_DaoConfig">Dao::DaoConfig</a>&lt;TokenT&gt;,
    &gt;(signer);
    // TODO: <b>assert</b> cap.account_address == token_issuer
    <b>let</b> cap = <a href="ModifyDaoConfigProposal.md#0x1_ModifyDaoConfigProposal_DaoConfigModifyCapability">DaoConfigModifyCapability</a> { cap: dao_config_modify_cap };
    move_to(signer, cap);
}
</code></pre>



</details>

<a name="0x1_ModifyDaoConfigProposal_propose"></a>

## Function `propose`



<pre><code><b>public</b> <b>fun</b> <a href="ModifyDaoConfigProposal.md#0x1_ModifyDaoConfigProposal_propose">propose</a>&lt;TokenT: <b>copyable</b>&gt;(signer: &signer, voting_delay: u64, voting_period: u64, voting_quorum_rate: u8, min_action_delay: u64, exec_delay: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ModifyDaoConfigProposal.md#0x1_ModifyDaoConfigProposal_propose">propose</a>&lt;TokenT: <b>copyable</b>&gt;(
    signer: &signer,
    voting_delay: u64,
    voting_period: u64,
    voting_quorum_rate: u8,
    min_action_delay: u64,
    exec_delay: u64,
) {
    <b>assert</b>(voting_quorum_rate &lt;= 100, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="ModifyDaoConfigProposal.md#0x1_ModifyDaoConfigProposal_ERR_QUORUM_RATE_INVALID">ERR_QUORUM_RATE_INVALID</a>));
    <b>let</b> action = <a href="ModifyDaoConfigProposal.md#0x1_ModifyDaoConfigProposal_DaoConfigUpdate">DaoConfigUpdate</a> {
        voting_delay,
        voting_period,
        voting_quorum_rate,
        min_action_delay,
    };
    <a href="Dao.md#0x1_Dao_propose">Dao::propose</a>&lt;TokenT, <a href="ModifyDaoConfigProposal.md#0x1_ModifyDaoConfigProposal_DaoConfigUpdate">DaoConfigUpdate</a>&gt;(signer, action, exec_delay);
}
</code></pre>



</details>

<a name="0x1_ModifyDaoConfigProposal_execute"></a>

## Function `execute`



<pre><code><b>public</b> <b>fun</b> <a href="ModifyDaoConfigProposal.md#0x1_ModifyDaoConfigProposal_execute">execute</a>&lt;TokenT: <b>copyable</b>&gt;(proposer_address: address, proposal_id: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ModifyDaoConfigProposal.md#0x1_ModifyDaoConfigProposal_execute">execute</a>&lt;TokenT: <b>copyable</b>&gt;(proposer_address: address, proposal_id: u64)
<b>acquires</b> <a href="ModifyDaoConfigProposal.md#0x1_ModifyDaoConfigProposal_DaoConfigModifyCapability">DaoConfigModifyCapability</a> {
    <b>let</b> <a href="ModifyDaoConfigProposal.md#0x1_ModifyDaoConfigProposal_DaoConfigUpdate">DaoConfigUpdate</a> {
        voting_delay,
        voting_period,
        voting_quorum_rate,
        min_action_delay,
    } = <a href="Dao.md#0x1_Dao_extract_proposal_action">Dao::extract_proposal_action</a>&lt;TokenT, <a href="ModifyDaoConfigProposal.md#0x1_ModifyDaoConfigProposal_DaoConfigUpdate">DaoConfigUpdate</a>&gt;(proposer_address, proposal_id);
    <b>let</b> cap = borrow_global_mut&lt;<a href="ModifyDaoConfigProposal.md#0x1_ModifyDaoConfigProposal_DaoConfigModifyCapability">DaoConfigModifyCapability</a>&lt;TokenT&gt;&gt;(
        <a href="Token.md#0x1_Token_token_address">Token::token_address</a>&lt;TokenT&gt;(),
    );
    <a href="Dao.md#0x1_Dao_modify_dao_config">Dao::modify_dao_config</a>(
        &<b>mut</b> cap.cap,
        voting_delay,
        voting_period,
        voting_quorum_rate,
        min_action_delay,
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


<pre><code><b>public</b> <b>fun</b> <a href="ModifyDaoConfigProposal.md#0x1_ModifyDaoConfigProposal_plugin">plugin</a>&lt;TokenT: <b>copyable</b>&gt;(signer: &signer)
</code></pre>




<pre><code><b>pragma</b> aborts_if_is_partial = <b>false</b>;
<a name="0x1_ModifyDaoConfigProposal_sender$3"></a>
<b>let</b> sender = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer);
<b>aborts_if</b> sender != <a href="Token.md#0x1_Token_SPEC_TOKEN_TEST_ADDRESS">Token::SPEC_TOKEN_TEST_ADDRESS</a>();
<b>include</b> <a href="Config.md#0x1_Config_AbortsIfCapNotExist">Config::AbortsIfCapNotExist</a>&lt;<a href="Dao.md#0x1_Dao_DaoConfig">Dao::DaoConfig</a>&lt;TokenT&gt;&gt;{account: sender};
<b>aborts_if</b> <b>exists</b>&lt;<a href="ModifyDaoConfigProposal.md#0x1_ModifyDaoConfigProposal_DaoConfigModifyCapability">DaoConfigModifyCapability</a>&lt;TokenT&gt;&gt;(sender);
<b>ensures</b> <b>exists</b>&lt;<a href="ModifyDaoConfigProposal.md#0x1_ModifyDaoConfigProposal_DaoConfigModifyCapability">DaoConfigModifyCapability</a>&lt;TokenT&gt;&gt;(sender);
</code></pre>



<a name="@Specification_1_propose"></a>

### Function `propose`


<pre><code><b>public</b> <b>fun</b> <a href="ModifyDaoConfigProposal.md#0x1_ModifyDaoConfigProposal_propose">propose</a>&lt;TokenT: <b>copyable</b>&gt;(signer: &signer, voting_delay: u64, voting_period: u64, voting_quorum_rate: u8, min_action_delay: u64, exec_delay: u64)
</code></pre>




<pre><code><b>pragma</b> aborts_if_is_partial = <b>false</b>;
<b>aborts_if</b> voting_quorum_rate &gt; 100;
<b>include</b> <a href="Dao.md#0x1_Dao_AbortIfDaoConfigNotExist">Dao::AbortIfDaoConfigNotExist</a>&lt;TokenT&gt;;
<b>include</b> <a href="Dao.md#0x1_Dao_AbortIfDaoInfoNotExist">Dao::AbortIfDaoInfoNotExist</a>&lt;TokenT&gt;;
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Timestamp.md#0x1_Timestamp_CurrentTimeMilliseconds">Timestamp::CurrentTimeMilliseconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
<b>aborts_if</b> exec_delay &gt; 0 && exec_delay &lt; <a href="Dao.md#0x1_Dao_spec_dao_config">Dao::spec_dao_config</a>&lt;TokenT&gt;().min_action_delay;
<b>include</b> <a href="Dao.md#0x1_Dao_CheckQuorumVotes">Dao::CheckQuorumVotes</a>&lt;TokenT&gt;;
<a name="0x1_ModifyDaoConfigProposal_sender$4"></a>
<b>let</b> sender = <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(signer);
<b>aborts_if</b> <b>exists</b>&lt;<a href="Dao.md#0x1_Dao_Proposal">Dao::Proposal</a>&lt;TokenT, <a href="ModifyDaoConfigProposal.md#0x1_ModifyDaoConfigProposal_DaoConfigUpdate">DaoConfigUpdate</a>&gt;&gt;(sender);
</code></pre>



<a name="@Specification_1_execute"></a>

### Function `execute`


<pre><code><b>public</b> <b>fun</b> <a href="ModifyDaoConfigProposal.md#0x1_ModifyDaoConfigProposal_execute">execute</a>&lt;TokenT: <b>copyable</b>&gt;(proposer_address: address, proposal_id: u64)
</code></pre>




<pre><code><b>pragma</b> aborts_if_is_partial = <b>true</b>;
<b>aborts_if</b> !<b>exists</b>&lt;<a href="ModifyDaoConfigProposal.md#0x1_ModifyDaoConfigProposal_DaoConfigModifyCapability">DaoConfigModifyCapability</a>&lt;TokenT&gt;&gt;(<a href="Token.md#0x1_Token_SPEC_TOKEN_TEST_ADDRESS">Token::SPEC_TOKEN_TEST_ADDRESS</a>());
</code></pre>
