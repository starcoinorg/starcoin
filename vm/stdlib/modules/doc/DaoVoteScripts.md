
<a name="0x1_DaoVoteScripts"></a>

# Module `0x1::DaoVoteScripts`



-  [Function `cast_vote`](#0x1_DaoVoteScripts_cast_vote)
-  [Function `revoke_vote`](#0x1_DaoVoteScripts_revoke_vote)
-  [Function `unstake_vote`](#0x1_DaoVoteScripts_unstake_vote)
-  [Specification](#@Specification_0)


<pre><code><b>use</b> <a href="Account.md#0x1_Account">0x1::Account</a>;
<b>use</b> <a href="Dao.md#0x1_Dao">0x1::Dao</a>;
<b>use</b> <a href="Signer.md#0x1_Signer">0x1::Signer</a>;
<b>use</b> <a href="Token.md#0x1_Token">0x1::Token</a>;
</code></pre>



<a name="0x1_DaoVoteScripts_cast_vote"></a>

## Function `cast_vote`



<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="DaoVoteScripts.md#0x1_DaoVoteScripts_cast_vote">cast_vote</a>&lt;<a href="Token.md#0x1_Token">Token</a>: <b>copy</b>, drop, store, ActionT: <b>copy</b>, drop, store&gt;(signer: signer, proposer_address: address, proposal_id: u64, agree: bool, votes: u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> ( <b>script</b> ) <b>fun</b> <a href="DaoVoteScripts.md#0x1_DaoVoteScripts_cast_vote">cast_vote</a>&lt;<a href="Token.md#0x1_Token">Token</a>: <b>copy</b> + drop + store, ActionT: <b>copy</b> + drop + store&gt;(
    signer: signer,
    proposer_address: address,
    proposal_id: u64,
    agree: bool,
    votes: u128,
) {
    <b>let</b> votes = <a href="Account.md#0x1_Account_withdraw">Account::withdraw</a>&lt;<a href="Token.md#0x1_Token">Token</a>&gt;(&signer, votes);
    <a href="Dao.md#0x1_Dao_cast_vote">Dao::cast_vote</a>&lt;<a href="Token.md#0x1_Token">Token</a>, ActionT&gt;(&signer, proposer_address, proposal_id, votes, agree);
}
</code></pre>



</details>

<a name="0x1_DaoVoteScripts_revoke_vote"></a>

## Function `revoke_vote`



<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="DaoVoteScripts.md#0x1_DaoVoteScripts_revoke_vote">revoke_vote</a>&lt;<a href="Token.md#0x1_Token">Token</a>: <b>copy</b>, drop, store, Action: <b>copy</b>, drop, store&gt;(signer: signer, proposer_address: address, proposal_id: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> ( <b>script</b> ) <b>fun</b> <a href="DaoVoteScripts.md#0x1_DaoVoteScripts_revoke_vote">revoke_vote</a>&lt;<a href="Token.md#0x1_Token">Token</a>: <b>copy</b> + drop + store, Action: <b>copy</b> + drop + store&gt;(
    signer: signer,
    proposer_address: address,
    proposal_id: u64,
) {
    <b>let</b> sender = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(&signer);
    <b>let</b> (_, power) = <a href="Dao.md#0x1_Dao_vote_of">Dao::vote_of</a>&lt;<a href="Token.md#0x1_Token">Token</a>&gt;(sender, proposer_address, proposal_id);
    <b>let</b> my_token = <a href="Dao.md#0x1_Dao_revoke_vote">Dao::revoke_vote</a>&lt;<a href="Token.md#0x1_Token">Token</a>, Action&gt;(&signer, proposer_address, proposal_id, power);
    <a href="Account.md#0x1_Account_deposit">Account::deposit</a>(sender, my_token);
}
</code></pre>



</details>

<a name="0x1_DaoVoteScripts_unstake_vote"></a>

## Function `unstake_vote`



<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="DaoVoteScripts.md#0x1_DaoVoteScripts_unstake_vote">unstake_vote</a>&lt;<a href="Token.md#0x1_Token">Token</a>: <b>copy</b>, drop, store, Action: <b>copy</b>, drop, store&gt;(signer: signer, proposer_address: address, proposal_id: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> ( <b>script</b> ) <b>fun</b> <a href="DaoVoteScripts.md#0x1_DaoVoteScripts_unstake_vote">unstake_vote</a>&lt;<a href="Token.md#0x1_Token">Token</a>: <b>copy</b> + drop + store, Action: <b>copy</b> + drop + store&gt;(
    signer: signer,
    proposer_address: address,
    proposal_id: u64,
) {
    <b>let</b> my_token = <a href="Dao.md#0x1_Dao_unstake_votes">Dao::unstake_votes</a>&lt;<a href="Token.md#0x1_Token">Token</a>, Action&gt;(&signer, proposer_address, proposal_id);
    <a href="Account.md#0x1_Account_deposit">Account::deposit</a>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(&signer), my_token);
}
</code></pre>



</details>

<a name="@Specification_0"></a>

## Specification



<pre><code><b>pragma</b> verify = <b>false</b>;
<b>pragma</b> aborts_if_is_partial = <b>false</b>;
<b>pragma</b> aborts_if_is_strict = <b>true</b>;
</code></pre>
