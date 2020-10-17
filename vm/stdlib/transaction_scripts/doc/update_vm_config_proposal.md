
<a name="update_vm_config"></a>

# Script `update_vm_config`



-  [Specification](#@Specification_0)
    -  [Function <code><a href="update_vm_config_proposal.md#update_vm_config">update_vm_config</a></code>](#@Specification_0_update_vm_config)



<pre><code><b>public</b> <b>fun</b> <a href="update_vm_config_proposal.md#update_vm_config">update_vm_config</a>(account: &signer, instruction_schedule: vector&lt;u8&gt;, native_schedule: vector&lt;u8&gt;, global_memory_per_byte_cost: u64, global_memory_per_byte_write_cost: u64, min_transaction_gas_units: u64, large_transaction_cutoff: u64, instrinsic_gas_per_byte: u64, maximum_number_of_gas_units: u64, min_price_per_gas_unit: u64, max_price_per_gas_unit: u64, max_transaction_size_in_bytes: u64, gas_unit_scaling_factor: u64, default_account_size: u64, exec_delay: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="update_vm_config_proposal.md#update_vm_config">update_vm_config</a>(account: &signer,
    instruction_schedule: vector&lt;u8&gt;,
    native_schedule: vector&lt;u8&gt;,
    global_memory_per_byte_cost: u64,
    global_memory_per_byte_write_cost: u64,
    min_transaction_gas_units: u64,
    large_transaction_cutoff: u64,
    instrinsic_gas_per_byte: u64,
    maximum_number_of_gas_units: u64,
    min_price_per_gas_unit: u64,
    max_price_per_gas_unit: u64,
    max_transaction_size_in_bytes: u64,
    gas_unit_scaling_factor: u64,
    default_account_size: u64,
    exec_delay: u64,) {
    <b>let</b> vm_config = <a href="../../modules/doc/VMConfig.md#0x1_VMConfig_new_vm_config">VMConfig::new_vm_config</a>(instruction_schedule,
                        native_schedule,
                        global_memory_per_byte_cost,
                        global_memory_per_byte_write_cost,
                        min_transaction_gas_units,
                        large_transaction_cutoff,
                        instrinsic_gas_per_byte,
                        maximum_number_of_gas_units,
                        min_price_per_gas_unit,
                        max_price_per_gas_unit,
                        max_transaction_size_in_bytes,
                        gas_unit_scaling_factor,
                        default_account_size);
    <a href="../../modules/doc/OnChainConfigDao.md#0x1_OnChainConfigDao_propose_update">OnChainConfigDao::propose_update</a>&lt;<a href="../../modules/doc/STC.md#0x1_STC_STC">STC::STC</a>, <a href="../../modules/doc/VMConfig.md#0x1_VMConfig_VMConfig">VMConfig::VMConfig</a>&gt;(account, vm_config, exec_delay);
}
</code></pre>



</details>

<a name="@Specification_0"></a>

## Specification


<a name="@Specification_0_update_vm_config"></a>

### Function `update_vm_config`


<pre><code><b>public</b> <b>fun</b> <a href="update_vm_config_proposal.md#update_vm_config">update_vm_config</a>(account: &signer, instruction_schedule: vector&lt;u8&gt;, native_schedule: vector&lt;u8&gt;, global_memory_per_byte_cost: u64, global_memory_per_byte_write_cost: u64, min_transaction_gas_units: u64, large_transaction_cutoff: u64, instrinsic_gas_per_byte: u64, maximum_number_of_gas_units: u64, min_price_per_gas_unit: u64, max_price_per_gas_unit: u64, max_transaction_size_in_bytes: u64, gas_unit_scaling_factor: u64, default_account_size: u64, exec_delay: u64)
</code></pre>




<pre><code>pragma verify = <b>false</b>;
</code></pre>
