
<a name="submit_module_upgrade_plan"></a>

# Script `submit_module_upgrade_plan`






<pre><code><b>public</b> <b>fun</b> <a href="submit_module_upgrade_plan.md#submit_module_upgrade_plan">submit_module_upgrade_plan</a>&lt;<a href="../../modules/doc/Token.md#0x1_Token">Token</a>: <b>copyable</b>&gt;(_signer: &signer, proposer_address: address, proposal_id: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="submit_module_upgrade_plan.md#submit_module_upgrade_plan">submit_module_upgrade_plan</a>&lt;<a href="../../modules/doc/Token.md#0x1_Token">Token</a>: <b>copyable</b>&gt;(
    _signer: &signer,
    proposer_address: address,
    proposal_id: u64,
) {
    <a href="../../modules/doc/UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_submit_module_upgrade_plan">UpgradeModuleDaoProposal::submit_module_upgrade_plan</a>&lt;<a href="../../modules/doc/Token.md#0x1_Token">Token</a>&gt;(proposer_address, proposal_id);
}
</code></pre>



</details>
