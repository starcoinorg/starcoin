
<a name="revoke_vote"></a>

# Script `revoke_vote`





<pre><code><b>use</b> <a href="../../modules/doc/Account.md#0x1_Account">0x1::Account</a>;
<b>use</b> <a href="../../modules/doc/Dao.md#0x1_Dao">0x1::Dao</a>;
<b>use</b> <a href="../../modules/doc/Signer.md#0x1_Signer">0x1::Signer</a>;
<b>use</b> <a href="../../modules/doc/Token.md#0x1_Token">0x1::Token</a>;
</code></pre>




<pre><code><b>public</b> <b>fun</b> <a href="revoke_vote.md#revoke_vote">revoke_vote</a>&lt;<a href="../../modules/doc/Token.md#0x1_Token">Token</a>: <b>copyable</b>, Action&gt;(signer: &signer, proposer_address: address, proposal_id: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="revoke_vote.md#revoke_vote">revoke_vote</a>&lt;<a href="../../modules/doc/Token.md#0x1_Token">Token</a>: <b>copyable</b>, Action&gt;(
    signer: &signer,
    proposer_address: address,
    proposal_id: u64,
) {
    <b>let</b> sender = <a href="../../modules/doc/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer);
    <b>let</b> (_, power) = <a href="../../modules/doc/Dao.md#0x1_Dao_vote_of">Dao::vote_of</a>&lt;<a href="../../modules/doc/Token.md#0x1_Token">Token</a>&gt;(sender, proposer_address, proposal_id);
    <b>let</b> my_token = <a href="../../modules/doc/Dao.md#0x1_Dao_revoke_vote">Dao::revoke_vote</a>&lt;<a href="../../modules/doc/Token.md#0x1_Token">Token</a>, Action&gt;(signer, proposer_address, proposal_id, power);
    <a href="../../modules/doc/Account.md#0x1_Account_deposit">Account::deposit</a>(sender, my_token);
}
</code></pre>



</details>
