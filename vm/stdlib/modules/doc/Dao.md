
<a name="0x1_Dao"></a>

# Module `0x1::Dao`

### Table of Contents

-  [Resource `GovGlobalInfo`](#0x1_Dao_GovGlobalInfo)
-  [Struct `DaoConfig`](#0x1_Dao_DaoConfig)
-  [Resource `Proposal`](#0x1_Dao_Proposal)
-  [Resource `Vote`](#0x1_Dao_Vote)
-  [Const `VOTEING_DELAY`](#0x1_Dao_VOTEING_DELAY)
-  [Const `VOTEING_PERIOD`](#0x1_Dao_VOTEING_PERIOD)
-  [Const `VOTEING_QUORUM_RATE`](#0x1_Dao_VOTEING_QUORUM_RATE)
-  [Const `MIN_ACTION_DELAY`](#0x1_Dao_MIN_ACTION_DELAY)
-  [Const `PENDING`](#0x1_Dao_PENDING)
-  [Const `ACTIVE`](#0x1_Dao_ACTIVE)
-  [Const `DEFEATED`](#0x1_Dao_DEFEATED)
-  [Const `AGREED`](#0x1_Dao_AGREED)
-  [Const `QUEUED`](#0x1_Dao_QUEUED)
-  [Const `EXECUTABLE`](#0x1_Dao_EXECUTABLE)
-  [Const `EXTRACTED`](#0x1_Dao_EXTRACTED)
-  [Function `plugin`](#0x1_Dao_plugin)
-  [Function `propose`](#0x1_Dao_propose)
-  [Function `cast_vote`](#0x1_Dao_cast_vote)
-  [Function `unstake_votes`](#0x1_Dao_unstake_votes)
-  [Function `queue_proposal_action`](#0x1_Dao_queue_proposal_action)
-  [Function `extract_proposal_action`](#0x1_Dao_extract_proposal_action)
-  [Function `proposal_state`](#0x1_Dao_proposal_state)
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



<a name="0x1_Dao_GovGlobalInfo"></a>

## Resource `GovGlobalInfo`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_Dao_GovGlobalInfo">GovGlobalInfo</a>&lt;<a href="Token.md#0x1_Token">Token</a>&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>next_proposal_id: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_Dao_DaoConfig"></a>

## Struct `DaoConfig`



<pre><code><b>struct</b> <a href="#0x1_Dao_DaoConfig">DaoConfig</a>&lt;TokenT: <b>copyable</b>&gt;
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

<a name="0x1_Dao_Proposal"></a>

## Resource `Proposal`

TODO: support that one can propose mutli proposals.


<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_Dao_Proposal">Proposal</a>&lt;<a href="Token.md#0x1_Token">Token</a>, Action&gt;
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

<code>start_block: u64</code>
</dt>
<dd>

</dd>
<dt>

<code>end_block: u64</code>
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



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_Dao_Vote">Vote</a>&lt;TokenT&gt;
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

<a name="0x1_Dao_VOTEING_DELAY"></a>

## Const `VOTEING_DELAY`

make them into configs


<pre><code><b>const</b> VOTEING_DELAY: u64 = 100;
</code></pre>



<a name="0x1_Dao_VOTEING_PERIOD"></a>

## Const `VOTEING_PERIOD`



<pre><code><b>const</b> VOTEING_PERIOD: u64 = 200;
</code></pre>



<a name="0x1_Dao_VOTEING_QUORUM_RATE"></a>

## Const `VOTEING_QUORUM_RATE`

quorum rate: 4% of toal token supply.


<pre><code><b>const</b> VOTEING_QUORUM_RATE: u8 = 4;
</code></pre>



<a name="0x1_Dao_MIN_ACTION_DELAY"></a>

## Const `MIN_ACTION_DELAY`



<pre><code><b>const</b> MIN_ACTION_DELAY: u64 = 200;
</code></pre>



<a name="0x1_Dao_PENDING"></a>

## Const `PENDING`

Proposal state


<pre><code><b>const</b> PENDING: u8 = 1;
</code></pre>



<a name="0x1_Dao_ACTIVE"></a>

## Const `ACTIVE`



<pre><code><b>const</b> ACTIVE: u8 = 2;
</code></pre>



<a name="0x1_Dao_DEFEATED"></a>

## Const `DEFEATED`



<pre><code><b>const</b> DEFEATED: u8 = 3;
</code></pre>



<a name="0x1_Dao_AGREED"></a>

## Const `AGREED`



<pre><code><b>const</b> AGREED: u8 = 4;
</code></pre>



<a name="0x1_Dao_QUEUED"></a>

## Const `QUEUED`



<pre><code><b>const</b> QUEUED: u8 = 5;
</code></pre>



<a name="0x1_Dao_EXECUTABLE"></a>

## Const `EXECUTABLE`



<pre><code><b>const</b> EXECUTABLE: u8 = 6;
</code></pre>



<a name="0x1_Dao_EXTRACTED"></a>

## Const `EXTRACTED`



<pre><code><b>const</b> EXTRACTED: u8 = 7;
</code></pre>



<a name="0x1_Dao_plugin"></a>

## Function `plugin`

plug_in function, can only be called by token issuer.
Any token who wants to has gov functionality
can optin this moudle by call this
<code>register function</code>.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Dao_plugin">plugin</a>&lt;TokenT: <b>copyable</b>&gt;(signer: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Dao_plugin">plugin</a>&lt;TokenT: <b>copyable</b>&gt;(signer: &signer) {
    // TODO: we can add a token manage cap in <a href="Token.md#0x1_Token">Token</a> <b>module</b>.
    // and only token manager can register this.
    <b>let</b> token_issuer = <a href="Token.md#0x1_Token_token_address">Token::token_address</a>&lt;TokenT&gt;();
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer) == token_issuer, 401);
    // <b>let</b> proposal_id = ProposalId {next: 0};
    <b>let</b> gov_info = <a href="#0x1_Dao_GovGlobalInfo">GovGlobalInfo</a>&lt;TokenT&gt; { next_proposal_id: 0 };
    move_to(signer, gov_info);
    <b>let</b> config = <a href="#0x1_Dao_DaoConfig">DaoConfig</a>&lt;TokenT&gt; {
        voting_delay: VOTEING_DELAY,
        voting_period: VOTEING_PERIOD,
        voting_quorum_rate: VOTEING_QUORUM_RATE,
        min_action_delay: MIN_ACTION_DELAY,
    };
    <a href="Config.md#0x1_Config_publish_new_config">Config::publish_new_config</a>(signer, config);
}
</code></pre>



</details>

<a name="0x1_Dao_propose"></a>

## Function `propose`

propose a proposal.
<code>action</code>: the actual action to execute.
<code>action_delay</code>: the delay to execute after the proposal is agreed


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Dao_propose">propose</a>&lt;TokenT: <b>copyable</b>, ActionT&gt;(signer: &signer, action: ActionT, action_delay: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Dao_propose">propose</a>&lt;TokenT: <b>copyable</b>, ActionT&gt;(
    signer: &signer,
    action: ActionT,
    action_delay: u64,
) <b>acquires</b> <a href="#0x1_Dao_GovGlobalInfo">GovGlobalInfo</a> {
    <b>assert</b>(action_delay &gt;= <a href="#0x1_Dao_min_action_delay">min_action_delay</a>&lt;TokenT&gt;(), 401);
    <b>let</b> proposal_id = <a href="#0x1_Dao_generate_next_proposal_id">generate_next_proposal_id</a>&lt;TokenT&gt;();
    // TODO: make the delay configurable
    <b>let</b> start_block = <a href="Block.md#0x1_Block_get_current_block_number">Block::get_current_block_number</a>() + <a href="#0x1_Dao_voting_delay">voting_delay</a>&lt;TokenT&gt;();
    <b>let</b> proposal = <a href="#0x1_Dao_Proposal">Proposal</a>&lt;TokenT, ActionT&gt; {
        id: proposal_id,
        proposer: <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer),
        start_block,
        end_block: start_block + <a href="#0x1_Dao_voting_period">voting_period</a>&lt;TokenT&gt;(),
        for_votes: 0,
        against_votes: 0,
        eta: 0,
        action_delay,
        action: <a href="Option.md#0x1_Option_some">Option::some</a>(action),
    };
    move_to(signer, proposal);
}
</code></pre>



</details>

<a name="0x1_Dao_cast_vote"></a>

## Function `cast_vote`

votes for a proposal.
User can only vote once, then the stake is locked,
which can only be unstaked by user after the proposal is expired, or cancelled, or executed.
So think twice before casting vote.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Dao_cast_vote">cast_vote</a>&lt;TokenT: <b>copyable</b>, ActionT&gt;(signer: &signer, proposer_address: address, proposal_id: u64, stake: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenT&gt;, agree: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Dao_cast_vote">cast_vote</a>&lt;TokenT: <b>copyable</b>, ActionT&gt;(
    signer: &signer,
    proposer_address: address,
    proposal_id: u64,
    stake: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenT&gt;,
    agree: bool,
) <b>acquires</b> <a href="#0x1_Dao_Proposal">Proposal</a> {
    {
        <b>let</b> state = <a href="#0x1_Dao_proposal_state">proposal_state</a>&lt;TokenT, ActionT&gt;(proposer_address, proposal_id);
        <b>assert</b>(state &lt;= ACTIVE, 700);
    };
    <b>let</b> proposal = borrow_global_mut&lt;<a href="#0x1_Dao_Proposal">Proposal</a>&lt;TokenT, ActionT&gt;&gt;(proposer_address);
    <b>assert</b>(proposal.id == proposal_id, 500);
    <b>let</b> stake_value = <a href="Token.md#0x1_Token_value">Token::value</a>(&stake);
    <b>let</b> my_vote = <a href="#0x1_Dao_Vote">Vote</a>&lt;TokenT&gt; { proposer: proposer_address, id: proposal_id, stake, agree };
    <b>if</b> (agree) {
        proposal.for_votes = proposal.for_votes + stake_value;
    } <b>else</b> {
        proposal.against_votes = proposal.against_votes + stake_value;
    };
    move_to(signer, my_vote);
}
</code></pre>



</details>

<a name="0x1_Dao_unstake_votes"></a>

## Function `unstake_votes`

Retrieve back my staked token voted for a proposal.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Dao_unstake_votes">unstake_votes</a>&lt;TokenT: <b>copyable</b>, ActionT&gt;(signer: &signer, proposer_address: address, proposal_id: u64): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenT&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Dao_unstake_votes">unstake_votes</a>&lt;TokenT: <b>copyable</b>, ActionT&gt;(
    signer: &signer,
    proposer_address: address,
    proposal_id: u64,
): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenT&gt; <b>acquires</b> <a href="#0x1_Dao_Proposal">Proposal</a>, <a href="#0x1_Dao_Vote">Vote</a> {
    {
        <b>let</b> state = <a href="#0x1_Dao_proposal_state">proposal_state</a>&lt;TokenT, ActionT&gt;(proposer_address, proposal_id);
        // Only after vote period end, user can unstake his votes.
        <b>assert</b>(state &gt; ACTIVE, 800);
    };
    <b>let</b> <a href="#0x1_Dao_Vote">Vote</a> { proposer, id, stake, agree: _ } = move_from&lt;<a href="#0x1_Dao_Vote">Vote</a>&lt;TokenT&gt;&gt;(
        <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer),
    );
    <b>assert</b>(proposer == proposer_address, 100);
    <b>assert</b>(id == proposal_id, 101);
    stake
}
</code></pre>



</details>

<a name="0x1_Dao_queue_proposal_action"></a>

## Function `queue_proposal_action`

queue agreed proposal to execute.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Dao_queue_proposal_action">queue_proposal_action</a>&lt;TokenT: <b>copyable</b>, ActionT&gt;(proposer_address: address, proposal_id: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Dao_queue_proposal_action">queue_proposal_action</a>&lt;TokenT: <b>copyable</b>, ActionT&gt;(
    proposer_address: address,
    proposal_id: u64,
) <b>acquires</b> <a href="#0x1_Dao_Proposal">Proposal</a> {
    // Only agreed proposal can be submitted.
    <b>assert</b>(<a href="#0x1_Dao_proposal_state">proposal_state</a>&lt;TokenT, ActionT&gt;(proposer_address, proposal_id) == AGREED, 601);
    <b>let</b> proposal = borrow_global_mut&lt;<a href="#0x1_Dao_Proposal">Proposal</a>&lt;TokenT, ActionT&gt;&gt;(proposer_address);
    proposal.eta = <a href="Block.md#0x1_Block_get_current_block_number">Block::get_current_block_number</a>() + proposal.action_delay;
}
</code></pre>



</details>

<a name="0x1_Dao_extract_proposal_action"></a>

## Function `extract_proposal_action`

extract proposal action to execute.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Dao_extract_proposal_action">extract_proposal_action</a>&lt;TokenT: <b>copyable</b>, ActionT&gt;(proposer_address: address, proposal_id: u64): ActionT
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Dao_extract_proposal_action">extract_proposal_action</a>&lt;TokenT: <b>copyable</b>, ActionT&gt;(
    proposer_address: address,
    proposal_id: u64,
): ActionT <b>acquires</b> <a href="#0x1_Dao_Proposal">Proposal</a> {
    // Only executable proposal's action can be extracted.
    <b>assert</b>(<a href="#0x1_Dao_proposal_state">proposal_state</a>&lt;TokenT, ActionT&gt;(proposer_address, proposal_id) == EXECUTABLE, 601);
    <b>let</b> proposal = borrow_global_mut&lt;<a href="#0x1_Dao_Proposal">Proposal</a>&lt;TokenT, ActionT&gt;&gt;(proposer_address);
    <b>let</b> action: ActionT = <a href="Option.md#0x1_Option_extract">Option::extract</a>(&<b>mut</b> proposal.action);
    action
}
</code></pre>



</details>

<a name="0x1_Dao_proposal_state"></a>

## Function `proposal_state`



<pre><code><b>fun</b> <a href="#0x1_Dao_proposal_state">proposal_state</a>&lt;TokenT: <b>copyable</b>, ActionT&gt;(proposer_address: address, proposal_id: u64): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_Dao_proposal_state">proposal_state</a>&lt;TokenT: <b>copyable</b>, ActionT&gt;(proposer_address: address, proposal_id: u64): u8
<b>acquires</b> <a href="#0x1_Dao_Proposal">Proposal</a> {
    <b>let</b> proposal = borrow_global&lt;<a href="#0x1_Dao_Proposal">Proposal</a>&lt;TokenT, ActionT&gt;&gt;(proposer_address);
    <b>assert</b>(proposal.id == proposal_id, 500);
    <b>let</b> current_block_number = <a href="Block.md#0x1_Block_get_current_block_number">Block::get_current_block_number</a>();
    <b>if</b> (current_block_number &lt;= proposal.start_block) {
        // Pending
        PENDING
    } <b>else</b> <b>if</b> (current_block_number &lt;= proposal.end_block) {
        // Active
        ACTIVE
    } <b>else</b> <b>if</b> (proposal.for_votes &lt;= proposal.against_votes ||
        proposal.for_votes &lt; <a href="#0x1_Dao_quorum_votes">quorum_votes</a>&lt;TokenT&gt;()) {
        // Defeated
        DEFEATED
    } <b>else</b> <b>if</b> (proposal.eta == 0) {
        // Agreed.
        AGREED
    } <b>else</b> <b>if</b> (proposal.eta &lt; current_block_number) {
        // Queued, waiting <b>to</b> execute
        QUEUED
    } <b>else</b> <b>if</b> (<a href="Option.md#0x1_Option_is_some">Option::is_some</a>(&proposal.action)) {
        EXECUTABLE
    } <b>else</b> {
        EXTRACTED
    }
}
</code></pre>



</details>

<a name="0x1_Dao_quorum_votes"></a>

## Function `quorum_votes`

Quorum votes to make proposal pass.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Dao_quorum_votes">quorum_votes</a>&lt;TokenT: <b>copyable</b>&gt;(): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Dao_quorum_votes">quorum_votes</a>&lt;TokenT: <b>copyable</b>&gt;(): u128 {
    <b>let</b> supply = <a href="Token.md#0x1_Token_market_cap">Token::market_cap</a>&lt;TokenT&gt;();
    supply / 100 * (<a href="#0x1_Dao_voting_quorum_rate">voting_quorum_rate</a>&lt;TokenT&gt;() <b>as</b> u128)
}
</code></pre>



</details>

<a name="0x1_Dao_generate_next_proposal_id"></a>

## Function `generate_next_proposal_id`



<pre><code><b>fun</b> <a href="#0x1_Dao_generate_next_proposal_id">generate_next_proposal_id</a>&lt;TokenT&gt;(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_Dao_generate_next_proposal_id">generate_next_proposal_id</a>&lt;TokenT&gt;(): u64 <b>acquires</b> <a href="#0x1_Dao_GovGlobalInfo">GovGlobalInfo</a> {
    <b>let</b> gov_info = borrow_global_mut&lt;<a href="#0x1_Dao_GovGlobalInfo">GovGlobalInfo</a>&lt;TokenT&gt;&gt;(<a href="Token.md#0x1_Token_token_address">Token::token_address</a>&lt;TokenT&gt;());
    <b>let</b> proposal_id = gov_info.next_proposal_id;
    gov_info.next_proposal_id = proposal_id + 1;
    proposal_id
}
</code></pre>



</details>

<a name="0x1_Dao_voting_delay"></a>

## Function `voting_delay`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Dao_voting_delay">voting_delay</a>&lt;TokenT: <b>copyable</b>&gt;(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Dao_voting_delay">voting_delay</a>&lt;TokenT: <b>copyable</b>&gt;(): u64 {
    <a href="#0x1_Dao_get_config">get_config</a>&lt;TokenT&gt;().voting_delay
}
</code></pre>



</details>

<a name="0x1_Dao_voting_period"></a>

## Function `voting_period`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Dao_voting_period">voting_period</a>&lt;TokenT: <b>copyable</b>&gt;(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Dao_voting_period">voting_period</a>&lt;TokenT: <b>copyable</b>&gt;(): u64 {
    <a href="#0x1_Dao_get_config">get_config</a>&lt;TokenT&gt;().voting_period
}
</code></pre>



</details>

<a name="0x1_Dao_voting_quorum_rate"></a>

## Function `voting_quorum_rate`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Dao_voting_quorum_rate">voting_quorum_rate</a>&lt;TokenT: <b>copyable</b>&gt;(): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Dao_voting_quorum_rate">voting_quorum_rate</a>&lt;TokenT: <b>copyable</b>&gt;(): u8 {
    <a href="#0x1_Dao_get_config">get_config</a>&lt;TokenT&gt;().voting_quorum_rate
}
</code></pre>



</details>

<a name="0x1_Dao_min_action_delay"></a>

## Function `min_action_delay`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Dao_min_action_delay">min_action_delay</a>&lt;TokenT: <b>copyable</b>&gt;(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Dao_min_action_delay">min_action_delay</a>&lt;TokenT: <b>copyable</b>&gt;(): u64 {
    <a href="#0x1_Dao_get_config">get_config</a>&lt;TokenT&gt;().min_action_delay
}
</code></pre>



</details>

<a name="0x1_Dao_get_config"></a>

## Function `get_config`



<pre><code><b>fun</b> <a href="#0x1_Dao_get_config">get_config</a>&lt;TokenT: <b>copyable</b>&gt;(): <a href="#0x1_Dao_DaoConfig">Dao::DaoConfig</a>&lt;TokenT&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_Dao_get_config">get_config</a>&lt;TokenT: <b>copyable</b>&gt;(): <a href="#0x1_Dao_DaoConfig">DaoConfig</a>&lt;TokenT&gt; {
    <b>let</b> token_issuer = <a href="Token.md#0x1_Token_token_address">Token::token_address</a>&lt;TokenT&gt;();
    <a href="Config.md#0x1_Config_get_by_address">Config::get_by_address</a>&lt;<a href="#0x1_Dao_DaoConfig">DaoConfig</a>&lt;TokenT&gt;&gt;(token_issuer)
}
</code></pre>



</details>

<a name="0x1_Dao_modify_dao_config"></a>

## Function `modify_dao_config`

update function
TODO: cap should not be mut to set data.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Dao_modify_dao_config">modify_dao_config</a>&lt;TokenT: <b>copyable</b>&gt;(cap: &<b>mut</b> <a href="Config.md#0x1_Config_ModifyConfigCapability">Config::ModifyConfigCapability</a>&lt;<a href="#0x1_Dao_DaoConfig">Dao::DaoConfig</a>&lt;TokenT&gt;&gt;, voting_delay: <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;u64&gt;, voting_period: <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;u64&gt;, voting_quorum_rate: <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;u8&gt;, min_action_delay: <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;u64&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Dao_modify_dao_config">modify_dao_config</a>&lt;TokenT: <b>copyable</b>&gt;(
    cap: &<b>mut</b> <a href="Config.md#0x1_Config_ModifyConfigCapability">Config::ModifyConfigCapability</a>&lt;<a href="#0x1_Dao_DaoConfig">DaoConfig</a>&lt;TokenT&gt;&gt;,
    voting_delay: <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;u64&gt;,
    voting_period: <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;u64&gt;,
    voting_quorum_rate: <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;u8&gt;,
    min_action_delay: <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;u64&gt;,
) {
    <b>let</b> config = <a href="#0x1_Dao_get_config">get_config</a>&lt;TokenT&gt;();
    <b>if</b> (<a href="Option.md#0x1_Option_is_some">Option::is_some</a>(&voting_period)) {
        <b>let</b> v = <a href="Option.md#0x1_Option_extract">Option::extract</a>(&<b>mut</b> voting_period);
        config.voting_period = v;
    };
    <b>if</b> (<a href="Option.md#0x1_Option_is_some">Option::is_some</a>(&voting_delay)) {
        <b>let</b> v = <a href="Option.md#0x1_Option_extract">Option::extract</a>(&<b>mut</b> voting_delay);
        config.voting_delay = v;
    };
    <b>if</b> (<a href="Option.md#0x1_Option_is_some">Option::is_some</a>(&voting_quorum_rate)) {
        <b>let</b> v = <a href="Option.md#0x1_Option_extract">Option::extract</a>(&<b>mut</b> voting_quorum_rate);
        config.voting_quorum_rate = v;
    };
    <b>if</b> (<a href="Option.md#0x1_Option_is_some">Option::is_some</a>(&min_action_delay)) {
        <b>let</b> v = <a href="Option.md#0x1_Option_extract">Option::extract</a>(&<b>mut</b> min_action_delay);
        config.min_action_delay = v;
    };
    <a href="Config.md#0x1_Config_set_with_capability">Config::set_with_capability</a>&lt;<a href="#0x1_Dao_DaoConfig">DaoConfig</a>&lt;TokenT&gt;&gt;(cap, config);
}
</code></pre>



</details>

<a name="0x1_Dao_set_voting_delay"></a>

## Function `set_voting_delay`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Dao_set_voting_delay">set_voting_delay</a>&lt;TokenT: <b>copyable</b>&gt;(cap: &<b>mut</b> <a href="Config.md#0x1_Config_ModifyConfigCapability">Config::ModifyConfigCapability</a>&lt;<a href="#0x1_Dao_DaoConfig">Dao::DaoConfig</a>&lt;TokenT&gt;&gt;, value: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Dao_set_voting_delay">set_voting_delay</a>&lt;TokenT: <b>copyable</b>&gt;(
    cap: &<b>mut</b> <a href="Config.md#0x1_Config_ModifyConfigCapability">Config::ModifyConfigCapability</a>&lt;<a href="#0x1_Dao_DaoConfig">DaoConfig</a>&lt;TokenT&gt;&gt;,
    value: u64,
) {
    <b>let</b> config = <a href="#0x1_Dao_get_config">get_config</a>&lt;TokenT&gt;();
    config.voting_delay = value;
    <a href="Config.md#0x1_Config_set_with_capability">Config::set_with_capability</a>&lt;<a href="#0x1_Dao_DaoConfig">DaoConfig</a>&lt;TokenT&gt;&gt;(cap, config);
}
</code></pre>



</details>

<a name="0x1_Dao_set_voting_period"></a>

## Function `set_voting_period`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Dao_set_voting_period">set_voting_period</a>&lt;TokenT: <b>copyable</b>&gt;(cap: &<b>mut</b> <a href="Config.md#0x1_Config_ModifyConfigCapability">Config::ModifyConfigCapability</a>&lt;<a href="#0x1_Dao_DaoConfig">Dao::DaoConfig</a>&lt;TokenT&gt;&gt;, value: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Dao_set_voting_period">set_voting_period</a>&lt;TokenT: <b>copyable</b>&gt;(
    cap: &<b>mut</b> <a href="Config.md#0x1_Config_ModifyConfigCapability">Config::ModifyConfigCapability</a>&lt;<a href="#0x1_Dao_DaoConfig">DaoConfig</a>&lt;TokenT&gt;&gt;,
    value: u64,
) {
    <b>let</b> config = <a href="#0x1_Dao_get_config">get_config</a>&lt;TokenT&gt;();
    config.voting_period = value;
    <a href="Config.md#0x1_Config_set_with_capability">Config::set_with_capability</a>&lt;<a href="#0x1_Dao_DaoConfig">DaoConfig</a>&lt;TokenT&gt;&gt;(cap, config);
}
</code></pre>



</details>

<a name="0x1_Dao_set_voting_quorum_rate"></a>

## Function `set_voting_quorum_rate`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Dao_set_voting_quorum_rate">set_voting_quorum_rate</a>&lt;TokenT: <b>copyable</b>&gt;(cap: &<b>mut</b> <a href="Config.md#0x1_Config_ModifyConfigCapability">Config::ModifyConfigCapability</a>&lt;<a href="#0x1_Dao_DaoConfig">Dao::DaoConfig</a>&lt;TokenT&gt;&gt;, value: u8)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Dao_set_voting_quorum_rate">set_voting_quorum_rate</a>&lt;TokenT: <b>copyable</b>&gt;(
    cap: &<b>mut</b> <a href="Config.md#0x1_Config_ModifyConfigCapability">Config::ModifyConfigCapability</a>&lt;<a href="#0x1_Dao_DaoConfig">DaoConfig</a>&lt;TokenT&gt;&gt;,
    value: u8,
) {
    <b>let</b> config = <a href="#0x1_Dao_get_config">get_config</a>&lt;TokenT&gt;();
    config.voting_quorum_rate = value;
    <a href="Config.md#0x1_Config_set_with_capability">Config::set_with_capability</a>&lt;<a href="#0x1_Dao_DaoConfig">DaoConfig</a>&lt;TokenT&gt;&gt;(cap, config);
}
</code></pre>



</details>

<a name="0x1_Dao_set_min_action_delay"></a>

## Function `set_min_action_delay`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Dao_set_min_action_delay">set_min_action_delay</a>&lt;TokenT: <b>copyable</b>&gt;(cap: &<b>mut</b> <a href="Config.md#0x1_Config_ModifyConfigCapability">Config::ModifyConfigCapability</a>&lt;<a href="#0x1_Dao_DaoConfig">Dao::DaoConfig</a>&lt;TokenT&gt;&gt;, value: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Dao_set_min_action_delay">set_min_action_delay</a>&lt;TokenT: <b>copyable</b>&gt;(
    cap: &<b>mut</b> <a href="Config.md#0x1_Config_ModifyConfigCapability">Config::ModifyConfigCapability</a>&lt;<a href="#0x1_Dao_DaoConfig">DaoConfig</a>&lt;TokenT&gt;&gt;,
    value: u64,
) {
    <b>let</b> config = <a href="#0x1_Dao_get_config">get_config</a>&lt;TokenT&gt;();
    config.min_action_delay = value;
    <a href="Config.md#0x1_Config_set_with_capability">Config::set_with_capability</a>&lt;<a href="#0x1_Dao_DaoConfig">DaoConfig</a>&lt;TokenT&gt;&gt;(cap, config);
}
</code></pre>



</details>
