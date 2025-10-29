
<a id="0x1_dao_vote_scripts"></a>

# Module `0x1::dao_vote_scripts`



-  [Constants](#@Constants_0)
-  [Function `cast_vote`](#0x1_dao_vote_scripts_cast_vote)
-  [Function `revoke_vote`](#0x1_dao_vote_scripts_revoke_vote)
-  [Function `flip_vote`](#0x1_dao_vote_scripts_flip_vote)
-  [Function `revoke_vote_of_power`](#0x1_dao_vote_scripts_revoke_vote_of_power)
-  [Function `unstake_vote`](#0x1_dao_vote_scripts_unstake_vote)
-  [Specification](#@Specification_1)


<pre><code><b>use</b> <a href="coin.md#0x1_coin">0x1::coin</a>;
<b>use</b> <a href="dao.md#0x1_dao">0x1::dao</a>;
<b>use</b> <a href="../../move-stdlib/doc/error.md#0x1_error">0x1::error</a>;
<b>use</b> <a href="fungible_asset.md#0x1_fungible_asset">0x1::fungible_asset</a>;
<b>use</b> <a href="object.md#0x1_object">0x1::object</a>;
<b>use</b> <a href="../../move-stdlib/doc/option.md#0x1_option">0x1::option</a>;
<b>use</b> <a href="primary_fungible_store.md#0x1_primary_fungible_store">0x1::primary_fungible_store</a>;
<b>use</b> <a href="../../move-stdlib/doc/signer.md#0x1_signer">0x1::signer</a>;
</code></pre>



<a id="@Constants_0"></a>

## Constants


<a id="0x1_dao_vote_scripts_ERR_CAST_VOTE_FA_NOT_FOUND"></a>



<pre><code><b>const</b> <a href="dao_vote_scripts.md#0x1_dao_vote_scripts_ERR_CAST_VOTE_FA_NOT_FOUND">ERR_CAST_VOTE_FA_NOT_FOUND</a>: u64 = 1;
</code></pre>



<a id="0x1_dao_vote_scripts_ERR_CAST_VOTE_WITHDRAW_FA_NOT_MATCH"></a>



<pre><code><b>const</b> <a href="dao_vote_scripts.md#0x1_dao_vote_scripts_ERR_CAST_VOTE_WITHDRAW_FA_NOT_MATCH">ERR_CAST_VOTE_WITHDRAW_FA_NOT_MATCH</a>: u64 = 2;
</code></pre>



<a id="0x1_dao_vote_scripts_cast_vote"></a>

## Function `cast_vote`



<pre><code><b>public</b> entry <b>fun</b> <a href="dao_vote_scripts.md#0x1_dao_vote_scripts_cast_vote">cast_vote</a>&lt;Token, ActionT: <b>copy</b>, drop, store&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, proposer_address: <b>address</b>, proposal_id: u64, agree: bool, votes: u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="dao_vote_scripts.md#0x1_dao_vote_scripts_cast_vote">cast_vote</a>&lt;Token, ActionT: <b>copy</b> + drop + store&gt;(
    <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    proposer_address: <b>address</b>,
    proposal_id: u64,
    agree: bool,
    votes: u128,
) {
    <b>let</b> sender = <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>);
    <b>if</b> (<a href="dao.md#0x1_dao_has_vote">dao::has_vote</a>&lt;Token&gt;(sender, proposer_address, proposal_id)) {
        // <b>if</b> already voted, and vote is not same <b>as</b> the current cast, change the existing vote.
        // resolve https://github.com/starcoinorg/starcoin/issues/2925.
        <b>let</b> (agree_voted, _) = <a href="dao.md#0x1_dao_vote_of">dao::vote_of</a>&lt;Token&gt;(sender, proposer_address, proposal_id);
        <b>if</b> (agree_voted != agree) {
            <a href="dao.md#0x1_dao_change_vote">dao::change_vote</a>&lt;Token, ActionT&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, proposer_address, proposal_id, agree);
        }
    };

    <b>let</b> metadata = <a href="coin.md#0x1_coin_paired_metadata">coin::paired_metadata</a>&lt;Token&gt;();
    <b>assert</b>!(<a href="../../move-stdlib/doc/option.md#0x1_option_is_some">option::is_some</a>(&metadata), <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_state">error::invalid_state</a>(<a href="dao_vote_scripts.md#0x1_dao_vote_scripts_ERR_CAST_VOTE_FA_NOT_FOUND">ERR_CAST_VOTE_FA_NOT_FOUND</a>));

    <b>let</b> votes_fa = <a href="primary_fungible_store.md#0x1_primary_fungible_store_withdraw">primary_fungible_store::withdraw</a>(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, <a href="../../move-stdlib/doc/option.md#0x1_option_destroy_some">option::destroy_some</a>(metadata), (votes <b>as</b> u64));
    <b>assert</b>!(
        <a href="fungible_asset.md#0x1_fungible_asset_amount">fungible_asset::amount</a>(&votes_fa) == (votes <b>as</b> u64),
        <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_state">error::invalid_state</a>(<a href="dao_vote_scripts.md#0x1_dao_vote_scripts_ERR_CAST_VOTE_WITHDRAW_FA_NOT_MATCH">ERR_CAST_VOTE_WITHDRAW_FA_NOT_MATCH</a>)
    );

    <a href="dao.md#0x1_dao_cast_vote">dao::cast_vote</a>&lt;Token, ActionT&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, proposer_address, proposal_id, votes_fa, agree);
}
</code></pre>



</details>

<a id="0x1_dao_vote_scripts_revoke_vote"></a>

## Function `revoke_vote`

revoke all votes on a proposal


<pre><code><b>public</b> entry <b>fun</b> <a href="dao_vote_scripts.md#0x1_dao_vote_scripts_revoke_vote">revoke_vote</a>&lt;Token, Action: <b>copy</b>, drop, store&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, proposer_address: <b>address</b>, proposal_id: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="dao_vote_scripts.md#0x1_dao_vote_scripts_revoke_vote">revoke_vote</a>&lt;Token, Action: <b>copy</b> + drop + store&gt;(
    <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    proposer_address: <b>address</b>,
    proposal_id: u64,
) {
    <b>let</b> sender = <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>);
    <b>let</b> (_, power) = <a href="dao.md#0x1_dao_vote_of">dao::vote_of</a>&lt;Token&gt;(sender, proposer_address, proposal_id);
    <b>let</b> my_token = <a href="dao.md#0x1_dao_revoke_vote">dao::revoke_vote</a>&lt;Token, Action&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, proposer_address, proposal_id, power);
    <a href="primary_fungible_store.md#0x1_primary_fungible_store_deposit">primary_fungible_store::deposit</a>(sender, my_token);
}
</code></pre>



</details>

<a id="0x1_dao_vote_scripts_flip_vote"></a>

## Function `flip_vote`

Let user change their vote during the voting time.


<pre><code><b>public</b> entry <b>fun</b> <a href="dao_vote_scripts.md#0x1_dao_vote_scripts_flip_vote">flip_vote</a>&lt;TokenT, ActionT: <b>copy</b>, drop, store&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, proposer_address: <b>address</b>, proposal_id: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="dao_vote_scripts.md#0x1_dao_vote_scripts_flip_vote">flip_vote</a>&lt;TokenT, ActionT: <b>copy</b> + drop + store&gt;(
    <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    proposer_address: <b>address</b>,
    proposal_id: u64,
) {
    <b>let</b> (agree, _) = <a href="dao.md#0x1_dao_vote_of">dao::vote_of</a>&lt;TokenT&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>), proposer_address, proposal_id);
    <a href="dao.md#0x1_dao_change_vote">dao::change_vote</a>&lt;TokenT, ActionT&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, proposer_address, proposal_id, !agree);
}
</code></pre>



</details>

<a id="0x1_dao_vote_scripts_revoke_vote_of_power"></a>

## Function `revoke_vote_of_power`

revoke some votes on a proposal


<pre><code><b>public</b> entry <b>fun</b> <a href="dao_vote_scripts.md#0x1_dao_vote_scripts_revoke_vote_of_power">revoke_vote_of_power</a>&lt;Token, Action: <b>copy</b>, drop, store&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, proposer_address: <b>address</b>, proposal_id: u64, power: u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="dao_vote_scripts.md#0x1_dao_vote_scripts_revoke_vote_of_power">revoke_vote_of_power</a>&lt;Token, Action: <b>copy</b> + drop + store&gt;(
    <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    proposer_address: <b>address</b>,
    proposal_id: u64,
    power: u128,
) {
    <b>let</b> sender = <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>);
    <b>let</b> my_token = <a href="dao.md#0x1_dao_revoke_vote">dao::revoke_vote</a>&lt;Token, Action&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, proposer_address, proposal_id, power);
    <a href="primary_fungible_store.md#0x1_primary_fungible_store_deposit">primary_fungible_store::deposit</a>(sender, my_token);
}
</code></pre>



</details>

<a id="0x1_dao_vote_scripts_unstake_vote"></a>

## Function `unstake_vote`



<pre><code><b>public</b> entry <b>fun</b> <a href="dao_vote_scripts.md#0x1_dao_vote_scripts_unstake_vote">unstake_vote</a>&lt;Token, Action: <b>copy</b>, drop, store&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, proposer_address: <b>address</b>, proposal_id: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="dao_vote_scripts.md#0x1_dao_vote_scripts_unstake_vote">unstake_vote</a>&lt;Token, Action: <b>copy</b> + drop + store&gt;(
    <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    proposer_address: <b>address</b>,
    proposal_id: u64,
) {
    <b>let</b> my_token = <a href="dao.md#0x1_dao_unstake_votes">dao::unstake_votes</a>&lt;Token, Action&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, proposer_address, proposal_id);
    <a href="primary_fungible_store.md#0x1_primary_fungible_store_deposit">primary_fungible_store::deposit</a>(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>), my_token);
}
</code></pre>



</details>

<a id="@Specification_1"></a>

## Specification



<pre><code><b>pragma</b> verify = <b>false</b>;
<b>pragma</b> aborts_if_is_partial = <b>false</b>;
<b>pragma</b> aborts_if_is_strict = <b>true</b>;
</code></pre>


[move-book]: https://starcoin.dev/move/book/SUMMARY
