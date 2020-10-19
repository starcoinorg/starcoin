
<a name="execute_proposal"></a>

# Script `execute_proposal`



-  [Specification](#@Specification_0)
    -  [Function `execute_proposal`](#@Specification_0_execute_proposal)


<pre><code><b>use</b> <a href="../../modules/doc/OnChainConfigDao.md#0x1_OnChainConfigDao">0x1::OnChainConfigDao</a>;
<b>use</b> <a href="../../modules/doc/STC.md#0x1_STC">0x1::STC</a>;
<b>use</b> <a href="../../modules/doc/Signer.md#0x1_Signer">0x1::Signer</a>;
</code></pre>




<pre><code><b>public</b> <b>fun</b> <a href="execute_proposal.md#execute_proposal">execute_proposal</a>&lt;ConfigT: <b>copyable</b>&gt;(account: &signer, proposal_id: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="execute_proposal.md#execute_proposal">execute_proposal</a>&lt;ConfigT: <b>copyable</b>&gt;(account: &signer, proposal_id: u64) {
    <a href="../../modules/doc/OnChainConfigDao.md#0x1_OnChainConfigDao_execute">OnChainConfigDao::execute</a>&lt;<a href="../../modules/doc/STC.md#0x1_STC_STC">STC::STC</a>, ConfigT&gt;(<a href="../../modules/doc/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account), proposal_id);
}
</code></pre>



</details>

<a name="@Specification_0"></a>

## Specification


<a name="@Specification_0_execute_proposal"></a>

### Function `execute_proposal`


<pre><code><b>public</b> <b>fun</b> <a href="execute_proposal.md#execute_proposal">execute_proposal</a>&lt;ConfigT: <b>copyable</b>&gt;(account: &signer, proposal_id: u64)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>
