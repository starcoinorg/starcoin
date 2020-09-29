
<a name="cast_vote"></a>

# Script `cast_vote`






<pre><code><b>public</b> <b>fun</b> <a href="cast_vote.md#cast_vote">cast_vote</a>&lt;<a href="../../modules/doc/Token.md#0x1_Token">Token</a>: <b>copyable</b>, ActionT&gt;(signer: &signer, proposer_address: address, proposal_id: u64, agree: bool, votes: u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="cast_vote.md#cast_vote">cast_vote</a>&lt;<a href="../../modules/doc/Token.md#0x1_Token">Token</a>: <b>copyable</b>, ActionT&gt;(
    signer: &signer,
    proposer_address: address,
    proposal_id: u64,
    agree: bool,
    votes: u128,
) {
    <b>let</b> votes = <a href="../../modules/doc/Account.md#0x1_Account_withdraw">Account::withdraw</a>&lt;<a href="../../modules/doc/Token.md#0x1_Token">Token</a>&gt;(signer, votes);
    <a href="../../modules/doc/Dao.md#0x1_Dao_cast_vote">Dao::cast_vote</a>&lt;<a href="../../modules/doc/Token.md#0x1_Token">Token</a>, ActionT&gt;(signer, proposer_address, proposal_id, votes, agree);
}
</code></pre>



</details>
