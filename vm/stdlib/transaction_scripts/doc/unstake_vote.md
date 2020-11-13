
<a name="unstake_vote"></a>

# Script `unstake_vote`





<pre><code><b>use</b> <a href="../../modules/doc/Account.md#0x1_Account">0x1::Account</a>;
<b>use</b> <a href="../../modules/doc/Dao.md#0x1_Dao">0x1::Dao</a>;
<b>use</b> <a href="../../modules/doc/Signer.md#0x1_Signer">0x1::Signer</a>;
<b>use</b> <a href="../../modules/doc/Token.md#0x1_Token">0x1::Token</a>;
</code></pre>




<pre><code><b>public</b> <b>fun</b> <a href="unstake_vote.md#unstake_vote">unstake_vote</a>&lt;<a href="../../modules/doc/Token.md#0x1_Token">Token</a>: <b>copyable</b>, Action: <b>copyable</b>&gt;(signer: &signer, proposer_address: address, proposal_id: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="unstake_vote.md#unstake_vote">unstake_vote</a>&lt;<a href="../../modules/doc/Token.md#0x1_Token">Token</a>: <b>copyable</b>, Action: <b>copyable</b>&gt;(
    signer: &signer,
    proposer_address: address,
    proposal_id: u64,
) {
    <b>let</b> my_token = <a href="../../modules/doc/Dao.md#0x1_Dao_unstake_votes">Dao::unstake_votes</a>&lt;<a href="../../modules/doc/Token.md#0x1_Token">Token</a>, Action&gt;(signer, proposer_address, proposal_id);
    <a href="../../modules/doc/Account.md#0x1_Account_deposit">Account::deposit</a>(<a href="../../modules/doc/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer), my_token);
}
</code></pre>



</details>
