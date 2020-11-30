
<a name="cancel_upgrade_plan"></a>

# Script `cancel_upgrade_plan`



-  [Specification](#@Specification_0)
    -  [Function `cancel_upgrade_plan`](#@Specification_0_cancel_upgrade_plan)


<pre><code><b>use</b> <a href="../../modules/doc/PackageTxnManager.md#0x1_PackageTxnManager">0x1::PackageTxnManager</a>;
</code></pre>




<pre><code><b>public</b> <b>fun</b> <a href="cancel_upgrade_plan.md#cancel_upgrade_plan">cancel_upgrade_plan</a>(signer: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="cancel_upgrade_plan.md#cancel_upgrade_plan">cancel_upgrade_plan</a>(
    signer: &signer,
) {
    <a href="../../modules/doc/PackageTxnManager.md#0x1_PackageTxnManager_cancel_upgrade_plan">PackageTxnManager::cancel_upgrade_plan</a>(signer);
}
</code></pre>



</details>

<a name="@Specification_0"></a>

## Specification


<a name="@Specification_0_cancel_upgrade_plan"></a>

### Function `cancel_upgrade_plan`


<pre><code><b>public</b> <b>fun</b> <a href="cancel_upgrade_plan.md#cancel_upgrade_plan">cancel_upgrade_plan</a>(signer: &signer)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>
