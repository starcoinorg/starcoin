
<a name="SCRIPT"></a>

# Script `queue_proposal_action.move`

### Table of Contents

-  [Function `main`](#SCRIPT_main)



<a name="SCRIPT_main"></a>

## Function `main`



<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_main">main</a>&lt;<a href="../../modules/doc/Token.md#0x1_Token">Token</a>: <b>copyable</b>, Action&gt;(_signer: &signer, proposer_address: address, proposal_id: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_main">main</a>&lt;<a href="../../modules/doc/Token.md#0x1_Token">Token</a>: <b>copyable</b>, Action&gt;(
    _signer: &signer,
    proposer_address: address,
    proposal_id: u64,
) {
    <a href="../../modules/doc/Dao.md#0x1_Dao_queue_proposal_action">Dao::queue_proposal_action</a>&lt;<a href="../../modules/doc/Token.md#0x1_Token">Token</a>, Action&gt;(proposer_address, proposal_id);
}
</code></pre>



</details>
