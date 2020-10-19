
<a name="update_txn_timeout_config_proposal"></a>

# Script `update_txn_timeout_config_proposal`



-  [Specification](#@Specification_0)
    -  [Function `update_txn_timeout_config_proposal`](#@Specification_0_update_txn_timeout_config_proposal)


<pre><code><b>use</b> <a href="../../modules/doc/OnChainConfigDao.md#0x1_OnChainConfigDao">0x1::OnChainConfigDao</a>;
<b>use</b> <a href="../../modules/doc/STC.md#0x1_STC">0x1::STC</a>;
<b>use</b> <a href="../../modules/doc/TransactionTimeoutConfig.md#0x1_TransactionTimeoutConfig">0x1::TransactionTimeoutConfig</a>;
</code></pre>




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




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>
