
<a name="queue_proposal_action"></a>

# Script `queue_proposal_action`





<pre><code><b>use</b> <a href="../../modules/doc/Dao.md#0x1_Dao">0x1::Dao</a>;
</code></pre>




<pre><code><b>public</b> <b>fun</b> <a href="queue_proposal_action.md#queue_proposal_action">queue_proposal_action</a>&lt;<a href="../../modules/doc/Token.md#0x1_Token">Token</a>: <b>copyable</b>, Action&gt;(_signer: &signer, proposer_address: address, proposal_id: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="queue_proposal_action.md#queue_proposal_action">queue_proposal_action</a>&lt;<a href="../../modules/doc/Token.md#0x1_Token">Token</a>: <b>copyable</b>, Action&gt;(
    _signer: &signer,
    proposer_address: address,
    proposal_id: u64,
) {
    <a href="../../modules/doc/Dao.md#0x1_Dao_queue_proposal_action">Dao::queue_proposal_action</a>&lt;<a href="../../modules/doc/Token.md#0x1_Token">Token</a>, Action&gt;(proposer_address, proposal_id);
}
</code></pre>



</details>
