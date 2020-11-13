
<a name="update_module_upgrade_strategy"></a>

# Script `update_module_upgrade_strategy`





<pre><code><b>use</b> <a href="../../modules/doc/Config.md#0x1_Config">0x1::Config</a>;
<b>use</b> <a href="../../modules/doc/PackageTxnManager.md#0x1_PackageTxnManager">0x1::PackageTxnManager</a>;
<b>use</b> <a href="../../modules/doc/Signer.md#0x1_Signer">0x1::Signer</a>;
<b>use</b> <a href="../../modules/doc/Version.md#0x1_Version">0x1::Version</a>;
</code></pre>




<pre><code><b>public</b> <b>fun</b> <a href="update_module_upgrade_strategy.md#update_module_upgrade_strategy">update_module_upgrade_strategy</a>(signer: &signer, strategy: u8)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="update_module_upgrade_strategy.md#update_module_upgrade_strategy">update_module_upgrade_strategy</a>(
    signer: &signer,
    strategy: u8,
) {
    // 1. check version
    <b>if</b> (strategy == <a href="../../modules/doc/PackageTxnManager.md#0x1_PackageTxnManager_get_strategy_two_phase">PackageTxnManager::get_strategy_two_phase</a>()){
        <b>if</b> (!<a href="../../modules/doc/Config.md#0x1_Config_config_exist_by_address">Config::config_exist_by_address</a>&lt;<a href="../../modules/doc/Version.md#0x1_Version_Version">Version::Version</a>&gt;(<a href="../../modules/doc/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer))) {
            <a href="../../modules/doc/Config.md#0x1_Config_publish_new_config">Config::publish_new_config</a>&lt;<a href="../../modules/doc/Version.md#0x1_Version_Version">Version::Version</a>&gt;(signer, <a href="../../modules/doc/Version.md#0x1_Version_new_version">Version::new_version</a>(1));
        }
    };

    // 2. <b>update</b> strategy
    <a href="../../modules/doc/PackageTxnManager.md#0x1_PackageTxnManager_update_module_upgrade_strategy">PackageTxnManager::update_module_upgrade_strategy</a>(
        signer,
        strategy,
    );
}
</code></pre>



</details>
