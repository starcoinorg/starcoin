
<a name="propose_modify_dao_config"></a>

# Script `propose_modify_dao_config`






<pre><code><b>public</b> <b>fun</b> <a href="propose_modify_dao_config.md#propose_modify_dao_config">propose_modify_dao_config</a>&lt;<a href="../../modules/doc/Token.md#0x1_Token">Token</a>: <b>copyable</b>&gt;(signer: &signer, voting_delay: u64, voting_period: u64, voting_quorum_rate: u8, min_action_delay: u64, exec_delay: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="propose_modify_dao_config.md#propose_modify_dao_config">propose_modify_dao_config</a>&lt;<a href="../../modules/doc/Token.md#0x1_Token">Token</a>: <b>copyable</b>&gt;(
    signer: &signer,
    voting_delay: u64,
    voting_period: u64,
    voting_quorum_rate: u8,
    min_action_delay: u64,
    exec_delay: u64,
) {
    <a href="../../modules/doc/ModifyDaoConfigProposal.md#0x1_ModifyDaoConfigProposal_propose">ModifyDaoConfigProposal::propose</a>&lt;<a href="../../modules/doc/Token.md#0x1_Token">Token</a>&gt;(signer, voting_delay, voting_period, voting_quorum_rate, min_action_delay, exec_delay);
}
</code></pre>



</details>
