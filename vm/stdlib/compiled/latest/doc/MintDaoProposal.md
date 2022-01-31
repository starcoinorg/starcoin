
<a name="0x1_MintDaoProposal"></a>

# Module `0x1::MintDaoProposal`

MintDaoProposal is a dao proposal for mint extra tokens.


-  [Resource `WrappedMintCapability`](#0x1_MintDaoProposal_WrappedMintCapability)
-  [Struct `MintToken`](#0x1_MintDaoProposal_MintToken)
-  [Constants](#@Constants_0)
-  [Function `plugin`](#0x1_MintDaoProposal_plugin)
-  [Function `propose_mint_to`](#0x1_MintDaoProposal_propose_mint_to)
-  [Function `execute_mint_proposal`](#0x1_MintDaoProposal_execute_mint_proposal)
-  [Specification](#@Specification_1)
    -  [Function `plugin`](#@Specification_1_plugin)
    -  [Function `propose_mint_to`](#@Specification_1_propose_mint_to)
    -  [Function `execute_mint_proposal`](#@Specification_1_execute_mint_proposal)


<pre><code><b>use</b> <a href="Account.md#0x1_Account">0x1::Account</a>;
<b>use</b> <a href="Dao.md#0x1_Dao">0x1::Dao</a>;
<b>use</b> <a href="Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="Signer.md#0x1_Signer">0x1::Signer</a>;
<b>use</b> <a href="Token.md#0x1_Token">0x1::Token</a>;
</code></pre>



<a name="0x1_MintDaoProposal_WrappedMintCapability"></a>

## Resource `WrappedMintCapability`

A wrapper of Token MintCapability.


<pre><code><b>struct</b> <a href="MintDaoProposal.md#0x1_MintDaoProposal_WrappedMintCapability">WrappedMintCapability</a>&lt;TokenType&gt; <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>cap: <a href="Token.md#0x1_Token_MintCapability">Token::MintCapability</a>&lt;TokenType&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_MintDaoProposal_MintToken"></a>

## Struct `MintToken`

MintToken request.


<pre><code><b>struct</b> <a href="MintDaoProposal.md#0x1_MintDaoProposal_MintToken">MintToken</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>receiver: <b>address</b></code>
</dt>
<dd>
 the receiver of minted tokens.
</dd>
<dt>
<code>amount: u128</code>
</dt>
<dd>
 how many tokens to mint.
</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_MintDaoProposal_ERR_NOT_AUTHORIZED"></a>



<pre><code><b>const</b> <a href="MintDaoProposal.md#0x1_MintDaoProposal_ERR_NOT_AUTHORIZED">ERR_NOT_AUTHORIZED</a>: u64 = 401;
</code></pre>



<a name="0x1_MintDaoProposal_plugin"></a>

## Function `plugin`

Plugin method of the module.
Should be called by token issuer.


<pre><code><b>public</b> <b>fun</b> <a href="MintDaoProposal.md#0x1_MintDaoProposal_plugin">plugin</a>&lt;TokenT: store&gt;(signer: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="MintDaoProposal.md#0x1_MintDaoProposal_plugin">plugin</a>&lt;TokenT: store&gt;(signer: &signer) {
    <b>let</b> token_issuer = <a href="Token.md#0x1_Token_token_address">Token::token_address</a>&lt;TokenT&gt;();
    <b>assert</b>!(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer) == token_issuer, <a href="Errors.md#0x1_Errors_requires_address">Errors::requires_address</a>(<a href="MintDaoProposal.md#0x1_MintDaoProposal_ERR_NOT_AUTHORIZED">ERR_NOT_AUTHORIZED</a>));
    <b>let</b> mint_cap = <a href="Token.md#0x1_Token_remove_mint_capability">Token::remove_mint_capability</a>&lt;TokenT&gt;(signer);
    <b>move_to</b>(signer, <a href="MintDaoProposal.md#0x1_MintDaoProposal_WrappedMintCapability">WrappedMintCapability</a> { cap: mint_cap });
}
</code></pre>



</details>

<a name="0x1_MintDaoProposal_propose_mint_to"></a>

## Function `propose_mint_to`

Entrypoint for the proposal.


<pre><code><b>public</b> <b>fun</b> <a href="MintDaoProposal.md#0x1_MintDaoProposal_propose_mint_to">propose_mint_to</a>&lt;TokenT: <b>copy</b>, drop, store&gt;(signer: &signer, receiver: <b>address</b>, amount: u128, exec_delay: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="MintDaoProposal.md#0x1_MintDaoProposal_propose_mint_to">propose_mint_to</a>&lt;TokenT: <b>copy</b> + drop + store&gt;(signer: &signer, receiver: <b>address</b>, amount: u128, exec_delay: u64) {
    <a href="Dao.md#0x1_Dao_propose">Dao::propose</a>&lt;TokenT, <a href="MintDaoProposal.md#0x1_MintDaoProposal_MintToken">MintToken</a>&gt;(
        signer,
        <a href="MintDaoProposal.md#0x1_MintDaoProposal_MintToken">MintToken</a> { receiver, amount },
        exec_delay,
    );
}
</code></pre>



</details>

<a name="0x1_MintDaoProposal_execute_mint_proposal"></a>

## Function `execute_mint_proposal`

Once the proposal is agreed, anyone can call the method to make the proposal happen.


<pre><code><b>public</b> <b>fun</b> <a href="MintDaoProposal.md#0x1_MintDaoProposal_execute_mint_proposal">execute_mint_proposal</a>&lt;TokenT: <b>copy</b>, drop, store&gt;(proposer_address: <b>address</b>, proposal_id: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="MintDaoProposal.md#0x1_MintDaoProposal_execute_mint_proposal">execute_mint_proposal</a>&lt;TokenT: <b>copy</b> + drop + store&gt;(
    proposer_address: <b>address</b>,
    proposal_id: u64,
) <b>acquires</b> <a href="MintDaoProposal.md#0x1_MintDaoProposal_WrappedMintCapability">WrappedMintCapability</a> {
    <b>let</b> <a href="MintDaoProposal.md#0x1_MintDaoProposal_MintToken">MintToken</a> { receiver, amount } = <a href="Dao.md#0x1_Dao_extract_proposal_action">Dao::extract_proposal_action</a>&lt;TokenT, <a href="MintDaoProposal.md#0x1_MintDaoProposal_MintToken">MintToken</a>&gt;(
        proposer_address,
        proposal_id,
    );
    <b>let</b> cap = <b>borrow_global</b>&lt;<a href="MintDaoProposal.md#0x1_MintDaoProposal_WrappedMintCapability">WrappedMintCapability</a>&lt;TokenT&gt;&gt;(<a href="Token.md#0x1_Token_token_address">Token::token_address</a>&lt;TokenT&gt;());
    <b>let</b> tokens = <a href="Token.md#0x1_Token_mint_with_capability">Token::mint_with_capability</a>&lt;TokenT&gt;(&cap.cap, amount);
    <a href="Account.md#0x1_Account_deposit">Account::deposit</a>(receiver, tokens);
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


<pre><code><b>public</b> <b>fun</b> <a href="MintDaoProposal.md#0x1_MintDaoProposal_plugin">plugin</a>&lt;TokenT: store&gt;(signer: &signer)
</code></pre>




<pre><code><b>pragma</b> aborts_if_is_partial = <b>false</b>;
<b>let</b> sender = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer);
<b>aborts_if</b> sender != <a href="Token.md#0x1_Token_SPEC_TOKEN_TEST_ADDRESS">Token::SPEC_TOKEN_TEST_ADDRESS</a>();
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Token.md#0x1_Token_MintCapability">Token::MintCapability</a>&lt;TokenT&gt;&gt;(sender);
<b>aborts_if</b> <b>exists</b>&lt;<a href="MintDaoProposal.md#0x1_MintDaoProposal_WrappedMintCapability">WrappedMintCapability</a>&lt;TokenT&gt;&gt;(sender);
<b>ensures</b> !<b>exists</b>&lt;<a href="Token.md#0x1_Token_MintCapability">Token::MintCapability</a>&lt;TokenT&gt;&gt;(sender);
<b>ensures</b> <b>exists</b>&lt;<a href="MintDaoProposal.md#0x1_MintDaoProposal_WrappedMintCapability">WrappedMintCapability</a>&lt;TokenT&gt;&gt;(sender);
</code></pre>



<a name="@Specification_1_propose_mint_to"></a>

### Function `propose_mint_to`


<pre><code><b>public</b> <b>fun</b> <a href="MintDaoProposal.md#0x1_MintDaoProposal_propose_mint_to">propose_mint_to</a>&lt;TokenT: <b>copy</b>, drop, store&gt;(signer: &signer, receiver: <b>address</b>, amount: u128, exec_delay: u64)
</code></pre>




<pre><code><b>pragma</b> aborts_if_is_partial = <b>false</b>;
<b>include</b> <a href="Dao.md#0x1_Dao_AbortIfDaoConfigNotExist">Dao::AbortIfDaoConfigNotExist</a>&lt;TokenT&gt;;
<b>include</b> <a href="Dao.md#0x1_Dao_AbortIfDaoInfoNotExist">Dao::AbortIfDaoInfoNotExist</a>&lt;TokenT&gt;;
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Timestamp.md#0x1_Timestamp_CurrentTimeMilliseconds">Timestamp::CurrentTimeMilliseconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
<b>aborts_if</b> exec_delay &gt; 0 && exec_delay &lt; <a href="Dao.md#0x1_Dao_spec_dao_config">Dao::spec_dao_config</a>&lt;TokenT&gt;().min_action_delay;
<b>include</b> <a href="Dao.md#0x1_Dao_CheckQuorumVotes">Dao::CheckQuorumVotes</a>&lt;TokenT&gt;;
<b>let</b> sender = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer);
<b>aborts_if</b> <b>exists</b>&lt;<a href="Dao.md#0x1_Dao_Proposal">Dao::Proposal</a>&lt;TokenT, <a href="MintDaoProposal.md#0x1_MintDaoProposal_MintToken">MintToken</a>&gt;&gt;(sender);
</code></pre>



<a name="@Specification_1_execute_mint_proposal"></a>

### Function `execute_mint_proposal`


<pre><code><b>public</b> <b>fun</b> <a href="MintDaoProposal.md#0x1_MintDaoProposal_execute_mint_proposal">execute_mint_proposal</a>&lt;TokenT: <b>copy</b>, drop, store&gt;(proposer_address: <b>address</b>, proposal_id: u64)
</code></pre>




<pre><code><b>pragma</b> aborts_if_is_partial = <b>true</b>;
<b>let</b> expected_states = vec&lt;u8&gt;(6);
<b>include</b> <a href="Dao.md#0x1_Dao_CheckProposalStates">Dao::CheckProposalStates</a>&lt;TokenT, <a href="MintDaoProposal.md#0x1_MintDaoProposal_MintToken">MintToken</a>&gt;{expected_states};
<b>let</b> proposal = <b>global</b>&lt;<a href="Dao.md#0x1_Dao_Proposal">Dao::Proposal</a>&lt;TokenT, <a href="MintDaoProposal.md#0x1_MintDaoProposal_MintToken">MintToken</a>&gt;&gt;(proposer_address);
<b>aborts_if</b> <a href="Option.md#0x1_Option_is_none">Option::is_none</a>(proposal.action);
<b>aborts_if</b> !<b>exists</b>&lt;<a href="MintDaoProposal.md#0x1_MintDaoProposal_WrappedMintCapability">WrappedMintCapability</a>&lt;TokenT&gt;&gt;(<a href="Token.md#0x1_Token_SPEC_TOKEN_TEST_ADDRESS">Token::SPEC_TOKEN_TEST_ADDRESS</a>());
</code></pre>
