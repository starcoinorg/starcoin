
<a name="propose_module_upgrade"></a>

# Script `propose_module_upgrade`






<pre><code><b>public</b> <b>fun</b> <a href="propose_module_upgrade.md#propose_module_upgrade">propose_module_upgrade</a>&lt;<a href="../../modules/doc/Token.md#0x1_Token">Token</a>: <b>copyable</b>&gt;(signer: &signer, module_address: address, package_hash: vector&lt;u8&gt;, exec_delay: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="propose_module_upgrade.md#propose_module_upgrade">propose_module_upgrade</a>&lt;<a href="../../modules/doc/Token.md#0x1_Token">Token</a>: <b>copyable</b>&gt;(
    signer: &signer,
    module_address: address,
    package_hash: vector&lt;u8&gt;,
    exec_delay: u64,
) {
    <a href="../../modules/doc/UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_propose_module_upgrade">UpgradeModuleDaoProposal::propose_module_upgrade</a>&lt;<a href="../../modules/doc/Token.md#0x1_Token">Token</a>&gt;(
        signer,
        module_address,
        package_hash,
        exec_delay,
    );
}
</code></pre>



</details>
