
<a name="0x1_ModuleUpgradeScripts"></a>

# Module `0x1::ModuleUpgradeScripts`



-  [Function `propose_module_upgrade_v2`](#0x1_ModuleUpgradeScripts_propose_module_upgrade_v2)
-  [Function `update_module_upgrade_strategy`](#0x1_ModuleUpgradeScripts_update_module_upgrade_strategy)
-  [Function `submit_module_upgrade_plan`](#0x1_ModuleUpgradeScripts_submit_module_upgrade_plan)
-  [Function `cancel_upgrade_plan`](#0x1_ModuleUpgradeScripts_cancel_upgrade_plan)
-  [Specification](#@Specification_0)
    -  [Function `cancel_upgrade_plan`](#@Specification_0_cancel_upgrade_plan)


<pre><code><b>use</b> <a href="Config.md#0x1_Config">0x1::Config</a>;
<b>use</b> <a href="Option.md#0x1_Option">0x1::Option</a>;
<b>use</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager">0x1::PackageTxnManager</a>;
<b>use</b> <a href="Signer.md#0x1_Signer">0x1::Signer</a>;
<b>use</b> <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal">0x1::UpgradeModuleDaoProposal</a>;
<b>use</b> <a href="Version.md#0x1_Version">0x1::Version</a>;
</code></pre>



<a name="0x1_ModuleUpgradeScripts_propose_module_upgrade_v2"></a>

## Function `propose_module_upgrade_v2`



<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="ModuleUpgradeScripts.md#0x1_ModuleUpgradeScripts_propose_module_upgrade_v2">propose_module_upgrade_v2</a>&lt;<a href="Token.md#0x1_Token">Token</a>: <b>copyable</b>&gt;(signer: signer, module_address: address, package_hash: vector&lt;u8&gt;, version: u64, exec_delay: u64, enforced: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="ModuleUpgradeScripts.md#0x1_ModuleUpgradeScripts_propose_module_upgrade_v2">propose_module_upgrade_v2</a>&lt;<a href="Token.md#0x1_Token">Token</a>: <b>copy</b> + drop + store&gt;(
    signer: signer,
    module_address: address,
    package_hash: vector&lt;u8&gt;,
    version: u64,
    exec_delay: u64,
    enforced: bool,
) {
    <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_propose_module_upgrade_v2">UpgradeModuleDaoProposal::propose_module_upgrade_v2</a>&lt;<a href="Token.md#0x1_Token">Token</a>&gt;(
        &signer,
        module_address,
        package_hash,
        version,
        exec_delay,
        enforced
    );
}
</code></pre>



</details>

<a name="0x1_ModuleUpgradeScripts_update_module_upgrade_strategy"></a>

## Function `update_module_upgrade_strategy`



<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="ModuleUpgradeScripts.md#0x1_ModuleUpgradeScripts_update_module_upgrade_strategy">update_module_upgrade_strategy</a>(signer: signer, strategy: u8)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="ModuleUpgradeScripts.md#0x1_ModuleUpgradeScripts_update_module_upgrade_strategy">update_module_upgrade_strategy</a>(
    signer: signer,
    strategy: u8,
) {
    // 1. check version
    <b>if</b> (strategy == <a href="PackageTxnManager.md#0x1_PackageTxnManager_get_strategy_two_phase">PackageTxnManager::get_strategy_two_phase</a>()) {
        <b>if</b> (!<a href="Config.md#0x1_Config_config_exist_by_address">Config::config_exist_by_address</a>&lt;<a href="Version.md#0x1_Version_Version">Version::Version</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(&signer))) {
            <a href="Config.md#0x1_Config_publish_new_config">Config::publish_new_config</a>&lt;<a href="Version.md#0x1_Version_Version">Version::Version</a>&gt;(&signer, <a href="Version.md#0x1_Version_new_version">Version::new_version</a>(1));
        }
    };

    // 2. <b>update</b> strategy
    <a href="PackageTxnManager.md#0x1_PackageTxnManager_update_module_upgrade_strategy">PackageTxnManager::update_module_upgrade_strategy</a>(
        &signer,
        strategy,
        <a href="Option.md#0x1_Option_none">Option::none</a>&lt;u64&gt;(),
    );
}
</code></pre>



</details>

<a name="0x1_ModuleUpgradeScripts_submit_module_upgrade_plan"></a>

## Function `submit_module_upgrade_plan`



<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="ModuleUpgradeScripts.md#0x1_ModuleUpgradeScripts_submit_module_upgrade_plan">submit_module_upgrade_plan</a>&lt;<a href="Token.md#0x1_Token">Token</a>: <b>copyable</b>&gt;(_signer: signer, proposer_address: address, proposal_id: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="ModuleUpgradeScripts.md#0x1_ModuleUpgradeScripts_submit_module_upgrade_plan">submit_module_upgrade_plan</a>&lt;<a href="Token.md#0x1_Token">Token</a>: <b>copy</b> + drop + store&gt;(
    _signer: signer,
    proposer_address: address,
    proposal_id: u64,
) {
    <a href="UpgradeModuleDaoProposal.md#0x1_UpgradeModuleDaoProposal_submit_module_upgrade_plan">UpgradeModuleDaoProposal::submit_module_upgrade_plan</a>&lt;<a href="Token.md#0x1_Token">Token</a>&gt;(proposer_address, proposal_id);
}
</code></pre>



</details>

<a name="0x1_ModuleUpgradeScripts_cancel_upgrade_plan"></a>

## Function `cancel_upgrade_plan`



<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="ModuleUpgradeScripts.md#0x1_ModuleUpgradeScripts_cancel_upgrade_plan">cancel_upgrade_plan</a>(signer: signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="ModuleUpgradeScripts.md#0x1_ModuleUpgradeScripts_cancel_upgrade_plan">cancel_upgrade_plan</a>(
    signer: signer,
) {
    <a href="PackageTxnManager.md#0x1_PackageTxnManager_cancel_upgrade_plan">PackageTxnManager::cancel_upgrade_plan</a>(&signer);
}
</code></pre>



</details>

<a name="@Specification_0"></a>

## Specification



<pre><code><b>pragma</b> verify = <b>false</b>;
<b>pragma</b> aborts_if_is_partial = <b>false</b>;
<b>pragma</b> aborts_if_is_strict = <b>true</b>;
</code></pre>



<a name="@Specification_0_cancel_upgrade_plan"></a>

### Function `cancel_upgrade_plan`


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="ModuleUpgradeScripts.md#0x1_ModuleUpgradeScripts_cancel_upgrade_plan">cancel_upgrade_plan</a>(signer: signer)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>
