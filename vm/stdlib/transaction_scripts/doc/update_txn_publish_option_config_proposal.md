
<a name="update_txn_publish_option"></a>

# Script `update_txn_publish_option`



-  [Specification](#@Specification_0)
    -  [Function <code><a href="update_txn_publish_option_config_proposal.md#update_txn_publish_option">update_txn_publish_option</a></code>](#@Specification_0_update_txn_publish_option)



<pre><code><b>public</b> <b>fun</b> <a href="update_txn_publish_option_config_proposal.md#update_txn_publish_option">update_txn_publish_option</a>(account: &signer, script_allow_list: vector&lt;vector&lt;u8&gt;&gt;, module_publishing_allowed: bool, exec_delay: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="update_txn_publish_option_config_proposal.md#update_txn_publish_option">update_txn_publish_option</a>(account: &signer,
    script_allow_list: vector&lt;vector&lt;u8&gt;&gt;,
    module_publishing_allowed: bool,
    exec_delay: u64) {
    <b>let</b> txn_publish_option = <a href="../../modules/doc/TransactionPublishOption.md#0x1_TransactionPublishOption_new_transaction_publish_option">TransactionPublishOption::new_transaction_publish_option</a>(script_allow_list, module_publishing_allowed);
    <a href="../../modules/doc/OnChainConfigDao.md#0x1_OnChainConfigDao_propose_update">OnChainConfigDao::propose_update</a>&lt;<a href="../../modules/doc/STC.md#0x1_STC_STC">STC::STC</a>, <a href="../../modules/doc/TransactionPublishOption.md#0x1_TransactionPublishOption_TransactionPublishOption">TransactionPublishOption::TransactionPublishOption</a>&gt;(account, txn_publish_option, exec_delay);
}
</code></pre>



</details>

<a name="@Specification_0"></a>

## Specification


<a name="@Specification_0_update_txn_publish_option"></a>

### Function `update_txn_publish_option`


<pre><code><b>public</b> <b>fun</b> <a href="update_txn_publish_option_config_proposal.md#update_txn_publish_option">update_txn_publish_option</a>(account: &signer, script_allow_list: vector&lt;vector&lt;u8&gt;&gt;, module_publishing_allowed: bool, exec_delay: u64)
</code></pre>




<pre><code>pragma verify = <b>false</b>;
</code></pre>
