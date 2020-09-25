
<a name="SCRIPT"></a>

# Script `propose_module_upgrade.move`

### Table of Contents

-  [Function `propose_module_upgrade`](#SCRIPT_propose_module_upgrade)



<a name="SCRIPT_propose_module_upgrade"></a>

## Function `propose_module_upgrade`



<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_propose_module_upgrade">propose_module_upgrade</a>&lt;<a href="../../modules/doc/Token.md#0x1_Token">Token</a>: <b>copyable</b>&gt;(signer: &signer, module_address: address, package_hash: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_propose_module_upgrade">propose_module_upgrade</a>&lt;<a href="../../modules/doc/Token.md#0x1_Token">Token</a>: <b>copyable</b>&gt;(
    signer: &signer,
    module_address: address,
    package_hash: vector&lt;u8&gt;,
) {
    <a href="../../modules/doc/UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_propose_module_upgrade">UpgradeModuleDaoProposal::propose_module_upgrade</a>&lt;<a href="../../modules/doc/Token.md#0x1_Token">Token</a>&gt;(
        signer,
        module_address,
        package_hash,
    );
}
</code></pre>



</details>
