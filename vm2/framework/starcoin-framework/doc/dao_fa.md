
<a id="0x1_dao_fa"></a>

# Module `0x1::dao_fa`



-  [Resource `DaoGlobalInfo`](#0x1_dao_fa_DaoGlobalInfo)
-  [Struct `DaoConfig`](#0x1_dao_fa_DaoConfig)
-  [Struct `ProposalCreatedEvent`](#0x1_dao_fa_ProposalCreatedEvent)
-  [Struct `VoteChangedEvent`](#0x1_dao_fa_VoteChangedEvent)
-  [Resource `Proposal`](#0x1_dao_fa_Proposal)
-  [Resource `Vote`](#0x1_dao_fa_Vote)
-  [Constants](#@Constants_0)
-  [Function `plugin`](#0x1_dao_fa_plugin)
-  [Function `new_dao_config`](#0x1_dao_fa_new_dao_config)
-  [Function `propose`](#0x1_dao_fa_propose)
-  [Function `cast_vote`](#0x1_dao_fa_cast_vote)
-  [Function `do_cast_vote`](#0x1_dao_fa_do_cast_vote)
-  [Function `change_vote`](#0x1_dao_fa_change_vote)
-  [Function `do_flip_vote`](#0x1_dao_fa_do_flip_vote)
-  [Function `revoke_vote`](#0x1_dao_fa_revoke_vote)
-  [Function `do_revoke_vote`](#0x1_dao_fa_do_revoke_vote)
-  [Function `unstake_votes`](#0x1_dao_fa_unstake_votes)
-  [Function `queue_proposal_action`](#0x1_dao_fa_queue_proposal_action)
-  [Function `extract_proposal_action`](#0x1_dao_fa_extract_proposal_action)
-  [Function `destroy_terminated_proposal`](#0x1_dao_fa_destroy_terminated_proposal)
-  [Function `proposal_exists`](#0x1_dao_fa_proposal_exists)
-  [Function `proposal_state`](#0x1_dao_fa_proposal_state)
-  [Function `do_proposal_state`](#0x1_dao_fa_do_proposal_state)
-  [Function `proposal_info`](#0x1_dao_fa_proposal_info)
-  [Function `vote_of`](#0x1_dao_fa_vote_of)
-  [Function `has_vote`](#0x1_dao_fa_has_vote)
-  [Function `generate_next_proposal_id`](#0x1_dao_fa_generate_next_proposal_id)
-  [Function `voting_delay`](#0x1_dao_fa_voting_delay)
-  [Function `voting_period`](#0x1_dao_fa_voting_period)
-  [Function `coin_to_fa_metadata`](#0x1_dao_fa_coin_to_fa_metadata)
-  [Function `quorum_votes`](#0x1_dao_fa_quorum_votes)
-  [Function `voting_quorum_rate`](#0x1_dao_fa_voting_quorum_rate)
-  [Function `min_action_delay`](#0x1_dao_fa_min_action_delay)
-  [Function `get_config`](#0x1_dao_fa_get_config)
-  [Function `modify_dao_config`](#0x1_dao_fa_modify_dao_config)
-  [Function `set_voting_delay`](#0x1_dao_fa_set_voting_delay)
-  [Function `set_voting_period`](#0x1_dao_fa_set_voting_period)
-  [Function `set_voting_quorum_rate`](#0x1_dao_fa_set_voting_quorum_rate)
-  [Function `set_min_action_delay`](#0x1_dao_fa_set_min_action_delay)


<pre><code><b>use</b> <a href="account.md#0x1_account">0x1::account</a>;
<b>use</b> <a href="coin.md#0x1_coin">0x1::coin</a>;
<b>use</b> <a href="create_signer.md#0x1_create_signer">0x1::create_signer</a>;
<b>use</b> <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug">0x1::debug</a>;
<b>use</b> <a href="../../move-stdlib/doc/error.md#0x1_error">0x1::error</a>;
<b>use</b> <a href="event.md#0x1_event">0x1::event</a>;
<b>use</b> <a href="fungible_asset.md#0x1_fungible_asset">0x1::fungible_asset</a>;
<b>use</b> <a href="object.md#0x1_object">0x1::object</a>;
<b>use</b> <a href="on_chain_config.md#0x1_on_chain_config">0x1::on_chain_config</a>;
<b>use</b> <a href="../../move-stdlib/doc/option.md#0x1_option">0x1::option</a>;
<b>use</b> <a href="../../move-stdlib/doc/signer.md#0x1_signer">0x1::signer</a>;
<b>use</b> <a href="stc_util.md#0x1_stc_util">0x1::stc_util</a>;
<b>use</b> <a href="../../move-stdlib/doc/string.md#0x1_string">0x1::string</a>;
<b>use</b> <a href="system_addresses.md#0x1_system_addresses">0x1::system_addresses</a>;
<b>use</b> <a href="timestamp.md#0x1_timestamp">0x1::timestamp</a>;
<b>use</b> <a href="treasury_fa.md#0x1_treasury_fa">0x1::treasury_fa</a>;
</code></pre>



<a id="0x1_dao_fa_DaoGlobalInfo"></a>

## Resource `DaoGlobalInfo`

global DAO info of the specified token type <code>Token</code>.


<pre><code><b>struct</b> <a href="dao_fa.md#0x1_dao_fa_DaoGlobalInfo">DaoGlobalInfo</a>&lt;Token&gt; <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>next_proposal_id: u64</code>
</dt>
<dd>
 next proposal id.
</dd>
<dt>
<code>proposal_create_event: <a href="event.md#0x1_event_EventHandle">event::EventHandle</a>&lt;<a href="dao_fa.md#0x1_dao_fa_ProposalCreatedEvent">dao_fa::ProposalCreatedEvent</a>&gt;</code>
</dt>
<dd>
 proposal creating event.
</dd>
<dt>
<code>vote_changed_event: <a href="event.md#0x1_event_EventHandle">event::EventHandle</a>&lt;<a href="dao_fa.md#0x1_dao_fa_VoteChangedEvent">dao_fa::VoteChangedEvent</a>&gt;</code>
</dt>
<dd>
 voting event.
</dd>
</dl>


</details>

<a id="0x1_dao_fa_DaoConfig"></a>

## Struct `DaoConfig`

Configuration of the <code>Token</code>'s DAO.


<pre><code><b>struct</b> <a href="dao_fa.md#0x1_dao_fa_DaoConfig">DaoConfig</a>&lt;CoinT&gt; <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>voting_delay: u64</code>
</dt>
<dd>
 after proposal created, how long use should wait before he can vote (in milliseconds)
</dd>
<dt>
<code>voting_period: u64</code>
</dt>
<dd>
 how long the voting window is (in milliseconds).
</dd>
<dt>
<code>voting_quorum_rate: u8</code>
</dt>
<dd>
 the quorum rate to agree on the proposal.
 if 50% votes needed, then the voting_quorum_rate should be 50.
 it should between (0, 100].
</dd>
<dt>
<code>min_action_delay: u64</code>
</dt>
<dd>
 how long the proposal should wait before it can be executed (in milliseconds).
</dd>
</dl>


</details>

<a id="0x1_dao_fa_ProposalCreatedEvent"></a>

## Struct `ProposalCreatedEvent`

emitted when proposal created.


<pre><code><b>struct</b> <a href="dao_fa.md#0x1_dao_fa_ProposalCreatedEvent">ProposalCreatedEvent</a> <b>has</b> drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>proposal_id: u64</code>
</dt>
<dd>
 the proposal id.
</dd>
<dt>
<code>proposer: <b>address</b></code>
</dt>
<dd>
 proposer is the user who create the proposal.
</dd>
</dl>


</details>

<a id="0x1_dao_fa_VoteChangedEvent"></a>

## Struct `VoteChangedEvent`

emitted when user vote/revoke_vote.


<pre><code><b>struct</b> <a href="dao_fa.md#0x1_dao_fa_VoteChangedEvent">VoteChangedEvent</a> <b>has</b> drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>proposal_id: u64</code>
</dt>
<dd>
 the proposal id.
</dd>
<dt>
<code>voter: <b>address</b></code>
</dt>
<dd>
 the voter.
</dd>
<dt>
<code>proposer: <b>address</b></code>
</dt>
<dd>
 creator of the proposal.
</dd>
<dt>
<code>agree: bool</code>
</dt>
<dd>
 agree with the proposal or not
</dd>
<dt>
<code>vote: u128</code>
</dt>
<dd>
 latest vote count of the voter.
</dd>
</dl>


</details>

<a id="0x1_dao_fa_Proposal"></a>

## Resource `Proposal`

Proposal data struct.


<pre><code><b>struct</b> <a href="dao_fa.md#0x1_dao_fa_Proposal">Proposal</a>&lt;Token, Action: store&gt; <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>id: u64</code>
</dt>
<dd>
 id of the proposal
</dd>
<dt>
<code>proposer: <b>address</b></code>
</dt>
<dd>
 creator of the proposal
</dd>
<dt>
<code>start_time: u64</code>
</dt>
<dd>
 when voting begins.
</dd>
<dt>
<code>end_time: u64</code>
</dt>
<dd>
 when voting ends.
</dd>
<dt>
<code>for_votes: u128</code>
</dt>
<dd>
 count of voters who agree with the proposal
</dd>
<dt>
<code>against_votes: u128</code>
</dt>
<dd>
 count of voters who're against the proposal
</dd>
<dt>
<code>eta: u64</code>
</dt>
<dd>
 executable after this time.
</dd>
<dt>
<code>action_delay: u64</code>
</dt>
<dd>
 after how long, the agreed proposal can be executed.
</dd>
<dt>
<code>quorum_votes: u128</code>
</dt>
<dd>
 how many votes to reach to make the proposal pass.
</dd>
<dt>
<code>action: <a href="../../move-stdlib/doc/option.md#0x1_option_Option">option::Option</a>&lt;Action&gt;</code>
</dt>
<dd>
 proposal action.
</dd>
</dl>


</details>

<a id="0x1_dao_fa_Vote"></a>

## Resource `Vote`

User vote info.


<pre><code><b>struct</b> <a href="dao_fa.md#0x1_dao_fa_Vote">Vote</a>&lt;CoinT&gt; <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>proposer: <b>address</b></code>
</dt>
<dd>
 vote for the proposal under the <code>proposer</code>.
</dd>
<dt>
<code>id: u64</code>
</dt>
<dd>
 proposal id.
</dd>
<dt>
<code>stake_store: <a href="object.md#0x1_object_Object">object::Object</a>&lt;<a href="fungible_asset.md#0x1_fungible_asset_FungibleStore">fungible_asset::FungibleStore</a>&gt;</code>
</dt>
<dd>
 how many tokens to stake.
</dd>
<dt>
<code>stake_store_delete_ref: <a href="object.md#0x1_object_DeleteRef">object::DeleteRef</a></code>
</dt>
<dd>
 Delete ref for delete stake store
</dd>
<dt>
<code>agree: bool</code>
</dt>
<dd>
 vote for or vote against.
</dd>
</dl>


</details>

<a id="@Constants_0"></a>

## Constants


<a id="0x1_dao_fa_ERR_NOT_AUTHORIZED"></a>



<pre><code><b>const</b> <a href="dao_fa.md#0x1_dao_fa_ERR_NOT_AUTHORIZED">ERR_NOT_AUTHORIZED</a>: u64 = 1401;
</code></pre>



<a id="0x1_dao_fa_ACTIVE"></a>



<pre><code><b>const</b> <a href="dao_fa.md#0x1_dao_fa_ACTIVE">ACTIVE</a>: u8 = 2;
</code></pre>



<a id="0x1_dao_fa_AGREED"></a>



<pre><code><b>const</b> <a href="dao_fa.md#0x1_dao_fa_AGREED">AGREED</a>: u8 = 4;
</code></pre>



<a id="0x1_dao_fa_DEFEATED"></a>



<pre><code><b>const</b> <a href="dao_fa.md#0x1_dao_fa_DEFEATED">DEFEATED</a>: u8 = 3;
</code></pre>



<a id="0x1_dao_fa_ERR_ACTION_DELAY_TOO_SMALL"></a>



<pre><code><b>const</b> <a href="dao_fa.md#0x1_dao_fa_ERR_ACTION_DELAY_TOO_SMALL">ERR_ACTION_DELAY_TOO_SMALL</a>: u64 = 1402;
</code></pre>



<a id="0x1_dao_fa_ERR_ACTION_MUST_EXIST"></a>



<pre><code><b>const</b> <a href="dao_fa.md#0x1_dao_fa_ERR_ACTION_MUST_EXIST">ERR_ACTION_MUST_EXIST</a>: u64 = 1409;
</code></pre>



<a id="0x1_dao_fa_ERR_CONFIG_PARAM_INVALID"></a>



<pre><code><b>const</b> <a href="dao_fa.md#0x1_dao_fa_ERR_CONFIG_PARAM_INVALID">ERR_CONFIG_PARAM_INVALID</a>: u64 = 1407;
</code></pre>



<a id="0x1_dao_fa_ERR_PROPOSAL_ID_MISMATCH"></a>



<pre><code><b>const</b> <a href="dao_fa.md#0x1_dao_fa_ERR_PROPOSAL_ID_MISMATCH">ERR_PROPOSAL_ID_MISMATCH</a>: u64 = 1404;
</code></pre>



<a id="0x1_dao_fa_ERR_PROPOSAL_STATE_INVALID"></a>



<pre><code><b>const</b> <a href="dao_fa.md#0x1_dao_fa_ERR_PROPOSAL_STATE_INVALID">ERR_PROPOSAL_STATE_INVALID</a>: u64 = 1403;
</code></pre>



<a id="0x1_dao_fa_ERR_PROPOSER_MISMATCH"></a>



<pre><code><b>const</b> <a href="dao_fa.md#0x1_dao_fa_ERR_PROPOSER_MISMATCH">ERR_PROPOSER_MISMATCH</a>: u64 = 1405;
</code></pre>



<a id="0x1_dao_fa_ERR_QUORUM_RATE_INVALID"></a>



<pre><code><b>const</b> <a href="dao_fa.md#0x1_dao_fa_ERR_QUORUM_RATE_INVALID">ERR_QUORUM_RATE_INVALID</a>: u64 = 1406;
</code></pre>



<a id="0x1_dao_fa_ERR_TOKEN_NOT_REGISTER"></a>



<pre><code><b>const</b> <a href="dao_fa.md#0x1_dao_fa_ERR_TOKEN_NOT_REGISTER">ERR_TOKEN_NOT_REGISTER</a>: u64 = 1411;
</code></pre>



<a id="0x1_dao_fa_ERR_VOTED_OTHERS_ALREADY"></a>



<pre><code><b>const</b> <a href="dao_fa.md#0x1_dao_fa_ERR_VOTED_OTHERS_ALREADY">ERR_VOTED_OTHERS_ALREADY</a>: u64 = 1410;
</code></pre>



<a id="0x1_dao_fa_ERR_VOTE_STATE_MISMATCH"></a>



<pre><code><b>const</b> <a href="dao_fa.md#0x1_dao_fa_ERR_VOTE_STATE_MISMATCH">ERR_VOTE_STATE_MISMATCH</a>: u64 = 1408;
</code></pre>



<a id="0x1_dao_fa_EXECUTABLE"></a>



<pre><code><b>const</b> <a href="dao_fa.md#0x1_dao_fa_EXECUTABLE">EXECUTABLE</a>: u8 = 6;
</code></pre>



<a id="0x1_dao_fa_EXTRACTED"></a>



<pre><code><b>const</b> <a href="dao_fa.md#0x1_dao_fa_EXTRACTED">EXTRACTED</a>: u8 = 7;
</code></pre>



<a id="0x1_dao_fa_PENDING"></a>

Proposal state


<pre><code><b>const</b> <a href="dao_fa.md#0x1_dao_fa_PENDING">PENDING</a>: u8 = 1;
</code></pre>



<a id="0x1_dao_fa_QUEUED"></a>



<pre><code><b>const</b> <a href="dao_fa.md#0x1_dao_fa_QUEUED">QUEUED</a>: u8 = 5;
</code></pre>



<a id="0x1_dao_fa_ERR_COIN_NOT_FOUND"></a>



<pre><code><b>const</b> <a href="dao_fa.md#0x1_dao_fa_ERR_COIN_NOT_FOUND">ERR_COIN_NOT_FOUND</a>: u64 = 1412;
</code></pre>



<a id="0x1_dao_fa_ERR_FUNGIBLE_ASSET_NOT_MATCH"></a>



<pre><code><b>const</b> <a href="dao_fa.md#0x1_dao_fa_ERR_FUNGIBLE_ASSET_NOT_MATCH">ERR_FUNGIBLE_ASSET_NOT_MATCH</a>: u64 = 1413;
</code></pre>



<a id="0x1_dao_fa_ERR_REVOKE_INSUFFICIENT_BALANCE"></a>



<pre><code><b>const</b> <a href="dao_fa.md#0x1_dao_fa_ERR_REVOKE_INSUFFICIENT_BALANCE">ERR_REVOKE_INSUFFICIENT_BALANCE</a>: u64 = 1414;
</code></pre>



<a id="0x1_dao_fa_ERR_REVOKE_WITHDRAW_WRONG_FA"></a>



<pre><code><b>const</b> <a href="dao_fa.md#0x1_dao_fa_ERR_REVOKE_WITHDRAW_WRONG_FA">ERR_REVOKE_WITHDRAW_WRONG_FA</a>: u64 = 1415;
</code></pre>



<a id="0x1_dao_fa_plugin"></a>

## Function `plugin`

plugin function, can only be called by token issuer.
Any token who wants to have gov functionality
can optin this module by call this <code>register function</code>.


<pre><code><b>public</b> <b>fun</b> <a href="dao_fa.md#0x1_dao_fa_plugin">plugin</a>&lt;CoinT&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, voting_delay: u64, voting_period: u64, voting_quorum_rate: u8, min_action_delay: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="dao_fa.md#0x1_dao_fa_plugin">plugin</a>&lt;CoinT&gt;(
    <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    voting_delay: u64,
    voting_period: u64,
    voting_quorum_rate: u8,
    min_action_delay: u64,
) {
    <b>let</b> token_issuer = <a href="stc_util.md#0x1_stc_util_token_issuer">stc_util::token_issuer</a>&lt;CoinT&gt;();
    <b>assert</b>!(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>) == token_issuer, <a href="../../move-stdlib/doc/error.md#0x1_error_not_found">error::not_found</a>(<a href="dao_fa.md#0x1_dao_fa_ERR_NOT_AUTHORIZED">ERR_NOT_AUTHORIZED</a>));
    // <b>let</b> proposal_id = ProposalId {next: 0};
    <b>let</b> gov_info = <a href="dao_fa.md#0x1_dao_fa_DaoGlobalInfo">DaoGlobalInfo</a>&lt;CoinT&gt; {
        next_proposal_id: 0,
        proposal_create_event: <a href="account.md#0x1_account_new_event_handle">account::new_event_handle</a>&lt;<a href="dao_fa.md#0x1_dao_fa_ProposalCreatedEvent">ProposalCreatedEvent</a>&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>),
        vote_changed_event: <a href="account.md#0x1_account_new_event_handle">account::new_event_handle</a>&lt;<a href="dao_fa.md#0x1_dao_fa_VoteChangedEvent">VoteChangedEvent</a>&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>),
    };
    <b>move_to</b>(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, gov_info);
    <b>let</b> config = <a href="dao_fa.md#0x1_dao_fa_new_dao_config">new_dao_config</a>&lt;CoinT&gt;(
        voting_delay,
        voting_period,
        voting_quorum_rate,
        min_action_delay,
    );
    <a href="on_chain_config.md#0x1_on_chain_config_publish_new_config">on_chain_config::publish_new_config</a>(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, config);
}
</code></pre>



</details>

<a id="0x1_dao_fa_new_dao_config"></a>

## Function `new_dao_config`

create a dao config


<pre><code><b>public</b> <b>fun</b> <a href="dao_fa.md#0x1_dao_fa_new_dao_config">new_dao_config</a>&lt;CoinT&gt;(voting_delay: u64, voting_period: u64, voting_quorum_rate: u8, min_action_delay: u64): <a href="dao_fa.md#0x1_dao_fa_DaoConfig">dao_fa::DaoConfig</a>&lt;CoinT&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="dao_fa.md#0x1_dao_fa_new_dao_config">new_dao_config</a>&lt;CoinT&gt;(
    voting_delay: u64,
    voting_period: u64,
    voting_quorum_rate: u8,
    min_action_delay: u64,
): <a href="dao_fa.md#0x1_dao_fa_DaoConfig">DaoConfig</a>&lt;CoinT&gt; {
    <b>assert</b>!(voting_delay &gt; 0, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="dao_fa.md#0x1_dao_fa_ERR_CONFIG_PARAM_INVALID">ERR_CONFIG_PARAM_INVALID</a>));
    <b>assert</b>!(voting_period &gt; 0, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="dao_fa.md#0x1_dao_fa_ERR_CONFIG_PARAM_INVALID">ERR_CONFIG_PARAM_INVALID</a>));
    <b>assert</b>!(
        voting_quorum_rate &gt; 0 && <a href="dao_fa.md#0x1_dao_fa_voting_quorum_rate">voting_quorum_rate</a> &lt;= 100,
        <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="dao_fa.md#0x1_dao_fa_ERR_CONFIG_PARAM_INVALID">ERR_CONFIG_PARAM_INVALID</a>),
    );
    <b>assert</b>!(min_action_delay &gt; 0, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="dao_fa.md#0x1_dao_fa_ERR_CONFIG_PARAM_INVALID">ERR_CONFIG_PARAM_INVALID</a>));
    <a href="dao_fa.md#0x1_dao_fa_DaoConfig">DaoConfig</a> { voting_delay, voting_period, voting_quorum_rate, min_action_delay }
}
</code></pre>



</details>

<a id="0x1_dao_fa_propose"></a>

## Function `propose`

propose a proposal.
<code>action</code>: the actual action to execute.
<code>action_delay</code>: the delay to execute after the proposal is agreed


<pre><code><b>public</b> <b>fun</b> <a href="dao_fa.md#0x1_dao_fa_propose">propose</a>&lt;CoinT, ActionT: <b>copy</b>, drop, store&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, action: ActionT, action_delay: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="dao_fa.md#0x1_dao_fa_propose">propose</a>&lt;CoinT, ActionT: <b>copy</b> + drop + store&gt;(
    <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    action: ActionT,
    action_delay: u64,
) <b>acquires</b> <a href="dao_fa.md#0x1_dao_fa_DaoGlobalInfo">DaoGlobalInfo</a> {
    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"dao::proposal | Entered"));

    <b>if</b> (action_delay == 0) {
        action_delay = <a href="dao_fa.md#0x1_dao_fa_min_action_delay">min_action_delay</a>&lt;CoinT&gt;();
    } <b>else</b> {
        <b>assert</b>!(action_delay &gt;= <a href="dao_fa.md#0x1_dao_fa_min_action_delay">min_action_delay</a>&lt;CoinT&gt;(), <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="dao_fa.md#0x1_dao_fa_ERR_ACTION_DELAY_TOO_SMALL">ERR_ACTION_DELAY_TOO_SMALL</a>));
    };
    <b>let</b> proposal_id = <a href="dao_fa.md#0x1_dao_fa_generate_next_proposal_id">generate_next_proposal_id</a>&lt;CoinT&gt;();
    <b>let</b> proposer = <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>);
    <b>let</b> start_time = <a href="timestamp.md#0x1_timestamp_now_milliseconds">timestamp::now_milliseconds</a>() + <a href="dao_fa.md#0x1_dao_fa_voting_delay">voting_delay</a>&lt;CoinT&gt;();
    <b>let</b> quorum_votes = <a href="dao_fa.md#0x1_dao_fa_quorum_votes">quorum_votes</a>&lt;CoinT&gt;();

    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"dao::proposal | <a href="dao_fa.md#0x1_dao_fa_Proposal">Proposal</a> "));
    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&proposal_id);
    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&start_time);

    <b>let</b> proposal = <a href="dao_fa.md#0x1_dao_fa_Proposal">Proposal</a>&lt;CoinT, ActionT&gt; {
        id: proposal_id,
        proposer,
        start_time,
        end_time: start_time + <a href="dao_fa.md#0x1_dao_fa_voting_period">voting_period</a>&lt;CoinT&gt;(),
        for_votes: 0,
        against_votes: 0,
        eta: 0,
        action_delay,
        quorum_votes,
        action: <a href="../../move-stdlib/doc/option.md#0x1_option_some">option::some</a>(action),
    };
    <b>move_to</b>(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, proposal);
    // emit <a href="event.md#0x1_event">event</a>
    <b>let</b> gov_info = <b>borrow_global_mut</b>&lt;<a href="dao_fa.md#0x1_dao_fa_DaoGlobalInfo">DaoGlobalInfo</a>&lt;CoinT&gt;&gt;(<a href="stc_util.md#0x1_stc_util_token_issuer">stc_util::token_issuer</a>&lt;CoinT&gt;());

    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"dao::proposal | emit <a href="event.md#0x1_event">event</a>"));

    <a href="event.md#0x1_event_emit_event">event::emit_event</a>(
        &<b>mut</b> gov_info.proposal_create_event,
        <a href="dao_fa.md#0x1_dao_fa_ProposalCreatedEvent">ProposalCreatedEvent</a> { proposal_id, proposer },
    );

    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"dao::proposal | Exited"));
}
</code></pre>



</details>

<a id="0x1_dao_fa_cast_vote"></a>

## Function `cast_vote`

votes for a proposal.
User can only vote once, then the stake is locked,
which can only be unstaked by user after the proposal is expired, or cancelled, or executed.
So think twice before casting vote.


<pre><code><b>public</b> <b>fun</b> <a href="dao_fa.md#0x1_dao_fa_cast_vote">cast_vote</a>&lt;CoinT, ActionT: <b>copy</b>, drop, store&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, proposer_address: <b>address</b>, proposal_id: u64, stake: <a href="fungible_asset.md#0x1_fungible_asset_FungibleAsset">fungible_asset::FungibleAsset</a>, agree: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="dao_fa.md#0x1_dao_fa_cast_vote">cast_vote</a>&lt;CoinT, ActionT: <b>copy</b> + drop + store&gt;(
    <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    proposer_address: <b>address</b>,
    proposal_id: u64,
    stake: FungibleAsset,
    agree: bool,
) <b>acquires</b> <a href="dao_fa.md#0x1_dao_fa_Proposal">Proposal</a>, <a href="dao_fa.md#0x1_dao_fa_DaoGlobalInfo">DaoGlobalInfo</a>, <a href="dao_fa.md#0x1_dao_fa_Vote">Vote</a> {
    {
        <b>let</b> state = <a href="dao_fa.md#0x1_dao_fa_proposal_state">proposal_state</a>&lt;CoinT, ActionT&gt;(proposer_address, proposal_id);
        // only when proposal is active, <b>use</b> can cast vote.
        <b>assert</b>!(state == <a href="dao_fa.md#0x1_dao_fa_ACTIVE">ACTIVE</a>, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_state">error::invalid_state</a>(<a href="dao_fa.md#0x1_dao_fa_ERR_PROPOSAL_STATE_INVALID">ERR_PROPOSAL_STATE_INVALID</a>));
    };

    <b>let</b> fa_metadata = <a href="fungible_asset.md#0x1_fungible_asset_metadata_from_asset">fungible_asset::metadata_from_asset</a>(&stake);
    <b>let</b> coin_metadata = <a href="coin.md#0x1_coin_paired_metadata">coin::paired_metadata</a>&lt;CoinT&gt;();
    <b>assert</b>!(<a href="../../move-stdlib/doc/option.md#0x1_option_is_some">option::is_some</a>(&coin_metadata), <a href="../../move-stdlib/doc/error.md#0x1_error_not_found">error::not_found</a>(<a href="dao_fa.md#0x1_dao_fa_ERR_COIN_NOT_FOUND">ERR_COIN_NOT_FOUND</a>));

    <b>let</b> coin_metadata = <a href="../../move-stdlib/doc/option.md#0x1_option_destroy_some">option::destroy_some</a>(coin_metadata);
    <b>assert</b>!(fa_metadata == coin_metadata, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(
        <a href="dao_fa.md#0x1_dao_fa_ERR_FUNGIBLE_ASSET_NOT_MATCH">ERR_FUNGIBLE_ASSET_NOT_MATCH</a>
    ));

    <b>let</b> proposal = <b>borrow_global_mut</b>&lt;<a href="dao_fa.md#0x1_dao_fa_Proposal">Proposal</a>&lt;CoinT, ActionT&gt;&gt;(proposer_address);
    <b>assert</b>!(proposal.id == proposal_id, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="dao_fa.md#0x1_dao_fa_ERR_PROPOSAL_ID_MISMATCH">ERR_PROPOSAL_ID_MISMATCH</a>));
    <b>let</b> sender = <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>);
    <b>let</b> total_voted = <b>if</b> (<b>exists</b>&lt;<a href="dao_fa.md#0x1_dao_fa_Vote">Vote</a>&lt;CoinT&gt;&gt;(sender)) {
        <b>let</b> my_vote = <b>borrow_global_mut</b>&lt;<a href="dao_fa.md#0x1_dao_fa_Vote">Vote</a>&lt;CoinT&gt;&gt;(sender);
        <b>assert</b>!(my_vote.id == proposal_id, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="dao_fa.md#0x1_dao_fa_ERR_VOTED_OTHERS_ALREADY">ERR_VOTED_OTHERS_ALREADY</a>));
        <b>assert</b>!(my_vote.agree == agree, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_state">error::invalid_state</a>(<a href="dao_fa.md#0x1_dao_fa_ERR_VOTE_STATE_MISMATCH">ERR_VOTE_STATE_MISMATCH</a>));

        <a href="dao_fa.md#0x1_dao_fa_do_cast_vote">do_cast_vote</a>(proposal, my_vote, stake);
        <a href="fungible_asset.md#0x1_fungible_asset_balance">fungible_asset::balance</a>(my_vote.stake_store)
    } <b>else</b> {
        <b>let</b> construct_ref = <a href="object.md#0x1_object_create_object">object::create_object</a>(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>));
        <b>let</b> stake_store = <a href="fungible_asset.md#0x1_fungible_asset_create_store">fungible_asset::create_store</a>(&construct_ref, coin_metadata);
        <b>let</b> my_vote = <a href="dao_fa.md#0x1_dao_fa_Vote">Vote</a>&lt;CoinT&gt; {
            proposer: proposer_address,
            id: proposal_id,
            stake_store,
            stake_store_delete_ref: <a href="object.md#0x1_object_generate_delete_ref">object::generate_delete_ref</a>(&construct_ref),
            agree,
        };
        <a href="dao_fa.md#0x1_dao_fa_do_cast_vote">do_cast_vote</a>(proposal, &<b>mut</b> my_vote, stake);
        <b>let</b> total_voted = <a href="fungible_asset.md#0x1_fungible_asset_balance">fungible_asset::balance</a>(my_vote.stake_store);
        <b>move_to</b>(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, my_vote);
        total_voted
    };

    // emit <a href="event.md#0x1_event">event</a>
    <b>let</b> gov_info = <b>borrow_global_mut</b>&lt;<a href="dao_fa.md#0x1_dao_fa_DaoGlobalInfo">DaoGlobalInfo</a>&lt;CoinT&gt;&gt;(<a href="stc_util.md#0x1_stc_util_token_issuer">stc_util::token_issuer</a>&lt;CoinT&gt;());
    <a href="event.md#0x1_event_emit_event">event::emit_event</a>(
        &<b>mut</b> gov_info.vote_changed_event,
        <a href="dao_fa.md#0x1_dao_fa_VoteChangedEvent">VoteChangedEvent</a> {
            proposal_id,
            proposer: proposer_address,
            voter: sender,
            agree,
            vote: (total_voted <b>as</b> u128),
        },
    );
}
</code></pre>



</details>

<a id="0x1_dao_fa_do_cast_vote"></a>

## Function `do_cast_vote`



<pre><code><b>fun</b> <a href="dao_fa.md#0x1_dao_fa_do_cast_vote">do_cast_vote</a>&lt;CoinT, ActionT: <b>copy</b>, drop, store&gt;(proposal: &<b>mut</b> <a href="dao_fa.md#0x1_dao_fa_Proposal">dao_fa::Proposal</a>&lt;CoinT, ActionT&gt;, vote: &<b>mut</b> <a href="dao_fa.md#0x1_dao_fa_Vote">dao_fa::Vote</a>&lt;CoinT&gt;, stake_fa: <a href="fungible_asset.md#0x1_fungible_asset_FungibleAsset">fungible_asset::FungibleAsset</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="dao_fa.md#0x1_dao_fa_do_cast_vote">do_cast_vote</a>&lt;CoinT, ActionT: <b>copy</b> + drop + store&gt;(
    proposal: &<b>mut</b> <a href="dao_fa.md#0x1_dao_fa_Proposal">Proposal</a>&lt;CoinT, ActionT&gt;,
    vote: &<b>mut</b> <a href="dao_fa.md#0x1_dao_fa_Vote">Vote</a>&lt;CoinT&gt;,
    stake_fa: <a href="fungible_asset.md#0x1_fungible_asset_FungibleAsset">fungible_asset::FungibleAsset</a>,
) {
    <b>let</b> stake_value = <a href="fungible_asset.md#0x1_fungible_asset_amount">fungible_asset::amount</a>(&stake_fa);
    <a href="fungible_asset.md#0x1_fungible_asset_deposit">fungible_asset::deposit</a>(vote.stake_store, stake_fa);
    <b>if</b> (vote.agree) {
        proposal.for_votes = proposal.for_votes + (stake_value <b>as</b> u128);
    } <b>else</b> {
        proposal.against_votes = proposal.against_votes + (stake_value <b>as</b> u128);
    };
}
</code></pre>



</details>

<a id="0x1_dao_fa_change_vote"></a>

## Function `change_vote`

Let user change their vote during the voting time.


<pre><code><b>public</b> <b>fun</b> <a href="dao_fa.md#0x1_dao_fa_change_vote">change_vote</a>&lt;CoinT, ActionT: <b>copy</b>, drop, store&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, proposer_address: <b>address</b>, proposal_id: u64, agree: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="dao_fa.md#0x1_dao_fa_change_vote">change_vote</a>&lt;CoinT, ActionT: <b>copy</b> + drop + store&gt;(
    <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    proposer_address: <b>address</b>,
    proposal_id: u64,
    agree: bool,
) <b>acquires</b> <a href="dao_fa.md#0x1_dao_fa_Proposal">Proposal</a>, <a href="dao_fa.md#0x1_dao_fa_DaoGlobalInfo">DaoGlobalInfo</a>, <a href="dao_fa.md#0x1_dao_fa_Vote">Vote</a> {
    {
        <b>let</b> state = <a href="dao_fa.md#0x1_dao_fa_proposal_state">proposal_state</a>&lt;CoinT, ActionT&gt;(proposer_address, proposal_id);
        // only when proposal is active, user can change vote.
        <b>assert</b>!(state == <a href="dao_fa.md#0x1_dao_fa_ACTIVE">ACTIVE</a>, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_state">error::invalid_state</a>(<a href="dao_fa.md#0x1_dao_fa_ERR_PROPOSAL_STATE_INVALID">ERR_PROPOSAL_STATE_INVALID</a>));
    };
    <b>let</b> proposal = <b>borrow_global_mut</b>&lt;<a href="dao_fa.md#0x1_dao_fa_Proposal">Proposal</a>&lt;CoinT, ActionT&gt;&gt;(proposer_address);
    <b>assert</b>!(proposal.id == proposal_id, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="dao_fa.md#0x1_dao_fa_ERR_PROPOSAL_ID_MISMATCH">ERR_PROPOSAL_ID_MISMATCH</a>));
    <b>let</b> my_vote = <b>borrow_global_mut</b>&lt;<a href="dao_fa.md#0x1_dao_fa_Vote">Vote</a>&lt;CoinT&gt;&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>));
    {
        <b>assert</b>!(my_vote.proposer == proposer_address, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="dao_fa.md#0x1_dao_fa_ERR_PROPOSER_MISMATCH">ERR_PROPOSER_MISMATCH</a>));
        <b>assert</b>!(my_vote.id == proposal_id, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="dao_fa.md#0x1_dao_fa_ERR_VOTED_OTHERS_ALREADY">ERR_VOTED_OTHERS_ALREADY</a>));
    };

    // flip the vote
    <b>if</b> (my_vote.agree != agree) {
        <b>let</b> total_voted = <a href="dao_fa.md#0x1_dao_fa_do_flip_vote">do_flip_vote</a>(my_vote, proposal);
        // emit <a href="event.md#0x1_event">event</a>
        <b>let</b> gov_info = <b>borrow_global_mut</b>&lt;<a href="dao_fa.md#0x1_dao_fa_DaoGlobalInfo">DaoGlobalInfo</a>&lt;CoinT&gt;&gt;(<a href="stc_util.md#0x1_stc_util_token_issuer">stc_util::token_issuer</a>&lt;CoinT&gt;());
        <a href="event.md#0x1_event_emit_event">event::emit_event</a>(
            &<b>mut</b> gov_info.vote_changed_event,
            <a href="dao_fa.md#0x1_dao_fa_VoteChangedEvent">VoteChangedEvent</a> {
                proposal_id,
                proposer: proposer_address,
                voter: <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>),
                agree,
                vote: total_voted,
            },
        );
    };
}
</code></pre>



</details>

<a id="0x1_dao_fa_do_flip_vote"></a>

## Function `do_flip_vote`



<pre><code><b>fun</b> <a href="dao_fa.md#0x1_dao_fa_do_flip_vote">do_flip_vote</a>&lt;CoinT, ActionT: <b>copy</b>, drop, store&gt;(my_vote: &<b>mut</b> <a href="dao_fa.md#0x1_dao_fa_Vote">dao_fa::Vote</a>&lt;CoinT&gt;, proposal: &<b>mut</b> <a href="dao_fa.md#0x1_dao_fa_Proposal">dao_fa::Proposal</a>&lt;CoinT, ActionT&gt;): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="dao_fa.md#0x1_dao_fa_do_flip_vote">do_flip_vote</a>&lt;CoinT, ActionT: <b>copy</b> + drop + store&gt;(
    my_vote: &<b>mut</b> <a href="dao_fa.md#0x1_dao_fa_Vote">Vote</a>&lt;CoinT&gt;,
    proposal: &<b>mut</b> <a href="dao_fa.md#0x1_dao_fa_Proposal">Proposal</a>&lt;CoinT, ActionT&gt;
): u128 {
    my_vote.agree = !my_vote.agree;
    <b>let</b> total_voted = (<a href="fungible_asset.md#0x1_fungible_asset_balance">fungible_asset::balance</a>(my_vote.stake_store) <b>as</b> u128);
    <b>if</b> (my_vote.agree) {
        proposal.for_votes = proposal.for_votes + total_voted;
        proposal.against_votes = proposal.against_votes - total_voted;
    } <b>else</b> {
        proposal.for_votes = proposal.for_votes - total_voted;
        proposal.against_votes = proposal.against_votes + total_voted;
    };
    total_voted
}
</code></pre>



</details>

<a id="0x1_dao_fa_revoke_vote"></a>

## Function `revoke_vote`

Revoke some voting powers from vote on <code>proposal_id</code> of <code>proposer_address</code>.


<pre><code><b>public</b> <b>fun</b> <a href="dao_fa.md#0x1_dao_fa_revoke_vote">revoke_vote</a>&lt;CoinT, ActionT: <b>copy</b>, drop, store&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, proposer_address: <b>address</b>, proposal_id: u64, voting_power: u128): <a href="fungible_asset.md#0x1_fungible_asset_FungibleAsset">fungible_asset::FungibleAsset</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="dao_fa.md#0x1_dao_fa_revoke_vote">revoke_vote</a>&lt;CoinT, ActionT: <b>copy</b> + drop + store&gt;(
    <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    proposer_address: <b>address</b>,
    proposal_id: u64,
    voting_power: u128,
): FungibleAsset <b>acquires</b> <a href="dao_fa.md#0x1_dao_fa_Proposal">Proposal</a>, <a href="dao_fa.md#0x1_dao_fa_Vote">Vote</a>, <a href="dao_fa.md#0x1_dao_fa_DaoGlobalInfo">DaoGlobalInfo</a> {
    {
        <b>let</b> state = <a href="dao_fa.md#0x1_dao_fa_proposal_state">proposal_state</a>&lt;CoinT, ActionT&gt;(proposer_address, proposal_id);
        // only when proposal is active, user can revoke vote.
        <b>assert</b>!(state == <a href="dao_fa.md#0x1_dao_fa_ACTIVE">ACTIVE</a>, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_state">error::invalid_state</a>(<a href="dao_fa.md#0x1_dao_fa_ERR_PROPOSAL_STATE_INVALID">ERR_PROPOSAL_STATE_INVALID</a>));
    };
    // get proposal
    <b>let</b> proposal = <b>borrow_global_mut</b>&lt;<a href="dao_fa.md#0x1_dao_fa_Proposal">Proposal</a>&lt;CoinT, ActionT&gt;&gt;(proposer_address);

    // get vote
    <b>let</b> my_vote = <b>move_from</b>&lt;<a href="dao_fa.md#0x1_dao_fa_Vote">Vote</a>&lt;CoinT&gt;&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>));
    {
        <b>assert</b>!(my_vote.proposer == proposer_address, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="dao_fa.md#0x1_dao_fa_ERR_PROPOSER_MISMATCH">ERR_PROPOSER_MISMATCH</a>));
        <b>assert</b>!(my_vote.id == proposal_id, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="dao_fa.md#0x1_dao_fa_ERR_VOTED_OTHERS_ALREADY">ERR_VOTED_OTHERS_ALREADY</a>));
    };
    // revoke vote on proposal
    <b>let</b> reverted_stake = <a href="dao_fa.md#0x1_dao_fa_do_revoke_vote">do_revoke_vote</a>(proposal, &<b>mut</b> my_vote, voting_power);
    // emit vote changed <a href="event.md#0x1_event">event</a>
    <b>let</b> gov_info = <b>borrow_global_mut</b>&lt;<a href="dao_fa.md#0x1_dao_fa_DaoGlobalInfo">DaoGlobalInfo</a>&lt;CoinT&gt;&gt;(<a href="object.md#0x1_object_owner">object::owner</a>(<a href="dao_fa.md#0x1_dao_fa_coin_to_fa_metadata">coin_to_fa_metadata</a>&lt;CoinT&gt;()));
    <a href="event.md#0x1_event_emit_event">event::emit_event</a>(
        &<b>mut</b> gov_info.vote_changed_event,
        <a href="dao_fa.md#0x1_dao_fa_VoteChangedEvent">VoteChangedEvent</a> {
            proposal_id,
            proposer: proposer_address,
            voter: <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>),
            agree: my_vote.agree,
            vote: (<a href="fungible_asset.md#0x1_fungible_asset_balance">fungible_asset::balance</a>(my_vote.stake_store) <b>as</b> u128),
        },
    );

    // <b>if</b> user <b>has</b> no stake, destroy his vote. resolve https://github.com/starcoinorg/starcoin/issues/2925.
    <b>if</b> (<a href="fungible_asset.md#0x1_fungible_asset_balance">fungible_asset::balance</a>(my_vote.stake_store) == 0) {
        <b>let</b> <a href="dao_fa.md#0x1_dao_fa_Vote">Vote</a> { stake_store: _, stake_store_delete_ref: store_delete_ref, proposer: _, id: _, agree: _ } = my_vote;
        <a href="fungible_asset.md#0x1_fungible_asset_remove_store">fungible_asset::remove_store</a>(&store_delete_ref);
    } <b>else</b> {
        <b>move_to</b>(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, my_vote);
    };

    reverted_stake
}
</code></pre>



</details>

<a id="0x1_dao_fa_do_revoke_vote"></a>

## Function `do_revoke_vote`



<pre><code><b>fun</b> <a href="dao_fa.md#0x1_dao_fa_do_revoke_vote">do_revoke_vote</a>&lt;CoinT, ActionT: <b>copy</b>, drop, store&gt;(proposal: &<b>mut</b> <a href="dao_fa.md#0x1_dao_fa_Proposal">dao_fa::Proposal</a>&lt;CoinT, ActionT&gt;, vote: &<b>mut</b> <a href="dao_fa.md#0x1_dao_fa_Vote">dao_fa::Vote</a>&lt;CoinT&gt;, to_revoke: u128): <a href="fungible_asset.md#0x1_fungible_asset_FungibleAsset">fungible_asset::FungibleAsset</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="dao_fa.md#0x1_dao_fa_do_revoke_vote">do_revoke_vote</a>&lt;CoinT, ActionT: <b>copy</b> + drop + store&gt;(
    proposal: &<b>mut</b> <a href="dao_fa.md#0x1_dao_fa_Proposal">Proposal</a>&lt;CoinT, ActionT&gt;,
    vote: &<b>mut</b> <a href="dao_fa.md#0x1_dao_fa_Vote">Vote</a>&lt;CoinT&gt;,
    to_revoke: u128
): FungibleAsset {
    <b>let</b> stake_amount = <a href="fungible_asset.md#0x1_fungible_asset_balance">fungible_asset::balance</a>(vote.stake_store);
    <b>assert</b>!((stake_amount <b>as</b> u128) &lt;= to_revoke, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="dao_fa.md#0x1_dao_fa_ERR_REVOKE_INSUFFICIENT_BALANCE">ERR_REVOKE_INSUFFICIENT_BALANCE</a>));

    <b>let</b> <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a> = <a href="create_signer.md#0x1_create_signer">create_signer</a>(<a href="object.md#0x1_object_owner">object::owner</a>(vote.stake_store));
    <b>let</b> reverted_stake = <a href="fungible_asset.md#0x1_fungible_asset_withdraw">fungible_asset::withdraw</a>(&<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, vote.stake_store, (to_revoke <b>as</b> u64));
    <b>if</b> (vote.agree) {
        proposal.for_votes = proposal.for_votes - to_revoke;
    } <b>else</b> {
        proposal.against_votes = proposal.against_votes - to_revoke;
    };
    <b>assert</b>!(
        to_revoke == (<a href="fungible_asset.md#0x1_fungible_asset_amount">fungible_asset::amount</a>(&reverted_stake) <b>as</b> u128),
        <a href="../../move-stdlib/doc/error.md#0x1_error_unavailable">error::unavailable</a>(<a href="dao_fa.md#0x1_dao_fa_ERR_REVOKE_WITHDRAW_WRONG_FA">ERR_REVOKE_WITHDRAW_WRONG_FA</a>)
    );
    reverted_stake
}
</code></pre>



</details>

<a id="0x1_dao_fa_unstake_votes"></a>

## Function `unstake_votes`

Retrieve back my staked token voted for a proposal.


<pre><code><b>public</b> <b>fun</b> <a href="dao_fa.md#0x1_dao_fa_unstake_votes">unstake_votes</a>&lt;CoinT, ActionT: <b>copy</b>, drop, store&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, proposer_address: <b>address</b>, proposal_id: u64): <a href="fungible_asset.md#0x1_fungible_asset_FungibleAsset">fungible_asset::FungibleAsset</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="dao_fa.md#0x1_dao_fa_unstake_votes">unstake_votes</a>&lt;CoinT, ActionT: <b>copy</b> + drop + store&gt;(
    <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    proposer_address: <b>address</b>,
    proposal_id: u64,
): FungibleAsset <b>acquires</b> <a href="dao_fa.md#0x1_dao_fa_Proposal">Proposal</a>, <a href="dao_fa.md#0x1_dao_fa_Vote">Vote</a> {
    // only check state when proposal <b>exists</b>.
    // because proposal can be destroyed after it ends in <a href="dao_fa.md#0x1_dao_fa_DEFEATED">DEFEATED</a> or <a href="dao_fa.md#0x1_dao_fa_EXTRACTED">EXTRACTED</a> state.
    <b>if</b> (<a href="dao_fa.md#0x1_dao_fa_proposal_exists">proposal_exists</a>&lt;CoinT, ActionT&gt;(proposer_address, proposal_id)) {
        <b>let</b> state = <a href="dao_fa.md#0x1_dao_fa_proposal_state">proposal_state</a>&lt;CoinT, ActionT&gt;(proposer_address, proposal_id);
        // Only after vote period end, user can unstake his votes.
        <b>assert</b>!(state &gt; <a href="dao_fa.md#0x1_dao_fa_ACTIVE">ACTIVE</a>, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_state">error::invalid_state</a>(<a href="dao_fa.md#0x1_dao_fa_ERR_PROPOSAL_STATE_INVALID">ERR_PROPOSAL_STATE_INVALID</a>));
    };
    <b>let</b> <a href="dao_fa.md#0x1_dao_fa_Vote">Vote</a> {
        proposer,
        id,
        stake_store,
        stake_store_delete_ref,
        agree: _
    } = <b>move_from</b>&lt;<a href="dao_fa.md#0x1_dao_fa_Vote">Vote</a>&lt;CoinT&gt;&gt;(
        <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>),
    );
    // these checks are still required.
    <b>assert</b>!(proposer == proposer_address, <a href="../../move-stdlib/doc/error.md#0x1_error_not_found">error::not_found</a>(<a href="dao_fa.md#0x1_dao_fa_ERR_PROPOSER_MISMATCH">ERR_PROPOSER_MISMATCH</a>));
    <b>assert</b>!(id == proposal_id, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="dao_fa.md#0x1_dao_fa_ERR_VOTED_OTHERS_ALREADY">ERR_VOTED_OTHERS_ALREADY</a>));

    <b>let</b> staking_amount = <a href="fungible_asset.md#0x1_fungible_asset_balance">fungible_asset::balance</a>(stake_store);
    <b>let</b> asset = <a href="fungible_asset.md#0x1_fungible_asset_withdraw">fungible_asset::withdraw</a>(
        &<a href="create_signer.md#0x1_create_signer_create_signer">create_signer::create_signer</a>(<a href="object.md#0x1_object_owner">object::owner</a>(stake_store)),
        stake_store,
        staking_amount
    );
    <a href="fungible_asset.md#0x1_fungible_asset_remove_store">fungible_asset::remove_store</a>(&stake_store_delete_ref);
    asset
}
</code></pre>



</details>

<a id="0x1_dao_fa_queue_proposal_action"></a>

## Function `queue_proposal_action`

queue agreed proposal to execute.


<pre><code><b>public</b> entry <b>fun</b> <a href="dao_fa.md#0x1_dao_fa_queue_proposal_action">queue_proposal_action</a>&lt;CoinT, ActionT: <b>copy</b>, drop, store&gt;(proposer_address: <b>address</b>, proposal_id: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="dao_fa.md#0x1_dao_fa_queue_proposal_action">queue_proposal_action</a>&lt;CoinT, ActionT: <b>copy</b> + drop + store&gt;(
    proposer_address: <b>address</b>,
    proposal_id: u64,
) <b>acquires</b> <a href="dao_fa.md#0x1_dao_fa_Proposal">Proposal</a> {
    // Only agreed proposal can be submitted.
    <b>assert</b>!(
        <a href="dao_fa.md#0x1_dao_fa_proposal_state">proposal_state</a>&lt;CoinT, ActionT&gt;(proposer_address, proposal_id) == <a href="dao_fa.md#0x1_dao_fa_AGREED">AGREED</a>,
        <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_state">error::invalid_state</a>(<a href="dao_fa.md#0x1_dao_fa_ERR_PROPOSAL_STATE_INVALID">ERR_PROPOSAL_STATE_INVALID</a>)
    );
    <b>let</b> proposal = <b>borrow_global_mut</b>&lt;<a href="dao_fa.md#0x1_dao_fa_Proposal">Proposal</a>&lt;CoinT, ActionT&gt;&gt;(proposer_address);
    proposal.eta = <a href="timestamp.md#0x1_timestamp_now_milliseconds">timestamp::now_milliseconds</a>() + proposal.action_delay;
}
</code></pre>



</details>

<a id="0x1_dao_fa_extract_proposal_action"></a>

## Function `extract_proposal_action`

extract proposal action to execute.


<pre><code><b>public</b> <b>fun</b> <a href="dao_fa.md#0x1_dao_fa_extract_proposal_action">extract_proposal_action</a>&lt;CoinT, ActionT: <b>copy</b>, drop, store&gt;(proposer_address: <b>address</b>, proposal_id: u64): ActionT
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="dao_fa.md#0x1_dao_fa_extract_proposal_action">extract_proposal_action</a>&lt;CoinT, ActionT: <b>copy</b> + drop + store&gt;(
    proposer_address: <b>address</b>,
    proposal_id: u64,
): ActionT <b>acquires</b> <a href="dao_fa.md#0x1_dao_fa_Proposal">Proposal</a> {
    // Only executable proposal's action can be extracted.
    <b>assert</b>!(
        <a href="dao_fa.md#0x1_dao_fa_proposal_state">proposal_state</a>&lt;CoinT, ActionT&gt;(proposer_address, proposal_id) == <a href="dao_fa.md#0x1_dao_fa_EXECUTABLE">EXECUTABLE</a>,
        <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_state">error::invalid_state</a>(<a href="dao_fa.md#0x1_dao_fa_ERR_PROPOSAL_STATE_INVALID">ERR_PROPOSAL_STATE_INVALID</a>),
    );
    <b>let</b> proposal = <b>borrow_global_mut</b>&lt;<a href="dao_fa.md#0x1_dao_fa_Proposal">Proposal</a>&lt;CoinT, ActionT&gt;&gt;(proposer_address);
    <b>let</b> action: ActionT = <a href="../../move-stdlib/doc/option.md#0x1_option_extract">option::extract</a>(&<b>mut</b> proposal.action);
    action
}
</code></pre>



</details>

<a id="0x1_dao_fa_destroy_terminated_proposal"></a>

## Function `destroy_terminated_proposal`

remove terminated proposal from proposer


<pre><code><b>public</b> entry <b>fun</b> <a href="dao_fa.md#0x1_dao_fa_destroy_terminated_proposal">destroy_terminated_proposal</a>&lt;CoinT, ActionT: <b>copy</b>, drop, store&gt;(proposer_address: <b>address</b>, proposal_id: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="dao_fa.md#0x1_dao_fa_destroy_terminated_proposal">destroy_terminated_proposal</a>&lt;CoinT, ActionT: <b>copy</b> + drop + store&gt;(
    proposer_address: <b>address</b>,
    proposal_id: u64,
) <b>acquires</b> <a href="dao_fa.md#0x1_dao_fa_Proposal">Proposal</a> {
    <b>let</b> proposal_state = <a href="dao_fa.md#0x1_dao_fa_proposal_state">proposal_state</a>&lt;CoinT, ActionT&gt;(proposer_address, proposal_id);
    <b>assert</b>!(
        proposal_state == <a href="dao_fa.md#0x1_dao_fa_DEFEATED">DEFEATED</a> || proposal_state == <a href="dao_fa.md#0x1_dao_fa_EXTRACTED">EXTRACTED</a>,
        <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_state">error::invalid_state</a>(<a href="dao_fa.md#0x1_dao_fa_ERR_PROPOSAL_STATE_INVALID">ERR_PROPOSAL_STATE_INVALID</a>),
    );
    <b>let</b> <a href="dao_fa.md#0x1_dao_fa_Proposal">Proposal</a> {
        id: _,
        proposer: _,
        start_time: _,
        end_time: _,
        for_votes: _,
        against_votes: _,
        eta: _,
        action_delay: _,
        quorum_votes: _,
        action,
    } = <b>move_from</b>&lt;<a href="dao_fa.md#0x1_dao_fa_Proposal">Proposal</a>&lt;CoinT, ActionT&gt;&gt;(proposer_address);
    <b>if</b> (proposal_state == <a href="dao_fa.md#0x1_dao_fa_DEFEATED">DEFEATED</a>) {
        <b>let</b> _ = <a href="../../move-stdlib/doc/option.md#0x1_option_extract">option::extract</a>(&<b>mut</b> action);
    };
    <a href="../../move-stdlib/doc/option.md#0x1_option_destroy_none">option::destroy_none</a>(action);
}
</code></pre>



</details>

<a id="0x1_dao_fa_proposal_exists"></a>

## Function `proposal_exists`

check whether a proposal exists in <code>proposer_address</code> with id <code>proposal_id</code>.


<pre><code><b>public</b> <b>fun</b> <a href="dao_fa.md#0x1_dao_fa_proposal_exists">proposal_exists</a>&lt;CoinT, ActionT: <b>copy</b>, drop, store&gt;(proposer_address: <b>address</b>, proposal_id: u64): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="dao_fa.md#0x1_dao_fa_proposal_exists">proposal_exists</a>&lt;CoinT, ActionT: <b>copy</b> + drop + store&gt;(
    proposer_address: <b>address</b>,
    proposal_id: u64,
): bool <b>acquires</b> <a href="dao_fa.md#0x1_dao_fa_Proposal">Proposal</a> {
    <b>if</b> (<b>exists</b>&lt;<a href="dao_fa.md#0x1_dao_fa_Proposal">Proposal</a>&lt;CoinT, ActionT&gt;&gt;(proposer_address)) {
        <b>let</b> proposal = <b>borrow_global</b>&lt;<a href="dao_fa.md#0x1_dao_fa_Proposal">Proposal</a>&lt;CoinT, ActionT&gt;&gt;(proposer_address);
        <b>return</b> proposal.id == proposal_id
    };
    <b>false</b>
}
</code></pre>



</details>

<a id="0x1_dao_fa_proposal_state"></a>

## Function `proposal_state`

Get the proposal state.


<pre><code><b>public</b> <b>fun</b> <a href="dao_fa.md#0x1_dao_fa_proposal_state">proposal_state</a>&lt;CoinT, ActionT: <b>copy</b>, drop, store&gt;(proposer_address: <b>address</b>, proposal_id: u64): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="dao_fa.md#0x1_dao_fa_proposal_state">proposal_state</a>&lt;CoinT, ActionT: <b>copy</b> + drop + store&gt;(
    proposer_address: <b>address</b>,
    proposal_id: u64,
): u8 <b>acquires</b> <a href="dao_fa.md#0x1_dao_fa_Proposal">Proposal</a> {
    <b>let</b> proposal = <b>borrow_global</b>&lt;<a href="dao_fa.md#0x1_dao_fa_Proposal">Proposal</a>&lt;CoinT, ActionT&gt;&gt;(proposer_address);
    <b>assert</b>!(proposal.id == proposal_id, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="dao_fa.md#0x1_dao_fa_ERR_PROPOSAL_ID_MISMATCH">ERR_PROPOSAL_ID_MISMATCH</a>));
    <b>let</b> current_time = <a href="timestamp.md#0x1_timestamp_now_milliseconds">timestamp::now_milliseconds</a>();
    <a href="dao_fa.md#0x1_dao_fa_do_proposal_state">do_proposal_state</a>(proposal, current_time)
}
</code></pre>



</details>

<a id="0x1_dao_fa_do_proposal_state"></a>

## Function `do_proposal_state`



<pre><code><b>fun</b> <a href="dao_fa.md#0x1_dao_fa_do_proposal_state">do_proposal_state</a>&lt;CoinT, ActionT: <b>copy</b>, drop, store&gt;(proposal: &<a href="dao_fa.md#0x1_dao_fa_Proposal">dao_fa::Proposal</a>&lt;CoinT, ActionT&gt;, current_time: u64): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="dao_fa.md#0x1_dao_fa_do_proposal_state">do_proposal_state</a>&lt;CoinT, ActionT: <b>copy</b> + drop + store&gt;(
    proposal: &<a href="dao_fa.md#0x1_dao_fa_Proposal">Proposal</a>&lt;CoinT, ActionT&gt;,
    current_time: u64,
): u8 {
    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"do_proposal_state | entered "));
    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(proposal);

    <b>if</b> (current_time &lt; proposal.start_time) {
        // Pending
        <a href="dao_fa.md#0x1_dao_fa_PENDING">PENDING</a>
    } <b>else</b> <b>if</b> (current_time &lt;= proposal.end_time) {
        // Active
        <a href="dao_fa.md#0x1_dao_fa_ACTIVE">ACTIVE</a>
    } <b>else</b> <b>if</b> (proposal.for_votes &lt;= proposal.against_votes ||
        proposal.for_votes &lt; proposal.quorum_votes) {
        // Defeated
        <a href="dao_fa.md#0x1_dao_fa_DEFEATED">DEFEATED</a>
    } <b>else</b> <b>if</b> (proposal.eta == 0) {
        // Agreed.
        <a href="dao_fa.md#0x1_dao_fa_AGREED">AGREED</a>
    } <b>else</b> <b>if</b> (current_time &lt; proposal.eta) {
        // Queued, waiting <b>to</b> execute
        <a href="dao_fa.md#0x1_dao_fa_QUEUED">QUEUED</a>
    } <b>else</b> <b>if</b> (<a href="../../move-stdlib/doc/option.md#0x1_option_is_some">option::is_some</a>(&proposal.action)) {
        <a href="dao_fa.md#0x1_dao_fa_EXECUTABLE">EXECUTABLE</a>
    } <b>else</b> {
        <a href="dao_fa.md#0x1_dao_fa_EXTRACTED">EXTRACTED</a>
    }
}
</code></pre>



</details>

<a id="0x1_dao_fa_proposal_info"></a>

## Function `proposal_info`

get proposal's information.
return: (id, start_time, end_time, for_votes, against_votes).


<pre><code><b>public</b> <b>fun</b> <a href="dao_fa.md#0x1_dao_fa_proposal_info">proposal_info</a>&lt;CoinT, ActionT: <b>copy</b>, drop, store&gt;(proposer_address: <b>address</b>): (u64, u64, u64, u128, u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="dao_fa.md#0x1_dao_fa_proposal_info">proposal_info</a>&lt;CoinT, ActionT: <b>copy</b> + drop + store&gt;(
    proposer_address: <b>address</b>,
): (u64, u64, u64, u128, u128) <b>acquires</b> <a href="dao_fa.md#0x1_dao_fa_Proposal">Proposal</a> {
    <b>let</b> proposal = <b>borrow_global</b>&lt;<a href="dao_fa.md#0x1_dao_fa_Proposal">Proposal</a>&lt;CoinT, ActionT&gt;&gt;(proposer_address);
    (proposal.id, proposal.start_time, proposal.end_time, proposal.for_votes, proposal.against_votes)
}
</code></pre>



</details>

<a id="0x1_dao_fa_vote_of"></a>

## Function `vote_of`

Get voter's vote info on proposal with <code>proposal_id</code> of <code>proposer_address</code>.


<pre><code><b>public</b> <b>fun</b> <a href="dao_fa.md#0x1_dao_fa_vote_of">vote_of</a>&lt;CoinT&gt;(voter: <b>address</b>, proposer_address: <b>address</b>, proposal_id: u64): (bool, u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="dao_fa.md#0x1_dao_fa_vote_of">vote_of</a>&lt;CoinT&gt;(
    voter: <b>address</b>,
    proposer_address: <b>address</b>,
    proposal_id: u64,
): (bool, u128) <b>acquires</b> <a href="dao_fa.md#0x1_dao_fa_Vote">Vote</a> {
    <b>let</b> vote = <b>borrow_global</b>&lt;<a href="dao_fa.md#0x1_dao_fa_Vote">Vote</a>&lt;CoinT&gt;&gt;(voter);
    <b>assert</b>!(vote.proposer == proposer_address, <a href="../../move-stdlib/doc/error.md#0x1_error_not_found">error::not_found</a>(<a href="dao_fa.md#0x1_dao_fa_ERR_PROPOSER_MISMATCH">ERR_PROPOSER_MISMATCH</a>));
    <b>assert</b>!(vote.id == proposal_id, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="dao_fa.md#0x1_dao_fa_ERR_VOTED_OTHERS_ALREADY">ERR_VOTED_OTHERS_ALREADY</a>));
    (vote.agree, (<a href="fungible_asset.md#0x1_fungible_asset_balance">fungible_asset::balance</a>(vote.stake_store) <b>as</b> u128))
}
</code></pre>



</details>

<a id="0x1_dao_fa_has_vote"></a>

## Function `has_vote`

Check whether voter has voted on proposal with <code>proposal_id</code> of <code>proposer_address</code>.


<pre><code><b>public</b> <b>fun</b> <a href="dao_fa.md#0x1_dao_fa_has_vote">has_vote</a>&lt;CoinT&gt;(voter: <b>address</b>, proposer_address: <b>address</b>, proposal_id: u64): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="dao_fa.md#0x1_dao_fa_has_vote">has_vote</a>&lt;CoinT&gt;(
    voter: <b>address</b>,
    proposer_address: <b>address</b>,
    proposal_id: u64,
): bool <b>acquires</b> <a href="dao_fa.md#0x1_dao_fa_Vote">Vote</a> {
    <b>if</b> (!<b>exists</b>&lt;<a href="dao_fa.md#0x1_dao_fa_Vote">Vote</a>&lt;CoinT&gt;&gt;(voter)) {
        <b>return</b> <b>false</b>
    };

    <b>let</b> vote = <b>borrow_global</b>&lt;<a href="dao_fa.md#0x1_dao_fa_Vote">Vote</a>&lt;CoinT&gt;&gt;(voter);
    vote.proposer == proposer_address && vote.id == proposal_id
}
</code></pre>



</details>

<a id="0x1_dao_fa_generate_next_proposal_id"></a>

## Function `generate_next_proposal_id`



<pre><code><b>fun</b> <a href="dao_fa.md#0x1_dao_fa_generate_next_proposal_id">generate_next_proposal_id</a>&lt;CoinT&gt;(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="dao_fa.md#0x1_dao_fa_generate_next_proposal_id">generate_next_proposal_id</a>&lt;CoinT&gt;(): u64 <b>acquires</b> <a href="dao_fa.md#0x1_dao_fa_DaoGlobalInfo">DaoGlobalInfo</a> {
    <b>let</b> gov_info = <b>borrow_global_mut</b>&lt;<a href="dao_fa.md#0x1_dao_fa_DaoGlobalInfo">DaoGlobalInfo</a>&lt;CoinT&gt;&gt;(<a href="stc_util.md#0x1_stc_util_token_issuer">stc_util::token_issuer</a>&lt;CoinT&gt;());
    <b>let</b> proposal_id = gov_info.next_proposal_id;
    gov_info.next_proposal_id = proposal_id + 1;
    proposal_id
}
</code></pre>



</details>

<a id="0x1_dao_fa_voting_delay"></a>

## Function `voting_delay`

get default voting delay of the DAO.


<pre><code><b>public</b> <b>fun</b> <a href="dao_fa.md#0x1_dao_fa_voting_delay">voting_delay</a>&lt;CoinT&gt;(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="dao_fa.md#0x1_dao_fa_voting_delay">voting_delay</a>&lt;CoinT&gt;(): u64 {
    <a href="dao_fa.md#0x1_dao_fa_get_config">get_config</a>&lt;CoinT&gt;().voting_delay
}
</code></pre>



</details>

<a id="0x1_dao_fa_voting_period"></a>

## Function `voting_period`

get the default voting period of the DAO.


<pre><code><b>public</b> <b>fun</b> <a href="dao_fa.md#0x1_dao_fa_voting_period">voting_period</a>&lt;CoinT&gt;(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="dao_fa.md#0x1_dao_fa_voting_period">voting_period</a>&lt;CoinT&gt;(): u64 {
    <a href="dao_fa.md#0x1_dao_fa_get_config">get_config</a>&lt;CoinT&gt;().voting_period
}
</code></pre>



</details>

<a id="0x1_dao_fa_coin_to_fa_metadata"></a>

## Function `coin_to_fa_metadata`



<pre><code><b>fun</b> <a href="dao_fa.md#0x1_dao_fa_coin_to_fa_metadata">coin_to_fa_metadata</a>&lt;CoinT&gt;(): <a href="object.md#0x1_object_Object">object::Object</a>&lt;<a href="fungible_asset.md#0x1_fungible_asset_Metadata">fungible_asset::Metadata</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="dao_fa.md#0x1_dao_fa_coin_to_fa_metadata">coin_to_fa_metadata</a>&lt;CoinT&gt;(): Object&lt;Metadata&gt; {
    <b>let</b> coin_metadata = <a href="coin.md#0x1_coin_paired_metadata">coin::paired_metadata</a>&lt;CoinT&gt;();
    <b>assert</b>!(<a href="../../move-stdlib/doc/option.md#0x1_option_is_some">option::is_some</a>(&coin_metadata), <a href="../../move-stdlib/doc/error.md#0x1_error_not_found">error::not_found</a>(<a href="dao_fa.md#0x1_dao_fa_ERR_COIN_NOT_FOUND">ERR_COIN_NOT_FOUND</a>));
    <a href="../../move-stdlib/doc/option.md#0x1_option_destroy_some">option::destroy_some</a>(coin_metadata)
}
</code></pre>



</details>

<a id="0x1_dao_fa_quorum_votes"></a>

## Function `quorum_votes`

Quorum votes to make proposal pass.


<pre><code><b>public</b> <b>fun</b> <a href="dao_fa.md#0x1_dao_fa_quorum_votes">quorum_votes</a>&lt;CoinT&gt;(): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="dao_fa.md#0x1_dao_fa_quorum_votes">quorum_votes</a>&lt;CoinT&gt;(): u128 {
    <b>let</b> supply = <a href="fungible_asset.md#0x1_fungible_asset_supply">fungible_asset::supply</a>(<a href="dao_fa.md#0x1_dao_fa_coin_to_fa_metadata">Self::coin_to_fa_metadata</a>&lt;CoinT&gt;());
    <b>assert</b>!(<a href="../../move-stdlib/doc/option.md#0x1_option_is_some">option::is_some</a>(&supply), <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_state">error::invalid_state</a>(<a href="dao_fa.md#0x1_dao_fa_ERR_TOKEN_NOT_REGISTER">ERR_TOKEN_NOT_REGISTER</a>));

    <b>let</b> market_cap = <a href="../../move-stdlib/doc/option.md#0x1_option_destroy_some">option::destroy_some</a>(supply);
    <b>let</b> balance_in_treasury = <a href="treasury_fa.md#0x1_treasury_fa_balance">treasury_fa::balance</a>&lt;CoinT&gt;(get_starcoin_framework());
    <b>let</b> supply = market_cap - balance_in_treasury;
    <b>let</b> rate = <a href="dao_fa.md#0x1_dao_fa_voting_quorum_rate">voting_quorum_rate</a>&lt;CoinT&gt;();
    <b>let</b> rate = (rate <b>as</b> u128);

    supply * rate / 100
}
</code></pre>



</details>

<a id="0x1_dao_fa_voting_quorum_rate"></a>

## Function `voting_quorum_rate`

Get the quorum rate in percent.


<pre><code><b>public</b> <b>fun</b> <a href="dao_fa.md#0x1_dao_fa_voting_quorum_rate">voting_quorum_rate</a>&lt;CoinT&gt;(): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="dao_fa.md#0x1_dao_fa_voting_quorum_rate">voting_quorum_rate</a>&lt;CoinT&gt;(): u8 {
    <a href="dao_fa.md#0x1_dao_fa_get_config">get_config</a>&lt;CoinT&gt;().voting_quorum_rate
}
</code></pre>



</details>

<a id="0x1_dao_fa_min_action_delay"></a>

## Function `min_action_delay`

Get the min_action_delay of the DAO.


<pre><code><b>public</b> <b>fun</b> <a href="dao_fa.md#0x1_dao_fa_min_action_delay">min_action_delay</a>&lt;CoinT&gt;(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="dao_fa.md#0x1_dao_fa_min_action_delay">min_action_delay</a>&lt;CoinT&gt;(): u64 {
    <a href="dao_fa.md#0x1_dao_fa_get_config">get_config</a>&lt;CoinT&gt;().min_action_delay
}
</code></pre>



</details>

<a id="0x1_dao_fa_get_config"></a>

## Function `get_config`



<pre><code><b>fun</b> <a href="dao_fa.md#0x1_dao_fa_get_config">get_config</a>&lt;CoinT&gt;(): <a href="dao_fa.md#0x1_dao_fa_DaoConfig">dao_fa::DaoConfig</a>&lt;CoinT&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="dao_fa.md#0x1_dao_fa_get_config">get_config</a>&lt;CoinT&gt;(): <a href="dao_fa.md#0x1_dao_fa_DaoConfig">DaoConfig</a>&lt;CoinT&gt; {
    <b>let</b> token_issuer = <a href="stc_util.md#0x1_stc_util_token_issuer">stc_util::token_issuer</a>&lt;CoinT&gt;();
    <a href="on_chain_config.md#0x1_on_chain_config_get_by_address">on_chain_config::get_by_address</a>&lt;<a href="dao_fa.md#0x1_dao_fa_DaoConfig">DaoConfig</a>&lt;CoinT&gt;&gt;(token_issuer)
}
</code></pre>



</details>

<a id="0x1_dao_fa_modify_dao_config"></a>

## Function `modify_dao_config`

update function, modify dao config.
if any param is 0, it means no change to that param.


<pre><code><b>public</b> <b>fun</b> <a href="dao_fa.md#0x1_dao_fa_modify_dao_config">modify_dao_config</a>&lt;CoinT&gt;(cap: &<b>mut</b> <a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapability">on_chain_config::ModifyConfigCapability</a>&lt;<a href="dao_fa.md#0x1_dao_fa_DaoConfig">dao_fa::DaoConfig</a>&lt;CoinT&gt;&gt;, voting_delay: u64, voting_period: u64, voting_quorum_rate: u8, min_action_delay: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="dao_fa.md#0x1_dao_fa_modify_dao_config">modify_dao_config</a>&lt;CoinT&gt;(
    cap: &<b>mut</b> <a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapability">on_chain_config::ModifyConfigCapability</a>&lt;<a href="dao_fa.md#0x1_dao_fa_DaoConfig">DaoConfig</a>&lt;CoinT&gt;&gt;,
    voting_delay: u64,
    voting_period: u64,
    voting_quorum_rate: u8,
    min_action_delay: u64,
) {
    <b>assert</b>!(
        <a href="on_chain_config.md#0x1_on_chain_config_account_address">on_chain_config::account_address</a>(cap) == <a href="stc_util.md#0x1_stc_util_token_issuer">stc_util::token_issuer</a>&lt;CoinT&gt;(),
        <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="dao_fa.md#0x1_dao_fa_ERR_NOT_AUTHORIZED">ERR_NOT_AUTHORIZED</a>)
    );
    <b>let</b> config = <a href="dao_fa.md#0x1_dao_fa_get_config">get_config</a>&lt;CoinT&gt;();
    <b>if</b> (voting_period &gt; 0) {
        config.voting_period = voting_period;
    };
    <b>if</b> (voting_delay &gt; 0) {
        config.voting_delay = voting_delay;
    };
    <b>if</b> (voting_quorum_rate &gt; 0) {
        <b>assert</b>!(<a href="dao_fa.md#0x1_dao_fa_voting_quorum_rate">voting_quorum_rate</a> &lt;= 100, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="dao_fa.md#0x1_dao_fa_ERR_QUORUM_RATE_INVALID">ERR_QUORUM_RATE_INVALID</a>));
        config.voting_quorum_rate = voting_quorum_rate;
    };
    <b>if</b> (min_action_delay &gt; 0) {
        config.min_action_delay = min_action_delay;
    };
    <a href="on_chain_config.md#0x1_on_chain_config_set_with_capability">on_chain_config::set_with_capability</a>&lt;<a href="dao_fa.md#0x1_dao_fa_DaoConfig">DaoConfig</a>&lt;CoinT&gt;&gt;(cap, config);
}
</code></pre>



</details>

<a id="0x1_dao_fa_set_voting_delay"></a>

## Function `set_voting_delay`

set voting delay


<pre><code><b>public</b> <b>fun</b> <a href="dao_fa.md#0x1_dao_fa_set_voting_delay">set_voting_delay</a>&lt;CoinT&gt;(cap: &<b>mut</b> <a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapability">on_chain_config::ModifyConfigCapability</a>&lt;<a href="dao_fa.md#0x1_dao_fa_DaoConfig">dao_fa::DaoConfig</a>&lt;CoinT&gt;&gt;, value: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="dao_fa.md#0x1_dao_fa_set_voting_delay">set_voting_delay</a>&lt;CoinT&gt;(
    cap: &<b>mut</b> <a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapability">on_chain_config::ModifyConfigCapability</a>&lt;<a href="dao_fa.md#0x1_dao_fa_DaoConfig">DaoConfig</a>&lt;CoinT&gt;&gt;,
    value: u64,
) {
    <b>assert</b>!(
        <a href="on_chain_config.md#0x1_on_chain_config_account_address">on_chain_config::account_address</a>(cap) == <a href="stc_util.md#0x1_stc_util_token_issuer">stc_util::token_issuer</a>&lt;CoinT&gt;(),
        <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="dao_fa.md#0x1_dao_fa_ERR_NOT_AUTHORIZED">ERR_NOT_AUTHORIZED</a>)
    );
    <b>assert</b>!(value &gt; 0, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="dao_fa.md#0x1_dao_fa_ERR_CONFIG_PARAM_INVALID">ERR_CONFIG_PARAM_INVALID</a>));
    <b>let</b> config = <a href="dao_fa.md#0x1_dao_fa_get_config">get_config</a>&lt;CoinT&gt;();
    config.voting_delay = value;
    <a href="on_chain_config.md#0x1_on_chain_config_set_with_capability">on_chain_config::set_with_capability</a>&lt;<a href="dao_fa.md#0x1_dao_fa_DaoConfig">DaoConfig</a>&lt;CoinT&gt;&gt;(cap, config);
}
</code></pre>



</details>

<a id="0x1_dao_fa_set_voting_period"></a>

## Function `set_voting_period`

set voting period


<pre><code><b>public</b> <b>fun</b> <a href="dao_fa.md#0x1_dao_fa_set_voting_period">set_voting_period</a>&lt;CoinT&gt;(cap: &<b>mut</b> <a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapability">on_chain_config::ModifyConfigCapability</a>&lt;<a href="dao_fa.md#0x1_dao_fa_DaoConfig">dao_fa::DaoConfig</a>&lt;CoinT&gt;&gt;, value: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="dao_fa.md#0x1_dao_fa_set_voting_period">set_voting_period</a>&lt;CoinT&gt;(
    cap: &<b>mut</b> <a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapability">on_chain_config::ModifyConfigCapability</a>&lt;<a href="dao_fa.md#0x1_dao_fa_DaoConfig">DaoConfig</a>&lt;CoinT&gt;&gt;,
    value: u64,
) {
    <b>assert</b>!(
        <a href="on_chain_config.md#0x1_on_chain_config_account_address">on_chain_config::account_address</a>(cap) == <a href="stc_util.md#0x1_stc_util_token_issuer">stc_util::token_issuer</a>&lt;CoinT&gt;(),
        <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="dao_fa.md#0x1_dao_fa_ERR_NOT_AUTHORIZED">ERR_NOT_AUTHORIZED</a>)
    );
    <b>assert</b>!(value &gt; 0, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="dao_fa.md#0x1_dao_fa_ERR_CONFIG_PARAM_INVALID">ERR_CONFIG_PARAM_INVALID</a>));
    <b>let</b> config = <a href="dao_fa.md#0x1_dao_fa_get_config">get_config</a>&lt;CoinT&gt;();
    config.voting_period = value;
    <a href="on_chain_config.md#0x1_on_chain_config_set_with_capability">on_chain_config::set_with_capability</a>&lt;<a href="dao_fa.md#0x1_dao_fa_DaoConfig">DaoConfig</a>&lt;CoinT&gt;&gt;(cap, config);
}
</code></pre>



</details>

<a id="0x1_dao_fa_set_voting_quorum_rate"></a>

## Function `set_voting_quorum_rate`

set voting quorum rate


<pre><code><b>public</b> <b>fun</b> <a href="dao_fa.md#0x1_dao_fa_set_voting_quorum_rate">set_voting_quorum_rate</a>&lt;CoinT&gt;(cap: &<b>mut</b> <a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapability">on_chain_config::ModifyConfigCapability</a>&lt;<a href="dao_fa.md#0x1_dao_fa_DaoConfig">dao_fa::DaoConfig</a>&lt;CoinT&gt;&gt;, value: u8)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="dao_fa.md#0x1_dao_fa_set_voting_quorum_rate">set_voting_quorum_rate</a>&lt;CoinT&gt;(
    cap: &<b>mut</b> <a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapability">on_chain_config::ModifyConfigCapability</a>&lt;<a href="dao_fa.md#0x1_dao_fa_DaoConfig">DaoConfig</a>&lt;CoinT&gt;&gt;,
    value: u8,
) {
    <b>assert</b>!(
        <a href="on_chain_config.md#0x1_on_chain_config_account_address">on_chain_config::account_address</a>(cap) == <a href="stc_util.md#0x1_stc_util_token_issuer">stc_util::token_issuer</a>&lt;CoinT&gt;(),
        <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="dao_fa.md#0x1_dao_fa_ERR_NOT_AUTHORIZED">ERR_NOT_AUTHORIZED</a>)
    );
    <b>assert</b>!(value &lt;= 100 && value &gt; 0, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="dao_fa.md#0x1_dao_fa_ERR_QUORUM_RATE_INVALID">ERR_QUORUM_RATE_INVALID</a>));
    <b>let</b> config = <a href="dao_fa.md#0x1_dao_fa_get_config">get_config</a>&lt;CoinT&gt;();
    config.voting_quorum_rate = value;
    <a href="on_chain_config.md#0x1_on_chain_config_set_with_capability">on_chain_config::set_with_capability</a>&lt;<a href="dao_fa.md#0x1_dao_fa_DaoConfig">DaoConfig</a>&lt;CoinT&gt;&gt;(cap, config);
}
</code></pre>



</details>

<a id="0x1_dao_fa_set_min_action_delay"></a>

## Function `set_min_action_delay`

set min action delay


<pre><code><b>public</b> <b>fun</b> <a href="dao_fa.md#0x1_dao_fa_set_min_action_delay">set_min_action_delay</a>&lt;CoinT&gt;(cap: &<b>mut</b> <a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapability">on_chain_config::ModifyConfigCapability</a>&lt;<a href="dao_fa.md#0x1_dao_fa_DaoConfig">dao_fa::DaoConfig</a>&lt;CoinT&gt;&gt;, value: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="dao_fa.md#0x1_dao_fa_set_min_action_delay">set_min_action_delay</a>&lt;CoinT&gt;(
    cap: &<b>mut</b> <a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapability">on_chain_config::ModifyConfigCapability</a>&lt;<a href="dao_fa.md#0x1_dao_fa_DaoConfig">DaoConfig</a>&lt;CoinT&gt;&gt;,
    value: u64,
) {
    <b>assert</b>!(
        <a href="on_chain_config.md#0x1_on_chain_config_account_address">on_chain_config::account_address</a>(cap) == <a href="stc_util.md#0x1_stc_util_token_issuer">stc_util::token_issuer</a>&lt;CoinT&gt;(),
        <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="dao_fa.md#0x1_dao_fa_ERR_NOT_AUTHORIZED">ERR_NOT_AUTHORIZED</a>)
    );
    <b>assert</b>!(value &gt; 0, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="dao_fa.md#0x1_dao_fa_ERR_CONFIG_PARAM_INVALID">ERR_CONFIG_PARAM_INVALID</a>));
    <b>let</b> config = <a href="dao_fa.md#0x1_dao_fa_get_config">get_config</a>&lt;CoinT&gt;();
    config.min_action_delay = value;
    <a href="on_chain_config.md#0x1_on_chain_config_set_with_capability">on_chain_config::set_with_capability</a>&lt;<a href="dao_fa.md#0x1_dao_fa_DaoConfig">DaoConfig</a>&lt;CoinT&gt;&gt;(cap, config);
}
</code></pre>



</details>


[move-book]: https://starcoin.dev/move/book/SUMMARY
