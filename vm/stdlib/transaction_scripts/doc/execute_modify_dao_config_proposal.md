
<a name="execute_modify_dao_config_proposal"></a>

# Script `execute_modify_dao_config_proposal`





<pre><code><b>use</b> <a href="../../modules/doc/ModifyDaoConfigProposal.md#0x1_ModifyDaoConfigProposal">0x1::ModifyDaoConfigProposal</a>;
</code></pre>




<pre><code><b>public</b> <b>fun</b> <a href="execute_modify_dao_config_proposal.md#execute_modify_dao_config_proposal">execute_modify_dao_config_proposal</a>&lt;<a href="../../modules/doc/Token.md#0x1_Token">Token</a>: <b>copyable</b>&gt;(_signer: &signer, proposer_address: address, proposal_id: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="execute_modify_dao_config_proposal.md#execute_modify_dao_config_proposal">execute_modify_dao_config_proposal</a>&lt;<a href="../../modules/doc/Token.md#0x1_Token">Token</a>: <b>copyable</b>&gt;(
    _signer: &signer,
    proposer_address: address,
    proposal_id: u64,
) {
    <a href="../../modules/doc/ModifyDaoConfigProposal.md#0x1_ModifyDaoConfigProposal_execute">ModifyDaoConfigProposal::execute</a>&lt;<a href="../../modules/doc/Token.md#0x1_Token">Token</a>&gt;(proposer_address, proposal_id);
}
</code></pre>



</details>
