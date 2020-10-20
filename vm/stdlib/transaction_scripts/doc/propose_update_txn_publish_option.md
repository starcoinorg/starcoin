
<a name="propose_update_txn_publish_option"></a>

# Script `propose_update_txn_publish_option`



-  [Specification](#@Specification_0)
    -  [Function `propose_update_txn_publish_option`](#@Specification_0_propose_update_txn_publish_option)


<pre><code><b>use</b> <a href="../../modules/doc/OnChainConfigDao.md#0x1_OnChainConfigDao">0x1::OnChainConfigDao</a>;
<b>use</b> <a href="../../modules/doc/STC.md#0x1_STC">0x1::STC</a>;
<b>use</b> <a href="../../modules/doc/TransactionPublishOption.md#0x1_TransactionPublishOption">0x1::TransactionPublishOption</a>;
</code></pre>




<pre><code><b>public</b> <b>fun</b> <a href="propose_update_txn_publish_option.md#propose_update_txn_publish_option">propose_update_txn_publish_option</a>(account: &signer, script_allow_list: vector&lt;u8&gt;, module_publishing_allowed: bool, exec_delay: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="propose_update_txn_publish_option.md#propose_update_txn_publish_option">propose_update_txn_publish_option</a>(account: &signer,
    script_allow_list: vector&lt;u8&gt;,
    module_publishing_allowed: bool,
    exec_delay: u64) {
    <b>let</b> txn_publish_option = <a href="../../modules/doc/TransactionPublishOption.md#0x1_TransactionPublishOption_new_transaction_publish_option">TransactionPublishOption::new_transaction_publish_option</a>(script_allow_list, module_publishing_allowed);
    <a href="../../modules/doc/OnChainConfigDao.md#0x1_OnChainConfigDao_propose_update">OnChainConfigDao::propose_update</a>&lt;<a href="../../modules/doc/STC.md#0x1_STC_STC">STC::STC</a>, <a href="../../modules/doc/TransactionPublishOption.md#0x1_TransactionPublishOption_TransactionPublishOption">TransactionPublishOption::TransactionPublishOption</a>&gt;(account, txn_publish_option, exec_delay);
}
</code></pre>



</details>

<a name="@Specification_0"></a>

## Specification


<a name="@Specification_0_propose_update_txn_publish_option"></a>

### Function `propose_update_txn_publish_option`


<pre><code><b>public</b> <b>fun</b> <a href="propose_update_txn_publish_option.md#propose_update_txn_publish_option">propose_update_txn_publish_option</a>(account: &signer, script_allow_list: vector&lt;u8&gt;, module_publishing_allowed: bool, exec_delay: u64)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>
