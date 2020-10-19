
<a name="0x1_Dao"></a>

# Module `0x1::Dao`



-  [Resource `DaoGlobalInfo`](#0x1_Dao_DaoGlobalInfo)
-  [Struct `DaoConfig`](#0x1_Dao_DaoConfig)
-  [Struct `ProposalCreatedEvent`](#0x1_Dao_ProposalCreatedEvent)
-  [Struct `VoteChangedEvent`](#0x1_Dao_VoteChangedEvent)
-  [Resource `Proposal`](#0x1_Dao_Proposal)
-  [Resource `Vote`](#0x1_Dao_Vote)
-  [Constants](#@Constants_0)
-  [Function `default_min_action_delay`](#0x1_Dao_default_min_action_delay)
-  [Function `default_voting_delay`](#0x1_Dao_default_voting_delay)
-  [Function `default_voting_period`](#0x1_Dao_default_voting_period)
-  [Function `default_voting_quorum_rate`](#0x1_Dao_default_voting_quorum_rate)
-  [Function `plugin`](#0x1_Dao_plugin)
-  [Function `new_dao_config`](#0x1_Dao_new_dao_config)
-  [Function `propose`](#0x1_Dao_propose)
-  [Function `cast_vote`](#0x1_Dao_cast_vote)
-  [Function `change_vote`](#0x1_Dao_change_vote)
-  [Function `revoke_vote`](#0x1_Dao_revoke_vote)
-  [Function `proposal_exists`](#0x1_Dao_proposal_exists)
-  [Function `unstake_votes`](#0x1_Dao_unstake_votes)
-  [Function `queue_proposal_action`](#0x1_Dao_queue_proposal_action)
-  [Function `extract_proposal_action`](#0x1_Dao_extract_proposal_action)
-  [Function `destroy_terminated_proposal`](#0x1_Dao_destroy_terminated_proposal)
-  [Function `proposal_state`](#0x1_Dao_proposal_state)
-  [Function `proposal_info`](#0x1_Dao_proposal_info)
-  [Function `vote_of`](#0x1_Dao_vote_of)
-  [Function `quorum_votes`](#0x1_Dao_quorum_votes)
-  [Function `generate_next_proposal_id`](#0x1_Dao_generate_next_proposal_id)
-  [Function `voting_delay`](#0x1_Dao_voting_delay)
-  [Function `voting_period`](#0x1_Dao_voting_period)
-  [Function `voting_quorum_rate`](#0x1_Dao_voting_quorum_rate)
-  [Function `min_action_delay`](#0x1_Dao_min_action_delay)
-  [Function `get_config`](#0x1_Dao_get_config)
-  [Function `modify_dao_config`](#0x1_Dao_modify_dao_config)
-  [Function `set_voting_delay`](#0x1_Dao_set_voting_delay)
-  [Function `set_voting_period`](#0x1_Dao_set_voting_period)
-  [Function `set_voting_quorum_rate`](#0x1_Dao_set_voting_quorum_rate)
-  [Function `set_min_action_delay`](#0x1_Dao_set_min_action_delay)


<pre><code><b>use</b> <a href="Config.md#0x1_Config">0x1::Config</a>;
<b>use</b> <a href="Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="Event.md#0x1_Event">0x1::Event</a>;
<b>use</b> <a href="Option.md#0x1_Option">0x1::Option</a>;
<b>use</b> <a href="Signer.md#0x1_Signer">0x1::Signer</a>;
<b>use</b> <a href="Timestamp.md#0x1_Timestamp">0x1::Timestamp</a>;
<b>use</b> <a href="Token.md#0x1_Token">0x1::Token</a>;
</code></pre>



<a name="0x1_Dao_DaoGlobalInfo"></a>

## Resource `DaoGlobalInfo`



<pre><code><b>resource</b> <b>struct</b> <a href="Dao.md#0x1_Dao_DaoGlobalInfo">DaoGlobalInfo</a>&lt;<a href="Token.md#0x1_Token">Token</a>&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>next_proposal_id: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>proposal_create_event: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="Dao.md#0x1_Dao_ProposalCreatedEvent">Dao::ProposalCreatedEvent</a>&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>vote_changed_event: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="Dao.md#0x1_Dao_VoteChangedEvent">Dao::VoteChangedEvent</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_Dao_DaoConfig"></a>

## Struct `DaoConfig`



<pre><code><b>struct</b> <a href="Dao.md#0x1_Dao_DaoConfig">DaoConfig</a>&lt;TokenT: <b>copyable</b>&gt;
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


<pre><code><b>struct</b> <a href="Dao.md#0x1_Dao_ProposalCreatedEvent">ProposalCreatedEvent</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>proposal_id: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>proposer: address</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_Dao_VoteChangedEvent"></a>

## Struct `VoteChangedEvent`

emitted when user vote/revoke_vote.


<pre><code><b>struct</b> <a href="Dao.md#0x1_Dao_VoteChangedEvent">VoteChangedEvent</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>proposal_id: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>proposer: address</code>
</dt>
<dd>

</dd>
<dt>
<code>voter: address</code>
</dt>
<dd>

</dd>
<dt>
<code>agree: bool</code>
</dt>
<dd>

</dd>
<dt>
<code>vote: u128</code>
</dt>
<dd>
 latest vote of the voter.
</dd>
</dl>


</details>

<a name="0x1_Dao_Proposal"></a>

## Resource `Proposal`

TODO: support that one can propose mutli proposals.


<pre><code><b>resource</b> <b>struct</b> <a href="Dao.md#0x1_Dao_Proposal">Proposal</a>&lt;<a href="Token.md#0x1_Token">Token</a>, Action&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>id: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>proposer: address</code>
</dt>
<dd>

</dd>
<dt>
<code>start_time: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>end_time: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>for_votes: u128</code>
</dt>
<dd>

</dd>
<dt>
<code>against_votes: u128</code>
</dt>
<dd>

</dd>
<dt>
<code>eta: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>action_delay: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>action: <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;Action&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_Dao_Vote"></a>

## Resource `Vote`



<pre><code><b>resource</b> <b>struct</b> <a href="Dao.md#0x1_Dao_Vote">Vote</a>&lt;TokenT&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>proposer: address</code>
</dt>
<dd>

</dd>
<dt>
<code>id: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>stake: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenT&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>agree: bool</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_Dao_ACTIVE"></a>



<pre><code><b>const</b> <a href="Dao.md#0x1_Dao_ACTIVE">ACTIVE</a>: u8 = 2;
</code></pre>



<a name="0x1_Dao_AGREED"></a>



<pre><code><b>const</b> <a href="Dao.md#0x1_Dao_AGREED">AGREED</a>: u8 = 4;
</code></pre>



<a name="0x1_Dao_DEFAULT_MIN_ACTION_DELAY"></a>

default action_delay: 1days


<pre><code><b>const</b> <a href="Dao.md#0x1_Dao_DEFAULT_MIN_ACTION_DELAY">DEFAULT_MIN_ACTION_DELAY</a>: u64 = 86400;
</code></pre>



<a name="0x1_Dao_DEFAULT_VOTEING_QUORUM_RATE"></a>

default quorum rate: 4% of toal token supply.


<pre><code><b>const</b> <a href="Dao.md#0x1_Dao_DEFAULT_VOTEING_QUORUM_RATE">DEFAULT_VOTEING_QUORUM_RATE</a>: u8 = 4;
</code></pre>



<a name="0x1_Dao_DEFAULT_VOTING_DELAY"></a>

default voting_delay: 1hour


<pre><code><b>const</b> <a href="Dao.md#0x1_Dao_DEFAULT_VOTING_DELAY">DEFAULT_VOTING_DELAY</a>: u64 = 3600;
</code></pre>



<a name="0x1_Dao_DEFAULT_VOTING_PERIOD"></a>

default voting_period: 2days


<pre><code><b>const</b> <a href="Dao.md#0x1_Dao_DEFAULT_VOTING_PERIOD">DEFAULT_VOTING_PERIOD</a>: u64 = 172800;
</code></pre>



<a name="0x1_Dao_DEFEATED"></a>



<pre><code><b>const</b> <a href="Dao.md#0x1_Dao_DEFEATED">DEFEATED</a>: u8 = 3;
</code></pre>



<a name="0x1_Dao_ERR_ACTION_DELAY_TOO_SMALL"></a>



<pre><code><b>const</b> <a href="Dao.md#0x1_Dao_ERR_ACTION_DELAY_TOO_SMALL">ERR_ACTION_DELAY_TOO_SMALL</a>: u64 = 1402;
</code></pre>



<a name="0x1_Dao_ERR_CONFIG_PARAM_INVALID"></a>



<pre><code><b>const</b> <a href="Dao.md#0x1_Dao_ERR_CONFIG_PARAM_INVALID">ERR_CONFIG_PARAM_INVALID</a>: u64 = 1407;
</code></pre>



<a name="0x1_Dao_ERR_NOT_AUTHORIZED"></a>



<pre><code><b>const</b> <a href="Dao.md#0x1_Dao_ERR_NOT_AUTHORIZED">ERR_NOT_AUTHORIZED</a>: u64 = 1401;
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



<a name="0x1_Dao_ERR_QUROM_RATE_INVALID"></a>



<pre><code><b>const</b> <a href="Dao.md#0x1_Dao_ERR_QUROM_RATE_INVALID">ERR_QUROM_RATE_INVALID</a>: u64 = 1406;
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



<a name="0x1_Dao_default_min_action_delay"></a>

## Function `default_min_action_delay`

default min_action_delay


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_default_min_action_delay">default_min_action_delay</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_default_min_action_delay">default_min_action_delay</a>(): u64 {
    <a href="Dao.md#0x1_Dao_DEFAULT_MIN_ACTION_DELAY">DEFAULT_MIN_ACTION_DELAY</a>
}
</code></pre>



</details>

<a name="0x1_Dao_default_voting_delay"></a>

## Function `default_voting_delay`



<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_default_voting_delay">default_voting_delay</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_default_voting_delay">default_voting_delay</a>(): u64 {
    <a href="Dao.md#0x1_Dao_DEFAULT_VOTING_DELAY">DEFAULT_VOTING_DELAY</a>
}
</code></pre>



</details>

<a name="0x1_Dao_default_voting_period"></a>

## Function `default_voting_period`



<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_default_voting_period">default_voting_period</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_default_voting_period">default_voting_period</a>(): u64 {
    <a href="Dao.md#0x1_Dao_DEFAULT_VOTING_PERIOD">DEFAULT_VOTING_PERIOD</a>
}
</code></pre>



</details>

<a name="0x1_Dao_default_voting_quorum_rate"></a>

## Function `default_voting_quorum_rate`



<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_default_voting_quorum_rate">default_voting_quorum_rate</a>(): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_default_voting_quorum_rate">default_voting_quorum_rate</a>(): u8 {
    <a href="Dao.md#0x1_Dao_DEFAULT_VOTEING_QUORUM_RATE">DEFAULT_VOTEING_QUORUM_RATE</a>
}
</code></pre>



</details>

<a name="0x1_Dao_plugin"></a>

## Function `plugin`

plugin function, can only be called by token issuer.
Any token who wants to has gov functionality
can optin this moudle by call this <code>register function</code>.


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_plugin">plugin</a>&lt;TokenT: <b>copyable</b>&gt;(signer: &signer, voting_delay: u64, voting_period: u64, voting_quorum_rate: u8, min_action_delay: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_plugin">plugin</a>&lt;TokenT: <b>copyable</b>&gt;(
    signer: &signer,
    voting_delay: u64,
    voting_period: u64,
    voting_quorum_rate: u8,
    min_action_delay: u64,
) {
    // TODO: we can add a token manage cap in <a href="Token.md#0x1_Token">Token</a> <b>module</b>.
    // and only token manager can register this.
    <b>let</b> token_issuer = <a href="Token.md#0x1_Token_token_address">Token::token_address</a>&lt;TokenT&gt;();
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer) == token_issuer, <a href="Errors.md#0x1_Errors_requires_address">Errors::requires_address</a>(<a href="Dao.md#0x1_Dao_ERR_NOT_AUTHORIZED">ERR_NOT_AUTHORIZED</a>));
    // <b>let</b> proposal_id = ProposalId {next: 0};
    <b>let</b> gov_info = <a href="Dao.md#0x1_Dao_DaoGlobalInfo">DaoGlobalInfo</a>&lt;TokenT&gt; {
        next_proposal_id: 0,
        proposal_create_event: <a href="Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="Dao.md#0x1_Dao_ProposalCreatedEvent">ProposalCreatedEvent</a>&gt;(signer),
        vote_changed_event: <a href="Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="Dao.md#0x1_Dao_VoteChangedEvent">VoteChangedEvent</a>&gt;(signer),
    };
    move_to(signer, gov_info);
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


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_new_dao_config">new_dao_config</a>&lt;TokenT: <b>copyable</b>&gt;(voting_delay: u64, voting_period: u64, voting_quorum_rate: u8, min_action_delay: u64): <a href="Dao.md#0x1_Dao_DaoConfig">Dao::DaoConfig</a>&lt;TokenT&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_new_dao_config">new_dao_config</a>&lt;TokenT: <b>copyable</b>&gt;(
    voting_delay: u64,
    voting_period: u64,
    voting_quorum_rate: u8,
    min_action_delay: u64,
): <a href="Dao.md#0x1_Dao_DaoConfig">DaoConfig</a>&lt;TokenT&gt; {
    <b>assert</b>(voting_delay &gt; 0, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Dao.md#0x1_Dao_ERR_CONFIG_PARAM_INVALID">ERR_CONFIG_PARAM_INVALID</a>));
    <b>assert</b>(voting_period &gt; 0, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Dao.md#0x1_Dao_ERR_CONFIG_PARAM_INVALID">ERR_CONFIG_PARAM_INVALID</a>));
    <b>assert</b>(voting_quorum_rate &gt; 0 && <a href="Dao.md#0x1_Dao_voting_quorum_rate">voting_quorum_rate</a> &lt;= 100, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Dao.md#0x1_Dao_ERR_CONFIG_PARAM_INVALID">ERR_CONFIG_PARAM_INVALID</a>));
    <b>assert</b>(min_action_delay &gt; 0, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Dao.md#0x1_Dao_ERR_CONFIG_PARAM_INVALID">ERR_CONFIG_PARAM_INVALID</a>));
    <a href="Dao.md#0x1_Dao_DaoConfig">DaoConfig</a> { voting_delay, voting_period, voting_quorum_rate, min_action_delay }
}
</code></pre>



</details>

<a name="0x1_Dao_propose"></a>

## Function `propose`

propose a proposal.
<code>action</code>: the actual action to execute.
<code>action_delay</code>: the delay to execute after the proposal is agreed


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_propose">propose</a>&lt;TokenT: <b>copyable</b>, ActionT&gt;(signer: &signer, action: ActionT, action_delay: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_propose">propose</a>&lt;TokenT: <b>copyable</b>, ActionT&gt;(
    signer: &signer,
    action: ActionT,
    action_delay: u64,
) <b>acquires</b> <a href="Dao.md#0x1_Dao_DaoGlobalInfo">DaoGlobalInfo</a> {
    <b>if</b> (action_delay == 0) {
        action_delay = <a href="Dao.md#0x1_Dao_min_action_delay">min_action_delay</a>&lt;TokenT&gt;();
    } <b>else</b> {
        <b>assert</b>(action_delay &gt;= <a href="Dao.md#0x1_Dao_min_action_delay">min_action_delay</a>&lt;TokenT&gt;(), <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Dao.md#0x1_Dao_ERR_ACTION_DELAY_TOO_SMALL">ERR_ACTION_DELAY_TOO_SMALL</a>));
    };
    <b>let</b> proposal_id = <a href="Dao.md#0x1_Dao_generate_next_proposal_id">generate_next_proposal_id</a>&lt;TokenT&gt;();
    <b>let</b> proposer = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer);
    <b>let</b> start_time = <a href="Timestamp.md#0x1_Timestamp_now_seconds">Timestamp::now_seconds</a>() + <a href="Dao.md#0x1_Dao_voting_delay">voting_delay</a>&lt;TokenT&gt;();
    <b>let</b> proposal = <a href="Dao.md#0x1_Dao_Proposal">Proposal</a>&lt;TokenT, ActionT&gt; {
        id: proposal_id,
        proposer,
        start_time,
        end_time: start_time + <a href="Dao.md#0x1_Dao_voting_period">voting_period</a>&lt;TokenT&gt;(),
        for_votes: 0,
        against_votes: 0,
        eta: 0,
        action_delay,
        action: <a href="Option.md#0x1_Option_some">Option::some</a>(action),
    };
    move_to(signer, proposal);
    // emit event
    <b>let</b> gov_info = borrow_global_mut&lt;<a href="Dao.md#0x1_Dao_DaoGlobalInfo">DaoGlobalInfo</a>&lt;TokenT&gt;&gt;(<a href="Token.md#0x1_Token_token_address">Token::token_address</a>&lt;TokenT&gt;());
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


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_cast_vote">cast_vote</a>&lt;TokenT: <b>copyable</b>, ActionT&gt;(signer: &signer, proposer_address: address, proposal_id: u64, stake: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenT&gt;, agree: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_cast_vote">cast_vote</a>&lt;TokenT: <b>copyable</b>, ActionT&gt;(
    signer: &signer,
    proposer_address: address,
    proposal_id: u64,
    stake: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenT&gt;,
    agree: bool,
) <b>acquires</b> <a href="Dao.md#0x1_Dao_Proposal">Proposal</a>, <a href="Dao.md#0x1_Dao_DaoGlobalInfo">DaoGlobalInfo</a>, <a href="Dao.md#0x1_Dao_Vote">Vote</a> {
    {
        <b>let</b> state = <a href="Dao.md#0x1_Dao_proposal_state">proposal_state</a>&lt;TokenT, ActionT&gt;(proposer_address, proposal_id);
        // only when proposal is active, <b>use</b> can cast vote.
        <b>assert</b>(state == <a href="Dao.md#0x1_Dao_ACTIVE">ACTIVE</a>, <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="Dao.md#0x1_Dao_ERR_PROPOSAL_STATE_INVALID">ERR_PROPOSAL_STATE_INVALID</a>));
    };
    <b>let</b> proposal = borrow_global_mut&lt;<a href="Dao.md#0x1_Dao_Proposal">Proposal</a>&lt;TokenT, ActionT&gt;&gt;(proposer_address);
    <b>assert</b>(proposal.id == proposal_id, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Dao.md#0x1_Dao_ERR_PROPOSAL_ID_MISMATCH">ERR_PROPOSAL_ID_MISMATCH</a>));
    <b>let</b> stake_value = <a href="Token.md#0x1_Token_value">Token::value</a>(&stake);
    <b>let</b> total_voted = <b>if</b> (<b>exists</b>&lt;<a href="Dao.md#0x1_Dao_Vote">Vote</a>&lt;TokenT&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer))) {
        <b>let</b> my_vote = borrow_global_mut&lt;<a href="Dao.md#0x1_Dao_Vote">Vote</a>&lt;TokenT&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer));
        <b>assert</b>(my_vote.id == proposal_id, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Dao.md#0x1_Dao_ERR_PROPOSAL_ID_MISMATCH">ERR_PROPOSAL_ID_MISMATCH</a>));
        <b>assert</b>(my_vote.agree == agree, <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="Dao.md#0x1_Dao_ERR_VOTE_STATE_MISMATCH">ERR_VOTE_STATE_MISMATCH</a>));
        <a href="Token.md#0x1_Token_deposit">Token::deposit</a>(&<b>mut</b> my_vote.stake, stake);
        <a href="Token.md#0x1_Token_value">Token::value</a>(&my_vote.stake)
    } <b>else</b> {
        <b>let</b> my_vote = <a href="Dao.md#0x1_Dao_Vote">Vote</a>&lt;TokenT&gt; {
            proposer: proposer_address,
            id: proposal_id,
            stake,
            agree,
        };
        move_to(signer, my_vote);
        stake_value
    };
    <b>if</b> (agree) {
        proposal.for_votes = proposal.for_votes + stake_value;
    } <b>else</b> {
        proposal.against_votes = proposal.against_votes + stake_value;
    };
    // emit event
    <b>let</b> gov_info = borrow_global_mut&lt;<a href="Dao.md#0x1_Dao_DaoGlobalInfo">DaoGlobalInfo</a>&lt;TokenT&gt;&gt;(<a href="Token.md#0x1_Token_token_address">Token::token_address</a>&lt;TokenT&gt;());
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
}
</code></pre>



</details>

<a name="0x1_Dao_change_vote"></a>

## Function `change_vote`

Let user change their vote during the voting time.


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_change_vote">change_vote</a>&lt;TokenT: <b>copyable</b>, ActionT&gt;(signer: &signer, proposer_address: address, proposal_id: u64, agree: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_change_vote">change_vote</a>&lt;TokenT: <b>copyable</b>, ActionT&gt;(
    signer: &signer,
    proposer_address: address,
    proposal_id: u64,
    agree: bool,
) <b>acquires</b> <a href="Dao.md#0x1_Dao_Proposal">Proposal</a>, <a href="Dao.md#0x1_Dao_DaoGlobalInfo">DaoGlobalInfo</a>, <a href="Dao.md#0x1_Dao_Vote">Vote</a> {
    {
        <b>let</b> state = <a href="Dao.md#0x1_Dao_proposal_state">proposal_state</a>&lt;TokenT, ActionT&gt;(proposer_address, proposal_id);
        // only when proposal is active, <b>use</b> can change vote.
        <b>assert</b>(state == <a href="Dao.md#0x1_Dao_ACTIVE">ACTIVE</a>, <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="Dao.md#0x1_Dao_ERR_PROPOSAL_STATE_INVALID">ERR_PROPOSAL_STATE_INVALID</a>));
    };
    <b>let</b> proposal = borrow_global_mut&lt;<a href="Dao.md#0x1_Dao_Proposal">Proposal</a>&lt;TokenT, ActionT&gt;&gt;(proposer_address);
    <b>assert</b>(proposal.id == proposal_id, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Dao.md#0x1_Dao_ERR_PROPOSAL_ID_MISMATCH">ERR_PROPOSAL_ID_MISMATCH</a>));
    <b>let</b> my_vote = borrow_global_mut&lt;<a href="Dao.md#0x1_Dao_Vote">Vote</a>&lt;TokenT&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer));
    <b>assert</b>(my_vote.id == proposal_id, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Dao.md#0x1_Dao_ERR_PROPOSAL_ID_MISMATCH">ERR_PROPOSAL_ID_MISMATCH</a>));
    // flip the vote
    <b>if</b> (my_vote.agree != agree) {
        my_vote.agree = agree;
        <b>let</b> total_voted = <a href="Token.md#0x1_Token_value">Token::value</a>(&my_vote.stake);
        <b>if</b> (agree) {
            proposal.for_votes = proposal.for_votes + total_voted;
            proposal.against_votes = proposal.against_votes - total_voted;
        } <b>else</b> {
            proposal.for_votes = proposal.for_votes - total_voted;
            proposal.against_votes = proposal.against_votes + total_voted;
        };
        // emit event
        <b>let</b> gov_info = borrow_global_mut&lt;<a href="Dao.md#0x1_Dao_DaoGlobalInfo">DaoGlobalInfo</a>&lt;TokenT&gt;&gt;(<a href="Token.md#0x1_Token_token_address">Token::token_address</a>&lt;TokenT&gt;());
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

<a name="0x1_Dao_revoke_vote"></a>

## Function `revoke_vote`

Revoke some voting powers from vote on <code>proposal_id</code> of <code>proposer_address</code>.


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_revoke_vote">revoke_vote</a>&lt;TokenT: <b>copyable</b>, ActionT&gt;(signer: &signer, proposer_address: address, proposal_id: u64, voting_power: u128): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenT&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_revoke_vote">revoke_vote</a>&lt;TokenT: <b>copyable</b>, ActionT&gt;(
    signer: &signer,
    proposer_address: address,
    proposal_id: u64,
    voting_power: u128,
): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenT&gt; <b>acquires</b> <a href="Dao.md#0x1_Dao_Proposal">Proposal</a>, <a href="Dao.md#0x1_Dao_Vote">Vote</a>, <a href="Dao.md#0x1_Dao_DaoGlobalInfo">DaoGlobalInfo</a> {
    {
        <b>let</b> state = <a href="Dao.md#0x1_Dao_proposal_state">proposal_state</a>&lt;TokenT, ActionT&gt;(proposer_address, proposal_id);
        // only when proposal is active, <b>use</b> can revoke vote.
        <b>assert</b>(state == <a href="Dao.md#0x1_Dao_ACTIVE">ACTIVE</a>, <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="Dao.md#0x1_Dao_ERR_PROPOSAL_STATE_INVALID">ERR_PROPOSAL_STATE_INVALID</a>));
    };
    <b>let</b> proposal = borrow_global_mut&lt;<a href="Dao.md#0x1_Dao_Proposal">Proposal</a>&lt;TokenT, ActionT&gt;&gt;(proposer_address);
    <b>assert</b>(proposal.id == proposal_id, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Dao.md#0x1_Dao_ERR_PROPOSAL_ID_MISMATCH">ERR_PROPOSAL_ID_MISMATCH</a>));
    <b>let</b> my_vote = borrow_global_mut&lt;<a href="Dao.md#0x1_Dao_Vote">Vote</a>&lt;TokenT&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer));
    <b>assert</b>(my_vote.proposer == proposer_address, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Dao.md#0x1_Dao_ERR_PROPOSER_MISMATCH">ERR_PROPOSER_MISMATCH</a>));
    <b>assert</b>(my_vote.id == proposal_id, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Dao.md#0x1_Dao_ERR_PROPOSAL_ID_MISMATCH">ERR_PROPOSAL_ID_MISMATCH</a>));
    <b>let</b> reverted_stake = <a href="Token.md#0x1_Token_withdraw">Token::withdraw</a>(&<b>mut</b> my_vote.stake, voting_power);
    <b>if</b> (my_vote.agree) {
        proposal.for_votes = proposal.for_votes - voting_power;
    } <b>else</b> {
        proposal.against_votes = proposal.against_votes - voting_power;
    };
    // emit vote changed event
    <b>let</b> gov_info = borrow_global_mut&lt;<a href="Dao.md#0x1_Dao_DaoGlobalInfo">DaoGlobalInfo</a>&lt;TokenT&gt;&gt;(<a href="Token.md#0x1_Token_token_address">Token::token_address</a>&lt;TokenT&gt;());
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
    reverted_stake
}
</code></pre>



</details>

<a name="0x1_Dao_proposal_exists"></a>

## Function `proposal_exists`



<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_proposal_exists">proposal_exists</a>&lt;TokenT: <b>copyable</b>, ActionT&gt;(proposer_address: address, proposal_id: u64): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_proposal_exists">proposal_exists</a>&lt;TokenT: <b>copyable</b>, ActionT&gt;(
    proposer_address: address,
    proposal_id: u64,
): bool <b>acquires</b> <a href="Dao.md#0x1_Dao_Proposal">Proposal</a> {
    <b>if</b> (<b>exists</b>&lt;<a href="Dao.md#0x1_Dao_Proposal">Proposal</a>&lt;TokenT, ActionT&gt;&gt;(proposer_address)) {
        <b>let</b> proposal = borrow_global&lt;<a href="Dao.md#0x1_Dao_Proposal">Proposal</a>&lt;TokenT, ActionT&gt;&gt;(proposer_address);
        <b>if</b> (proposal.id == proposal_id) {
            <b>return</b> <b>true</b>
        };
    };
    <b>false</b>
}
</code></pre>



</details>

<a name="0x1_Dao_unstake_votes"></a>

## Function `unstake_votes`

Retrieve back my staked token voted for a proposal.


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_unstake_votes">unstake_votes</a>&lt;TokenT: <b>copyable</b>, ActionT&gt;(signer: &signer, proposer_address: address, proposal_id: u64): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenT&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_unstake_votes">unstake_votes</a>&lt;TokenT: <b>copyable</b>, ActionT&gt;(
    signer: &signer,
    proposer_address: address,
    proposal_id: u64,
): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenT&gt; <b>acquires</b> <a href="Dao.md#0x1_Dao_Proposal">Proposal</a>, <a href="Dao.md#0x1_Dao_Vote">Vote</a> {
    // only check state when proposal <b>exists</b>.
    // because proposal can be destroyed after it ends in <a href="Dao.md#0x1_Dao_DEFEATED">DEFEATED</a> or <a href="Dao.md#0x1_Dao_EXTRACTED">EXTRACTED</a> state.
    <b>if</b> (<a href="Dao.md#0x1_Dao_proposal_exists">proposal_exists</a>&lt;TokenT, ActionT&gt;(proposer_address, proposal_id)) {
        <b>let</b> state = <a href="Dao.md#0x1_Dao_proposal_state">proposal_state</a>&lt;TokenT, ActionT&gt;(proposer_address, proposal_id);
        // Only after vote period end, user can unstake his votes.
        <b>assert</b>(state &gt; <a href="Dao.md#0x1_Dao_ACTIVE">ACTIVE</a>, <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="Dao.md#0x1_Dao_ERR_PROPOSAL_STATE_INVALID">ERR_PROPOSAL_STATE_INVALID</a>));
    };
    <b>let</b> <a href="Dao.md#0x1_Dao_Vote">Vote</a> { proposer, id, stake, agree: _ } = move_from&lt;<a href="Dao.md#0x1_Dao_Vote">Vote</a>&lt;TokenT&gt;&gt;(
        <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer),
    );
    // these checks are still required.
    <b>assert</b>(proposer == proposer_address, <a href="Errors.md#0x1_Errors_requires_address">Errors::requires_address</a>(<a href="Dao.md#0x1_Dao_ERR_PROPOSER_MISMATCH">ERR_PROPOSER_MISMATCH</a>));
    <b>assert</b>(id == proposal_id, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Dao.md#0x1_Dao_ERR_PROPOSAL_ID_MISMATCH">ERR_PROPOSAL_ID_MISMATCH</a>));
    stake
}
</code></pre>



</details>

<a name="0x1_Dao_queue_proposal_action"></a>

## Function `queue_proposal_action`

queue agreed proposal to execute.


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_queue_proposal_action">queue_proposal_action</a>&lt;TokenT: <b>copyable</b>, ActionT&gt;(proposer_address: address, proposal_id: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_queue_proposal_action">queue_proposal_action</a>&lt;TokenT: <b>copyable</b>, ActionT&gt;(
    proposer_address: address,
    proposal_id: u64,
) <b>acquires</b> <a href="Dao.md#0x1_Dao_Proposal">Proposal</a> {
    // Only agreed proposal can be submitted.
    <b>assert</b>(<a href="Dao.md#0x1_Dao_proposal_state">proposal_state</a>&lt;TokenT, ActionT&gt;(proposer_address, proposal_id) == <a href="Dao.md#0x1_Dao_AGREED">AGREED</a>, 601); //todo
    <b>let</b> proposal = borrow_global_mut&lt;<a href="Dao.md#0x1_Dao_Proposal">Proposal</a>&lt;TokenT, ActionT&gt;&gt;(proposer_address);
    proposal.eta = <a href="Timestamp.md#0x1_Timestamp_now_seconds">Timestamp::now_seconds</a>() + proposal.action_delay;
}
</code></pre>



</details>

<a name="0x1_Dao_extract_proposal_action"></a>

## Function `extract_proposal_action`

extract proposal action to execute.


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_extract_proposal_action">extract_proposal_action</a>&lt;TokenT: <b>copyable</b>, ActionT&gt;(proposer_address: address, proposal_id: u64): ActionT
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_extract_proposal_action">extract_proposal_action</a>&lt;TokenT: <b>copyable</b>, ActionT&gt;(
    proposer_address: address,
    proposal_id: u64,
): ActionT <b>acquires</b> <a href="Dao.md#0x1_Dao_Proposal">Proposal</a> {
    // Only executable proposal's action can be extracted.
    <b>assert</b>(
        <a href="Dao.md#0x1_Dao_proposal_state">proposal_state</a>&lt;TokenT, ActionT&gt;(proposer_address, proposal_id) == <a href="Dao.md#0x1_Dao_EXECUTABLE">EXECUTABLE</a>,
        <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="Dao.md#0x1_Dao_ERR_PROPOSAL_STATE_INVALID">ERR_PROPOSAL_STATE_INVALID</a>),
    );
    <b>let</b> proposal = borrow_global_mut&lt;<a href="Dao.md#0x1_Dao_Proposal">Proposal</a>&lt;TokenT, ActionT&gt;&gt;(proposer_address);
    <b>let</b> action: ActionT = <a href="Option.md#0x1_Option_extract">Option::extract</a>(&<b>mut</b> proposal.action);
    action
}
</code></pre>



</details>

<a name="0x1_Dao_destroy_terminated_proposal"></a>

## Function `destroy_terminated_proposal`

remove terminated proposal from proposer


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_destroy_terminated_proposal">destroy_terminated_proposal</a>&lt;TokenT: <b>copyable</b>, ActionT&gt;(proposer_address: address, proposal_id: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_destroy_terminated_proposal">destroy_terminated_proposal</a>&lt;TokenT: <b>copyable</b>, ActionT&gt;(
    proposer_address: address,
    proposal_id: u64,
) <b>acquires</b> <a href="Dao.md#0x1_Dao_Proposal">Proposal</a> {
    <b>let</b> proposal_state = <a href="Dao.md#0x1_Dao_proposal_state">proposal_state</a>&lt;TokenT, ActionT&gt;(proposer_address, proposal_id);
    <b>assert</b>(
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
        action,
    } = move_from&lt;<a href="Dao.md#0x1_Dao_Proposal">Proposal</a>&lt;TokenT, ActionT&gt;&gt;(proposer_address);
    <a href="Option.md#0x1_Option_destroy_none">Option::destroy_none</a>(action);
}
</code></pre>



</details>

<a name="0x1_Dao_proposal_state"></a>

## Function `proposal_state`



<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_proposal_state">proposal_state</a>&lt;TokenT: <b>copyable</b>, ActionT&gt;(proposer_address: address, proposal_id: u64): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_proposal_state">proposal_state</a>&lt;TokenT: <b>copyable</b>, ActionT&gt;(
    proposer_address: address,
    proposal_id: u64,
): u8 <b>acquires</b> <a href="Dao.md#0x1_Dao_Proposal">Proposal</a> {
    <b>let</b> proposal = borrow_global&lt;<a href="Dao.md#0x1_Dao_Proposal">Proposal</a>&lt;TokenT, ActionT&gt;&gt;(proposer_address);
    <b>assert</b>(proposal.id == proposal_id, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Dao.md#0x1_Dao_ERR_PROPOSAL_ID_MISMATCH">ERR_PROPOSAL_ID_MISMATCH</a>));
    <b>let</b> current_time = <a href="Timestamp.md#0x1_Timestamp_now_seconds">Timestamp::now_seconds</a>();
    <b>if</b> (current_time &lt; proposal.start_time) {
        // Pending
        <a href="Dao.md#0x1_Dao_PENDING">PENDING</a>
    } <b>else</b> <b>if</b> (current_time &lt;= proposal.end_time) {
        // Active
        <a href="Dao.md#0x1_Dao_ACTIVE">ACTIVE</a>
    } <b>else</b> <b>if</b> (proposal.for_votes &lt;= proposal.against_votes ||
        proposal.for_votes &lt; <a href="Dao.md#0x1_Dao_quorum_votes">quorum_votes</a>&lt;TokenT&gt;()) {
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
return: (start_time, end_time, for_votes, against_votes).


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_proposal_info">proposal_info</a>&lt;TokenT: <b>copyable</b>, ActionT&gt;(proposer_address: address, proposal_id: u64): (u64, u64, u128, u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_proposal_info">proposal_info</a>&lt;TokenT: <b>copyable</b>, ActionT&gt;(
    proposer_address: address,
    proposal_id: u64,
): (u64, u64, u128, u128) <b>acquires</b> <a href="Dao.md#0x1_Dao_Proposal">Proposal</a> {
    <b>let</b> proposal = borrow_global&lt;<a href="Dao.md#0x1_Dao_Proposal">Proposal</a>&lt;TokenT, ActionT&gt;&gt;(proposer_address);
    <b>assert</b>(proposal.id == proposal_id, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Dao.md#0x1_Dao_ERR_PROPOSAL_ID_MISMATCH">ERR_PROPOSAL_ID_MISMATCH</a>));
    (proposal.start_time, proposal.end_time, proposal.for_votes, proposal.against_votes)
}
</code></pre>



</details>

<a name="0x1_Dao_vote_of"></a>

## Function `vote_of`

Get voter's vote info on proposal with <code>proposal_id</code> of <code>proposer_address</code>.


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_vote_of">vote_of</a>&lt;TokenT: <b>copyable</b>&gt;(voter: address, proposer_address: address, proposal_id: u64): (bool, u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_vote_of">vote_of</a>&lt;TokenT: <b>copyable</b>&gt;(
    voter: address,
    proposer_address: address,
    proposal_id: u64,
): (bool, u128) <b>acquires</b> <a href="Dao.md#0x1_Dao_Vote">Vote</a> {
    <b>let</b> vote = borrow_global&lt;<a href="Dao.md#0x1_Dao_Vote">Vote</a>&lt;TokenT&gt;&gt;(voter);
    <b>assert</b>(vote.proposer == proposer_address, <a href="Errors.md#0x1_Errors_requires_address">Errors::requires_address</a>(<a href="Dao.md#0x1_Dao_ERR_PROPOSER_MISMATCH">ERR_PROPOSER_MISMATCH</a>));
    <b>assert</b>(vote.id == proposal_id, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Dao.md#0x1_Dao_ERR_PROPOSAL_ID_MISMATCH">ERR_PROPOSAL_ID_MISMATCH</a>));
    (vote.agree, <a href="Token.md#0x1_Token_value">Token::value</a>(&vote.stake))
}
</code></pre>



</details>

<a name="0x1_Dao_quorum_votes"></a>

## Function `quorum_votes`

Quorum votes to make proposal pass.


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_quorum_votes">quorum_votes</a>&lt;TokenT: <b>copyable</b>&gt;(): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_quorum_votes">quorum_votes</a>&lt;TokenT: <b>copyable</b>&gt;(): u128 {
    <b>let</b> supply = <a href="Token.md#0x1_Token_market_cap">Token::market_cap</a>&lt;TokenT&gt;();
    supply / 100 * (<a href="Dao.md#0x1_Dao_voting_quorum_rate">voting_quorum_rate</a>&lt;TokenT&gt;() <b>as</b> u128)
}
</code></pre>



</details>

<a name="0x1_Dao_generate_next_proposal_id"></a>

## Function `generate_next_proposal_id`



<pre><code><b>fun</b> <a href="Dao.md#0x1_Dao_generate_next_proposal_id">generate_next_proposal_id</a>&lt;TokenT&gt;(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="Dao.md#0x1_Dao_generate_next_proposal_id">generate_next_proposal_id</a>&lt;TokenT&gt;(): u64 <b>acquires</b> <a href="Dao.md#0x1_Dao_DaoGlobalInfo">DaoGlobalInfo</a> {
    <b>let</b> gov_info = borrow_global_mut&lt;<a href="Dao.md#0x1_Dao_DaoGlobalInfo">DaoGlobalInfo</a>&lt;TokenT&gt;&gt;(<a href="Token.md#0x1_Token_token_address">Token::token_address</a>&lt;TokenT&gt;());
    <b>let</b> proposal_id = gov_info.next_proposal_id;
    gov_info.next_proposal_id = proposal_id + 1;
    proposal_id
}
</code></pre>



</details>

<a name="0x1_Dao_voting_delay"></a>

## Function `voting_delay`



<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_voting_delay">voting_delay</a>&lt;TokenT: <b>copyable</b>&gt;(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_voting_delay">voting_delay</a>&lt;TokenT: <b>copyable</b>&gt;(): u64 {
    <a href="Dao.md#0x1_Dao_get_config">get_config</a>&lt;TokenT&gt;().voting_delay
}
</code></pre>



</details>

<a name="0x1_Dao_voting_period"></a>

## Function `voting_period`



<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_voting_period">voting_period</a>&lt;TokenT: <b>copyable</b>&gt;(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_voting_period">voting_period</a>&lt;TokenT: <b>copyable</b>&gt;(): u64 {
    <a href="Dao.md#0x1_Dao_get_config">get_config</a>&lt;TokenT&gt;().voting_period
}
</code></pre>



</details>

<a name="0x1_Dao_voting_quorum_rate"></a>

## Function `voting_quorum_rate`



<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_voting_quorum_rate">voting_quorum_rate</a>&lt;TokenT: <b>copyable</b>&gt;(): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_voting_quorum_rate">voting_quorum_rate</a>&lt;TokenT: <b>copyable</b>&gt;(): u8 {
    <a href="Dao.md#0x1_Dao_get_config">get_config</a>&lt;TokenT&gt;().voting_quorum_rate
}
</code></pre>



</details>

<a name="0x1_Dao_min_action_delay"></a>

## Function `min_action_delay`



<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_min_action_delay">min_action_delay</a>&lt;TokenT: <b>copyable</b>&gt;(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_min_action_delay">min_action_delay</a>&lt;TokenT: <b>copyable</b>&gt;(): u64 {
    <a href="Dao.md#0x1_Dao_get_config">get_config</a>&lt;TokenT&gt;().min_action_delay
}
</code></pre>



</details>

<a name="0x1_Dao_get_config"></a>

## Function `get_config`



<pre><code><b>fun</b> <a href="Dao.md#0x1_Dao_get_config">get_config</a>&lt;TokenT: <b>copyable</b>&gt;(): <a href="Dao.md#0x1_Dao_DaoConfig">Dao::DaoConfig</a>&lt;TokenT&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="Dao.md#0x1_Dao_get_config">get_config</a>&lt;TokenT: <b>copyable</b>&gt;(): <a href="Dao.md#0x1_Dao_DaoConfig">DaoConfig</a>&lt;TokenT&gt; {
    <b>let</b> token_issuer = <a href="Token.md#0x1_Token_token_address">Token::token_address</a>&lt;TokenT&gt;();
    <a href="Config.md#0x1_Config_get_by_address">Config::get_by_address</a>&lt;<a href="Dao.md#0x1_Dao_DaoConfig">DaoConfig</a>&lt;TokenT&gt;&gt;(token_issuer)
}
</code></pre>



</details>

<a name="0x1_Dao_modify_dao_config"></a>

## Function `modify_dao_config`

update function
TODO: cap should not be mut to set data.


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_modify_dao_config">modify_dao_config</a>&lt;TokenT: <b>copyable</b>&gt;(cap: &<b>mut</b> <a href="Config.md#0x1_Config_ModifyConfigCapability">Config::ModifyConfigCapability</a>&lt;<a href="Dao.md#0x1_Dao_DaoConfig">Dao::DaoConfig</a>&lt;TokenT&gt;&gt;, voting_delay: u64, voting_period: u64, voting_quorum_rate: u8, min_action_delay: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_modify_dao_config">modify_dao_config</a>&lt;TokenT: <b>copyable</b>&gt;(
    cap: &<b>mut</b> <a href="Config.md#0x1_Config_ModifyConfigCapability">Config::ModifyConfigCapability</a>&lt;<a href="Dao.md#0x1_Dao_DaoConfig">DaoConfig</a>&lt;TokenT&gt;&gt;,
    voting_delay: u64,
    voting_period: u64,
    voting_quorum_rate: u8,
    min_action_delay: u64,
) {
    <b>let</b> config = <a href="Dao.md#0x1_Dao_get_config">get_config</a>&lt;TokenT&gt;();
    <b>if</b> (voting_period &gt; 0) {
        config.voting_period = voting_period;
    };
    <b>if</b> (voting_delay &gt; 0) {
        config.voting_delay = voting_delay;
    };
    <b>if</b> (voting_quorum_rate &gt; 0) {
        <b>assert</b>(<a href="Dao.md#0x1_Dao_voting_quorum_rate">voting_quorum_rate</a> &lt;= 100, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Dao.md#0x1_Dao_ERR_QUROM_RATE_INVALID">ERR_QUROM_RATE_INVALID</a>));
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



<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_set_voting_delay">set_voting_delay</a>&lt;TokenT: <b>copyable</b>&gt;(cap: &<b>mut</b> <a href="Config.md#0x1_Config_ModifyConfigCapability">Config::ModifyConfigCapability</a>&lt;<a href="Dao.md#0x1_Dao_DaoConfig">Dao::DaoConfig</a>&lt;TokenT&gt;&gt;, value: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_set_voting_delay">set_voting_delay</a>&lt;TokenT: <b>copyable</b>&gt;(
    cap: &<b>mut</b> <a href="Config.md#0x1_Config_ModifyConfigCapability">Config::ModifyConfigCapability</a>&lt;<a href="Dao.md#0x1_Dao_DaoConfig">DaoConfig</a>&lt;TokenT&gt;&gt;,
    value: u64,
) {
    <b>assert</b>(value &gt; 0, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Dao.md#0x1_Dao_ERR_CONFIG_PARAM_INVALID">ERR_CONFIG_PARAM_INVALID</a>));
    <b>let</b> config = <a href="Dao.md#0x1_Dao_get_config">get_config</a>&lt;TokenT&gt;();
    config.voting_delay = value;
    <a href="Config.md#0x1_Config_set_with_capability">Config::set_with_capability</a>&lt;<a href="Dao.md#0x1_Dao_DaoConfig">DaoConfig</a>&lt;TokenT&gt;&gt;(cap, config);
}
</code></pre>



</details>

<a name="0x1_Dao_set_voting_period"></a>

## Function `set_voting_period`



<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_set_voting_period">set_voting_period</a>&lt;TokenT: <b>copyable</b>&gt;(cap: &<b>mut</b> <a href="Config.md#0x1_Config_ModifyConfigCapability">Config::ModifyConfigCapability</a>&lt;<a href="Dao.md#0x1_Dao_DaoConfig">Dao::DaoConfig</a>&lt;TokenT&gt;&gt;, value: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_set_voting_period">set_voting_period</a>&lt;TokenT: <b>copyable</b>&gt;(
    cap: &<b>mut</b> <a href="Config.md#0x1_Config_ModifyConfigCapability">Config::ModifyConfigCapability</a>&lt;<a href="Dao.md#0x1_Dao_DaoConfig">DaoConfig</a>&lt;TokenT&gt;&gt;,
    value: u64,
) {
    <b>assert</b>(value &gt; 0, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Dao.md#0x1_Dao_ERR_CONFIG_PARAM_INVALID">ERR_CONFIG_PARAM_INVALID</a>));
    <b>let</b> config = <a href="Dao.md#0x1_Dao_get_config">get_config</a>&lt;TokenT&gt;();
    config.voting_period = value;
    <a href="Config.md#0x1_Config_set_with_capability">Config::set_with_capability</a>&lt;<a href="Dao.md#0x1_Dao_DaoConfig">DaoConfig</a>&lt;TokenT&gt;&gt;(cap, config);
}
</code></pre>



</details>

<a name="0x1_Dao_set_voting_quorum_rate"></a>

## Function `set_voting_quorum_rate`



<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_set_voting_quorum_rate">set_voting_quorum_rate</a>&lt;TokenT: <b>copyable</b>&gt;(cap: &<b>mut</b> <a href="Config.md#0x1_Config_ModifyConfigCapability">Config::ModifyConfigCapability</a>&lt;<a href="Dao.md#0x1_Dao_DaoConfig">Dao::DaoConfig</a>&lt;TokenT&gt;&gt;, value: u8)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_set_voting_quorum_rate">set_voting_quorum_rate</a>&lt;TokenT: <b>copyable</b>&gt;(
    cap: &<b>mut</b> <a href="Config.md#0x1_Config_ModifyConfigCapability">Config::ModifyConfigCapability</a>&lt;<a href="Dao.md#0x1_Dao_DaoConfig">DaoConfig</a>&lt;TokenT&gt;&gt;,
    value: u8,
) {
    <b>assert</b>(value &lt;= 100 && value &gt; 0, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Dao.md#0x1_Dao_ERR_QUROM_RATE_INVALID">ERR_QUROM_RATE_INVALID</a>));
    <b>let</b> config = <a href="Dao.md#0x1_Dao_get_config">get_config</a>&lt;TokenT&gt;();
    config.voting_quorum_rate = value;
    <a href="Config.md#0x1_Config_set_with_capability">Config::set_with_capability</a>&lt;<a href="Dao.md#0x1_Dao_DaoConfig">DaoConfig</a>&lt;TokenT&gt;&gt;(cap, config);
}
</code></pre>



</details>

<a name="0x1_Dao_set_min_action_delay"></a>

## Function `set_min_action_delay`



<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_set_min_action_delay">set_min_action_delay</a>&lt;TokenT: <b>copyable</b>&gt;(cap: &<b>mut</b> <a href="Config.md#0x1_Config_ModifyConfigCapability">Config::ModifyConfigCapability</a>&lt;<a href="Dao.md#0x1_Dao_DaoConfig">Dao::DaoConfig</a>&lt;TokenT&gt;&gt;, value: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Dao.md#0x1_Dao_set_min_action_delay">set_min_action_delay</a>&lt;TokenT: <b>copyable</b>&gt;(
    cap: &<b>mut</b> <a href="Config.md#0x1_Config_ModifyConfigCapability">Config::ModifyConfigCapability</a>&lt;<a href="Dao.md#0x1_Dao_DaoConfig">DaoConfig</a>&lt;TokenT&gt;&gt;,
    value: u64,
) {
    <b>assert</b>(value &gt; 0, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Dao.md#0x1_Dao_ERR_CONFIG_PARAM_INVALID">ERR_CONFIG_PARAM_INVALID</a>));
    <b>let</b> config = <a href="Dao.md#0x1_Dao_get_config">get_config</a>&lt;TokenT&gt;();
    config.min_action_delay = value;
    <a href="Config.md#0x1_Config_set_with_capability">Config::set_with_capability</a>&lt;<a href="Dao.md#0x1_Dao_DaoConfig">DaoConfig</a>&lt;TokenT&gt;&gt;(cap, config);
}
</code></pre>



</details>
