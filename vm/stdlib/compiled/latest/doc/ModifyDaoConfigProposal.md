
<a name="0x1_ModifyDaoConfigProposal"></a>

# Module `0x1::ModifyDaoConfigProposal`

A proposal module which is used to modify Token's DAO configuration.


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

A wrapper of <code><a href="Config.md#0x1_Config_ModifyConfigCapability">Config::ModifyConfigCapability</a>&lt;<a href="Dao.md#0x1_Dao_DaoConfig">Dao::DaoConfig</a>&lt;TokenT&gt;&gt;</code>.


<pre><code><b>struct</b> <a href="ModifyDaoConfigProposal.md#0x1_ModifyDaoConfigProposal_DaoConfigModifyCapability">DaoConfigModifyCapability</a>&lt;TokenT: <b>copy</b>, drop, store&gt; <b>has</b> key
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


<pre><code><b>struct</b> <a href="ModifyDaoConfigProposal.md#0x1_ModifyDaoConfigProposal_DaoConfigUpdate">DaoConfigUpdate</a> <b>has</b> <b>copy</b>, drop, store
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

Plugin method of the module.
Should be called by token issuer.


<pre><code><b>public</b> <b>fun</b> <a href="ModifyDaoConfigProposal.md#0x1_ModifyDaoConfigProposal_plugin">plugin</a>&lt;TokenT: <b>copy</b>, drop, store&gt;(signer: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ModifyDaoConfigProposal.md#0x1_ModifyDaoConfigProposal_plugin">plugin</a>&lt;TokenT: <b>copy</b> + drop + store&gt;(signer: &signer) {
    <b>let</b> token_issuer = <a href="Token.md#0x1_Token_token_address">Token::token_address</a>&lt;TokenT&gt;();
    <b>assert</b>!(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer) == token_issuer, <a href="Errors.md#0x1_Errors_requires_address">Errors::requires_address</a>(<a href="ModifyDaoConfigProposal.md#0x1_ModifyDaoConfigProposal_ERR_NOT_AUTHORIZED">ERR_NOT_AUTHORIZED</a>));
    <b>let</b> dao_config_modify_cap = <a href="Config.md#0x1_Config_extract_modify_config_capability">Config::extract_modify_config_capability</a>&lt;
        <a href="Dao.md#0x1_Dao_DaoConfig">Dao::DaoConfig</a>&lt;TokenT&gt;,
    &gt;(signer);
    <b>assert</b>!(<a href="Config.md#0x1_Config_account_address">Config::account_address</a>(&dao_config_modify_cap) == token_issuer, <a href="Errors.md#0x1_Errors_requires_address">Errors::requires_address</a>(<a href="ModifyDaoConfigProposal.md#0x1_ModifyDaoConfigProposal_ERR_NOT_AUTHORIZED">ERR_NOT_AUTHORIZED</a>));
    <b>let</b> cap = <a href="ModifyDaoConfigProposal.md#0x1_ModifyDaoConfigProposal_DaoConfigModifyCapability">DaoConfigModifyCapability</a> { cap: dao_config_modify_cap };
    <b>move_to</b>(signer, cap);
}
</code></pre>



</details>

<a name="0x1_ModifyDaoConfigProposal_propose"></a>

## Function `propose`

Entrypoint for the proposal.


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="ModifyDaoConfigProposal.md#0x1_ModifyDaoConfigProposal_propose">propose</a>&lt;TokenT: <b>copy</b>, drop, store&gt;(signer: signer, voting_delay: u64, voting_period: u64, voting_quorum_rate: u8, min_action_delay: u64, exec_delay: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="ModifyDaoConfigProposal.md#0x1_ModifyDaoConfigProposal_propose">propose</a>&lt;TokenT: <b>copy</b> + drop + store&gt;(
    signer: signer,
    voting_delay: u64,
    voting_period: u64,
    voting_quorum_rate: u8,
    min_action_delay: u64,
    exec_delay: u64,
) {
    <b>assert</b>!(voting_quorum_rate &lt;= 100, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="ModifyDaoConfigProposal.md#0x1_ModifyDaoConfigProposal_ERR_QUORUM_RATE_INVALID">ERR_QUORUM_RATE_INVALID</a>));
    <b>let</b> action = <a href="ModifyDaoConfigProposal.md#0x1_ModifyDaoConfigProposal_DaoConfigUpdate">DaoConfigUpdate</a> {
        voting_delay,
        voting_period,
        voting_quorum_rate,
        min_action_delay,
    };
    <a href="Dao.md#0x1_Dao_propose">Dao::propose</a>&lt;TokenT, <a href="ModifyDaoConfigProposal.md#0x1_ModifyDaoConfigProposal_DaoConfigUpdate">DaoConfigUpdate</a>&gt;(&signer, action, exec_delay);
}
</code></pre>



</details>

<a name="0x1_ModifyDaoConfigProposal_execute"></a>

## Function `execute`

Once the proposal is agreed, anyone can call the method to make the proposal happen.


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="ModifyDaoConfigProposal.md#0x1_ModifyDaoConfigProposal_execute">execute</a>&lt;TokenT: <b>copy</b>, drop, store&gt;(proposer_address: <b>address</b>, proposal_id: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="ModifyDaoConfigProposal.md#0x1_ModifyDaoConfigProposal_execute">execute</a>&lt;TokenT: <b>copy</b> + drop + store&gt;(proposer_address: <b>address</b>, proposal_id: u64)
<b>acquires</b> <a href="ModifyDaoConfigProposal.md#0x1_ModifyDaoConfigProposal_DaoConfigModifyCapability">DaoConfigModifyCapability</a> {
    <b>let</b> <a href="ModifyDaoConfigProposal.md#0x1_ModifyDaoConfigProposal_DaoConfigUpdate">DaoConfigUpdate</a> {
        voting_delay,
        voting_period,
        voting_quorum_rate,
        min_action_delay,
    } = <a href="Dao.md#0x1_Dao_extract_proposal_action">Dao::extract_proposal_action</a>&lt;TokenT, <a href="ModifyDaoConfigProposal.md#0x1_ModifyDaoConfigProposal_DaoConfigUpdate">DaoConfigUpdate</a>&gt;(proposer_address, proposal_id);
    <b>let</b> cap = <b>borrow_global_mut</b>&lt;<a href="ModifyDaoConfigProposal.md#0x1_ModifyDaoConfigProposal_DaoConfigModifyCapability">DaoConfigModifyCapability</a>&lt;TokenT&gt;&gt;(
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



<pre><code><b>pragma</b> verify = <b>false</b>;
<b>pragma</b> aborts_if_is_strict;
<b>pragma</b> aborts_if_is_partial;
</code></pre>



<a name="@Specification_1_plugin"></a>

### Function `plugin`


<pre><code><b>public</b> <b>fun</b> <a href="ModifyDaoConfigProposal.md#0x1_ModifyDaoConfigProposal_plugin">plugin</a>&lt;TokenT: <b>copy</b>, drop, store&gt;(signer: &signer)
</code></pre>




<pre><code><b>pragma</b> aborts_if_is_partial = <b>false</b>;
<b>let</b> sender = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer);
<b>aborts_if</b> sender != <a href="Token.md#0x1_Token_SPEC_TOKEN_TEST_ADDRESS">Token::SPEC_TOKEN_TEST_ADDRESS</a>();
<b>include</b> <a href="Config.md#0x1_Config_AbortsIfCapNotExist">Config::AbortsIfCapNotExist</a>&lt;<a href="Dao.md#0x1_Dao_DaoConfig">Dao::DaoConfig</a>&lt;TokenT&gt;&gt;{account: sender};
<b>let</b> config_cap = <a href="Config.md#0x1_Config_spec_cap">Config::spec_cap</a>&lt;<a href="Dao.md#0x1_Dao_DaoConfig">Dao::DaoConfig</a>&lt;TokenT&gt;&gt;(sender);
<b>aborts_if</b> <a href="Option.md#0x1_Option_is_none">Option::is_none</a>(config_cap);
<b>aborts_if</b> <a href="Option.md#0x1_Option_borrow">Option::borrow</a>(config_cap).account_address != sender;
<b>aborts_if</b> <b>exists</b>&lt;<a href="ModifyDaoConfigProposal.md#0x1_ModifyDaoConfigProposal_DaoConfigModifyCapability">DaoConfigModifyCapability</a>&lt;TokenT&gt;&gt;(sender);
<b>ensures</b> <b>exists</b>&lt;<a href="ModifyDaoConfigProposal.md#0x1_ModifyDaoConfigProposal_DaoConfigModifyCapability">DaoConfigModifyCapability</a>&lt;TokenT&gt;&gt;(sender);
</code></pre>



<a name="@Specification_1_propose"></a>

### Function `propose`


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="ModifyDaoConfigProposal.md#0x1_ModifyDaoConfigProposal_propose">propose</a>&lt;TokenT: <b>copy</b>, drop, store&gt;(signer: signer, voting_delay: u64, voting_period: u64, voting_quorum_rate: u8, min_action_delay: u64, exec_delay: u64)
</code></pre>




<pre><code><b>pragma</b> aborts_if_is_partial = <b>false</b>;
<b>aborts_if</b> voting_quorum_rate &gt; 100;
<b>include</b> <a href="Dao.md#0x1_Dao_AbortIfDaoConfigNotExist">Dao::AbortIfDaoConfigNotExist</a>&lt;TokenT&gt;;
<b>include</b> <a href="Dao.md#0x1_Dao_AbortIfDaoInfoNotExist">Dao::AbortIfDaoInfoNotExist</a>&lt;TokenT&gt;;
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Timestamp.md#0x1_Timestamp_CurrentTimeMilliseconds">Timestamp::CurrentTimeMilliseconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
<b>aborts_if</b> exec_delay &gt; 0 && exec_delay &lt; <a href="Dao.md#0x1_Dao_spec_dao_config">Dao::spec_dao_config</a>&lt;TokenT&gt;().min_action_delay;
<b>include</b> <a href="Dao.md#0x1_Dao_CheckQuorumVotes">Dao::CheckQuorumVotes</a>&lt;TokenT&gt;;
<b>let</b> sender = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer);
<b>aborts_if</b> <b>exists</b>&lt;<a href="Dao.md#0x1_Dao_Proposal">Dao::Proposal</a>&lt;TokenT, <a href="ModifyDaoConfigProposal.md#0x1_ModifyDaoConfigProposal_DaoConfigUpdate">DaoConfigUpdate</a>&gt;&gt;(sender);
</code></pre>



<a name="@Specification_1_execute"></a>

### Function `execute`


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="ModifyDaoConfigProposal.md#0x1_ModifyDaoConfigProposal_execute">execute</a>&lt;TokenT: <b>copy</b>, drop, store&gt;(proposer_address: <b>address</b>, proposal_id: u64)
</code></pre>




<pre><code><b>pragma</b> aborts_if_is_partial = <b>true</b>;
<b>aborts_if</b> !<b>exists</b>&lt;<a href="ModifyDaoConfigProposal.md#0x1_ModifyDaoConfigProposal_DaoConfigModifyCapability">DaoConfigModifyCapability</a>&lt;TokenT&gt;&gt;(<a href="Token.md#0x1_Token_SPEC_TOKEN_TEST_ADDRESS">Token::SPEC_TOKEN_TEST_ADDRESS</a>());
</code></pre>
