
<a name="update_version_proposal"></a>

# Script `update_version_proposal`



-  [Specification](#@Specification_0)
    -  [Function <code><a href="update_version_proposal.md#update_version_proposal">update_version_proposal</a></code>](#@Specification_0_update_version_proposal)



<pre><code><b>public</b> <b>fun</b> <a href="update_version_proposal.md#update_version_proposal">update_version_proposal</a>(account: &signer, major: u64, exec_delay: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="update_version_proposal.md#update_version_proposal">update_version_proposal</a>(account: &signer,
    major: u64,
    exec_delay: u64) {
    <b>let</b> version = <a href="../../modules/doc/Version.md#0x1_Version_new_version">Version::new_version</a>(major);
    <a href="../../modules/doc/OnChainConfigDao.md#0x1_OnChainConfigDao_propose_update">OnChainConfigDao::propose_update</a>&lt;<a href="../../modules/doc/STC.md#0x1_STC_STC">STC::STC</a>, <a href="../../modules/doc/Version.md#0x1_Version_Version">Version::Version</a>&gt;(account, version, exec_delay);
}
</code></pre>



</details>

<a name="@Specification_0"></a>

## Specification


<a name="@Specification_0_update_version_proposal"></a>

### Function `update_version_proposal`


<pre><code><b>public</b> <b>fun</b> <a href="update_version_proposal.md#update_version_proposal">update_version_proposal</a>(account: &signer, major: u64, exec_delay: u64)
</code></pre>




<pre><code>pragma verify = <b>false</b>;
</code></pre>
