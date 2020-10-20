
<a name="propose_update_version"></a>

# Script `propose_update_version`



-  [Specification](#@Specification_0)
    -  [Function `propose_update_version`](#@Specification_0_propose_update_version)


<pre><code><b>use</b> <a href="../../modules/doc/OnChainConfigDao.md#0x1_OnChainConfigDao">0x1::OnChainConfigDao</a>;
<b>use</b> <a href="../../modules/doc/STC.md#0x1_STC">0x1::STC</a>;
<b>use</b> <a href="../../modules/doc/Version.md#0x1_Version">0x1::Version</a>;
</code></pre>




<pre><code><b>public</b> <b>fun</b> <a href="propose_update_version.md#propose_update_version">propose_update_version</a>(account: &signer, major: u64, exec_delay: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="propose_update_version.md#propose_update_version">propose_update_version</a>(account: &signer,
    major: u64,
    exec_delay: u64) {
    <b>let</b> version = <a href="../../modules/doc/Version.md#0x1_Version_new_version">Version::new_version</a>(major);
    <a href="../../modules/doc/OnChainConfigDao.md#0x1_OnChainConfigDao_propose_update">OnChainConfigDao::propose_update</a>&lt;<a href="../../modules/doc/STC.md#0x1_STC_STC">STC::STC</a>, <a href="../../modules/doc/Version.md#0x1_Version_Version">Version::Version</a>&gt;(account, version, exec_delay);
}
</code></pre>



</details>

<a name="@Specification_0"></a>

## Specification


<a name="@Specification_0_propose_update_version"></a>

### Function `propose_update_version`


<pre><code><b>public</b> <b>fun</b> <a href="propose_update_version.md#propose_update_version">propose_update_version</a>(account: &signer, major: u64, exec_delay: u64)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>
