
<a name="0x1_VMConfig"></a>

# Module `0x1::VMConfig`

### Table of Contents

-  [Struct `VMConfig`](#0x1_VMConfig_VMConfig)
-  [Struct `GasSchedule`](#0x1_VMConfig_GasSchedule)
-  [Struct `GasConstants`](#0x1_VMConfig_GasConstants)
-  [Function `initialize`](#0x1_VMConfig_initialize)
-  [Function `set_publishing_option`](#0x1_VMConfig_set_publishing_option)
-  [Specification](#0x1_VMConfig_Specification)
    -  [Function `initialize`](#0x1_VMConfig_Specification_initialize)
    -  [Function `set_publishing_option`](#0x1_VMConfig_Specification_set_publishing_option)



<a name="0x1_VMConfig_VMConfig"></a>

## Struct `VMConfig`



<pre><code><b>struct</b> <a href="#0x1_VMConfig">VMConfig</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>publishing_option: vector&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>

<code>gas_schedule: <a href="#0x1_VMConfig_GasSchedule">VMConfig::GasSchedule</a></code>
</dt>
<dd>

</dd>
<dt>

<code>block_gas_limit: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_VMConfig_GasSchedule"></a>

## Struct `GasSchedule`



<pre><code><b>struct</b> <a href="#0x1_VMConfig_GasSchedule">GasSchedule</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>instruction_schedule: vector&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>

<code>native_schedule: vector&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>

<code>gas_constants: <a href="#0x1_VMConfig_GasConstants">VMConfig::GasConstants</a></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_VMConfig_GasConstants"></a>

## Struct `GasConstants`



<pre><code><b>struct</b> <a href="#0x1_VMConfig_GasConstants">GasConstants</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>global_memory_per_byte_cost: u64</code>
</dt>
<dd>
 The cost per-byte written to global storage.
</dd>
<dt>

<code>global_memory_per_byte_write_cost: u64</code>
</dt>
<dd>
 The cost per-byte written to storage.
</dd>
<dt>

<code>min_transaction_gas_units: u64</code>
</dt>
<dd>
 We charge one unit of gas per-byte for the first 600 bytes
</dd>
<dt>

<code>large_transaction_cutoff: u64</code>
</dt>
<dd>
 Any transaction over this size will be charged
<code>INTRINSIC_GAS_PER_BYTE</code> per byte
</dd>
<dt>

<code>instrinsic_gas_per_byte: u64</code>
</dt>
<dd>
 The units of gas that should be charged per byte for every transaction.
</dd>
<dt>

<code>maximum_number_of_gas_units: u64</code>
</dt>
<dd>
 1 nanosecond should equal one unit of computational gas. We bound the maximum
 computational time of any given transaction at 10 milliseconds. We want this number and
 <code>MAX_PRICE_PER_GAS_UNIT</code> to always satisfy the inequality that
         MAXIMUM_NUMBER_OF_GAS_UNITS * MAX_PRICE_PER_GAS_UNIT < min(u64::MAX, GasUnits<GasCarrier>::MAX)
</dd>
<dt>

<code>min_price_per_gas_unit: u64</code>
</dt>
<dd>
 The minimum gas price that a transaction can be submitted with.
</dd>
<dt>

<code>max_price_per_gas_unit: u64</code>
</dt>
<dd>
 The maximum gas unit price that a transaction can be submitted with.
</dd>
<dt>

<code>max_transaction_size_in_bytes: u64</code>
</dt>
<dd>

</dd>
<dt>

<code>gas_unit_scaling_factor: u64</code>
</dt>
<dd>

</dd>
<dt>

<code>default_account_size: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_VMConfig_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VMConfig_initialize">initialize</a>(account: &signer, publishing_option: vector&lt;u8&gt;, instruction_schedule: vector&lt;u8&gt;, native_schedule: vector&lt;u8&gt;, block_gas_limit: u64, global_memory_per_byte_cost: u64, global_memory_per_byte_write_cost: u64, min_transaction_gas_units: u64, large_transaction_cutoff: u64, instrinsic_gas_per_byte: u64, maximum_number_of_gas_units: u64, min_price_per_gas_unit: u64, max_price_per_gas_unit: u64, max_transaction_size_in_bytes: u64, gas_unit_scaling_factor: u64, default_account_size: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VMConfig_initialize">initialize</a>(
    account: &signer,
    publishing_option: vector&lt;u8&gt;,
    instruction_schedule: vector&lt;u8&gt;,
    native_schedule: vector&lt;u8&gt;,
    block_gas_limit: u64,
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
    default_account_size: u64
) {
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>(), 1);
    //TODO pass gas_constants <b>as</b> init argument and onchain config.
    <b>let</b> gas_constants = <a href="#0x1_VMConfig_GasConstants">GasConstants</a> {
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
        default_account_size
    };

    <a href="Config.md#0x1_Config_publish_new_config">Config::publish_new_config</a>&lt;<a href="#0x1_VMConfig">VMConfig</a>&gt;(
        account,
        <a href="#0x1_VMConfig">VMConfig</a> {
            publishing_option,
            gas_schedule: <a href="#0x1_VMConfig_GasSchedule">GasSchedule</a> {
                instruction_schedule,
                native_schedule,
                gas_constants,
            },
            block_gas_limit
        },
    );
}
</code></pre>



</details>

<a name="0x1_VMConfig_set_publishing_option"></a>

## Function `set_publishing_option`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VMConfig_set_publishing_option">set_publishing_option</a>(account: &signer, publishing_option: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VMConfig_set_publishing_option">set_publishing_option</a>(account: &signer, publishing_option: vector&lt;u8&gt;) {
    <b>let</b> current_config = <a href="Config.md#0x1_Config_get">Config::get</a>&lt;<a href="#0x1_VMConfig">VMConfig</a>&gt;(account);
    current_config.publishing_option = publishing_option;
    <a href="Config.md#0x1_Config_set">Config::set</a>&lt;<a href="#0x1_VMConfig">VMConfig</a>&gt;(account, current_config);
}
</code></pre>



</details>

<a name="0x1_VMConfig_Specification"></a>

## Specification



<pre><code>pragma verify;
pragma aborts_if_is_strict;
</code></pre>



<a name="0x1_VMConfig_Specification_initialize"></a>

### Function `initialize`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VMConfig_initialize">initialize</a>(account: &signer, publishing_option: vector&lt;u8&gt;, instruction_schedule: vector&lt;u8&gt;, native_schedule: vector&lt;u8&gt;, block_gas_limit: u64, global_memory_per_byte_cost: u64, global_memory_per_byte_write_cost: u64, min_transaction_gas_units: u64, large_transaction_cutoff: u64, instrinsic_gas_per_byte: u64, maximum_number_of_gas_units: u64, min_price_per_gas_unit: u64, max_price_per_gas_unit: u64, max_transaction_size_in_bytes: u64, gas_unit_scaling_factor: u64, default_account_size: u64)
</code></pre>




<pre><code><b>aborts_if</b> <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account) != <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>();
<b>aborts_if</b> exists&lt;<a href="Config.md#0x1_Config_Config">Config::Config</a>&lt;<a href="#0x1_VMConfig">VMConfig</a>&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
<b>aborts_if</b> exists&lt;<a href="Config.md#0x1_Config_ModifyConfigCapabilityHolder">Config::ModifyConfigCapabilityHolder</a>&lt;<a href="#0x1_VMConfig">VMConfig</a>&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
<b>ensures</b> exists&lt;<a href="Config.md#0x1_Config_Config">Config::Config</a>&lt;<a href="#0x1_VMConfig">VMConfig</a>&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
<b>ensures</b> exists&lt;<a href="Config.md#0x1_Config_ModifyConfigCapabilityHolder">Config::ModifyConfigCapabilityHolder</a>&lt;<a href="#0x1_VMConfig">VMConfig</a>&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
</code></pre>



<a name="0x1_VMConfig_Specification_set_publishing_option"></a>

### Function `set_publishing_option`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_VMConfig_set_publishing_option">set_publishing_option</a>(account: &signer, publishing_option: vector&lt;u8&gt;)
</code></pre>




<pre><code>pragma verify = <b>false</b>;
</code></pre>
