
<a name="0x1_ModifyDaoConfigProposal"></a>

# Module `0x1::ModifyDaoConfigProposal`

### Table of Contents

-  [Resource `DaoConfigModifyCapability`](#0x1_ModifyDaoConfigProposal_DaoConfigModifyCapability)
-  [Struct `DaoConfigUpdate`](#0x1_ModifyDaoConfigProposal_DaoConfigUpdate)
-  [Const `ERR_NOT_AUTHORIZED`](#0x1_ModifyDaoConfigProposal_ERR_NOT_AUTHORIZED)
-  [Const `ERR_QUROM_RATE_INVALID`](#0x1_ModifyDaoConfigProposal_ERR_QUROM_RATE_INVALID)
-  [Function `plugin`](#0x1_ModifyDaoConfigProposal_plugin)
-  [Function `propose`](#0x1_ModifyDaoConfigProposal_propose)
-  [Function `execute`](#0x1_ModifyDaoConfigProposal_execute)



<a name="0x1_ModifyDaoConfigProposal_DaoConfigModifyCapability"></a>

## Resource `DaoConfigModifyCapability`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_ModifyDaoConfigProposal_DaoConfigModifyCapability">DaoConfigModifyCapability</a>&lt;TokenT: <b>copyable</b>&gt;
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

a proposal action to udpate dao config.
if any field is
<code>0</code>, that means the proposal want to update.


<pre><code><b>struct</b> <a href="#0x1_ModifyDaoConfigProposal_DaoConfigUpdate">DaoConfigUpdate</a>
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

<a name="0x1_ModifyDaoConfigProposal_ERR_NOT_AUTHORIZED"></a>

## Const `ERR_NOT_AUTHORIZED`



<pre><code><b>const</b> ERR_NOT_AUTHORIZED: u64 = 401;
</code></pre>



<a name="0x1_ModifyDaoConfigProposal_ERR_QUROM_RATE_INVALID"></a>

## Const `ERR_QUROM_RATE_INVALID`



<pre><code><b>const</b> ERR_QUROM_RATE_INVALID: u64 = 402;
</code></pre>



<a name="0x1_ModifyDaoConfigProposal_plugin"></a>

## Function `plugin`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_ModifyDaoConfigProposal_plugin">plugin</a>&lt;TokenT: <b>copyable</b>&gt;(signer: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_ModifyDaoConfigProposal_plugin">plugin</a>&lt;TokenT: <b>copyable</b>&gt;(signer: &signer) {
    <b>let</b> token_issuer = <a href="Token.md#0x1_Token_token_address">Token::token_address</a>&lt;TokenT&gt;();
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer) == token_issuer, ERR_NOT_AUTHORIZED);
    <b>let</b> dao_config_moidify_cap = <a href="Config.md#0x1_Config_extract_modify_config_capability">Config::extract_modify_config_capability</a>&lt;
        <a href="Dao.md#0x1_Dao_DaoConfig">Dao::DaoConfig</a>&lt;TokenT&gt;,
    &gt;(signer);
    // TODO: <b>assert</b> cap.account_address == token_issuer
    <b>let</b> cap = <a href="#0x1_ModifyDaoConfigProposal_DaoConfigModifyCapability">DaoConfigModifyCapability</a> { cap: dao_config_moidify_cap };
    move_to(signer, cap);
}
</code></pre>



</details>

<a name="0x1_ModifyDaoConfigProposal_propose"></a>

## Function `propose`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_ModifyDaoConfigProposal_propose">propose</a>&lt;TokenT: <b>copyable</b>&gt;(signer: &signer, voting_delay: u64, voting_period: u64, voting_quorum_rate: u8, min_action_delay: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_ModifyDaoConfigProposal_propose">propose</a>&lt;TokenT: <b>copyable</b>&gt;(
    signer: &signer,
    voting_delay: u64,
    voting_period: u64,
    voting_quorum_rate: u8,
    min_action_delay: u64,
) {
    <b>assert</b>(voting_quorum_rate &lt;= 100, ERR_QUROM_RATE_INVALID);
    <b>let</b> action = <a href="#0x1_ModifyDaoConfigProposal_DaoConfigUpdate">DaoConfigUpdate</a> {
        voting_delay,
        voting_period,
        voting_quorum_rate,
        min_action_delay,
    };
    <a href="Dao.md#0x1_Dao_propose">Dao::propose</a>&lt;TokenT, <a href="#0x1_ModifyDaoConfigProposal_DaoConfigUpdate">DaoConfigUpdate</a>&gt;(signer, action, <a href="Dao.md#0x1_Dao_min_action_delay">Dao::min_action_delay</a>&lt;TokenT&gt;());
}
</code></pre>



</details>

<a name="0x1_ModifyDaoConfigProposal_execute"></a>

## Function `execute`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_ModifyDaoConfigProposal_execute">execute</a>&lt;TokenT: <b>copyable</b>&gt;(proposer_address: address, proposal_id: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_ModifyDaoConfigProposal_execute">execute</a>&lt;TokenT: <b>copyable</b>&gt;(proposer_address: address, proposal_id: u64)
<b>acquires</b> <a href="#0x1_ModifyDaoConfigProposal_DaoConfigModifyCapability">DaoConfigModifyCapability</a> {
    <b>let</b> <a href="#0x1_ModifyDaoConfigProposal_DaoConfigUpdate">DaoConfigUpdate</a> {
        voting_delay,
        voting_period,
        voting_quorum_rate,
        min_action_delay,
    } = <a href="Dao.md#0x1_Dao_extract_proposal_action">Dao::extract_proposal_action</a>&lt;TokenT, <a href="#0x1_ModifyDaoConfigProposal_DaoConfigUpdate">DaoConfigUpdate</a>&gt;(proposer_address, proposal_id);
    <b>let</b> cap = borrow_global_mut&lt;<a href="#0x1_ModifyDaoConfigProposal_DaoConfigModifyCapability">DaoConfigModifyCapability</a>&lt;TokenT&gt;&gt;(
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
