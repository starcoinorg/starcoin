
<a name="update_txn_timeout_config_proposal"></a>

# Script `update_txn_timeout_config_proposal`



-  [Specification](#@Specification_0)
    -  [Function <code><a href="update_txn_timeout_config_proposal.md#update_txn_timeout_config_proposal">update_txn_timeout_config_proposal</a></code>](#@Specification_0_update_txn_timeout_config_proposal)



<pre><code><b>public</b> <b>fun</b> <a href="update_txn_timeout_config_proposal.md#update_txn_timeout_config_proposal">update_txn_timeout_config_proposal</a>(account: &signer, duration_seconds: u64, exec_delay: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="update_txn_timeout_config_proposal.md#update_txn_timeout_config_proposal">update_txn_timeout_config_proposal</a>(account: &signer,
    duration_seconds: u64,
    exec_delay: u64) {
    <b>let</b> txn_timeout_config = <a href="../../modules/doc/TransactionTimeoutConfig.md#0x1_TransactionTimeoutConfig_new_transaction_timeout_config">TransactionTimeoutConfig::new_transaction_timeout_config</a>(duration_seconds);
    <a href="../../modules/doc/OnChainConfigDao.md#0x1_OnChainConfigDao_propose_update">OnChainConfigDao::propose_update</a>&lt;<a href="../../modules/doc/STC.md#0x1_STC_STC">STC::STC</a>, <a href="../../modules/doc/TransactionTimeoutConfig.md#0x1_TransactionTimeoutConfig_TransactionTimeoutConfig">TransactionTimeoutConfig::TransactionTimeoutConfig</a>&gt;(account, txn_timeout_config, exec_delay);
}
</code></pre>



</details>

<a name="@Specification_0"></a>

## Specification


<a name="@Specification_0_update_txn_timeout_config_proposal"></a>

### Function `update_txn_timeout_config_proposal`


<pre><code><b>public</b> <b>fun</b> <a href="update_txn_timeout_config_proposal.md#update_txn_timeout_config_proposal">update_txn_timeout_config_proposal</a>(account: &signer, duration_seconds: u64, exec_delay: u64)
</code></pre>




<pre><code>pragma verify = <b>false</b>;
</code></pre>
