
<a id="0x1_treasury_withdraw_dao_proposal"></a>

# Module `0x1::treasury_withdraw_dao_proposal`

TreasuryWithdrawDaoProposal is a dao proposal for withdraw Token from Treasury.


-  [Resource `WrappedWithdrawCapability`](#0x1_treasury_withdraw_dao_proposal_WrappedWithdrawCapability)
-  [Struct `WithdrawToken`](#0x1_treasury_withdraw_dao_proposal_WithdrawToken)
-  [Constants](#@Constants_0)
-  [Function `plugin`](#0x1_treasury_withdraw_dao_proposal_plugin)
-  [Function `propose_withdraw`](#0x1_treasury_withdraw_dao_proposal_propose_withdraw)
-  [Function `execute_withdraw_proposal`](#0x1_treasury_withdraw_dao_proposal_execute_withdraw_proposal)
-  [Function `withdraw_for_block_reward`](#0x1_treasury_withdraw_dao_proposal_withdraw_for_block_reward)
-  [Specification](#@Specification_1)
    -  [Function `plugin`](#@Specification_1_plugin)
    -  [Function `propose_withdraw`](#@Specification_1_propose_withdraw)
    -  [Function `execute_withdraw_proposal`](#@Specification_1_execute_withdraw_proposal)


<pre><code><b>use</b> <a href="coin.md#0x1_coin">0x1::coin</a>;
<b>use</b> <a href="dao.md#0x1_dao">0x1::dao</a>;
<b>use</b> <a href="../../move-stdlib/doc/error.md#0x1_error">0x1::error</a>;
<b>use</b> <a href="../../move-stdlib/doc/signer.md#0x1_signer">0x1::signer</a>;
<b>use</b> <a href="stc_util.md#0x1_stc_util">0x1::stc_util</a>;
<b>use</b> <a href="system_addresses.md#0x1_system_addresses">0x1::system_addresses</a>;
<b>use</b> <a href="treasury.md#0x1_treasury">0x1::treasury</a>;
</code></pre>



<a id="0x1_treasury_withdraw_dao_proposal_WrappedWithdrawCapability"></a>

## Resource `WrappedWithdrawCapability`

A wrapper of Token MintCapability.


<pre><code><b>struct</b> <a href="treasury_withdraw_dao_proposal.md#0x1_treasury_withdraw_dao_proposal_WrappedWithdrawCapability">WrappedWithdrawCapability</a>&lt;TokenT&gt; <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>cap: <a href="treasury.md#0x1_treasury_WithdrawCapability">treasury::WithdrawCapability</a>&lt;TokenT&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="0x1_treasury_withdraw_dao_proposal_WithdrawToken"></a>

## Struct `WithdrawToken`

WithdrawToken request.


<pre><code><b>struct</b> <a href="treasury_withdraw_dao_proposal.md#0x1_treasury_withdraw_dao_proposal_WithdrawToken">WithdrawToken</a> <b>has</b> <b>copy</b>, drop, store
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

<a id="@Constants_0"></a>

## Constants


<a id="0x1_treasury_withdraw_dao_proposal_ERR_NOT_AUTHORIZED"></a>



<pre><code><b>const</b> <a href="treasury_withdraw_dao_proposal.md#0x1_treasury_withdraw_dao_proposal_ERR_NOT_AUTHORIZED">ERR_NOT_AUTHORIZED</a>: u64 = 101;
</code></pre>



<a id="0x1_treasury_withdraw_dao_proposal_ERR_NEED_RECEIVER_TO_EXECUTE"></a>

Only receiver can execute TreasuryWithdrawDaoProposal


<pre><code><b>const</b> <a href="treasury_withdraw_dao_proposal.md#0x1_treasury_withdraw_dao_proposal_ERR_NEED_RECEIVER_TO_EXECUTE">ERR_NEED_RECEIVER_TO_EXECUTE</a>: u64 = 102;
</code></pre>



<a id="0x1_treasury_withdraw_dao_proposal_ERR_TOO_MANY_WITHDRAW_AMOUNT"></a>

The withdraw amount of propose is too many.


<pre><code><b>const</b> <a href="treasury_withdraw_dao_proposal.md#0x1_treasury_withdraw_dao_proposal_ERR_TOO_MANY_WITHDRAW_AMOUNT">ERR_TOO_MANY_WITHDRAW_AMOUNT</a>: u64 = 103;
</code></pre>



<a id="0x1_treasury_withdraw_dao_proposal_plugin"></a>

## Function `plugin`

Plugin method of the module.
Should be called by token issuer.


<pre><code><b>public</b> <b>fun</b> <a href="treasury_withdraw_dao_proposal.md#0x1_treasury_withdraw_dao_proposal_plugin">plugin</a>&lt;TokenT&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, cap: <a href="treasury.md#0x1_treasury_WithdrawCapability">treasury::WithdrawCapability</a>&lt;TokenT&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="treasury_withdraw_dao_proposal.md#0x1_treasury_withdraw_dao_proposal_plugin">plugin</a>&lt;TokenT&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, cap: <a href="treasury.md#0x1_treasury_WithdrawCapability">treasury::WithdrawCapability</a>&lt;TokenT&gt;) {
    <b>let</b> token_issuer = <a href="stc_util.md#0x1_stc_util_token_issuer">stc_util::token_issuer</a>&lt;TokenT&gt;();
    <b>assert</b>!(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>) == token_issuer, <a href="../../move-stdlib/doc/error.md#0x1_error_not_found">error::not_found</a>(<a href="treasury_withdraw_dao_proposal.md#0x1_treasury_withdraw_dao_proposal_ERR_NOT_AUTHORIZED">ERR_NOT_AUTHORIZED</a>));
    <b>move_to</b>(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, <a href="treasury_withdraw_dao_proposal.md#0x1_treasury_withdraw_dao_proposal_WrappedWithdrawCapability">WrappedWithdrawCapability</a>&lt;TokenT&gt; { cap });
}
</code></pre>



</details>

<a id="0x1_treasury_withdraw_dao_proposal_propose_withdraw"></a>

## Function `propose_withdraw`

Entrypoint for the proposal.


<pre><code><b>public</b> <b>fun</b> <a href="treasury_withdraw_dao_proposal.md#0x1_treasury_withdraw_dao_proposal_propose_withdraw">propose_withdraw</a>&lt;TokenT&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, receiver: <b>address</b>, amount: u128, period: u64, exec_delay: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="treasury_withdraw_dao_proposal.md#0x1_treasury_withdraw_dao_proposal_propose_withdraw">propose_withdraw</a>&lt;TokenT&gt;(
    <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    receiver: <b>address</b>,
    amount: u128,
    period: u64,
    exec_delay: u64
) {
    <b>let</b> quorum_votes = <a href="dao.md#0x1_dao_quorum_votes">dao::quorum_votes</a>&lt;TokenT&gt;();
    <b>assert</b>!(amount &lt;= quorum_votes, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="treasury_withdraw_dao_proposal.md#0x1_treasury_withdraw_dao_proposal_ERR_TOO_MANY_WITHDRAW_AMOUNT">ERR_TOO_MANY_WITHDRAW_AMOUNT</a>));
    <a href="dao.md#0x1_dao_propose">dao::propose</a>&lt;TokenT, <a href="treasury_withdraw_dao_proposal.md#0x1_treasury_withdraw_dao_proposal_WithdrawToken">WithdrawToken</a>&gt;(
        <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
        <a href="treasury_withdraw_dao_proposal.md#0x1_treasury_withdraw_dao_proposal_WithdrawToken">WithdrawToken</a> { receiver, amount, period },
        exec_delay,
    );
}
</code></pre>



</details>

<a id="0x1_treasury_withdraw_dao_proposal_execute_withdraw_proposal"></a>

## Function `execute_withdraw_proposal`

Once the proposal is agreed, anyone can call the method to make the proposal happen.


<pre><code><b>public</b> <b>fun</b> <a href="treasury_withdraw_dao_proposal.md#0x1_treasury_withdraw_dao_proposal_execute_withdraw_proposal">execute_withdraw_proposal</a>&lt;TokenT&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, proposer_address: <b>address</b>, proposal_id: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="treasury_withdraw_dao_proposal.md#0x1_treasury_withdraw_dao_proposal_execute_withdraw_proposal">execute_withdraw_proposal</a>&lt;TokenT&gt;(
    <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    proposer_address: <b>address</b>,
    proposal_id: u64,
) <b>acquires</b> <a href="treasury_withdraw_dao_proposal.md#0x1_treasury_withdraw_dao_proposal_WrappedWithdrawCapability">WrappedWithdrawCapability</a> {
    <b>let</b> <a href="treasury_withdraw_dao_proposal.md#0x1_treasury_withdraw_dao_proposal_WithdrawToken">WithdrawToken</a> { receiver, amount, period } = <a href="dao.md#0x1_dao_extract_proposal_action">dao::extract_proposal_action</a>&lt;TokenT, <a href="treasury_withdraw_dao_proposal.md#0x1_treasury_withdraw_dao_proposal_WithdrawToken">WithdrawToken</a>&gt;(
        proposer_address,
        proposal_id,
    );
    <b>assert</b>!(receiver == <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>), <a href="../../move-stdlib/doc/error.md#0x1_error_not_found">error::not_found</a>(<a href="treasury_withdraw_dao_proposal.md#0x1_treasury_withdraw_dao_proposal_ERR_NEED_RECEIVER_TO_EXECUTE">ERR_NEED_RECEIVER_TO_EXECUTE</a>));
    <b>let</b> cap =
        <b>borrow_global_mut</b>&lt;<a href="treasury_withdraw_dao_proposal.md#0x1_treasury_withdraw_dao_proposal_WrappedWithdrawCapability">WrappedWithdrawCapability</a>&lt;TokenT&gt;&gt;(<a href="stc_util.md#0x1_stc_util_token_issuer">stc_util::token_issuer</a>&lt;TokenT&gt;());
    <b>let</b> linear_cap =
        <a href="treasury.md#0x1_treasury_issue_linear_withdraw_capability">treasury::issue_linear_withdraw_capability</a>&lt;TokenT&gt;(&<b>mut</b> cap.cap, amount, period);
    <a href="treasury.md#0x1_treasury_add_linear_withdraw_capability">treasury::add_linear_withdraw_capability</a>(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, linear_cap);
}
</code></pre>



</details>

<a id="0x1_treasury_withdraw_dao_proposal_withdraw_for_block_reward"></a>

## Function `withdraw_for_block_reward`

Provider a port for get block reward STC from Treasury, only genesis account can invoke this function.
The TreasuryWithdrawCapability is locked in TreasuryWithdrawDaoProposal, and only can withdraw by DAO proposal.
This approach is not graceful, but restricts the operation to genesis accounts only, so there are no security issues either.


<pre><code><b>public</b> <b>fun</b> <a href="treasury_withdraw_dao_proposal.md#0x1_treasury_withdraw_dao_proposal_withdraw_for_block_reward">withdraw_for_block_reward</a>&lt;TokenT&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, reward: u128): <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;TokenT&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="treasury_withdraw_dao_proposal.md#0x1_treasury_withdraw_dao_proposal_withdraw_for_block_reward">withdraw_for_block_reward</a>&lt;TokenT&gt;(
    <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    reward: u128
): <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;TokenT&gt; <b>acquires</b> <a href="treasury_withdraw_dao_proposal.md#0x1_treasury_withdraw_dao_proposal_WrappedWithdrawCapability">WrappedWithdrawCapability</a> {
    <a href="system_addresses.md#0x1_system_addresses_assert_starcoin_framework">system_addresses::assert_starcoin_framework</a>(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>);
    <b>let</b> cap = <b>borrow_global_mut</b>&lt;<a href="treasury_withdraw_dao_proposal.md#0x1_treasury_withdraw_dao_proposal_WrappedWithdrawCapability">WrappedWithdrawCapability</a>&lt;TokenT&gt;&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>));
    <a href="treasury.md#0x1_treasury_withdraw_with_capability">treasury::withdraw_with_capability</a>(&<b>mut</b> cap.cap, reward)
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


<pre><code><b>public</b> <b>fun</b> <a href="treasury_withdraw_dao_proposal.md#0x1_treasury_withdraw_dao_proposal_plugin">plugin</a>&lt;TokenT&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, cap: <a href="treasury.md#0x1_treasury_WithdrawCapability">treasury::WithdrawCapability</a>&lt;TokenT&gt;)
</code></pre>




<pre><code><b>pragma</b> aborts_if_is_partial = <b>false</b>;
<b>let</b> sender = <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>);
<b>aborts_if</b> sender != @0x2;
<b>aborts_if</b> !<b>exists</b>&lt;<a href="treasury.md#0x1_treasury_WithdrawCapability">treasury::WithdrawCapability</a>&lt;TokenT&gt;&gt;(sender);
<b>aborts_if</b> <b>exists</b>&lt;<a href="treasury_withdraw_dao_proposal.md#0x1_treasury_withdraw_dao_proposal_WrappedWithdrawCapability">WrappedWithdrawCapability</a>&lt;TokenT&gt;&gt;(sender);
<b>ensures</b> !<b>exists</b>&lt;<a href="treasury.md#0x1_treasury_WithdrawCapability">treasury::WithdrawCapability</a>&lt;TokenT&gt;&gt;(sender);
<b>ensures</b> <b>exists</b>&lt;<a href="treasury_withdraw_dao_proposal.md#0x1_treasury_withdraw_dao_proposal_WrappedWithdrawCapability">WrappedWithdrawCapability</a>&lt;TokenT&gt;&gt;(sender);
</code></pre>



<a id="@Specification_1_propose_withdraw"></a>

### Function `propose_withdraw`


<pre><code><b>public</b> <b>fun</b> <a href="treasury_withdraw_dao_proposal.md#0x1_treasury_withdraw_dao_proposal_propose_withdraw">propose_withdraw</a>&lt;TokenT&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, receiver: <b>address</b>, amount: u128, period: u64, exec_delay: u64)
</code></pre>




<pre><code><b>pragma</b> aborts_if_is_partial = <b>false</b>;
<b>let</b> quorum_votes = <a href="dao.md#0x1_dao_spec_quorum_votes">dao::spec_quorum_votes</a>&lt;TokenT&gt;();
<b>aborts_if</b> amount &gt; quorum_votes;
<b>include</b> <a href="dao.md#0x1_dao_AbortIfDaoConfigNotExist">dao::AbortIfDaoConfigNotExist</a>&lt;TokenT&gt;;
<b>include</b> <a href="dao.md#0x1_dao_AbortIfDaoInfoNotExist">dao::AbortIfDaoInfoNotExist</a>&lt;TokenT&gt;;
<b>aborts_if</b> !<b>exists</b>&lt;<a href="timestamp.md#0x1_timestamp_CurrentTimeMicroseconds">timestamp::CurrentTimeMicroseconds</a>&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>());
<b>aborts_if</b> exec_delay &gt; 0 && exec_delay &lt; <a href="dao.md#0x1_dao_spec_dao_config">dao::spec_dao_config</a>&lt;TokenT&gt;().min_action_delay;
<b>include</b> <a href="dao.md#0x1_dao_CheckQuorumVotes">dao::CheckQuorumVotes</a>&lt;TokenT&gt;;
<b>let</b> sender = <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>);
<b>aborts_if</b> <b>exists</b>&lt;<a href="dao.md#0x1_dao_Proposal">dao::Proposal</a>&lt;TokenT, <a href="treasury_withdraw_dao_proposal.md#0x1_treasury_withdraw_dao_proposal_WithdrawToken">WithdrawToken</a>&gt;&gt;(sender);
</code></pre>



<a id="@Specification_1_execute_withdraw_proposal"></a>

### Function `execute_withdraw_proposal`


<pre><code><b>public</b> <b>fun</b> <a href="treasury_withdraw_dao_proposal.md#0x1_treasury_withdraw_dao_proposal_execute_withdraw_proposal">execute_withdraw_proposal</a>&lt;TokenT&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, proposer_address: <b>address</b>, proposal_id: u64)
</code></pre>




<pre><code><b>pragma</b> aborts_if_is_partial = <b>true</b>;
<b>let</b> expected_states = vec&lt;u8&gt;(6);
<b>include</b> <a href="dao.md#0x1_dao_CheckProposalStates">dao::CheckProposalStates</a>&lt;TokenT, <a href="treasury_withdraw_dao_proposal.md#0x1_treasury_withdraw_dao_proposal_WithdrawToken">WithdrawToken</a>&gt; { expected_states };
<b>let</b> proposal = <b>global</b>&lt;<a href="dao.md#0x1_dao_Proposal">dao::Proposal</a>&lt;TokenT, <a href="treasury_withdraw_dao_proposal.md#0x1_treasury_withdraw_dao_proposal_WithdrawToken">WithdrawToken</a>&gt;&gt;(proposer_address);
<b>aborts_if</b> <a href="../../move-stdlib/doc/option.md#0x1_option_is_none">option::is_none</a>(proposal.action);
<b>aborts_if</b> !<b>exists</b>&lt;<a href="treasury_withdraw_dao_proposal.md#0x1_treasury_withdraw_dao_proposal_WrappedWithdrawCapability">WrappedWithdrawCapability</a>&lt;TokenT&gt;&gt;(@0x2);
</code></pre>


[move-book]: https://starcoin.dev/move/book/SUMMARY
