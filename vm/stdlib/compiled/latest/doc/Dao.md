
<a name="0x1_Dao"></a>

# Module `0x1::Dao`



-  [Resource `DaoGlobalInfo`](#0x1_Dao_DaoGlobalInfo)
-  [Struct `DaoConfig`](#0x1_Dao_DaoConfig)
-  [Struct `ProposalCreatedEvent`](#0x1_Dao_ProposalCreatedEvent)
-  [Struct `VoteChangedEvent`](#0x1_Dao_VoteChangedEvent)
-  [Resource `Proposal`](#0x1_Dao_Proposal)
-  [Resource `Vote`](#0x1_Dao_Vote)
-  [Constants](#@Constants_0)
-  [Function `plugin`](#0x1_Dao_plugin)
-  [Function `new_dao_config`](#0x1_Dao_new_dao_config)
-  [Function `propose`](#0x1_Dao_propose)
-  [Function `cast_vote`](#0x1_Dao_cast_vote)
-  [Function `do_cast_vote`](#0x1_Dao_do_cast_vote)
-  [Function `change_vote`](#0x1_Dao_change_vote)
-  [Function `do_flip_vote`](#0x1_Dao_do_flip_vote)
-  [Function `revoke_vote`](#0x1_Dao_revoke_vote)
-  [Function `do_revoke_vote`](#0x1_Dao_do_revoke_vote)
-  [Function `unstake_votes`](#0x1_Dao_unstake_votes)
-  [Function `queue_proposal_action`](#0x1_Dao_queue_proposal_action)
-  [Function `extract_proposal_action`](#0x1_Dao_extract_proposal_action)
-  [Function `destroy_terminated_proposal`](#0x1_Dao_destroy_terminated_proposal)
-  [Function `proposal_exists`](#0x1_Dao_proposal_exists)
-  [Function `proposal_state`](#0x1_Dao_proposal_state)
-  [Function `do_proposal_state`](#0x1_Dao_do_proposal_state)
-  [Function `proposal_info`](#0x1_Dao_proposal_info)
-  [Function `vote_of`](#0x1_Dao_vote_of)
-  [Function `has_vote`](#0x1_Dao_has_vote)
-  [Function `generate_next_proposal_id`](#0x1_Dao_generate_next_proposal_id)
-  [Function `voting_delay`](#0x1_Dao_voting_delay)
-  [Function `voting_period`](#0x1_Dao_voting_period)
-  [Function `quorum_votes`](#0x1_Dao_quorum_votes)
-  [Function `voting_quorum_rate`](#0x1_Dao_voting_quorum_rate)
-  [Function `min_action_delay`](#0x1_Dao_min_action_delay)
-  [Function `get_config`](#0x1_Dao_get_config)
-  [Function `modify_dao_config`](#0x1_Dao_modify_dao_config)
-  [Function `set_voting_delay`](#0x1_Dao_set_voting_delay)
-  [Function `set_voting_period`](#0x1_Dao_set_voting_period)
-  [Function `set_voting_quorum_rate`](#0x1_Dao_set_voting_quorum_rate)
-  [Function `set_min_action_delay`](#0x1_Dao_set_min_action_delay)
-  [Specification](#@Specification_1)
    -  [Struct `DaoConfig`](#@Specification_1_DaoConfig)
    -  [Function `plugin`](#@Specification_1_plugin)
    -  [Function `new_dao_config`](#@Specification_1_new_dao_config)
    -  [Function `propose`](#@Specification_1_propose)
    -  [Function `cast_vote`](#@Specification_1_cast_vote)
    -  [Function `do_cast_vote`](#@Specification_1_do_cast_vote)
    -  [Function `change_vote`](#@Specification_1_change_vote)
    -  [Function `do_flip_vote`](#@Specification_1_do_flip_vote)
    -  [Function `revoke_vote`](#@Specification_1_revoke_vote)
    -  [Function `do_revoke_vote`](#@Specification_1_do_revoke_vote)
    -  [Function `unstake_votes`](#@Specification_1_unstake_votes)
    -  [Function `queue_proposal_action`](#@Specification_1_queue_proposal_action)
    -  [Function `extract_proposal_action`](#@Specification_1_extract_proposal_action)
    -  [Function `destroy_terminated_proposal`](#@Specification_1_destroy_terminated_proposal)
    -  [Function `proposal_exists`](#@Specification_1_proposal_exists)
    -  [Function `proposal_state`](#@Specification_1_proposal_state)
    -  [Function `proposal_info`](#@Specification_1_proposal_info)
    -  [Function `vote_of`](#@Specification_1_vote_of)
    -  [Function `generate_next_proposal_id`](#@Specification_1_generate_next_proposal_id)
    -  [Function `voting_delay`](#@Specification_1_voting_delay)
    -  [Function `voting_period`](#@Specification_1_voting_period)
    -  [Function `quorum_votes`](#@Specification_1_quorum_votes)
    -  [Function `voting_quorum_rate`](#@Specification_1_voting_quorum_rate)
    -  [Function `min_action_delay`](#@Specification_1_min_action_delay)
    -  [Function `get_config`](#@Specification_1_get_config)
    -  [Function `modify_dao_config`](#@Specification_1_modify_dao_config)
    -  [Function `set_voting_delay`](#@Specification_1_set_voting_delay)
    -  [Function `set_voting_period`](#@Specification_1_set_voting_period)
    -  [Function `set_voting_quorum_rate`](#@Specification_1_set_voting_quorum_rate)
    -  [Function `set_min_action_delay`](#@Specification_1_set_min_action_delay)


<pre><code><b>use</b> <a href="Config.md#0x1_Config">0x1::Config</a>;
<b>use</b> <a href="Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="Event.md#0x1_Event">0x1::Event</a>;
<b>use</b> <a href="Option.md#0x1_Option">0x1::Option</a>;
<b>use</b> <a href="Signer.md#0x1_Signer">0x1::Signer</a>;
<b>use</b> <a href="Timestamp.md#0x1_Timestamp">0x1::Timestamp</a>;
<b>use</b> <a href="Token.md#0x1_Token">0x1::Token</a>;
<b>use</b> <a href="Treasury.md#0x1_Treasury">0x1::Treasury</a>;
</code></pre>



<a name="0x1_Dao_DaoGlobalInfo"></a>

## Resource `DaoGlobalInfo`

global DAO info of the specified token type <code><a href="Token.md#0x1_Token">Token</a></code>.


<pre><code><b>struct</b> <a href="Dao.md#0x1_Dao_DaoGlobalInfo">DaoGlobalInfo</a>&lt;<a href="Token.md#0x1_Token">Token</a>: store&gt; <b>has</b> key
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
<code>proposal_create_event: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="Dao.md#0x1_Dao_ProposalCreatedEvent">Dao::ProposalCreatedEvent</a>&gt;</code>
</dt>
<dd>
 proposal creating event.
</dd>
<dt>
<code>vote_changed_event: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="Dao.md#0x1_Dao_VoteChangedEvent">Dao::VoteChangedEvent</a>&gt;</code>
</dt>
<dd>
 voting event.
</dd>
</dl>


</details>

<a name="0x1_Dao_DaoConfig"></a>

## Struct `DaoConfig`

Configuration of the <code><a href="Token.md#0x1_Token">Token</a></code>'s DAO.


<pre><code><b>struct</b> <a href="Dao.md#0x1_Dao_DaoConfig">DaoConfig</a>&lt;TokenT: <b>copy</b>, drop, store&gt; <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>voting_delay: u64</code>
</dt>
<dd>
 after proposal created, how long use should wait before he can vote.
</dd>
<dt>
<code>voting_period: u64</code>
</dt>
<dd>
 how long the voting window is.
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
 how long the proposal should wait before it can be executed.
</dd>
</dl>


</details>

<a name="0x1_Dao_ProposalCreatedEvent"></a>

## Struct `ProposalCreatedEvent`

emitted when proposal created.


<pre><code><b>struct</b> <a href="Dao.md#0x1_Dao_ProposalCreatedEvent">ProposalCreatedEvent</a> <b>has</b> drop, store
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

<a name="0x1_Dao_VoteChangedEvent"></a>

## Struct `VoteChangedEvent`

emitted when user vote/revoke_vote.


<pre><code><b>struct</b> <a href="Dao.md#0x1_Dao_VoteChangedEvent">VoteChangedEvent</a> <b>has</b> drop, store
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
 agree or againest.
</dd>
<dt>
<code>vote: u128</code>
</dt>
<dd>
 latest vote count of the voter.
</dd>
</dl>


</details>

<a name="0x1_Dao_Proposal"></a>

## Resource `Proposal`

Proposal data struct.


<pre><code><b>struct</b> <a href="Dao.md#0x1_Dao_Proposal">Proposal</a>&lt;<a href="Token.md#0x1_Token">Token</a>: store, Action: store&gt; <b>has</b> key
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
 count of votes for agree.
</dd>
<dt>
<code>against_votes: u128</code>
</dt>
<dd>
 count of votes for againest.
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
<code>action: <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;Action&gt;</code>
</dt>
<dd>
 proposal action.
</dd>
</dl>


</details>

<a name="0x1_Dao_Vote"></a>

## Resource `Vote`

User vote info.


<pre><code><b>struct</b> <a href="Dao.md#0x1_Dao_Vote">Vote</a>&lt;TokenT: store&gt; <b>has</b> key
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
<code>stake: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenT&gt;</code>
</dt>
<dd>
 how many tokens to stake.
</dd>
<dt>
<code>agree: bool</code>
</dt>
<dd>
 vote for or vote against.
</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_Dao_ERR_NOT_AUTHORIZED"></a>



<pre><code><b>const</b> <a href="Dao.md#0x1_Dao_ERR_NOT_AUTHORIZED">ERR_NOT_AUTHORIZED</a>: u64 = 1401;
</code></pre>



<a name="0x1_Dao_ACTIVE"></a>



<pre><code><b>const</b> <a href="Dao.md#0x1_Dao_ACTIVE">ACTIVE</a>: u8 = 2;
</code></pre>



<a name="0x1_Dao_AGREED"></a>



<pre><code><b>const</b> <a href="Dao.md#0x1_Dao_AGREED">AGREED</a>: u8 = 4;
</code></pre>



<a name="0x1_Dao_DEFEATED"></a>



<pre><code><b>const</b> <a href="Dao.md#0x1_Dao_DEFEATED">DEFEATED</a>: u8 = 3;
</code></pre>



<a name="0x1_Dao_ERR_ACTION_DELAY_TOO_SMALL"></a>



<pre><code><b>const</b> <a href="Dao.md#0x1_Dao_ERR_ACTION_DELAY_TOO_SMALL">ERR_ACTION_DELAY_TOO_SMALL</a>: u64 = 1402;
</code></pre>



<a name="0x1_Dao_ERR_ACTION_MUST_EXIST"></a>



<pre><code><b>const</b> <a href="Dao.md#0x1_Dao_ERR_ACTION_MUST_EXIST">ERR_ACTION_MUST_EXIST</a>: u64 = 1409;
</code></pre>



<a name="0x1_Dao_ERR_CONFIG_PARAM_INVALID"></a>



<pre><code><b>const</b> <a href="Dao.md#0x1_Dao_ERR_CONFIG_PARAM_INVALID">ERR_CONFIG_PARAM_INVALID</a>: u64 = 1407;
</code></pre>



<a name="0x1_Dao_ERR_PROPOSAL_ID_MISMATCH"></a>



<pre><code><b>const</b> <a href="Dao.md#0x1_Dao_ERR_PROPOSAL_ID_MISMATCH">ERR_PROPOSAL_ID_MISMATCH</a>: u64 = 1404;
</code></pre>



<a name="0x1_Dao_ERR_PROPOSAL_STATE_INVALID"></a>



<pre><code><b>const</b> <a href="Dao.md#0x1_Dao_ERR_PROPOSAL_STATE_INVALID">ERR_PROPOSAL_STATE_INVALID</a>: u64 = 1403;
</code></pre>



<a name="0x1_Dao_ERR_PROPOSER_MISMATCH"></a>



<pre><code><b>const</b> <a href="Dao.md#0x1_Dao_ERR_PROPOSER_MISMATCH">ERR_PROPOSER_MISMATCH</a>: u64 = 1405;
</code></pre>



<a name="0x1_Dao_ERR_QUORUM_RATE_INVALID"></a>



<pre><code><b>const</b> <a href="Dao.md#0x1_Dao_ERR_QUORUM_RATE_INVALID">ERR_QUORUM_RATE_INVALID</a>: u64 = 1406;
</code></pre>



<a name="0x1_Dao_ERR_VOTED_OTHERS_ALREADY"></a>



<pre><code><b>const</b> <a href="Dao.md#0x1_Dao_ERR_VOTED_OTHERS_ALREADY">ERR_VOTED_OTHERS_ALREADY</a>: u64 = 1410;
</code></pre>



<a name="0x1_Dao_ERR_VOTE_STATE_MISMATCH"></a>



<pre><code><b>const</b> <a href="Dao.md#0x1_Dao_ERR_VOTE_STATE_MISMATCH">ERR_VOTE_STATE_MISMATCH</a>: u64 = 1408;
</code></pre>



<a name="0x1_Dao_EXECUTABLE"></a>



<pre><code><b>const</b> <a href="Dao.md#0x1_Dao_EXECUTABLE">EXECUTABLE</a>: u8 = 6;
</code></pre>



<a name="0x1_Dao_EXTRACTED"></a>



<pre><code><b>const</b> <a href="Dao.md#0x1_Dao_EXTRACTED">EXTRACTED</a>: u8 = 7;
</code></pre>



<a name="0x1_Dao_PENDING"></a>

Proposal state


<pre><code><b>const</b> <a href="Dao.md#0x1_Dao_PENDING">PENDING</a>: u8 = 1;
</code></pre>



<a name="0x1_Dao_QUEUED"></a>



<pre><code><b>const</b> <a href="Dao.md#0x1_Dao_QUEUED">QUEUED</a>: u8 = 5;
</code></pre>



<a name="0x1_Dao_plugin"></a>

## Function `plugin`

plugin function, can only be called by token issuer.
Any token who wants to has gov functionality
can optin this module by call this <code>register function</code>.


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_plugin">plugin</a>&lt;TokenT: <b>copy</b>, drop, store&gt;(signer: &signer, voting_delay: u64, voting_period: u64, voting_quorum_rate: u8, min_action_delay: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_plugin">plugin</a>&lt;TokenT: <b>copy</b> + drop + store&gt;(
    signer: &signer,
    voting_delay: u64,
    voting_period: u64,
    voting_quorum_rate: u8,
    min_action_delay: u64,
) {
    <b>let</b> token_issuer = <a href="Token.md#0x1_Token_token_address">Token::token_address</a>&lt;TokenT&gt;();
    <b>assert</b>!(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer) == token_issuer, <a href="Errors.md#0x1_Errors_requires_address">Errors::requires_address</a>(<a href="Dao.md#0x1_Dao_ERR_NOT_AUTHORIZED">ERR_NOT_AUTHORIZED</a>));
    // <b>let</b> proposal_id = ProposalId {next: 0};
    <b>let</b> gov_info = <a href="Dao.md#0x1_Dao_DaoGlobalInfo">DaoGlobalInfo</a>&lt;TokenT&gt; {
        next_proposal_id: 0,
        proposal_create_event: <a href="Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="Dao.md#0x1_Dao_ProposalCreatedEvent">ProposalCreatedEvent</a>&gt;(signer),
        vote_changed_event: <a href="Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="Dao.md#0x1_Dao_VoteChangedEvent">VoteChangedEvent</a>&gt;(signer),
    };
    <b>move_to</b>(signer, gov_info);
    <b>let</b> config = <a href="Dao.md#0x1_Dao_new_dao_config">new_dao_config</a>&lt;TokenT&gt;(
        voting_delay,
        voting_period,
        voting_quorum_rate,
        min_action_delay,
    );
    <a href="Config.md#0x1_Config_publish_new_config">Config::publish_new_config</a>(signer, config);
}
</code></pre>



</details>

<a name="0x1_Dao_new_dao_config"></a>

## Function `new_dao_config`

create a dao config


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_new_dao_config">new_dao_config</a>&lt;TokenT: <b>copy</b>, drop, store&gt;(voting_delay: u64, voting_period: u64, voting_quorum_rate: u8, min_action_delay: u64): <a href="Dao.md#0x1_Dao_DaoConfig">Dao::DaoConfig</a>&lt;TokenT&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_new_dao_config">new_dao_config</a>&lt;TokenT: <b>copy</b> + drop + store&gt;(
    voting_delay: u64,
    voting_period: u64,
    voting_quorum_rate: u8,
    min_action_delay: u64,
): <a href="Dao.md#0x1_Dao_DaoConfig">DaoConfig</a>&lt;TokenT&gt; {
    <b>assert</b>!(voting_delay &gt; 0, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Dao.md#0x1_Dao_ERR_CONFIG_PARAM_INVALID">ERR_CONFIG_PARAM_INVALID</a>));
    <b>assert</b>!(voting_period &gt; 0, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Dao.md#0x1_Dao_ERR_CONFIG_PARAM_INVALID">ERR_CONFIG_PARAM_INVALID</a>));
    <b>assert</b>!(voting_quorum_rate &gt; 0 && <a href="Dao.md#0x1_Dao_voting_quorum_rate">voting_quorum_rate</a> &lt;= 100, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Dao.md#0x1_Dao_ERR_CONFIG_PARAM_INVALID">ERR_CONFIG_PARAM_INVALID</a>));
    <b>assert</b>!(min_action_delay &gt; 0, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Dao.md#0x1_Dao_ERR_CONFIG_PARAM_INVALID">ERR_CONFIG_PARAM_INVALID</a>));
    <a href="Dao.md#0x1_Dao_DaoConfig">DaoConfig</a> { voting_delay, voting_period, voting_quorum_rate, min_action_delay }
}
</code></pre>



</details>

<a name="0x1_Dao_propose"></a>

## Function `propose`

propose a proposal.
<code>action</code>: the actual action to execute.
<code>action_delay</code>: the delay to execute after the proposal is agreed


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_propose">propose</a>&lt;TokenT: <b>copy</b>, drop, store, ActionT: <b>copy</b>, drop, store&gt;(signer: &signer, action: ActionT, action_delay: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_propose">propose</a>&lt;TokenT: <b>copy</b> + drop + store, ActionT: <b>copy</b> + drop + store&gt;(
    signer: &signer,
    action: ActionT,
    action_delay: u64,
) <b>acquires</b> <a href="Dao.md#0x1_Dao_DaoGlobalInfo">DaoGlobalInfo</a> {
    <b>if</b> (action_delay == 0) {
        action_delay = <a href="Dao.md#0x1_Dao_min_action_delay">min_action_delay</a>&lt;TokenT&gt;();
    } <b>else</b> {
        <b>assert</b>!(action_delay &gt;= <a href="Dao.md#0x1_Dao_min_action_delay">min_action_delay</a>&lt;TokenT&gt;(), <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Dao.md#0x1_Dao_ERR_ACTION_DELAY_TOO_SMALL">ERR_ACTION_DELAY_TOO_SMALL</a>));
    };
    <b>let</b> proposal_id = <a href="Dao.md#0x1_Dao_generate_next_proposal_id">generate_next_proposal_id</a>&lt;TokenT&gt;();
    <b>let</b> proposer = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer);
    <b>let</b> start_time = <a href="Timestamp.md#0x1_Timestamp_now_milliseconds">Timestamp::now_milliseconds</a>() + <a href="Dao.md#0x1_Dao_voting_delay">voting_delay</a>&lt;TokenT&gt;();
    <b>let</b> quorum_votes = <a href="Dao.md#0x1_Dao_quorum_votes">quorum_votes</a>&lt;TokenT&gt;();
    <b>let</b> proposal = <a href="Dao.md#0x1_Dao_Proposal">Proposal</a>&lt;TokenT, ActionT&gt; {
        id: proposal_id,
        proposer,
        start_time,
        end_time: start_time + <a href="Dao.md#0x1_Dao_voting_period">voting_period</a>&lt;TokenT&gt;(),
        for_votes: 0,
        against_votes: 0,
        eta: 0,
        action_delay,
        quorum_votes: quorum_votes,
        action: <a href="Option.md#0x1_Option_some">Option::some</a>(action),
    };
    <b>move_to</b>(signer, proposal);
    // emit event
    <b>let</b> gov_info = <b>borrow_global_mut</b>&lt;<a href="Dao.md#0x1_Dao_DaoGlobalInfo">DaoGlobalInfo</a>&lt;TokenT&gt;&gt;(<a href="Token.md#0x1_Token_token_address">Token::token_address</a>&lt;TokenT&gt;());
    <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>(
        &<b>mut</b> gov_info.proposal_create_event,
        <a href="Dao.md#0x1_Dao_ProposalCreatedEvent">ProposalCreatedEvent</a> { proposal_id, proposer },
    );
}
</code></pre>



</details>

<a name="0x1_Dao_cast_vote"></a>

## Function `cast_vote`

votes for a proposal.
User can only vote once, then the stake is locked,
which can only be unstaked by user after the proposal is expired, or cancelled, or executed.
So think twice before casting vote.


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_cast_vote">cast_vote</a>&lt;TokenT: <b>copy</b>, drop, store, ActionT: <b>copy</b>, drop, store&gt;(signer: &signer, proposer_address: <b>address</b>, proposal_id: u64, stake: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenT&gt;, agree: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_cast_vote">cast_vote</a>&lt;TokenT: <b>copy</b> + drop + store, ActionT: <b>copy</b> + drop + store&gt;(
    signer: &signer,
    proposer_address: <b>address</b>,
    proposal_id: u64,
    stake: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenT&gt;,
    agree: bool,
) <b>acquires</b> <a href="Dao.md#0x1_Dao_Proposal">Proposal</a>, <a href="Dao.md#0x1_Dao_DaoGlobalInfo">DaoGlobalInfo</a>, <a href="Dao.md#0x1_Dao_Vote">Vote</a> {
    {
        <b>let</b> state = <a href="Dao.md#0x1_Dao_proposal_state">proposal_state</a>&lt;TokenT, ActionT&gt;(proposer_address, proposal_id);
        // only when proposal is active, <b>use</b> can cast vote.
        <b>assert</b>!(state == <a href="Dao.md#0x1_Dao_ACTIVE">ACTIVE</a>, <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="Dao.md#0x1_Dao_ERR_PROPOSAL_STATE_INVALID">ERR_PROPOSAL_STATE_INVALID</a>));
    };
    <b>let</b> proposal = <b>borrow_global_mut</b>&lt;<a href="Dao.md#0x1_Dao_Proposal">Proposal</a>&lt;TokenT, ActionT&gt;&gt;(proposer_address);
    <b>assert</b>!(proposal.id == proposal_id, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Dao.md#0x1_Dao_ERR_PROPOSAL_ID_MISMATCH">ERR_PROPOSAL_ID_MISMATCH</a>));
    <b>let</b> sender = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer);
    <b>let</b> total_voted = <b>if</b> (<b>exists</b>&lt;<a href="Dao.md#0x1_Dao_Vote">Vote</a>&lt;TokenT&gt;&gt;(sender)) {
        <b>let</b> my_vote = <b>borrow_global_mut</b>&lt;<a href="Dao.md#0x1_Dao_Vote">Vote</a>&lt;TokenT&gt;&gt;(sender);
        <b>assert</b>!(my_vote.id == proposal_id, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Dao.md#0x1_Dao_ERR_VOTED_OTHERS_ALREADY">ERR_VOTED_OTHERS_ALREADY</a>));
        <b>assert</b>!(my_vote.agree == agree, <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="Dao.md#0x1_Dao_ERR_VOTE_STATE_MISMATCH">ERR_VOTE_STATE_MISMATCH</a>));

        <a href="Dao.md#0x1_Dao_do_cast_vote">do_cast_vote</a>(proposal, my_vote, stake);
        <a href="Token.md#0x1_Token_value">Token::value</a>(&my_vote.stake)
    } <b>else</b> {
        <b>let</b> my_vote = <a href="Dao.md#0x1_Dao_Vote">Vote</a>&lt;TokenT&gt; {
            proposer: proposer_address,
            id: proposal_id,
            stake: <a href="Token.md#0x1_Token_zero">Token::zero</a>(),
            agree,
        };
        <a href="Dao.md#0x1_Dao_do_cast_vote">do_cast_vote</a>(proposal, &<b>mut</b> my_vote, stake);
        <b>let</b> total_voted = <a href="Token.md#0x1_Token_value">Token::value</a>(&my_vote.stake);
        <b>move_to</b>(signer, my_vote);
        total_voted
    };

    // emit event
    <b>let</b> gov_info = <b>borrow_global_mut</b>&lt;<a href="Dao.md#0x1_Dao_DaoGlobalInfo">DaoGlobalInfo</a>&lt;TokenT&gt;&gt;(<a href="Token.md#0x1_Token_token_address">Token::token_address</a>&lt;TokenT&gt;());
    <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>(
        &<b>mut</b> gov_info.vote_changed_event,
        <a href="Dao.md#0x1_Dao_VoteChangedEvent">VoteChangedEvent</a> {
            proposal_id,
            proposer: proposer_address,
            voter: sender,
            agree,
            vote: total_voted,
        },
    );
}
</code></pre>



</details>

<a name="0x1_Dao_do_cast_vote"></a>

## Function `do_cast_vote`



<pre><code><b>fun</b> <a href="Dao.md#0x1_Dao_do_cast_vote">do_cast_vote</a>&lt;TokenT: <b>copy</b>, drop, store, ActionT: <b>copy</b>, drop, store&gt;(proposal: &<b>mut</b> <a href="Dao.md#0x1_Dao_Proposal">Dao::Proposal</a>&lt;TokenT, ActionT&gt;, vote: &<b>mut</b> <a href="Dao.md#0x1_Dao_Vote">Dao::Vote</a>&lt;TokenT&gt;, stake: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenT&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="Dao.md#0x1_Dao_do_cast_vote">do_cast_vote</a>&lt;TokenT: <b>copy</b> + drop + store, ActionT: <b>copy</b> + drop + store&gt;(proposal: &<b>mut</b> <a href="Dao.md#0x1_Dao_Proposal">Proposal</a>&lt;TokenT, ActionT&gt;, vote: &<b>mut</b> <a href="Dao.md#0x1_Dao_Vote">Vote</a>&lt;TokenT&gt;, stake: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenT&gt;) {
    <b>let</b> stake_value = <a href="Token.md#0x1_Token_value">Token::value</a>(&stake);
    <a href="Token.md#0x1_Token_deposit">Token::deposit</a>(&<b>mut</b> vote.stake, stake);
    <b>if</b> (vote.agree) {
        proposal.for_votes = proposal.for_votes + stake_value;
    } <b>else</b> {
        proposal.against_votes = proposal.against_votes + stake_value;
    };
}
</code></pre>



</details>

<a name="0x1_Dao_change_vote"></a>

## Function `change_vote`

Let user change their vote during the voting time.


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_change_vote">change_vote</a>&lt;TokenT: <b>copy</b>, drop, store, ActionT: <b>copy</b>, drop, store&gt;(signer: &signer, proposer_address: <b>address</b>, proposal_id: u64, agree: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_change_vote">change_vote</a>&lt;TokenT: <b>copy</b> + drop + store, ActionT: <b>copy</b> + drop + store&gt;(
    signer: &signer,
    proposer_address: <b>address</b>,
    proposal_id: u64,
    agree: bool,
) <b>acquires</b> <a href="Dao.md#0x1_Dao_Proposal">Proposal</a>, <a href="Dao.md#0x1_Dao_DaoGlobalInfo">DaoGlobalInfo</a>, <a href="Dao.md#0x1_Dao_Vote">Vote</a> {
    {
        <b>let</b> state = <a href="Dao.md#0x1_Dao_proposal_state">proposal_state</a>&lt;TokenT, ActionT&gt;(proposer_address, proposal_id);
        // only when proposal is active, user can change vote.
        <b>assert</b>!(state == <a href="Dao.md#0x1_Dao_ACTIVE">ACTIVE</a>, <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="Dao.md#0x1_Dao_ERR_PROPOSAL_STATE_INVALID">ERR_PROPOSAL_STATE_INVALID</a>));
    };
    <b>let</b> proposal = <b>borrow_global_mut</b>&lt;<a href="Dao.md#0x1_Dao_Proposal">Proposal</a>&lt;TokenT, ActionT&gt;&gt;(proposer_address);
    <b>assert</b>!(proposal.id == proposal_id, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Dao.md#0x1_Dao_ERR_PROPOSAL_ID_MISMATCH">ERR_PROPOSAL_ID_MISMATCH</a>));
    <b>let</b> my_vote = <b>borrow_global_mut</b>&lt;<a href="Dao.md#0x1_Dao_Vote">Vote</a>&lt;TokenT&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer));
    {
        <b>assert</b>!(my_vote.proposer == proposer_address, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Dao.md#0x1_Dao_ERR_PROPOSER_MISMATCH">ERR_PROPOSER_MISMATCH</a>));
        <b>assert</b>!(my_vote.id == proposal_id, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Dao.md#0x1_Dao_ERR_VOTED_OTHERS_ALREADY">ERR_VOTED_OTHERS_ALREADY</a>));
    };

    // flip the vote
    <b>if</b> (my_vote.agree != agree) {
        <b>let</b> total_voted = <a href="Dao.md#0x1_Dao_do_flip_vote">do_flip_vote</a>(my_vote, proposal);
        // emit event
        <b>let</b> gov_info = <b>borrow_global_mut</b>&lt;<a href="Dao.md#0x1_Dao_DaoGlobalInfo">DaoGlobalInfo</a>&lt;TokenT&gt;&gt;(<a href="Token.md#0x1_Token_token_address">Token::token_address</a>&lt;TokenT&gt;());
        <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>(
            &<b>mut</b> gov_info.vote_changed_event,
            <a href="Dao.md#0x1_Dao_VoteChangedEvent">VoteChangedEvent</a> {
                proposal_id,
                proposer: proposer_address,
                voter: <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer),
                agree,
                vote: total_voted,
            },
        );
    };
}
</code></pre>



</details>

<a name="0x1_Dao_do_flip_vote"></a>

## Function `do_flip_vote`



<pre><code><b>fun</b> <a href="Dao.md#0x1_Dao_do_flip_vote">do_flip_vote</a>&lt;TokenT: <b>copy</b>, drop, store, ActionT: <b>copy</b>, drop, store&gt;(my_vote: &<b>mut</b> <a href="Dao.md#0x1_Dao_Vote">Dao::Vote</a>&lt;TokenT&gt;, proposal: &<b>mut</b> <a href="Dao.md#0x1_Dao_Proposal">Dao::Proposal</a>&lt;TokenT, ActionT&gt;): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="Dao.md#0x1_Dao_do_flip_vote">do_flip_vote</a>&lt;TokenT: <b>copy</b> + drop + store, ActionT: <b>copy</b> + drop + store&gt;(my_vote: &<b>mut</b> <a href="Dao.md#0x1_Dao_Vote">Vote</a>&lt;TokenT&gt;, proposal: &<b>mut</b> <a href="Dao.md#0x1_Dao_Proposal">Proposal</a>&lt;TokenT, ActionT&gt;): u128 {
    my_vote.agree = !my_vote.agree;
    <b>let</b> total_voted = <a href="Token.md#0x1_Token_value">Token::value</a>(&my_vote.stake);
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

<a name="0x1_Dao_revoke_vote"></a>

## Function `revoke_vote`

Revoke some voting powers from vote on <code>proposal_id</code> of <code>proposer_address</code>.


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_revoke_vote">revoke_vote</a>&lt;TokenT: <b>copy</b>, drop, store, ActionT: <b>copy</b>, drop, store&gt;(signer: &signer, proposer_address: <b>address</b>, proposal_id: u64, voting_power: u128): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenT&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_revoke_vote">revoke_vote</a>&lt;TokenT: <b>copy</b> + drop + store, ActionT: <b>copy</b> + drop + store&gt;(
    signer: &signer,
    proposer_address: <b>address</b>,
    proposal_id: u64,
    voting_power: u128,
): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenT&gt; <b>acquires</b> <a href="Dao.md#0x1_Dao_Proposal">Proposal</a>, <a href="Dao.md#0x1_Dao_Vote">Vote</a>, <a href="Dao.md#0x1_Dao_DaoGlobalInfo">DaoGlobalInfo</a> {
    {
        <b>let</b> state = <a href="Dao.md#0x1_Dao_proposal_state">proposal_state</a>&lt;TokenT, ActionT&gt;(proposer_address, proposal_id);
        // only when proposal is active, user can revoke vote.
        <b>assert</b>!(state == <a href="Dao.md#0x1_Dao_ACTIVE">ACTIVE</a>, <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="Dao.md#0x1_Dao_ERR_PROPOSAL_STATE_INVALID">ERR_PROPOSAL_STATE_INVALID</a>));
    };
    // get proposal
    <b>let</b> proposal = <b>borrow_global_mut</b>&lt;<a href="Dao.md#0x1_Dao_Proposal">Proposal</a>&lt;TokenT, ActionT&gt;&gt;(proposer_address);

    // get vote
    <b>let</b> my_vote = <b>move_from</b>&lt;<a href="Dao.md#0x1_Dao_Vote">Vote</a>&lt;TokenT&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer));
    {
        <b>assert</b>!(my_vote.proposer == proposer_address, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Dao.md#0x1_Dao_ERR_PROPOSER_MISMATCH">ERR_PROPOSER_MISMATCH</a>));
        <b>assert</b>!(my_vote.id == proposal_id, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Dao.md#0x1_Dao_ERR_VOTED_OTHERS_ALREADY">ERR_VOTED_OTHERS_ALREADY</a>));
    };
    // revoke vote on proposal
    <b>let</b> reverted_stake =<a href="Dao.md#0x1_Dao_do_revoke_vote">do_revoke_vote</a>(proposal, &<b>mut</b> my_vote, voting_power);
    // emit vote changed event
    <b>let</b> gov_info = <b>borrow_global_mut</b>&lt;<a href="Dao.md#0x1_Dao_DaoGlobalInfo">DaoGlobalInfo</a>&lt;TokenT&gt;&gt;(<a href="Token.md#0x1_Token_token_address">Token::token_address</a>&lt;TokenT&gt;());
    <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>(
        &<b>mut</b> gov_info.vote_changed_event,
        <a href="Dao.md#0x1_Dao_VoteChangedEvent">VoteChangedEvent</a> {
            proposal_id,
            proposer: proposer_address,
            voter: <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer),
            agree: my_vote.agree,
            vote: <a href="Token.md#0x1_Token_value">Token::value</a>(&my_vote.stake),
        },
    );

    // <b>if</b> user <b>has</b> no stake, destroy his vote. resolve https://github.com/starcoinorg/starcoin/issues/2925.
    <b>if</b> (<a href="Token.md#0x1_Token_value">Token::value</a>(&my_vote.stake) == 0u128) {
        <b>let</b> <a href="Dao.md#0x1_Dao_Vote">Vote</a> {stake, proposer: _, id: _, agree: _} = my_vote;
        <a href="Token.md#0x1_Token_destroy_zero">Token::destroy_zero</a>(stake);
    } <b>else</b> {
        <b>move_to</b>(signer, my_vote);
    };

    reverted_stake
}
</code></pre>



</details>

<a name="0x1_Dao_do_revoke_vote"></a>

## Function `do_revoke_vote`



<pre><code><b>fun</b> <a href="Dao.md#0x1_Dao_do_revoke_vote">do_revoke_vote</a>&lt;TokenT: <b>copy</b>, drop, store, ActionT: <b>copy</b>, drop, store&gt;(proposal: &<b>mut</b> <a href="Dao.md#0x1_Dao_Proposal">Dao::Proposal</a>&lt;TokenT, ActionT&gt;, vote: &<b>mut</b> <a href="Dao.md#0x1_Dao_Vote">Dao::Vote</a>&lt;TokenT&gt;, to_revoke: u128): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenT&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="Dao.md#0x1_Dao_do_revoke_vote">do_revoke_vote</a>&lt;TokenT: <b>copy</b> + drop + store, ActionT: <b>copy</b> + drop + store&gt;(proposal: &<b>mut</b> <a href="Dao.md#0x1_Dao_Proposal">Proposal</a>&lt;TokenT, ActionT&gt;, vote: &<b>mut</b> <a href="Dao.md#0x1_Dao_Vote">Vote</a>&lt;TokenT&gt;, to_revoke: u128): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenT&gt; {
    <b>spec</b> {
        <b>assume</b> vote.stake.value &gt;= to_revoke;
    };
    <b>let</b> reverted_stake = <a href="Token.md#0x1_Token_withdraw">Token::withdraw</a>(&<b>mut</b> vote.stake, to_revoke);
    <b>if</b> (vote.agree) {
        proposal.for_votes = proposal.for_votes - to_revoke;
    } <b>else</b> {
        proposal.against_votes = proposal.against_votes - to_revoke;
    };
    <b>spec</b> {
        <b>assert</b> <a href="Token.md#0x1_Token_value">Token::value</a>(reverted_stake) == to_revoke;
    };
    reverted_stake
}
</code></pre>



</details>

<a name="0x1_Dao_unstake_votes"></a>

## Function `unstake_votes`

Retrieve back my staked token voted for a proposal.


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_unstake_votes">unstake_votes</a>&lt;TokenT: <b>copy</b>, drop, store, ActionT: <b>copy</b>, drop, store&gt;(signer: &signer, proposer_address: <b>address</b>, proposal_id: u64): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenT&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_unstake_votes">unstake_votes</a>&lt;TokenT: <b>copy</b> + drop + store, ActionT: <b>copy</b> + drop + store&gt;(
    signer: &signer,
    proposer_address: <b>address</b>,
    proposal_id: u64,
): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenT&gt; <b>acquires</b> <a href="Dao.md#0x1_Dao_Proposal">Proposal</a>, <a href="Dao.md#0x1_Dao_Vote">Vote</a> {
    // only check state when proposal <b>exists</b>.
    // because proposal can be destroyed after it ends in <a href="Dao.md#0x1_Dao_DEFEATED">DEFEATED</a> or <a href="Dao.md#0x1_Dao_EXTRACTED">EXTRACTED</a> state.
    <b>if</b> (<a href="Dao.md#0x1_Dao_proposal_exists">proposal_exists</a>&lt;TokenT, ActionT&gt;(proposer_address, proposal_id)) {
        <b>let</b> state = <a href="Dao.md#0x1_Dao_proposal_state">proposal_state</a>&lt;TokenT, ActionT&gt;(proposer_address, proposal_id);
        // Only after vote period end, user can unstake his votes.
        <b>assert</b>!(state &gt; <a href="Dao.md#0x1_Dao_ACTIVE">ACTIVE</a>, <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="Dao.md#0x1_Dao_ERR_PROPOSAL_STATE_INVALID">ERR_PROPOSAL_STATE_INVALID</a>));
    };
    <b>let</b> <a href="Dao.md#0x1_Dao_Vote">Vote</a> { proposer, id, stake, agree: _ } = <b>move_from</b>&lt;<a href="Dao.md#0x1_Dao_Vote">Vote</a>&lt;TokenT&gt;&gt;(
        <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer),
    );
    // these checks are still required.
    <b>assert</b>!(proposer == proposer_address, <a href="Errors.md#0x1_Errors_requires_address">Errors::requires_address</a>(<a href="Dao.md#0x1_Dao_ERR_PROPOSER_MISMATCH">ERR_PROPOSER_MISMATCH</a>));
    <b>assert</b>!(id == proposal_id, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Dao.md#0x1_Dao_ERR_VOTED_OTHERS_ALREADY">ERR_VOTED_OTHERS_ALREADY</a>));
    stake
}
</code></pre>



</details>

<a name="0x1_Dao_queue_proposal_action"></a>

## Function `queue_proposal_action`

queue agreed proposal to execute.


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="Dao.md#0x1_Dao_queue_proposal_action">queue_proposal_action</a>&lt;TokenT: <b>copy</b>, drop, store, ActionT: <b>copy</b>, drop, store&gt;(proposer_address: <b>address</b>, proposal_id: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="Dao.md#0x1_Dao_queue_proposal_action">queue_proposal_action</a>&lt;TokenT: <b>copy</b> + drop + store, ActionT: <b>copy</b> + drop + store&gt;(
    proposer_address: <b>address</b>,
    proposal_id: u64,
) <b>acquires</b> <a href="Dao.md#0x1_Dao_Proposal">Proposal</a> {
    // Only agreed proposal can be submitted.
    <b>assert</b>!(
        <a href="Dao.md#0x1_Dao_proposal_state">proposal_state</a>&lt;TokenT, ActionT&gt;(proposer_address, proposal_id) == <a href="Dao.md#0x1_Dao_AGREED">AGREED</a>,
        <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="Dao.md#0x1_Dao_ERR_PROPOSAL_STATE_INVALID">ERR_PROPOSAL_STATE_INVALID</a>)
    );
    <b>let</b> proposal = <b>borrow_global_mut</b>&lt;<a href="Dao.md#0x1_Dao_Proposal">Proposal</a>&lt;TokenT, ActionT&gt;&gt;(proposer_address);
    proposal.eta = <a href="Timestamp.md#0x1_Timestamp_now_milliseconds">Timestamp::now_milliseconds</a>() + proposal.action_delay;
}
</code></pre>



</details>

<a name="0x1_Dao_extract_proposal_action"></a>

## Function `extract_proposal_action`

extract proposal action to execute.


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_extract_proposal_action">extract_proposal_action</a>&lt;TokenT: <b>copy</b>, drop, store, ActionT: <b>copy</b>, drop, store&gt;(proposer_address: <b>address</b>, proposal_id: u64): ActionT
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_extract_proposal_action">extract_proposal_action</a>&lt;TokenT: <b>copy</b> + drop + store, ActionT: <b>copy</b> + drop + store&gt;(
    proposer_address: <b>address</b>,
    proposal_id: u64,
): ActionT <b>acquires</b> <a href="Dao.md#0x1_Dao_Proposal">Proposal</a> {
    // Only executable proposal's action can be extracted.
    <b>assert</b>!(
        <a href="Dao.md#0x1_Dao_proposal_state">proposal_state</a>&lt;TokenT, ActionT&gt;(proposer_address, proposal_id) == <a href="Dao.md#0x1_Dao_EXECUTABLE">EXECUTABLE</a>,
        <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="Dao.md#0x1_Dao_ERR_PROPOSAL_STATE_INVALID">ERR_PROPOSAL_STATE_INVALID</a>),
    );
    <b>let</b> proposal = <b>borrow_global_mut</b>&lt;<a href="Dao.md#0x1_Dao_Proposal">Proposal</a>&lt;TokenT, ActionT&gt;&gt;(proposer_address);
    <b>let</b> action: ActionT = <a href="Option.md#0x1_Option_extract">Option::extract</a>(&<b>mut</b> proposal.action);
    action
}
</code></pre>



</details>

<a name="0x1_Dao_destroy_terminated_proposal"></a>

## Function `destroy_terminated_proposal`

remove terminated proposal from proposer


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="Dao.md#0x1_Dao_destroy_terminated_proposal">destroy_terminated_proposal</a>&lt;TokenT: <b>copy</b>, drop, store, ActionT: <b>copy</b>, drop, store&gt;(proposer_address: <b>address</b>, proposal_id: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="Dao.md#0x1_Dao_destroy_terminated_proposal">destroy_terminated_proposal</a>&lt;TokenT: <b>copy</b> + drop + store, ActionT: <b>copy</b> + drop + store&gt;(
    proposer_address: <b>address</b>,
    proposal_id: u64,
) <b>acquires</b> <a href="Dao.md#0x1_Dao_Proposal">Proposal</a> {
    <b>let</b> proposal_state = <a href="Dao.md#0x1_Dao_proposal_state">proposal_state</a>&lt;TokenT, ActionT&gt;(proposer_address, proposal_id);
    <b>assert</b>!(
        proposal_state == <a href="Dao.md#0x1_Dao_DEFEATED">DEFEATED</a> || proposal_state == <a href="Dao.md#0x1_Dao_EXTRACTED">EXTRACTED</a>,
        <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="Dao.md#0x1_Dao_ERR_PROPOSAL_STATE_INVALID">ERR_PROPOSAL_STATE_INVALID</a>),
    );
    <b>let</b> <a href="Dao.md#0x1_Dao_Proposal">Proposal</a> {
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
    } = <b>move_from</b>&lt;<a href="Dao.md#0x1_Dao_Proposal">Proposal</a>&lt;TokenT, ActionT&gt;&gt;(proposer_address);
    <b>if</b> (proposal_state == <a href="Dao.md#0x1_Dao_DEFEATED">DEFEATED</a>) {
        <b>let</b> _ = <a href="Option.md#0x1_Option_extract">Option::extract</a>(&<b>mut</b> action);
    };
    <a href="Option.md#0x1_Option_destroy_none">Option::destroy_none</a>(action);
}
</code></pre>



</details>

<a name="0x1_Dao_proposal_exists"></a>

## Function `proposal_exists`

check whether a proposal exists in <code>proposer_address</code> with id <code>proposal_id</code>.


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_proposal_exists">proposal_exists</a>&lt;TokenT: <b>copy</b>, drop, store, ActionT: <b>copy</b>, drop, store&gt;(proposer_address: <b>address</b>, proposal_id: u64): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_proposal_exists">proposal_exists</a>&lt;TokenT: <b>copy</b> + drop + store, ActionT: <b>copy</b> + drop + store&gt;(
    proposer_address: <b>address</b>,
    proposal_id: u64,
): bool <b>acquires</b> <a href="Dao.md#0x1_Dao_Proposal">Proposal</a> {
    <b>if</b> (<b>exists</b>&lt;<a href="Dao.md#0x1_Dao_Proposal">Proposal</a>&lt;TokenT, ActionT&gt;&gt;(proposer_address)) {
        <b>let</b> proposal = <b>borrow_global</b>&lt;<a href="Dao.md#0x1_Dao_Proposal">Proposal</a>&lt;TokenT, ActionT&gt;&gt;(proposer_address);
        <b>return</b> proposal.id == proposal_id
    };
    <b>false</b>
}
</code></pre>



</details>

<a name="0x1_Dao_proposal_state"></a>

## Function `proposal_state`

Get the proposal state.


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_proposal_state">proposal_state</a>&lt;TokenT: <b>copy</b>, drop, store, ActionT: <b>copy</b>, drop, store&gt;(proposer_address: <b>address</b>, proposal_id: u64): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_proposal_state">proposal_state</a>&lt;TokenT: <b>copy</b> + drop + store, ActionT: <b>copy</b> + drop + store&gt;(
    proposer_address: <b>address</b>,
    proposal_id: u64,
): u8 <b>acquires</b> <a href="Dao.md#0x1_Dao_Proposal">Proposal</a> {
    <b>let</b> proposal = <b>borrow_global</b>&lt;<a href="Dao.md#0x1_Dao_Proposal">Proposal</a>&lt;TokenT, ActionT&gt;&gt;(proposer_address);
    <b>assert</b>!(proposal.id == proposal_id, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Dao.md#0x1_Dao_ERR_PROPOSAL_ID_MISMATCH">ERR_PROPOSAL_ID_MISMATCH</a>));
    <b>let</b> current_time = <a href="Timestamp.md#0x1_Timestamp_now_milliseconds">Timestamp::now_milliseconds</a>();
    <a href="Dao.md#0x1_Dao_do_proposal_state">do_proposal_state</a>(proposal, current_time)
}
</code></pre>



</details>

<a name="0x1_Dao_do_proposal_state"></a>

## Function `do_proposal_state`



<pre><code><b>fun</b> <a href="Dao.md#0x1_Dao_do_proposal_state">do_proposal_state</a>&lt;TokenT: <b>copy</b>, drop, store, ActionT: <b>copy</b>, drop, store&gt;(proposal: &<a href="Dao.md#0x1_Dao_Proposal">Dao::Proposal</a>&lt;TokenT, ActionT&gt;, current_time: u64): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="Dao.md#0x1_Dao_do_proposal_state">do_proposal_state</a>&lt;TokenT: <b>copy</b> + drop + store, ActionT: <b>copy</b> + drop + store&gt;(
    proposal: &<a href="Dao.md#0x1_Dao_Proposal">Proposal</a>&lt;TokenT, ActionT&gt;,
    current_time: u64,
): u8 {
    <b>if</b> (current_time &lt; proposal.start_time) {
        // Pending
        <a href="Dao.md#0x1_Dao_PENDING">PENDING</a>
    } <b>else</b> <b>if</b> (current_time &lt;= proposal.end_time) {
        // Active
        <a href="Dao.md#0x1_Dao_ACTIVE">ACTIVE</a>
    } <b>else</b> <b>if</b> (proposal.for_votes &lt;= proposal.against_votes ||
        proposal.for_votes &lt; proposal.quorum_votes) {
        // Defeated
        <a href="Dao.md#0x1_Dao_DEFEATED">DEFEATED</a>
    } <b>else</b> <b>if</b> (proposal.eta == 0) {
        // Agreed.
        <a href="Dao.md#0x1_Dao_AGREED">AGREED</a>
    } <b>else</b> <b>if</b> (current_time &lt; proposal.eta) {
        // Queued, waiting <b>to</b> execute
        <a href="Dao.md#0x1_Dao_QUEUED">QUEUED</a>
    } <b>else</b> <b>if</b> (<a href="Option.md#0x1_Option_is_some">Option::is_some</a>(&proposal.action)) {
        <a href="Dao.md#0x1_Dao_EXECUTABLE">EXECUTABLE</a>
    } <b>else</b> {
        <a href="Dao.md#0x1_Dao_EXTRACTED">EXTRACTED</a>
    }
}
</code></pre>



</details>

<a name="0x1_Dao_proposal_info"></a>

## Function `proposal_info`

get proposal's information.
return: (id, start_time, end_time, for_votes, against_votes).


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_proposal_info">proposal_info</a>&lt;TokenT: <b>copy</b>, drop, store, ActionT: <b>copy</b>, drop, store&gt;(proposer_address: <b>address</b>): (u64, u64, u64, u128, u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_proposal_info">proposal_info</a>&lt;TokenT: <b>copy</b> + drop + store, ActionT: <b>copy</b> + drop + store&gt;(
    proposer_address: <b>address</b>,
): (u64, u64, u64, u128, u128) <b>acquires</b> <a href="Dao.md#0x1_Dao_Proposal">Proposal</a> {
    <b>let</b> proposal = <b>borrow_global</b>&lt;<a href="Dao.md#0x1_Dao_Proposal">Proposal</a>&lt;TokenT, ActionT&gt;&gt;(proposer_address);
    (proposal.id, proposal.start_time, proposal.end_time, proposal.for_votes, proposal.against_votes)
}
</code></pre>



</details>

<a name="0x1_Dao_vote_of"></a>

## Function `vote_of`

Get voter's vote info on proposal with <code>proposal_id</code> of <code>proposer_address</code>.


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_vote_of">vote_of</a>&lt;TokenT: <b>copy</b>, drop, store&gt;(voter: <b>address</b>, proposer_address: <b>address</b>, proposal_id: u64): (bool, u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_vote_of">vote_of</a>&lt;TokenT: <b>copy</b> + drop + store&gt;(
    voter: <b>address</b>,
    proposer_address: <b>address</b>,
    proposal_id: u64,
): (bool, u128) <b>acquires</b> <a href="Dao.md#0x1_Dao_Vote">Vote</a> {
    <b>let</b> vote = <b>borrow_global</b>&lt;<a href="Dao.md#0x1_Dao_Vote">Vote</a>&lt;TokenT&gt;&gt;(voter);
    <b>assert</b>!(vote.proposer == proposer_address, <a href="Errors.md#0x1_Errors_requires_address">Errors::requires_address</a>(<a href="Dao.md#0x1_Dao_ERR_PROPOSER_MISMATCH">ERR_PROPOSER_MISMATCH</a>));
    <b>assert</b>!(vote.id == proposal_id, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Dao.md#0x1_Dao_ERR_VOTED_OTHERS_ALREADY">ERR_VOTED_OTHERS_ALREADY</a>));
    (vote.agree, <a href="Token.md#0x1_Token_value">Token::value</a>(&vote.stake))
}
</code></pre>



</details>

<a name="0x1_Dao_has_vote"></a>

## Function `has_vote`

Check whether voter has voted on proposal with <code>proposal_id</code> of <code>proposer_address</code>.


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_has_vote">has_vote</a>&lt;TokenT: <b>copy</b>, drop, store&gt;(voter: <b>address</b>, proposer_address: <b>address</b>, proposal_id: u64): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_has_vote">has_vote</a>&lt;TokenT: <b>copy</b> + drop + store&gt;(
    voter: <b>address</b>,
    proposer_address: <b>address</b>,
    proposal_id: u64,
): bool <b>acquires</b> <a href="Dao.md#0x1_Dao_Vote">Vote</a> {
    <b>if</b> (!<b>exists</b>&lt;<a href="Dao.md#0x1_Dao_Vote">Vote</a>&lt;TokenT&gt;&gt;(voter)) {
        <b>return</b> <b>false</b>
    };

    <b>let</b> vote = <b>borrow_global</b>&lt;<a href="Dao.md#0x1_Dao_Vote">Vote</a>&lt;TokenT&gt;&gt;(voter);
    vote.proposer == proposer_address && vote.id == proposal_id
}
</code></pre>



</details>

<a name="0x1_Dao_generate_next_proposal_id"></a>

## Function `generate_next_proposal_id`



<pre><code><b>fun</b> <a href="Dao.md#0x1_Dao_generate_next_proposal_id">generate_next_proposal_id</a>&lt;TokenT: store&gt;(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="Dao.md#0x1_Dao_generate_next_proposal_id">generate_next_proposal_id</a>&lt;TokenT: store&gt;(): u64 <b>acquires</b> <a href="Dao.md#0x1_Dao_DaoGlobalInfo">DaoGlobalInfo</a> {
    <b>let</b> gov_info = <b>borrow_global_mut</b>&lt;<a href="Dao.md#0x1_Dao_DaoGlobalInfo">DaoGlobalInfo</a>&lt;TokenT&gt;&gt;(<a href="Token.md#0x1_Token_token_address">Token::token_address</a>&lt;TokenT&gt;());
    <b>let</b> proposal_id = gov_info.next_proposal_id;
    gov_info.next_proposal_id = proposal_id + 1;
    proposal_id
}
</code></pre>



</details>

<a name="0x1_Dao_voting_delay"></a>

## Function `voting_delay`

get default voting delay of the DAO.


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_voting_delay">voting_delay</a>&lt;TokenT: <b>copy</b>, drop, store&gt;(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_voting_delay">voting_delay</a>&lt;TokenT: <b>copy</b> + drop + store&gt;(): u64 {
    <a href="Dao.md#0x1_Dao_get_config">get_config</a>&lt;TokenT&gt;().voting_delay
}
</code></pre>



</details>

<a name="0x1_Dao_voting_period"></a>

## Function `voting_period`

get the default voting period of the DAO.


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_voting_period">voting_period</a>&lt;TokenT: <b>copy</b>, drop, store&gt;(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_voting_period">voting_period</a>&lt;TokenT: <b>copy</b> + drop + store&gt;(): u64 {
    <a href="Dao.md#0x1_Dao_get_config">get_config</a>&lt;TokenT&gt;().voting_period
}
</code></pre>



</details>

<a name="0x1_Dao_quorum_votes"></a>

## Function `quorum_votes`

Quorum votes to make proposal pass.


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_quorum_votes">quorum_votes</a>&lt;TokenT: <b>copy</b>, drop, store&gt;(): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_quorum_votes">quorum_votes</a>&lt;TokenT: <b>copy</b> + drop + store&gt;(): u128 {
    <b>let</b> market_cap = <a href="Token.md#0x1_Token_market_cap">Token::market_cap</a>&lt;TokenT&gt;();
    <b>let</b> balance_in_treasury = <a href="Treasury.md#0x1_Treasury_balance">Treasury::balance</a>&lt;TokenT&gt;();
    <b>let</b> supply = market_cap - balance_in_treasury;
    <b>let</b> rate = <a href="Dao.md#0x1_Dao_voting_quorum_rate">voting_quorum_rate</a>&lt;TokenT&gt;();
    <b>let</b> rate = (rate <b>as</b> u128);
    supply * rate / 100
}
</code></pre>



</details>

<a name="0x1_Dao_voting_quorum_rate"></a>

## Function `voting_quorum_rate`

Get the quorum rate in percent.


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_voting_quorum_rate">voting_quorum_rate</a>&lt;TokenT: <b>copy</b>, drop, store&gt;(): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_voting_quorum_rate">voting_quorum_rate</a>&lt;TokenT: <b>copy</b> + drop + store&gt;(): u8 {
    <a href="Dao.md#0x1_Dao_get_config">get_config</a>&lt;TokenT&gt;().voting_quorum_rate
}
</code></pre>



</details>

<a name="0x1_Dao_min_action_delay"></a>

## Function `min_action_delay`

Get the min_action_delay of the DAO.


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_min_action_delay">min_action_delay</a>&lt;TokenT: <b>copy</b>, drop, store&gt;(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_min_action_delay">min_action_delay</a>&lt;TokenT: <b>copy</b> + drop + store&gt;(): u64 {
    <a href="Dao.md#0x1_Dao_get_config">get_config</a>&lt;TokenT&gt;().min_action_delay
}
</code></pre>



</details>

<a name="0x1_Dao_get_config"></a>

## Function `get_config`



<pre><code><b>fun</b> <a href="Dao.md#0x1_Dao_get_config">get_config</a>&lt;TokenT: <b>copy</b>, drop, store&gt;(): <a href="Dao.md#0x1_Dao_DaoConfig">Dao::DaoConfig</a>&lt;TokenT&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="Dao.md#0x1_Dao_get_config">get_config</a>&lt;TokenT: <b>copy</b> + drop + store&gt;(): <a href="Dao.md#0x1_Dao_DaoConfig">DaoConfig</a>&lt;TokenT&gt; {
    <b>let</b> token_issuer = <a href="Token.md#0x1_Token_token_address">Token::token_address</a>&lt;TokenT&gt;();
    <a href="Config.md#0x1_Config_get_by_address">Config::get_by_address</a>&lt;<a href="Dao.md#0x1_Dao_DaoConfig">DaoConfig</a>&lt;TokenT&gt;&gt;(token_issuer)
}
</code></pre>



</details>

<a name="0x1_Dao_modify_dao_config"></a>

## Function `modify_dao_config`

update function, modify dao config.
if any param is 0, it means no change to that param.


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_modify_dao_config">modify_dao_config</a>&lt;TokenT: <b>copy</b>, drop, store&gt;(cap: &<b>mut</b> <a href="Config.md#0x1_Config_ModifyConfigCapability">Config::ModifyConfigCapability</a>&lt;<a href="Dao.md#0x1_Dao_DaoConfig">Dao::DaoConfig</a>&lt;TokenT&gt;&gt;, voting_delay: u64, voting_period: u64, voting_quorum_rate: u8, min_action_delay: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_modify_dao_config">modify_dao_config</a>&lt;TokenT: <b>copy</b> + drop + store&gt;(
    cap: &<b>mut</b> <a href="Config.md#0x1_Config_ModifyConfigCapability">Config::ModifyConfigCapability</a>&lt;<a href="Dao.md#0x1_Dao_DaoConfig">DaoConfig</a>&lt;TokenT&gt;&gt;,
    voting_delay: u64,
    voting_period: u64,
    voting_quorum_rate: u8,
    min_action_delay: u64,
) {
    <b>assert</b>!(<a href="Config.md#0x1_Config_account_address">Config::account_address</a>(cap) == <a href="Token.md#0x1_Token_token_address">Token::token_address</a>&lt;TokenT&gt;(), <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Dao.md#0x1_Dao_ERR_NOT_AUTHORIZED">ERR_NOT_AUTHORIZED</a>));
    <b>let</b> config = <a href="Dao.md#0x1_Dao_get_config">get_config</a>&lt;TokenT&gt;();
    <b>if</b> (voting_period &gt; 0) {
        config.voting_period = voting_period;
    };
    <b>if</b> (voting_delay &gt; 0) {
        config.voting_delay = voting_delay;
    };
    <b>if</b> (voting_quorum_rate &gt; 0) {
        <b>assert</b>!(<a href="Dao.md#0x1_Dao_voting_quorum_rate">voting_quorum_rate</a> &lt;= 100, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Dao.md#0x1_Dao_ERR_QUORUM_RATE_INVALID">ERR_QUORUM_RATE_INVALID</a>));
        config.voting_quorum_rate = voting_quorum_rate;
    };
    <b>if</b> (min_action_delay &gt; 0) {
        config.min_action_delay = min_action_delay;
    };
    <a href="Config.md#0x1_Config_set_with_capability">Config::set_with_capability</a>&lt;<a href="Dao.md#0x1_Dao_DaoConfig">DaoConfig</a>&lt;TokenT&gt;&gt;(cap, config);
}
</code></pre>



</details>

<a name="0x1_Dao_set_voting_delay"></a>

## Function `set_voting_delay`

set voting delay


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_set_voting_delay">set_voting_delay</a>&lt;TokenT: <b>copy</b>, drop, store&gt;(cap: &<b>mut</b> <a href="Config.md#0x1_Config_ModifyConfigCapability">Config::ModifyConfigCapability</a>&lt;<a href="Dao.md#0x1_Dao_DaoConfig">Dao::DaoConfig</a>&lt;TokenT&gt;&gt;, value: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_set_voting_delay">set_voting_delay</a>&lt;TokenT: <b>copy</b> + drop + store&gt;(
    cap: &<b>mut</b> <a href="Config.md#0x1_Config_ModifyConfigCapability">Config::ModifyConfigCapability</a>&lt;<a href="Dao.md#0x1_Dao_DaoConfig">DaoConfig</a>&lt;TokenT&gt;&gt;,
    value: u64,
) {
    <b>assert</b>!(<a href="Config.md#0x1_Config_account_address">Config::account_address</a>(cap) == <a href="Token.md#0x1_Token_token_address">Token::token_address</a>&lt;TokenT&gt;(), <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Dao.md#0x1_Dao_ERR_NOT_AUTHORIZED">ERR_NOT_AUTHORIZED</a>));
    <b>assert</b>!(value &gt; 0, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Dao.md#0x1_Dao_ERR_CONFIG_PARAM_INVALID">ERR_CONFIG_PARAM_INVALID</a>));
    <b>let</b> config = <a href="Dao.md#0x1_Dao_get_config">get_config</a>&lt;TokenT&gt;();
    config.voting_delay = value;
    <a href="Config.md#0x1_Config_set_with_capability">Config::set_with_capability</a>&lt;<a href="Dao.md#0x1_Dao_DaoConfig">DaoConfig</a>&lt;TokenT&gt;&gt;(cap, config);
}
</code></pre>



</details>

<a name="0x1_Dao_set_voting_period"></a>

## Function `set_voting_period`

set voting period


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_set_voting_period">set_voting_period</a>&lt;TokenT: <b>copy</b>, drop, store&gt;(cap: &<b>mut</b> <a href="Config.md#0x1_Config_ModifyConfigCapability">Config::ModifyConfigCapability</a>&lt;<a href="Dao.md#0x1_Dao_DaoConfig">Dao::DaoConfig</a>&lt;TokenT&gt;&gt;, value: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_set_voting_period">set_voting_period</a>&lt;TokenT: <b>copy</b> + drop + store&gt;(
    cap: &<b>mut</b> <a href="Config.md#0x1_Config_ModifyConfigCapability">Config::ModifyConfigCapability</a>&lt;<a href="Dao.md#0x1_Dao_DaoConfig">DaoConfig</a>&lt;TokenT&gt;&gt;,
    value: u64,
) {
    <b>assert</b>!(<a href="Config.md#0x1_Config_account_address">Config::account_address</a>(cap) == <a href="Token.md#0x1_Token_token_address">Token::token_address</a>&lt;TokenT&gt;(), <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Dao.md#0x1_Dao_ERR_NOT_AUTHORIZED">ERR_NOT_AUTHORIZED</a>));
    <b>assert</b>!(value &gt; 0, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Dao.md#0x1_Dao_ERR_CONFIG_PARAM_INVALID">ERR_CONFIG_PARAM_INVALID</a>));
    <b>let</b> config = <a href="Dao.md#0x1_Dao_get_config">get_config</a>&lt;TokenT&gt;();
    config.voting_period = value;
    <a href="Config.md#0x1_Config_set_with_capability">Config::set_with_capability</a>&lt;<a href="Dao.md#0x1_Dao_DaoConfig">DaoConfig</a>&lt;TokenT&gt;&gt;(cap, config);
}
</code></pre>



</details>

<a name="0x1_Dao_set_voting_quorum_rate"></a>

## Function `set_voting_quorum_rate`

set voting quorum rate


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_set_voting_quorum_rate">set_voting_quorum_rate</a>&lt;TokenT: <b>copy</b>, drop, store&gt;(cap: &<b>mut</b> <a href="Config.md#0x1_Config_ModifyConfigCapability">Config::ModifyConfigCapability</a>&lt;<a href="Dao.md#0x1_Dao_DaoConfig">Dao::DaoConfig</a>&lt;TokenT&gt;&gt;, value: u8)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_set_voting_quorum_rate">set_voting_quorum_rate</a>&lt;TokenT: <b>copy</b> + drop + store&gt;(
    cap: &<b>mut</b> <a href="Config.md#0x1_Config_ModifyConfigCapability">Config::ModifyConfigCapability</a>&lt;<a href="Dao.md#0x1_Dao_DaoConfig">DaoConfig</a>&lt;TokenT&gt;&gt;,
    value: u8,
) {
    <b>assert</b>!(<a href="Config.md#0x1_Config_account_address">Config::account_address</a>(cap) == <a href="Token.md#0x1_Token_token_address">Token::token_address</a>&lt;TokenT&gt;(), <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Dao.md#0x1_Dao_ERR_NOT_AUTHORIZED">ERR_NOT_AUTHORIZED</a>));
    <b>assert</b>!(value &lt;= 100 && value &gt; 0, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Dao.md#0x1_Dao_ERR_QUORUM_RATE_INVALID">ERR_QUORUM_RATE_INVALID</a>));
    <b>let</b> config = <a href="Dao.md#0x1_Dao_get_config">get_config</a>&lt;TokenT&gt;();
    config.voting_quorum_rate = value;
    <a href="Config.md#0x1_Config_set_with_capability">Config::set_with_capability</a>&lt;<a href="Dao.md#0x1_Dao_DaoConfig">DaoConfig</a>&lt;TokenT&gt;&gt;(cap, config);
}
</code></pre>



</details>

<a name="0x1_Dao_set_min_action_delay"></a>

## Function `set_min_action_delay`

set min action delay


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_set_min_action_delay">set_min_action_delay</a>&lt;TokenT: <b>copy</b>, drop, store&gt;(cap: &<b>mut</b> <a href="Config.md#0x1_Config_ModifyConfigCapability">Config::ModifyConfigCapability</a>&lt;<a href="Dao.md#0x1_Dao_DaoConfig">Dao::DaoConfig</a>&lt;TokenT&gt;&gt;, value: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_set_min_action_delay">set_min_action_delay</a>&lt;TokenT: <b>copy</b> + drop + store&gt;(
    cap: &<b>mut</b> <a href="Config.md#0x1_Config_ModifyConfigCapability">Config::ModifyConfigCapability</a>&lt;<a href="Dao.md#0x1_Dao_DaoConfig">DaoConfig</a>&lt;TokenT&gt;&gt;,
    value: u64,
) {
    <b>assert</b>!(<a href="Config.md#0x1_Config_account_address">Config::account_address</a>(cap) == <a href="Token.md#0x1_Token_token_address">Token::token_address</a>&lt;TokenT&gt;(), <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Dao.md#0x1_Dao_ERR_NOT_AUTHORIZED">ERR_NOT_AUTHORIZED</a>));
    <b>assert</b>!(value &gt; 0, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Dao.md#0x1_Dao_ERR_CONFIG_PARAM_INVALID">ERR_CONFIG_PARAM_INVALID</a>));
    <b>let</b> config = <a href="Dao.md#0x1_Dao_get_config">get_config</a>&lt;TokenT&gt;();
    config.min_action_delay = value;
    <a href="Config.md#0x1_Config_set_with_capability">Config::set_with_capability</a>&lt;<a href="Dao.md#0x1_Dao_DaoConfig">DaoConfig</a>&lt;TokenT&gt;&gt;(cap, config);
}
</code></pre>



</details>

<a name="@Specification_1"></a>

## Specification



<pre><code><b>pragma</b> verify = <b>false</b>;
<b>pragma</b> aborts_if_is_partial = <b>false</b>;
<b>pragma</b> aborts_if_is_strict = <b>true</b>;
</code></pre>



<a name="@Specification_1_DaoConfig"></a>

### Struct `DaoConfig`


<pre><code><b>struct</b> <a href="Dao.md#0x1_Dao_DaoConfig">DaoConfig</a>&lt;TokenT: <b>copy</b>, drop, store&gt; <b>has</b> <b>copy</b>, drop, store
</code></pre>



<dl>
<dt>
<code>voting_delay: u64</code>
</dt>
<dd>
 after proposal created, how long use should wait before he can vote.
</dd>
<dt>
<code>voting_period: u64</code>
</dt>
<dd>
 how long the voting window is.
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
 how long the proposal should wait before it can be executed.
</dd>
</dl>



<pre><code><b>invariant</b> voting_quorum_rate &gt; 0 && <a href="Dao.md#0x1_Dao_voting_quorum_rate">voting_quorum_rate</a> &lt;= 100;
<b>invariant</b> voting_delay &gt; 0;
<b>invariant</b> voting_period &gt; 0;
<b>invariant</b> min_action_delay &gt; 0;
</code></pre>



<a name="@Specification_1_plugin"></a>

### Function `plugin`


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_plugin">plugin</a>&lt;TokenT: <b>copy</b>, drop, store&gt;(signer: &signer, voting_delay: u64, voting_period: u64, voting_quorum_rate: u8, min_action_delay: u64)
</code></pre>




<pre><code><b>aborts_if</b> voting_delay == 0;
<b>aborts_if</b> voting_period == 0;
<b>aborts_if</b> voting_quorum_rate == 0 || voting_quorum_rate &gt; 100;
<b>aborts_if</b> min_action_delay == 0;
<b>let</b> sender = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer);
<b>aborts_if</b> sender != <a href="Token.md#0x1_Token_SPEC_TOKEN_TEST_ADDRESS">Token::SPEC_TOKEN_TEST_ADDRESS</a>();
<b>aborts_if</b> <b>exists</b>&lt;<a href="Dao.md#0x1_Dao_DaoGlobalInfo">DaoGlobalInfo</a>&lt;TokenT&gt;&gt;(sender);
<b>aborts_if</b> <b>exists</b>&lt;<a href="Config.md#0x1_Config_Config">Config::Config</a>&lt;<a href="Dao.md#0x1_Dao_DaoConfig">DaoConfig</a>&lt;TokenT&gt;&gt;&gt;(sender);
<b>aborts_if</b> <b>exists</b>&lt;<a href="Config.md#0x1_Config_ModifyConfigCapabilityHolder">Config::ModifyConfigCapabilityHolder</a>&lt;<a href="Dao.md#0x1_Dao_DaoConfig">DaoConfig</a>&lt;TokenT&gt;&gt;&gt;(sender);
</code></pre>




<a name="0x1_Dao_RequirePluginDao"></a>


<pre><code><b>schema</b> <a href="Dao.md#0x1_Dao_RequirePluginDao">RequirePluginDao</a>&lt;TokenT&gt; {
    <b>let</b> token_addr = <a href="Token.md#0x1_Token_SPEC_TOKEN_TEST_ADDRESS">Token::SPEC_TOKEN_TEST_ADDRESS</a>();
    <b>aborts_if</b> !<b>exists</b>&lt;<a href="Dao.md#0x1_Dao_DaoGlobalInfo">DaoGlobalInfo</a>&lt;TokenT&gt;&gt;(token_addr);
    <b>aborts_if</b> !<b>exists</b>&lt;<a href="Config.md#0x1_Config_Config">Config::Config</a>&lt;<a href="Dao.md#0x1_Dao_DaoConfig">DaoConfig</a>&lt;TokenT&gt;&gt;&gt;(token_addr);
}
</code></pre>




<a name="0x1_Dao_AbortIfDaoInfoNotExist"></a>


<pre><code><b>schema</b> <a href="Dao.md#0x1_Dao_AbortIfDaoInfoNotExist">AbortIfDaoInfoNotExist</a>&lt;TokenT&gt; {
    <b>let</b> token_addr = <a href="Token.md#0x1_Token_SPEC_TOKEN_TEST_ADDRESS">Token::SPEC_TOKEN_TEST_ADDRESS</a>();
    <b>aborts_if</b> !<b>exists</b>&lt;<a href="Dao.md#0x1_Dao_DaoGlobalInfo">DaoGlobalInfo</a>&lt;TokenT&gt;&gt;(token_addr);
}
</code></pre>




<a name="0x1_Dao_AbortIfDaoConfigNotExist"></a>


<pre><code><b>schema</b> <a href="Dao.md#0x1_Dao_AbortIfDaoConfigNotExist">AbortIfDaoConfigNotExist</a>&lt;TokenT&gt; {
    <b>let</b> token_addr = <a href="Token.md#0x1_Token_SPEC_TOKEN_TEST_ADDRESS">Token::SPEC_TOKEN_TEST_ADDRESS</a>();
    <b>aborts_if</b> !<b>exists</b>&lt;<a href="Config.md#0x1_Config_Config">Config::Config</a>&lt;<a href="Dao.md#0x1_Dao_DaoConfig">DaoConfig</a>&lt;TokenT&gt;&gt;&gt;(token_addr);
}
</code></pre>




<a name="0x1_Dao_AbortIfTimestampNotExist"></a>


<pre><code><b>schema</b> <a href="Dao.md#0x1_Dao_AbortIfTimestampNotExist">AbortIfTimestampNotExist</a> {
    <b>aborts_if</b> !<b>exists</b>&lt;<a href="Timestamp.md#0x1_Timestamp_CurrentTimeMilliseconds">Timestamp::CurrentTimeMilliseconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
}
</code></pre>




<pre><code><b>apply</b>
    <a href="Dao.md#0x1_Dao_AbortIfDaoInfoNotExist">AbortIfDaoInfoNotExist</a>&lt;TokenT&gt;
<b>to</b>
    <a href="Dao.md#0x1_Dao_generate_next_proposal_id">generate_next_proposal_id</a>&lt;TokenT&gt;;
<b>apply</b>
    <a href="Dao.md#0x1_Dao_AbortIfDaoConfigNotExist">AbortIfDaoConfigNotExist</a>&lt;TokenT&gt;
<b>to</b>
    <a href="Dao.md#0x1_Dao_get_config">get_config</a>&lt;TokenT&gt;,
    <a href="Dao.md#0x1_Dao_voting_delay">voting_delay</a>&lt;TokenT&gt;,
    <a href="Dao.md#0x1_Dao_voting_period">voting_period</a>&lt;TokenT&gt;,
    <a href="Dao.md#0x1_Dao_voting_quorum_rate">voting_quorum_rate</a>&lt;TokenT&gt;,
    <a href="Dao.md#0x1_Dao_min_action_delay">min_action_delay</a>&lt;TokenT&gt;,
    <a href="Dao.md#0x1_Dao_quorum_votes">quorum_votes</a>&lt;TokenT&gt;;
</code></pre>



<a name="@Specification_1_new_dao_config"></a>

### Function `new_dao_config`


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_new_dao_config">new_dao_config</a>&lt;TokenT: <b>copy</b>, drop, store&gt;(voting_delay: u64, voting_period: u64, voting_quorum_rate: u8, min_action_delay: u64): <a href="Dao.md#0x1_Dao_DaoConfig">Dao::DaoConfig</a>&lt;TokenT&gt;
</code></pre>




<pre><code><b>aborts_if</b> voting_delay == 0;
<b>aborts_if</b> voting_period == 0;
<b>aborts_if</b> voting_quorum_rate == 0 || voting_quorum_rate &gt; 100;
<b>aborts_if</b> min_action_delay == 0;
</code></pre>



<a name="@Specification_1_propose"></a>

### Function `propose`


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_propose">propose</a>&lt;TokenT: <b>copy</b>, drop, store, ActionT: <b>copy</b>, drop, store&gt;(signer: &signer, action: ActionT, action_delay: u64)
</code></pre>




<pre><code><b>pragma</b> addition_overflow_unchecked;
<b>include</b> <a href="Dao.md#0x1_Dao_AbortIfDaoConfigNotExist">AbortIfDaoConfigNotExist</a>&lt;TokenT&gt;;
<b>include</b> <a href="Dao.md#0x1_Dao_AbortIfDaoInfoNotExist">AbortIfDaoInfoNotExist</a>&lt;TokenT&gt;;
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Timestamp.md#0x1_Timestamp_CurrentTimeMilliseconds">Timestamp::CurrentTimeMilliseconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
<b>aborts_if</b> action_delay &gt; 0 && action_delay &lt; <a href="Dao.md#0x1_Dao_spec_dao_config">spec_dao_config</a>&lt;TokenT&gt;().min_action_delay;
<b>include</b> <a href="Dao.md#0x1_Dao_CheckQuorumVotes">CheckQuorumVotes</a>&lt;TokenT&gt;;
<b>let</b> sender = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer);
<b>aborts_if</b> <b>exists</b>&lt;<a href="Dao.md#0x1_Dao_Proposal">Proposal</a>&lt;TokenT, ActionT&gt;&gt;(sender);
<b>modifies</b> <b>global</b>&lt;<a href="Dao.md#0x1_Dao_DaoGlobalInfo">DaoGlobalInfo</a>&lt;TokenT&gt;&gt;(<a href="Token.md#0x1_Token_SPEC_TOKEN_TEST_ADDRESS">Token::SPEC_TOKEN_TEST_ADDRESS</a>());
<b>ensures</b> <b>exists</b>&lt;<a href="Dao.md#0x1_Dao_Proposal">Proposal</a>&lt;TokenT, ActionT&gt;&gt;(sender);
</code></pre>



<a name="@Specification_1_cast_vote"></a>

### Function `cast_vote`


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_cast_vote">cast_vote</a>&lt;TokenT: <b>copy</b>, drop, store, ActionT: <b>copy</b>, drop, store&gt;(signer: &signer, proposer_address: <b>address</b>, proposal_id: u64, stake: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenT&gt;, agree: bool)
</code></pre>




<pre><code><b>pragma</b> addition_overflow_unchecked = <b>true</b>;
<b>include</b> <a href="Dao.md#0x1_Dao_AbortIfDaoInfoNotExist">AbortIfDaoInfoNotExist</a>&lt;TokenT&gt;;
<b>let</b> expected_states = vec(<a href="Dao.md#0x1_Dao_ACTIVE">ACTIVE</a>);
<b>include</b> <a href="Dao.md#0x1_Dao_CheckProposalStates">CheckProposalStates</a>&lt;TokenT, ActionT&gt; {expected_states};
<b>let</b> sender = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer);
<b>let</b> vote_exists = <b>exists</b>&lt;<a href="Dao.md#0x1_Dao_Vote">Vote</a>&lt;TokenT&gt;&gt;(sender);
<b>include</b> vote_exists ==&gt; <a href="Dao.md#0x1_Dao_CheckVoteOnCast">CheckVoteOnCast</a>&lt;TokenT, ActionT&gt; {
    voter: sender,
    proposal_id: proposal_id,
    agree: agree,
    stake_value: stake.value,
};
<b>modifies</b> <b>global</b>&lt;<a href="Dao.md#0x1_Dao_Proposal">Proposal</a>&lt;TokenT, ActionT&gt;&gt;(proposer_address);
<b>ensures</b> !vote_exists ==&gt; <b>global</b>&lt;<a href="Dao.md#0x1_Dao_Vote">Vote</a>&lt;TokenT&gt;&gt;(sender).stake.value == stake.value;
</code></pre>



<a name="@Specification_1_do_cast_vote"></a>

### Function `do_cast_vote`


<pre><code><b>fun</b> <a href="Dao.md#0x1_Dao_do_cast_vote">do_cast_vote</a>&lt;TokenT: <b>copy</b>, drop, store, ActionT: <b>copy</b>, drop, store&gt;(proposal: &<b>mut</b> <a href="Dao.md#0x1_Dao_Proposal">Dao::Proposal</a>&lt;TokenT, ActionT&gt;, vote: &<b>mut</b> <a href="Dao.md#0x1_Dao_Vote">Dao::Vote</a>&lt;TokenT&gt;, stake: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenT&gt;)
</code></pre>




<pre><code><b>pragma</b> addition_overflow_unchecked = <b>true</b>;
<b>aborts_if</b> vote.stake.value + stake.value &gt; MAX_U128;
<b>ensures</b> vote.stake.value == <b>old</b>(vote).stake.value + stake.value;
<b>ensures</b> vote.agree ==&gt; <b>old</b>(proposal).for_votes + stake.value == proposal.for_votes;
<b>ensures</b> vote.agree ==&gt; <b>old</b>(proposal).against_votes == proposal.against_votes;
<b>ensures</b> !vote.agree ==&gt; <b>old</b>(proposal).against_votes + stake.value == proposal.against_votes;
<b>ensures</b> !vote.agree ==&gt; <b>old</b>(proposal).for_votes == proposal.for_votes;
</code></pre>



<a name="@Specification_1_change_vote"></a>

### Function `change_vote`


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_change_vote">change_vote</a>&lt;TokenT: <b>copy</b>, drop, store, ActionT: <b>copy</b>, drop, store&gt;(signer: &signer, proposer_address: <b>address</b>, proposal_id: u64, agree: bool)
</code></pre>




<pre><code><b>let</b> expected_states = vec(<a href="Dao.md#0x1_Dao_ACTIVE">ACTIVE</a>);
<b>include</b> <a href="Dao.md#0x1_Dao_CheckProposalStates">CheckProposalStates</a>&lt;TokenT, ActionT&gt;{expected_states};
<b>let</b> sender = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer);
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Dao.md#0x1_Dao_Vote">Vote</a>&lt;TokenT&gt;&gt;(sender);
<b>let</b> vote = <b>global</b>&lt;<a href="Dao.md#0x1_Dao_Vote">Vote</a>&lt;TokenT&gt;&gt;(sender);
<b>include</b> <a href="Dao.md#0x1_Dao_CheckVoteOnProposal">CheckVoteOnProposal</a>&lt;TokenT&gt;{vote, proposer_address, proposal_id};
<b>include</b> vote.agree != agree ==&gt; <a href="Dao.md#0x1_Dao_CheckChangeVote">CheckChangeVote</a>&lt;TokenT, ActionT&gt;{vote, proposer_address};
<b>ensures</b> vote.agree != agree ==&gt; vote.agree == agree;
</code></pre>



<a name="@Specification_1_do_flip_vote"></a>

### Function `do_flip_vote`


<pre><code><b>fun</b> <a href="Dao.md#0x1_Dao_do_flip_vote">do_flip_vote</a>&lt;TokenT: <b>copy</b>, drop, store, ActionT: <b>copy</b>, drop, store&gt;(my_vote: &<b>mut</b> <a href="Dao.md#0x1_Dao_Vote">Dao::Vote</a>&lt;TokenT&gt;, proposal: &<b>mut</b> <a href="Dao.md#0x1_Dao_Proposal">Dao::Proposal</a>&lt;TokenT, ActionT&gt;): u128
</code></pre>




<pre><code><b>include</b> <a href="Dao.md#0x1_Dao_CheckFlipVote">CheckFlipVote</a>&lt;TokenT, ActionT&gt;;
<b>ensures</b> my_vote.agree == !<b>old</b>(my_vote).agree;
</code></pre>



<a name="@Specification_1_revoke_vote"></a>

### Function `revoke_vote`


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_revoke_vote">revoke_vote</a>&lt;TokenT: <b>copy</b>, drop, store, ActionT: <b>copy</b>, drop, store&gt;(signer: &signer, proposer_address: <b>address</b>, proposal_id: u64, voting_power: u128): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenT&gt;
</code></pre>




<pre><code><b>include</b> <a href="Dao.md#0x1_Dao_AbortIfDaoInfoNotExist">AbortIfDaoInfoNotExist</a>&lt;TokenT&gt;;
<b>let</b> expected_states = vec(<a href="Dao.md#0x1_Dao_ACTIVE">ACTIVE</a>);
<b>include</b> <a href="Dao.md#0x1_Dao_CheckProposalStates">CheckProposalStates</a>&lt;TokenT, ActionT&gt; {expected_states};
<b>let</b> sender = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer);
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Dao.md#0x1_Dao_Vote">Vote</a>&lt;TokenT&gt;&gt;(sender);
<b>let</b> vote = <b>global</b>&lt;<a href="Dao.md#0x1_Dao_Vote">Vote</a>&lt;TokenT&gt;&gt;(sender);
<b>include</b> <a href="Dao.md#0x1_Dao_CheckVoteOnProposal">CheckVoteOnProposal</a>&lt;TokenT&gt; {vote, proposer_address, proposal_id};
<b>include</b> <a href="Dao.md#0x1_Dao_CheckRevokeVote">CheckRevokeVote</a>&lt;TokenT, ActionT&gt; {
    vote,
    proposal: <b>global</b>&lt;<a href="Dao.md#0x1_Dao_Proposal">Proposal</a>&lt;TokenT, ActionT&gt;&gt;(proposer_address),
    to_revoke: voting_power,
};
<b>modifies</b> <b>global</b>&lt;<a href="Dao.md#0x1_Dao_Vote">Vote</a>&lt;TokenT&gt;&gt;(sender);
<b>modifies</b> <b>global</b>&lt;<a href="Dao.md#0x1_Dao_Proposal">Proposal</a>&lt;TokenT, ActionT&gt;&gt;(proposer_address);
<b>modifies</b> <b>global</b>&lt;<a href="Dao.md#0x1_Dao_DaoGlobalInfo">DaoGlobalInfo</a>&lt;TokenT&gt;&gt;(<a href="Token.md#0x1_Token_SPEC_TOKEN_TEST_ADDRESS">Token::SPEC_TOKEN_TEST_ADDRESS</a>());
<b>ensures</b> <b>global</b>&lt;<a href="Dao.md#0x1_Dao_Vote">Vote</a>&lt;TokenT&gt;&gt;(sender).stake.value + result.value == <b>old</b>(<b>global</b>&lt;<a href="Dao.md#0x1_Dao_Vote">Vote</a>&lt;TokenT&gt;&gt;(sender)).stake.value;
<b>ensures</b> result.value == voting_power;
</code></pre>



<a name="@Specification_1_do_revoke_vote"></a>

### Function `do_revoke_vote`


<pre><code><b>fun</b> <a href="Dao.md#0x1_Dao_do_revoke_vote">do_revoke_vote</a>&lt;TokenT: <b>copy</b>, drop, store, ActionT: <b>copy</b>, drop, store&gt;(proposal: &<b>mut</b> <a href="Dao.md#0x1_Dao_Proposal">Dao::Proposal</a>&lt;TokenT, ActionT&gt;, vote: &<b>mut</b> <a href="Dao.md#0x1_Dao_Vote">Dao::Vote</a>&lt;TokenT&gt;, to_revoke: u128): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenT&gt;
</code></pre>




<pre><code><b>include</b> <a href="Dao.md#0x1_Dao_CheckRevokeVote">CheckRevokeVote</a>&lt;TokenT, ActionT&gt;;
<b>ensures</b> vote.agree ==&gt; <b>old</b>(proposal).for_votes == proposal.for_votes + to_revoke;
<b>ensures</b> !vote.agree ==&gt; <b>old</b>(proposal).against_votes == proposal.against_votes + to_revoke;
<b>ensures</b> result.value == to_revoke;
</code></pre>



<a name="@Specification_1_unstake_votes"></a>

### Function `unstake_votes`


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_unstake_votes">unstake_votes</a>&lt;TokenT: <b>copy</b>, drop, store, ActionT: <b>copy</b>, drop, store&gt;(signer: &signer, proposer_address: <b>address</b>, proposal_id: u64): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenT&gt;
</code></pre>




<pre><code><b>let</b> expected_states = vec(<a href="Dao.md#0x1_Dao_DEFEATED">DEFEATED</a>);
<b>let</b> expected_states1 = concat(expected_states,vec(<a href="Dao.md#0x1_Dao_AGREED">AGREED</a>));
<b>let</b> expected_states2 = concat(expected_states1,vec(<a href="Dao.md#0x1_Dao_QUEUED">QUEUED</a>));
<b>let</b> expected_states3 = concat(expected_states2,vec(<a href="Dao.md#0x1_Dao_EXECUTABLE">EXECUTABLE</a>));
<b>let</b> expected_states4 = concat(expected_states3,vec(<a href="Dao.md#0x1_Dao_EXTRACTED">EXTRACTED</a>));
<b>aborts_if</b> expected_states4[0] != <a href="Dao.md#0x1_Dao_DEFEATED">DEFEATED</a>;
<b>aborts_if</b> expected_states4[1] != <a href="Dao.md#0x1_Dao_AGREED">AGREED</a>;
<b>aborts_if</b> expected_states4[2] != <a href="Dao.md#0x1_Dao_QUEUED">QUEUED</a>;
<b>aborts_if</b> expected_states4[3] != <a href="Dao.md#0x1_Dao_EXECUTABLE">EXECUTABLE</a>;
<b>aborts_if</b> expected_states4[4] != <a href="Dao.md#0x1_Dao_EXTRACTED">EXTRACTED</a>;
<b>include</b> <a href="Dao.md#0x1_Dao_spec_proposal_exists">spec_proposal_exists</a>&lt;TokenT, ActionT&gt;(proposer_address, proposal_id) ==&gt;
            <a href="Dao.md#0x1_Dao_CheckProposalStates">CheckProposalStates</a>&lt;TokenT, ActionT&gt;{expected_states: expected_states4};
<b>let</b> sender = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer);
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Dao.md#0x1_Dao_Vote">Vote</a>&lt;TokenT&gt;&gt;(sender);
<b>let</b> vote = <b>global</b>&lt;<a href="Dao.md#0x1_Dao_Vote">Vote</a>&lt;TokenT&gt;&gt;(sender);
<b>include</b> <a href="Dao.md#0x1_Dao_CheckVoteOnProposal">CheckVoteOnProposal</a>&lt;TokenT&gt;{vote, proposer_address, proposal_id};
<b>ensures</b> !<b>exists</b>&lt;<a href="Dao.md#0x1_Dao_Vote">Vote</a>&lt;TokenT&gt;&gt;(sender);
<b>ensures</b> result.value == <b>old</b>(vote).stake.value;
</code></pre>



<a name="@Specification_1_queue_proposal_action"></a>

### Function `queue_proposal_action`


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="Dao.md#0x1_Dao_queue_proposal_action">queue_proposal_action</a>&lt;TokenT: <b>copy</b>, drop, store, ActionT: <b>copy</b>, drop, store&gt;(proposer_address: <b>address</b>, proposal_id: u64)
</code></pre>




<pre><code><b>let</b> expected_states = vec(<a href="Dao.md#0x1_Dao_AGREED">AGREED</a>);
<b>include</b> <a href="Dao.md#0x1_Dao_CheckProposalStates">CheckProposalStates</a>&lt;TokenT, ActionT&gt;{expected_states};
<b>let</b> proposal = <b>global</b>&lt;<a href="Dao.md#0x1_Dao_Proposal">Proposal</a>&lt;TokenT, ActionT&gt;&gt;(proposer_address);
<b>aborts_if</b> <a href="Timestamp.md#0x1_Timestamp_spec_now_millseconds">Timestamp::spec_now_millseconds</a>() + proposal.action_delay &gt; MAX_U64;
<b>ensures</b> proposal.eta &gt;= <a href="Timestamp.md#0x1_Timestamp_spec_now_millseconds">Timestamp::spec_now_millseconds</a>();
</code></pre>



<a name="@Specification_1_extract_proposal_action"></a>

### Function `extract_proposal_action`


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_extract_proposal_action">extract_proposal_action</a>&lt;TokenT: <b>copy</b>, drop, store, ActionT: <b>copy</b>, drop, store&gt;(proposer_address: <b>address</b>, proposal_id: u64): ActionT
</code></pre>




<pre><code><b>pragma</b> aborts_if_is_partial = <b>false</b>;
<b>let</b> expected_states = vec(<a href="Dao.md#0x1_Dao_EXECUTABLE">EXECUTABLE</a>);
<b>include</b> <a href="Dao.md#0x1_Dao_CheckProposalStates">CheckProposalStates</a>&lt;TokenT, ActionT&gt;{expected_states};
<b>modifies</b> <b>global</b>&lt;<a href="Dao.md#0x1_Dao_Proposal">Proposal</a>&lt;TokenT, ActionT&gt;&gt;(proposer_address);
<b>ensures</b> <a href="Option.md#0x1_Option_is_none">Option::is_none</a>(<b>global</b>&lt;<a href="Dao.md#0x1_Dao_Proposal">Proposal</a>&lt;TokenT, ActionT&gt;&gt;(proposer_address).action);
</code></pre>



<a name="@Specification_1_destroy_terminated_proposal"></a>

### Function `destroy_terminated_proposal`


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="Dao.md#0x1_Dao_destroy_terminated_proposal">destroy_terminated_proposal</a>&lt;TokenT: <b>copy</b>, drop, store, ActionT: <b>copy</b>, drop, store&gt;(proposer_address: <b>address</b>, proposal_id: u64)
</code></pre>




<pre><code><b>let</b> expected_states = concat(vec(<a href="Dao.md#0x1_Dao_DEFEATED">DEFEATED</a>), vec(<a href="Dao.md#0x1_Dao_EXTRACTED">EXTRACTED</a>));
<b>aborts_if</b> len(expected_states) != 2;
<b>aborts_if</b> expected_states[0] != <a href="Dao.md#0x1_Dao_DEFEATED">DEFEATED</a>;
<b>aborts_if</b> expected_states[1] != <a href="Dao.md#0x1_Dao_EXTRACTED">EXTRACTED</a>;
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Dao.md#0x1_Dao_Proposal">Proposal</a>&lt;TokenT, ActionT&gt;&gt;(proposer_address);
<b>let</b> proposal = <b>global</b>&lt;<a href="Dao.md#0x1_Dao_Proposal">Proposal</a>&lt;TokenT, ActionT&gt;&gt;(proposer_address);
<b>aborts_if</b> proposal.id != proposal_id;
<b>include</b> <a href="Dao.md#0x1_Dao_AbortIfTimestampNotExist">AbortIfTimestampNotExist</a>;
<b>let</b> current_time = <a href="Timestamp.md#0x1_Timestamp_spec_now_millseconds">Timestamp::spec_now_millseconds</a>();
<b>let</b> state = <a href="Dao.md#0x1_Dao_do_proposal_state">do_proposal_state</a>(proposal, current_time);
<b>aborts_if</b> (<b>forall</b> s in expected_states : s != state);
<b>aborts_if</b> state == <a href="Dao.md#0x1_Dao_DEFEATED">DEFEATED</a> && <a href="Option.md#0x1_Option_is_none">Option::is_none</a>(<b>global</b>&lt;<a href="Dao.md#0x1_Dao_Proposal">Proposal</a>&lt;TokenT, ActionT&gt;&gt;(proposer_address).action);
<b>aborts_if</b> state == <a href="Dao.md#0x1_Dao_EXTRACTED">EXTRACTED</a> && <a href="Option.md#0x1_Option_is_some">Option::is_some</a>(<b>global</b>&lt;<a href="Dao.md#0x1_Dao_Proposal">Proposal</a>&lt;TokenT, ActionT&gt;&gt;(proposer_address).action);
<b>modifies</b> <b>global</b>&lt;<a href="Dao.md#0x1_Dao_Proposal">Proposal</a>&lt;TokenT, ActionT&gt;&gt;(proposer_address);
</code></pre>



<a name="@Specification_1_proposal_exists"></a>

### Function `proposal_exists`


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_proposal_exists">proposal_exists</a>&lt;TokenT: <b>copy</b>, drop, store, ActionT: <b>copy</b>, drop, store&gt;(proposer_address: <b>address</b>, proposal_id: u64): bool
</code></pre>




<pre><code><b>ensures</b> <b>exists</b>&lt;<a href="Dao.md#0x1_Dao_Proposal">Proposal</a>&lt;TokenT, ActionT&gt;&gt;(proposer_address) &&
            <b>borrow_global</b>&lt;<a href="Dao.md#0x1_Dao_Proposal">Proposal</a>&lt;TokenT, ActionT&gt;&gt;(proposer_address).id == proposal_id ==&gt;
            result;
</code></pre>




<a name="0x1_Dao_spec_proposal_exists"></a>


<pre><code><b>fun</b> <a href="Dao.md#0x1_Dao_spec_proposal_exists">spec_proposal_exists</a>&lt;TokenT: <b>copy</b> + drop + store, ActionT: <b>copy</b> + drop + store&gt;(
   proposer_address: <b>address</b>,
   proposal_id: u64,
): bool {
   <b>if</b> (<b>exists</b>&lt;<a href="Dao.md#0x1_Dao_Proposal">Proposal</a>&lt;TokenT, ActionT&gt;&gt;(proposer_address)) {
       <b>let</b> proposal = <b>global</b>&lt;<a href="Dao.md#0x1_Dao_Proposal">Proposal</a>&lt;TokenT, ActionT&gt;&gt;(proposer_address);
       proposal.id == proposal_id
   } <b>else</b> {
       <b>false</b>
   }
}
</code></pre>



<a name="@Specification_1_proposal_state"></a>

### Function `proposal_state`


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_proposal_state">proposal_state</a>&lt;TokenT: <b>copy</b>, drop, store, ActionT: <b>copy</b>, drop, store&gt;(proposer_address: <b>address</b>, proposal_id: u64): u8
</code></pre>




<pre><code><b>include</b> <a href="Dao.md#0x1_Dao_AbortIfTimestampNotExist">AbortIfTimestampNotExist</a>;
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Timestamp.md#0x1_Timestamp_CurrentTimeMilliseconds">Timestamp::CurrentTimeMilliseconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Dao.md#0x1_Dao_Proposal">Proposal</a>&lt;TokenT, ActionT&gt;&gt;(proposer_address);
<b>let</b> proposal = <b>global</b>&lt;<a href="Dao.md#0x1_Dao_Proposal">Proposal</a>&lt;TokenT, ActionT&gt;&gt;(proposer_address);
<b>aborts_if</b> proposal.id != proposal_id;
</code></pre>



<a name="@Specification_1_proposal_info"></a>

### Function `proposal_info`


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_proposal_info">proposal_info</a>&lt;TokenT: <b>copy</b>, drop, store, ActionT: <b>copy</b>, drop, store&gt;(proposer_address: <b>address</b>): (u64, u64, u64, u128, u128)
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Dao.md#0x1_Dao_Proposal">Proposal</a>&lt;TokenT, ActionT&gt;&gt;(proposer_address);
</code></pre>



<a name="@Specification_1_vote_of"></a>

### Function `vote_of`


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_vote_of">vote_of</a>&lt;TokenT: <b>copy</b>, drop, store&gt;(voter: <b>address</b>, proposer_address: <b>address</b>, proposal_id: u64): (bool, u128)
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Dao.md#0x1_Dao_Vote">Vote</a>&lt;TokenT&gt;&gt;(voter);
<b>let</b> vote = <b>global</b>&lt;<a href="Dao.md#0x1_Dao_Vote">Vote</a>&lt;TokenT&gt;&gt;(voter);
<b>include</b> <a href="Dao.md#0x1_Dao_CheckVoteOnProposal">CheckVoteOnProposal</a>&lt;TokenT&gt;{vote, proposer_address, proposal_id};
</code></pre>



<a name="@Specification_1_generate_next_proposal_id"></a>

### Function `generate_next_proposal_id`


<pre><code><b>fun</b> <a href="Dao.md#0x1_Dao_generate_next_proposal_id">generate_next_proposal_id</a>&lt;TokenT: store&gt;(): u64
</code></pre>




<pre><code><b>pragma</b> addition_overflow_unchecked;
<b>modifies</b> <b>global</b>&lt;<a href="Dao.md#0x1_Dao_DaoGlobalInfo">DaoGlobalInfo</a>&lt;TokenT&gt;&gt;(<a href="Token.md#0x1_Token_SPEC_TOKEN_TEST_ADDRESS">Token::SPEC_TOKEN_TEST_ADDRESS</a>());
<b>ensures</b>
    <b>global</b>&lt;<a href="Dao.md#0x1_Dao_DaoGlobalInfo">DaoGlobalInfo</a>&lt;TokenT&gt;&gt;(<a href="Token.md#0x1_Token_SPEC_TOKEN_TEST_ADDRESS">Token::SPEC_TOKEN_TEST_ADDRESS</a>()).next_proposal_id ==
    <b>old</b>(<b>global</b>&lt;<a href="Dao.md#0x1_Dao_DaoGlobalInfo">DaoGlobalInfo</a>&lt;TokenT&gt;&gt;(<a href="Token.md#0x1_Token_SPEC_TOKEN_TEST_ADDRESS">Token::SPEC_TOKEN_TEST_ADDRESS</a>()).next_proposal_id) + 1;
<b>ensures</b> result == <b>old</b>(<b>global</b>&lt;<a href="Dao.md#0x1_Dao_DaoGlobalInfo">DaoGlobalInfo</a>&lt;TokenT&gt;&gt;(<a href="Token.md#0x1_Token_SPEC_TOKEN_TEST_ADDRESS">Token::SPEC_TOKEN_TEST_ADDRESS</a>()).next_proposal_id);
</code></pre>



<a name="@Specification_1_voting_delay"></a>

### Function `voting_delay`


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_voting_delay">voting_delay</a>&lt;TokenT: <b>copy</b>, drop, store&gt;(): u64
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="@Specification_1_voting_period"></a>

### Function `voting_period`


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_voting_period">voting_period</a>&lt;TokenT: <b>copy</b>, drop, store&gt;(): u64
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="@Specification_1_quorum_votes"></a>

### Function `quorum_votes`


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_quorum_votes">quorum_votes</a>&lt;TokenT: <b>copy</b>, drop, store&gt;(): u128
</code></pre>




<pre><code><b>include</b> <a href="Dao.md#0x1_Dao_CheckQuorumVotes">CheckQuorumVotes</a>&lt;TokenT&gt;;
</code></pre>




<a name="0x1_Dao_spec_quorum_votes"></a>


<pre><code><b>fun</b> <a href="Dao.md#0x1_Dao_spec_quorum_votes">spec_quorum_votes</a>&lt;TokenT: <b>copy</b> + drop + store&gt;(): u128 {
   <b>let</b> supply = <a href="Token.md#0x1_Token_spec_abstract_total_value">Token::spec_abstract_total_value</a>&lt;TokenT&gt;() - <a href="Treasury.md#0x1_Treasury_spec_balance">Treasury::spec_balance</a>&lt;TokenT&gt;();
   supply * <a href="Dao.md#0x1_Dao_spec_dao_config">spec_dao_config</a>&lt;TokenT&gt;().voting_quorum_rate / 100
}
</code></pre>



<a name="@Specification_1_voting_quorum_rate"></a>

### Function `voting_quorum_rate`


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_voting_quorum_rate">voting_quorum_rate</a>&lt;TokenT: <b>copy</b>, drop, store&gt;(): u8
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
<b>ensures</b> result == <b>global</b>&lt;<a href="Config.md#0x1_Config_Config">Config::Config</a>&lt;<a href="Dao.md#0x1_Dao_DaoConfig">DaoConfig</a>&lt;TokenT&gt;&gt;&gt;((<a href="Token.md#0x1_Token_SPEC_TOKEN_TEST_ADDRESS">Token::SPEC_TOKEN_TEST_ADDRESS</a>())).payload.voting_quorum_rate;
</code></pre>



<a name="@Specification_1_min_action_delay"></a>

### Function `min_action_delay`


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_min_action_delay">min_action_delay</a>&lt;TokenT: <b>copy</b>, drop, store&gt;(): u64
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
<b>ensures</b> result == <a href="Dao.md#0x1_Dao_spec_dao_config">spec_dao_config</a>&lt;TokenT&gt;().min_action_delay;
</code></pre>



<a name="@Specification_1_get_config"></a>

### Function `get_config`


<pre><code><b>fun</b> <a href="Dao.md#0x1_Dao_get_config">get_config</a>&lt;TokenT: <b>copy</b>, drop, store&gt;(): <a href="Dao.md#0x1_Dao_DaoConfig">Dao::DaoConfig</a>&lt;TokenT&gt;
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
<b>ensures</b> result == <b>global</b>&lt;<a href="Config.md#0x1_Config_Config">Config::Config</a>&lt;<a href="Dao.md#0x1_Dao_DaoConfig">DaoConfig</a>&lt;TokenT&gt;&gt;&gt;((<a href="Token.md#0x1_Token_SPEC_TOKEN_TEST_ADDRESS">Token::SPEC_TOKEN_TEST_ADDRESS</a>())).payload;
</code></pre>




<a name="0x1_Dao_spec_dao_config"></a>


<pre><code><b>fun</b> <a href="Dao.md#0x1_Dao_spec_dao_config">spec_dao_config</a>&lt;TokenT: <b>copy</b> + drop + store&gt;(): <a href="Dao.md#0x1_Dao_DaoConfig">DaoConfig</a>&lt;TokenT&gt; {
   <b>global</b>&lt;<a href="Config.md#0x1_Config_Config">Config::Config</a>&lt;<a href="Dao.md#0x1_Dao_DaoConfig">DaoConfig</a>&lt;TokenT&gt;&gt;&gt;((<a href="Token.md#0x1_Token_SPEC_TOKEN_TEST_ADDRESS">Token::SPEC_TOKEN_TEST_ADDRESS</a>())).payload
}
</code></pre>




<a name="0x1_Dao_CheckModifyConfigWithCap"></a>


<pre><code><b>schema</b> <a href="Dao.md#0x1_Dao_CheckModifyConfigWithCap">CheckModifyConfigWithCap</a>&lt;TokenT&gt; {
    cap: <a href="Config.md#0x1_Config_ModifyConfigCapability">Config::ModifyConfigCapability</a>&lt;<a href="Dao.md#0x1_Dao_DaoConfig">DaoConfig</a>&lt;TokenT&gt;&gt;;
    <b>aborts_if</b> cap.account_address != <a href="Token.md#0x1_Token_SPEC_TOKEN_TEST_ADDRESS">Token::SPEC_TOKEN_TEST_ADDRESS</a>();
    <b>aborts_if</b> !<b>exists</b>&lt;<a href="Config.md#0x1_Config_Config">Config::Config</a>&lt;<a href="Dao.md#0x1_Dao_DaoConfig">DaoConfig</a>&lt;TokenT&gt;&gt;&gt;(cap.account_address);
}
</code></pre>



<a name="@Specification_1_modify_dao_config"></a>

### Function `modify_dao_config`


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_modify_dao_config">modify_dao_config</a>&lt;TokenT: <b>copy</b>, drop, store&gt;(cap: &<b>mut</b> <a href="Config.md#0x1_Config_ModifyConfigCapability">Config::ModifyConfigCapability</a>&lt;<a href="Dao.md#0x1_Dao_DaoConfig">Dao::DaoConfig</a>&lt;TokenT&gt;&gt;, voting_delay: u64, voting_period: u64, voting_quorum_rate: u8, min_action_delay: u64)
</code></pre>




<pre><code><b>include</b> <a href="Dao.md#0x1_Dao_CheckModifyConfigWithCap">CheckModifyConfigWithCap</a>&lt;TokenT&gt;;
<b>aborts_if</b> voting_quorum_rate &gt; 0 && voting_quorum_rate &gt; 100;
</code></pre>



<a name="@Specification_1_set_voting_delay"></a>

### Function `set_voting_delay`


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_set_voting_delay">set_voting_delay</a>&lt;TokenT: <b>copy</b>, drop, store&gt;(cap: &<b>mut</b> <a href="Config.md#0x1_Config_ModifyConfigCapability">Config::ModifyConfigCapability</a>&lt;<a href="Dao.md#0x1_Dao_DaoConfig">Dao::DaoConfig</a>&lt;TokenT&gt;&gt;, value: u64)
</code></pre>




<pre><code><b>include</b> <a href="Dao.md#0x1_Dao_CheckModifyConfigWithCap">CheckModifyConfigWithCap</a>&lt;TokenT&gt;;
<b>aborts_if</b> value == 0;
</code></pre>



<a name="@Specification_1_set_voting_period"></a>

### Function `set_voting_period`


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_set_voting_period">set_voting_period</a>&lt;TokenT: <b>copy</b>, drop, store&gt;(cap: &<b>mut</b> <a href="Config.md#0x1_Config_ModifyConfigCapability">Config::ModifyConfigCapability</a>&lt;<a href="Dao.md#0x1_Dao_DaoConfig">Dao::DaoConfig</a>&lt;TokenT&gt;&gt;, value: u64)
</code></pre>




<pre><code><b>include</b> <a href="Dao.md#0x1_Dao_CheckModifyConfigWithCap">CheckModifyConfigWithCap</a>&lt;TokenT&gt;;
<b>aborts_if</b> value == 0;
</code></pre>



<a name="@Specification_1_set_voting_quorum_rate"></a>

### Function `set_voting_quorum_rate`


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_set_voting_quorum_rate">set_voting_quorum_rate</a>&lt;TokenT: <b>copy</b>, drop, store&gt;(cap: &<b>mut</b> <a href="Config.md#0x1_Config_ModifyConfigCapability">Config::ModifyConfigCapability</a>&lt;<a href="Dao.md#0x1_Dao_DaoConfig">Dao::DaoConfig</a>&lt;TokenT&gt;&gt;, value: u8)
</code></pre>




<pre><code><b>aborts_if</b> !(value &gt; 0 && value &lt;= 100);
<b>include</b> <a href="Dao.md#0x1_Dao_CheckModifyConfigWithCap">CheckModifyConfigWithCap</a>&lt;TokenT&gt;;
</code></pre>



<a name="@Specification_1_set_min_action_delay"></a>

### Function `set_min_action_delay`


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_set_min_action_delay">set_min_action_delay</a>&lt;TokenT: <b>copy</b>, drop, store&gt;(cap: &<b>mut</b> <a href="Config.md#0x1_Config_ModifyConfigCapability">Config::ModifyConfigCapability</a>&lt;<a href="Dao.md#0x1_Dao_DaoConfig">Dao::DaoConfig</a>&lt;TokenT&gt;&gt;, value: u64)
</code></pre>




<pre><code><b>aborts_if</b> value == 0;
<b>include</b> <a href="Dao.md#0x1_Dao_CheckModifyConfigWithCap">CheckModifyConfigWithCap</a>&lt;TokenT&gt;;
</code></pre>
