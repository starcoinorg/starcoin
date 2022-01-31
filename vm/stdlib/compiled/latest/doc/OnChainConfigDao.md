
<a name="0x1_OnChainConfigDao"></a>

# Module `0x1::OnChainConfigDao`

OnChainConfigDao is a DAO proposal for modify onchain configuration.


-  [Resource `WrappedConfigModifyCapability`](#0x1_OnChainConfigDao_WrappedConfigModifyCapability)
-  [Struct `OnChainConfigUpdate`](#0x1_OnChainConfigDao_OnChainConfigUpdate)
-  [Constants](#@Constants_0)
-  [Function `plugin`](#0x1_OnChainConfigDao_plugin)
-  [Function `propose_update`](#0x1_OnChainConfigDao_propose_update)
-  [Function `execute`](#0x1_OnChainConfigDao_execute)
-  [Specification](#@Specification_1)
    -  [Function `plugin`](#@Specification_1_plugin)
    -  [Function `propose_update`](#@Specification_1_propose_update)
    -  [Function `execute`](#@Specification_1_execute)


<pre><code><b>use</b> <a href="Config.md#0x1_Config">0x1::Config</a>;
<b>use</b> <a href="Dao.md#0x1_Dao">0x1::Dao</a>;
<b>use</b> <a href="Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="Signer.md#0x1_Signer">0x1::Signer</a>;
<b>use</b> <a href="Token.md#0x1_Token">0x1::Token</a>;
</code></pre>



<a name="0x1_OnChainConfigDao_WrappedConfigModifyCapability"></a>

## Resource `WrappedConfigModifyCapability`

A wrapper of <code><a href="Config.md#0x1_Config_ModifyConfigCapability">Config::ModifyConfigCapability</a>&lt;ConfigT&gt;</code>.


<pre><code><b>struct</b> <a href="OnChainConfigDao.md#0x1_OnChainConfigDao_WrappedConfigModifyCapability">WrappedConfigModifyCapability</a>&lt;TokenT, ConfigT: <b>copy</b>, drop, store&gt; <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>cap: <a href="Config.md#0x1_Config_ModifyConfigCapability">Config::ModifyConfigCapability</a>&lt;ConfigT&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_OnChainConfigDao_OnChainConfigUpdate"></a>

## Struct `OnChainConfigUpdate`

request of updating configuration.


<pre><code><b>struct</b> <a href="OnChainConfigDao.md#0x1_OnChainConfigDao_OnChainConfigUpdate">OnChainConfigUpdate</a>&lt;ConfigT: <b>copy</b>, drop, store&gt; <b>has</b> <b>copy</b>, drop, store
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

<a name="@Constants_0"></a>

## Constants


<a name="0x1_OnChainConfigDao_ERR_NOT_AUTHORIZED"></a>



<pre><code><b>const</b> <a href="OnChainConfigDao.md#0x1_OnChainConfigDao_ERR_NOT_AUTHORIZED">ERR_NOT_AUTHORIZED</a>: u64 = 401;
</code></pre>



<a name="0x1_OnChainConfigDao_plugin"></a>

## Function `plugin`

Plugin method of the module.
Should be called by token issuer.


<pre><code><b>public</b> <b>fun</b> <a href="OnChainConfigDao.md#0x1_OnChainConfigDao_plugin">plugin</a>&lt;TokenT: <b>copy</b>, drop, store, ConfigT: <b>copy</b>, drop, store&gt;(signer: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="OnChainConfigDao.md#0x1_OnChainConfigDao_plugin">plugin</a>&lt;TokenT: <b>copy</b> + drop + store, ConfigT: <b>copy</b> + drop + store&gt;(signer: &signer) {
    <b>let</b> token_issuer = <a href="Token.md#0x1_Token_token_address">Token::token_address</a>&lt;TokenT&gt;();
    <b>assert</b>!(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer) == token_issuer, <a href="Errors.md#0x1_Errors_requires_address">Errors::requires_address</a>(<a href="OnChainConfigDao.md#0x1_OnChainConfigDao_ERR_NOT_AUTHORIZED">ERR_NOT_AUTHORIZED</a>));
    <b>let</b> config_modify_cap = <a href="Config.md#0x1_Config_extract_modify_config_capability">Config::extract_modify_config_capability</a>&lt;ConfigT&gt;(signer);
    <b>let</b> cap = <a href="OnChainConfigDao.md#0x1_OnChainConfigDao_WrappedConfigModifyCapability">WrappedConfigModifyCapability</a>&lt;TokenT, ConfigT&gt; { cap: config_modify_cap };
    <b>move_to</b>(signer, cap);
}
</code></pre>



</details>

<a name="0x1_OnChainConfigDao_propose_update"></a>

## Function `propose_update`

issue a proposal to update config of ConfigT goved by TokenT


<pre><code><b>public</b> <b>fun</b> <a href="OnChainConfigDao.md#0x1_OnChainConfigDao_propose_update">propose_update</a>&lt;TokenT: <b>copy</b>, drop, store, ConfigT: <b>copy</b>, drop, store&gt;(signer: &signer, new_config: ConfigT, exec_delay: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="OnChainConfigDao.md#0x1_OnChainConfigDao_propose_update">propose_update</a>&lt;TokenT: <b>copy</b> + drop + store, ConfigT: <b>copy</b> + drop + store&gt;(
    signer: &signer,
    new_config: ConfigT,
    exec_delay: u64,
) {
    <a href="Dao.md#0x1_Dao_propose">Dao::propose</a>&lt;TokenT, <a href="OnChainConfigDao.md#0x1_OnChainConfigDao_OnChainConfigUpdate">OnChainConfigUpdate</a>&lt;ConfigT&gt;&gt;(
        signer,
        <a href="OnChainConfigDao.md#0x1_OnChainConfigDao_OnChainConfigUpdate">OnChainConfigUpdate</a> { value: new_config },
        exec_delay,
    );
}
</code></pre>



</details>

<a name="0x1_OnChainConfigDao_execute"></a>

## Function `execute`

Once the proposal is agreed, anyone can call the method to make the proposal happen.
Caller need to make sure that the proposal of <code>proposal_id</code> under <code>proposal_address</code> is
the kind of this proposal module.


<pre><code><b>public</b> <b>fun</b> <a href="OnChainConfigDao.md#0x1_OnChainConfigDao_execute">execute</a>&lt;TokenT: <b>copy</b>, drop, store, ConfigT: <b>copy</b>, drop, store&gt;(proposer_address: <b>address</b>, proposal_id: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="OnChainConfigDao.md#0x1_OnChainConfigDao_execute">execute</a>&lt;TokenT: <b>copy</b> + drop + store, ConfigT: <b>copy</b> + drop + store&gt;(
    proposer_address: <b>address</b>,
    proposal_id: u64,
) <b>acquires</b> <a href="OnChainConfigDao.md#0x1_OnChainConfigDao_WrappedConfigModifyCapability">WrappedConfigModifyCapability</a> {
    <b>let</b> <a href="OnChainConfigDao.md#0x1_OnChainConfigDao_OnChainConfigUpdate">OnChainConfigUpdate</a> { value } = <a href="Dao.md#0x1_Dao_extract_proposal_action">Dao::extract_proposal_action</a>&lt;
        TokenT,
        <a href="OnChainConfigDao.md#0x1_OnChainConfigDao_OnChainConfigUpdate">OnChainConfigUpdate</a>&lt;ConfigT&gt;,
    &gt;(proposer_address, proposal_id);
    <b>let</b> cap = <b>borrow_global_mut</b>&lt;<a href="OnChainConfigDao.md#0x1_OnChainConfigDao_WrappedConfigModifyCapability">WrappedConfigModifyCapability</a>&lt;TokenT, ConfigT&gt;&gt;(
        <a href="Token.md#0x1_Token_token_address">Token::token_address</a>&lt;TokenT&gt;(),
    );
    <a href="Config.md#0x1_Config_set_with_capability">Config::set_with_capability</a>(&<b>mut</b> cap.cap, value);
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


<pre><code><b>public</b> <b>fun</b> <a href="OnChainConfigDao.md#0x1_OnChainConfigDao_plugin">plugin</a>&lt;TokenT: <b>copy</b>, drop, store, ConfigT: <b>copy</b>, drop, store&gt;(signer: &signer)
</code></pre>




<pre><code><b>pragma</b> aborts_if_is_partial = <b>false</b>;
<b>let</b> sender = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer);
<b>aborts_if</b> sender != <a href="Token.md#0x1_Token_SPEC_TOKEN_TEST_ADDRESS">Token::SPEC_TOKEN_TEST_ADDRESS</a>();
<b>include</b> <a href="Config.md#0x1_Config_AbortsIfCapNotExist">Config::AbortsIfCapNotExist</a>&lt;ConfigT&gt;{account: sender};
<b>aborts_if</b> <b>exists</b>&lt;<a href="OnChainConfigDao.md#0x1_OnChainConfigDao_WrappedConfigModifyCapability">WrappedConfigModifyCapability</a>&lt;TokenT, ConfigT&gt;&gt;(sender);
<b>ensures</b> <b>exists</b>&lt;<a href="OnChainConfigDao.md#0x1_OnChainConfigDao_WrappedConfigModifyCapability">WrappedConfigModifyCapability</a>&lt;TokenT, ConfigT&gt;&gt;(sender);
</code></pre>



<a name="@Specification_1_propose_update"></a>

### Function `propose_update`


<pre><code><b>public</b> <b>fun</b> <a href="OnChainConfigDao.md#0x1_OnChainConfigDao_propose_update">propose_update</a>&lt;TokenT: <b>copy</b>, drop, store, ConfigT: <b>copy</b>, drop, store&gt;(signer: &signer, new_config: ConfigT, exec_delay: u64)
</code></pre>




<pre><code><b>pragma</b> aborts_if_is_partial = <b>false</b>;
<b>include</b> <a href="Dao.md#0x1_Dao_AbortIfDaoConfigNotExist">Dao::AbortIfDaoConfigNotExist</a>&lt;TokenT&gt;;
<b>include</b> <a href="Dao.md#0x1_Dao_AbortIfDaoInfoNotExist">Dao::AbortIfDaoInfoNotExist</a>&lt;TokenT&gt;;
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Timestamp.md#0x1_Timestamp_CurrentTimeMilliseconds">Timestamp::CurrentTimeMilliseconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
<b>aborts_if</b> exec_delay &gt; 0 && exec_delay &lt; <a href="Dao.md#0x1_Dao_spec_dao_config">Dao::spec_dao_config</a>&lt;TokenT&gt;().min_action_delay;
<b>include</b> <a href="Dao.md#0x1_Dao_CheckQuorumVotes">Dao::CheckQuorumVotes</a>&lt;TokenT&gt;;
<b>let</b> sender = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer);
<b>aborts_if</b> <b>exists</b>&lt;<a href="Dao.md#0x1_Dao_Proposal">Dao::Proposal</a>&lt;TokenT, <a href="OnChainConfigDao.md#0x1_OnChainConfigDao_OnChainConfigUpdate">OnChainConfigUpdate</a>&lt;ConfigT&gt;&gt;&gt;(sender);
</code></pre>



<a name="@Specification_1_execute"></a>

### Function `execute`


<pre><code><b>public</b> <b>fun</b> <a href="OnChainConfigDao.md#0x1_OnChainConfigDao_execute">execute</a>&lt;TokenT: <b>copy</b>, drop, store, ConfigT: <b>copy</b>, drop, store&gt;(proposer_address: <b>address</b>, proposal_id: u64)
</code></pre>




<pre><code><b>pragma</b> aborts_if_is_partial = <b>true</b>;
<b>let</b> expected_states = vec&lt;u8&gt;(6);
<b>include</b> <a href="Dao.md#0x1_Dao_CheckProposalStates">Dao::CheckProposalStates</a>&lt;TokenT, <a href="OnChainConfigDao.md#0x1_OnChainConfigDao_OnChainConfigUpdate">OnChainConfigUpdate</a>&lt;ConfigT&gt;&gt;{expected_states};
<b>aborts_if</b> !<b>exists</b>&lt;<a href="OnChainConfigDao.md#0x1_OnChainConfigDao_WrappedConfigModifyCapability">WrappedConfigModifyCapability</a>&lt;TokenT, ConfigT&gt;&gt;(<a href="Token.md#0x1_Token_SPEC_TOKEN_TEST_ADDRESS">Token::SPEC_TOKEN_TEST_ADDRESS</a>());
</code></pre>
