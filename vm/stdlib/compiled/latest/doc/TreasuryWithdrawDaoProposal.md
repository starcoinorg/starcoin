
<a name="0x1_TreasuryWithdrawDaoProposal"></a>

# Module `0x1::TreasuryWithdrawDaoProposal`

TreasuryWithdrawDaoProposal is a dao proposal for withdraw Token from Treasury.


-  [Resource `WrappedWithdrawCapability`](#0x1_TreasuryWithdrawDaoProposal_WrappedWithdrawCapability)
-  [Struct `WithdrawToken`](#0x1_TreasuryWithdrawDaoProposal_WithdrawToken)
-  [Constants](#@Constants_0)
-  [Function `plugin`](#0x1_TreasuryWithdrawDaoProposal_plugin)
-  [Function `propose_withdraw`](#0x1_TreasuryWithdrawDaoProposal_propose_withdraw)
-  [Function `execute_withdraw_proposal`](#0x1_TreasuryWithdrawDaoProposal_execute_withdraw_proposal)
-  [Function `withdraw_for_block_reward`](#0x1_TreasuryWithdrawDaoProposal_withdraw_for_block_reward)
-  [Specification](#@Specification_1)
    -  [Function `plugin`](#@Specification_1_plugin)
    -  [Function `propose_withdraw`](#@Specification_1_propose_withdraw)
    -  [Function `execute_withdraw_proposal`](#@Specification_1_execute_withdraw_proposal)


<pre><code><b>use</b> <a href="CoreAddresses.md#0x1_CoreAddresses">0x1::CoreAddresses</a>;
<b>use</b> <a href="Dao.md#0x1_Dao">0x1::Dao</a>;
<b>use</b> <a href="Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="Signer.md#0x1_Signer">0x1::Signer</a>;
<b>use</b> <a href="Token.md#0x1_Token">0x1::Token</a>;
<b>use</b> <a href="Treasury.md#0x1_Treasury">0x1::Treasury</a>;
</code></pre>



<a name="0x1_TreasuryWithdrawDaoProposal_WrappedWithdrawCapability"></a>

## Resource `WrappedWithdrawCapability`

A wrapper of Token MintCapability.


<pre><code><b>struct</b> <a href="TreasuryWithdrawDaoProposal.md#0x1_TreasuryWithdrawDaoProposal_WrappedWithdrawCapability">WrappedWithdrawCapability</a>&lt;TokenT&gt; <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>cap: <a href="Treasury.md#0x1_Treasury_WithdrawCapability">Treasury::WithdrawCapability</a>&lt;TokenT&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_TreasuryWithdrawDaoProposal_WithdrawToken"></a>

## Struct `WithdrawToken`

WithdrawToken request.


<pre><code><b>struct</b> <a href="TreasuryWithdrawDaoProposal.md#0x1_TreasuryWithdrawDaoProposal_WithdrawToken">WithdrawToken</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>receiver: <b>address</b></code>
</dt>
<dd>
 the receiver of withdraw tokens.
</dd>
<dt>
<code>amount: u128</code>
</dt>
<dd>
 how many tokens to mint.
</dd>
<dt>
<code>period: u64</code>
</dt>
<dd>
 How long in milliseconds does it take for the token to be released
</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_TreasuryWithdrawDaoProposal_ERR_NOT_AUTHORIZED"></a>



<pre><code><b>const</b> <a href="TreasuryWithdrawDaoProposal.md#0x1_TreasuryWithdrawDaoProposal_ERR_NOT_AUTHORIZED">ERR_NOT_AUTHORIZED</a>: u64 = 101;
</code></pre>



<a name="0x1_TreasuryWithdrawDaoProposal_ERR_NEED_RECEIVER_TO_EXECUTE"></a>

Only receiver can execute TreasuryWithdrawDaoProposal


<pre><code><b>const</b> <a href="TreasuryWithdrawDaoProposal.md#0x1_TreasuryWithdrawDaoProposal_ERR_NEED_RECEIVER_TO_EXECUTE">ERR_NEED_RECEIVER_TO_EXECUTE</a>: u64 = 102;
</code></pre>



<a name="0x1_TreasuryWithdrawDaoProposal_ERR_TOO_MANY_WITHDRAW_AMOUNT"></a>

The withdraw amount of propose is too many.


<pre><code><b>const</b> <a href="TreasuryWithdrawDaoProposal.md#0x1_TreasuryWithdrawDaoProposal_ERR_TOO_MANY_WITHDRAW_AMOUNT">ERR_TOO_MANY_WITHDRAW_AMOUNT</a>: u64 = 103;
</code></pre>



<a name="0x1_TreasuryWithdrawDaoProposal_plugin"></a>

## Function `plugin`

Plugin method of the module.
Should be called by token issuer.


<pre><code><b>public</b> <b>fun</b> <a href="TreasuryWithdrawDaoProposal.md#0x1_TreasuryWithdrawDaoProposal_plugin">plugin</a>&lt;TokenT: store&gt;(signer: &signer, cap: <a href="Treasury.md#0x1_Treasury_WithdrawCapability">Treasury::WithdrawCapability</a>&lt;TokenT&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="TreasuryWithdrawDaoProposal.md#0x1_TreasuryWithdrawDaoProposal_plugin">plugin</a>&lt;TokenT: store&gt;(signer: &signer, cap: <a href="Treasury.md#0x1_Treasury_WithdrawCapability">Treasury::WithdrawCapability</a>&lt;TokenT&gt;) {
    <b>let</b> token_issuer = <a href="Token.md#0x1_Token_token_address">Token::token_address</a>&lt;TokenT&gt;();
    <b>assert</b>!(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer) == token_issuer, <a href="Errors.md#0x1_Errors_requires_address">Errors::requires_address</a>(<a href="TreasuryWithdrawDaoProposal.md#0x1_TreasuryWithdrawDaoProposal_ERR_NOT_AUTHORIZED">ERR_NOT_AUTHORIZED</a>));
    <b>move_to</b>(signer, <a href="TreasuryWithdrawDaoProposal.md#0x1_TreasuryWithdrawDaoProposal_WrappedWithdrawCapability">WrappedWithdrawCapability</a>&lt;TokenT&gt; { cap: cap });
}
</code></pre>



</details>

<a name="0x1_TreasuryWithdrawDaoProposal_propose_withdraw"></a>

## Function `propose_withdraw`

Entrypoint for the proposal.


<pre><code><b>public</b> <b>fun</b> <a href="TreasuryWithdrawDaoProposal.md#0x1_TreasuryWithdrawDaoProposal_propose_withdraw">propose_withdraw</a>&lt;TokenT: <b>copy</b>, drop, store&gt;(signer: &signer, receiver: <b>address</b>, amount: u128, period: u64, exec_delay: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="TreasuryWithdrawDaoProposal.md#0x1_TreasuryWithdrawDaoProposal_propose_withdraw">propose_withdraw</a>&lt;TokenT: <b>copy</b> + drop + store&gt;(signer: &signer, receiver: <b>address</b>, amount: u128, period: u64, exec_delay: u64) {
    <b>let</b> quorum_votes = <a href="Dao.md#0x1_Dao_quorum_votes">Dao::quorum_votes</a>&lt;TokenT&gt;();
    <b>assert</b>!(amount &lt;= quorum_votes,  <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="TreasuryWithdrawDaoProposal.md#0x1_TreasuryWithdrawDaoProposal_ERR_TOO_MANY_WITHDRAW_AMOUNT">ERR_TOO_MANY_WITHDRAW_AMOUNT</a>));
    <a href="Dao.md#0x1_Dao_propose">Dao::propose</a>&lt;TokenT, <a href="TreasuryWithdrawDaoProposal.md#0x1_TreasuryWithdrawDaoProposal_WithdrawToken">WithdrawToken</a>&gt;(
        signer,
        <a href="TreasuryWithdrawDaoProposal.md#0x1_TreasuryWithdrawDaoProposal_WithdrawToken">WithdrawToken</a> { receiver, amount, period },
        exec_delay,
    );
}
</code></pre>



</details>

<a name="0x1_TreasuryWithdrawDaoProposal_execute_withdraw_proposal"></a>

## Function `execute_withdraw_proposal`

Once the proposal is agreed, anyone can call the method to make the proposal happen.


<pre><code><b>public</b> <b>fun</b> <a href="TreasuryWithdrawDaoProposal.md#0x1_TreasuryWithdrawDaoProposal_execute_withdraw_proposal">execute_withdraw_proposal</a>&lt;TokenT: <b>copy</b>, drop, store&gt;(signer: &signer, proposer_address: <b>address</b>, proposal_id: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="TreasuryWithdrawDaoProposal.md#0x1_TreasuryWithdrawDaoProposal_execute_withdraw_proposal">execute_withdraw_proposal</a>&lt;TokenT: <b>copy</b> + drop + store&gt;(
    signer: &signer,
    proposer_address: <b>address</b>,
    proposal_id: u64,
) <b>acquires</b> <a href="TreasuryWithdrawDaoProposal.md#0x1_TreasuryWithdrawDaoProposal_WrappedWithdrawCapability">WrappedWithdrawCapability</a> {
    <b>let</b> <a href="TreasuryWithdrawDaoProposal.md#0x1_TreasuryWithdrawDaoProposal_WithdrawToken">WithdrawToken</a> { receiver, amount, period } = <a href="Dao.md#0x1_Dao_extract_proposal_action">Dao::extract_proposal_action</a>&lt;TokenT, <a href="TreasuryWithdrawDaoProposal.md#0x1_TreasuryWithdrawDaoProposal_WithdrawToken">WithdrawToken</a>&gt;(
        proposer_address,
        proposal_id,
    );
    <b>assert</b>!(receiver == <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer), <a href="Errors.md#0x1_Errors_requires_address">Errors::requires_address</a>(<a href="TreasuryWithdrawDaoProposal.md#0x1_TreasuryWithdrawDaoProposal_ERR_NEED_RECEIVER_TO_EXECUTE">ERR_NEED_RECEIVER_TO_EXECUTE</a>));
    <b>let</b> cap = <b>borrow_global_mut</b>&lt;<a href="TreasuryWithdrawDaoProposal.md#0x1_TreasuryWithdrawDaoProposal_WrappedWithdrawCapability">WrappedWithdrawCapability</a>&lt;TokenT&gt;&gt;(<a href="Token.md#0x1_Token_token_address">Token::token_address</a>&lt;TokenT&gt;());
    <b>let</b> linear_cap = <a href="Treasury.md#0x1_Treasury_issue_linear_withdraw_capability">Treasury::issue_linear_withdraw_capability</a>&lt;TokenT&gt;(&<b>mut</b> cap.cap, amount, period);
    <a href="Treasury.md#0x1_Treasury_add_linear_withdraw_capability">Treasury::add_linear_withdraw_capability</a>(signer, linear_cap);
}
</code></pre>



</details>

<a name="0x1_TreasuryWithdrawDaoProposal_withdraw_for_block_reward"></a>

## Function `withdraw_for_block_reward`

Provider a port for get block reward STC from Treasury, only genesis account can invoke this function.
The TreasuryWithdrawCapability is locked in TreasuryWithdrawDaoProposal, and only can withdraw by DAO proposal.
This approach is not graceful, but restricts the operation to genesis accounts only, so there are no security issues either.


<pre><code><b>public</b> <b>fun</b> <a href="TreasuryWithdrawDaoProposal.md#0x1_TreasuryWithdrawDaoProposal_withdraw_for_block_reward">withdraw_for_block_reward</a>&lt;TokenT: store&gt;(signer: &signer, reward: u128): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenT&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="TreasuryWithdrawDaoProposal.md#0x1_TreasuryWithdrawDaoProposal_withdraw_for_block_reward">withdraw_for_block_reward</a>&lt;TokenT: store&gt;(signer: &signer, reward: u128):<a href="Token.md#0x1_Token">Token</a>&lt;TokenT&gt; <b>acquires</b> <a href="TreasuryWithdrawDaoProposal.md#0x1_TreasuryWithdrawDaoProposal_WrappedWithdrawCapability">WrappedWithdrawCapability</a>  {
    <a href="CoreAddresses.md#0x1_CoreAddresses_assert_genesis_address">CoreAddresses::assert_genesis_address</a>(signer);
    <b>let</b> cap = <b>borrow_global_mut</b>&lt;<a href="TreasuryWithdrawDaoProposal.md#0x1_TreasuryWithdrawDaoProposal_WrappedWithdrawCapability">WrappedWithdrawCapability</a>&lt;TokenT&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer));
    <a href="Treasury.md#0x1_Treasury_withdraw_with_capability">Treasury::withdraw_with_capability</a>(&<b>mut</b> cap.cap, reward)
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


<pre><code><b>public</b> <b>fun</b> <a href="TreasuryWithdrawDaoProposal.md#0x1_TreasuryWithdrawDaoProposal_plugin">plugin</a>&lt;TokenT: store&gt;(signer: &signer, cap: <a href="Treasury.md#0x1_Treasury_WithdrawCapability">Treasury::WithdrawCapability</a>&lt;TokenT&gt;)
</code></pre>




<pre><code><b>pragma</b> aborts_if_is_partial = <b>false</b>;
<b>let</b> sender = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer);
<b>aborts_if</b> sender != <a href="Token.md#0x1_Token_SPEC_TOKEN_TEST_ADDRESS">Token::SPEC_TOKEN_TEST_ADDRESS</a>();
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Treasury.md#0x1_Treasury_WithdrawCapability">Treasury::WithdrawCapability</a>&lt;TokenT&gt;&gt;(sender);
<b>aborts_if</b> <b>exists</b>&lt;<a href="TreasuryWithdrawDaoProposal.md#0x1_TreasuryWithdrawDaoProposal_WrappedWithdrawCapability">WrappedWithdrawCapability</a>&lt;TokenT&gt;&gt;(sender);
<b>ensures</b> !<b>exists</b>&lt;<a href="Treasury.md#0x1_Treasury_WithdrawCapability">Treasury::WithdrawCapability</a>&lt;TokenT&gt;&gt;(sender);
<b>ensures</b> <b>exists</b>&lt;<a href="TreasuryWithdrawDaoProposal.md#0x1_TreasuryWithdrawDaoProposal_WrappedWithdrawCapability">WrappedWithdrawCapability</a>&lt;TokenT&gt;&gt;(sender);
</code></pre>



<a name="@Specification_1_propose_withdraw"></a>

### Function `propose_withdraw`


<pre><code><b>public</b> <b>fun</b> <a href="TreasuryWithdrawDaoProposal.md#0x1_TreasuryWithdrawDaoProposal_propose_withdraw">propose_withdraw</a>&lt;TokenT: <b>copy</b>, drop, store&gt;(signer: &signer, receiver: <b>address</b>, amount: u128, period: u64, exec_delay: u64)
</code></pre>




<pre><code><b>pragma</b> aborts_if_is_partial = <b>false</b>;
<b>let</b> quorum_votes = <a href="Dao.md#0x1_Dao_spec_quorum_votes">Dao::spec_quorum_votes</a>&lt;TokenT&gt;();
<b>aborts_if</b> amount &gt; quorum_votes;
<b>include</b> <a href="Dao.md#0x1_Dao_AbortIfDaoConfigNotExist">Dao::AbortIfDaoConfigNotExist</a>&lt;TokenT&gt;;
<b>include</b> <a href="Dao.md#0x1_Dao_AbortIfDaoInfoNotExist">Dao::AbortIfDaoInfoNotExist</a>&lt;TokenT&gt;;
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Timestamp.md#0x1_Timestamp_CurrentTimeMilliseconds">Timestamp::CurrentTimeMilliseconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
<b>aborts_if</b> exec_delay &gt; 0 && exec_delay &lt; <a href="Dao.md#0x1_Dao_spec_dao_config">Dao::spec_dao_config</a>&lt;TokenT&gt;().min_action_delay;
<b>include</b> <a href="Dao.md#0x1_Dao_CheckQuorumVotes">Dao::CheckQuorumVotes</a>&lt;TokenT&gt;;
<b>let</b> sender = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer);
<b>aborts_if</b> <b>exists</b>&lt;<a href="Dao.md#0x1_Dao_Proposal">Dao::Proposal</a>&lt;TokenT, <a href="TreasuryWithdrawDaoProposal.md#0x1_TreasuryWithdrawDaoProposal_WithdrawToken">WithdrawToken</a>&gt;&gt;(sender);
</code></pre>



<a name="@Specification_1_execute_withdraw_proposal"></a>

### Function `execute_withdraw_proposal`


<pre><code><b>public</b> <b>fun</b> <a href="TreasuryWithdrawDaoProposal.md#0x1_TreasuryWithdrawDaoProposal_execute_withdraw_proposal">execute_withdraw_proposal</a>&lt;TokenT: <b>copy</b>, drop, store&gt;(signer: &signer, proposer_address: <b>address</b>, proposal_id: u64)
</code></pre>




<pre><code><b>pragma</b> aborts_if_is_partial = <b>true</b>;
<b>let</b> expected_states = vec&lt;u8&gt;(6);
<b>include</b> <a href="Dao.md#0x1_Dao_CheckProposalStates">Dao::CheckProposalStates</a>&lt;TokenT, <a href="TreasuryWithdrawDaoProposal.md#0x1_TreasuryWithdrawDaoProposal_WithdrawToken">WithdrawToken</a>&gt;{expected_states};
<b>let</b> proposal = <b>global</b>&lt;<a href="Dao.md#0x1_Dao_Proposal">Dao::Proposal</a>&lt;TokenT, <a href="TreasuryWithdrawDaoProposal.md#0x1_TreasuryWithdrawDaoProposal_WithdrawToken">WithdrawToken</a>&gt;&gt;(proposer_address);
<b>aborts_if</b> <a href="Option.md#0x1_Option_is_none">Option::is_none</a>(proposal.action);
<b>aborts_if</b> !<b>exists</b>&lt;<a href="TreasuryWithdrawDaoProposal.md#0x1_TreasuryWithdrawDaoProposal_WrappedWithdrawCapability">WrappedWithdrawCapability</a>&lt;TokenT&gt;&gt;(<a href="Token.md#0x1_Token_SPEC_TOKEN_TEST_ADDRESS">Token::SPEC_TOKEN_TEST_ADDRESS</a>());
</code></pre>
