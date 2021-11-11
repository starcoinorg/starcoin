
<a name="0x1_VMConfig"></a>

# Module `0x1::VMConfig`

<code><a href="VMConfig.md#0x1_VMConfig">VMConfig</a></code> keep track of VM related configuration, like gas schedule.


-  [Struct `VMConfig`](#0x1_VMConfig_VMConfig)
-  [Struct `GasSchedule`](#0x1_VMConfig_GasSchedule)
-  [Struct `GasConstants`](#0x1_VMConfig_GasConstants)
-  [Struct `GasCost`](#0x1_VMConfig_GasCost)
-  [Function `instruction_schedule`](#0x1_VMConfig_instruction_schedule)
-  [Function `native_schedule`](#0x1_VMConfig_native_schedule)
-  [Function `gas_constants`](#0x1_VMConfig_gas_constants)
-  [Function `new_gas_cost`](#0x1_VMConfig_new_gas_cost)
-  [Function `new_vm_config`](#0x1_VMConfig_new_vm_config)
-  [Function `initialize`](#0x1_VMConfig_initialize)
-  [Specification](#@Specification_0)
    -  [Function `initialize`](#@Specification_0_initialize)


<pre><code><b>use</b> <a href="ChainId.md#0x1_ChainId">0x1::ChainId</a>;
<b>use</b> <a href="Config.md#0x1_Config">0x1::Config</a>;
<b>use</b> <a href="CoreAddresses.md#0x1_CoreAddresses">0x1::CoreAddresses</a>;
<b>use</b> <a href="Vector.md#0x1_Vector">0x1::Vector</a>;
</code></pre>



<a name="0x1_VMConfig_VMConfig"></a>

## Struct `VMConfig`

The struct to hold all config data needed to operate the VM.
* gas_schedule: Cost of running the VM.


<pre><code><b>struct</b> <a href="VMConfig.md#0x1_VMConfig">VMConfig</a> has <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>gas_schedule: <a href="VMConfig.md#0x1_VMConfig_GasSchedule">VMConfig::GasSchedule</a></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_VMConfig_GasSchedule"></a>

## Struct `GasSchedule`

The gas schedule keeps two separate schedules for the gas:
* The instruction_schedule: This holds the gas for each bytecode instruction.
* The native_schedule: This holds the gas for used (per-byte operated over) for each native
function.
A couple notes:
1. In the case that an instruction is deleted from the bytecode, that part of the cost schedule
still needs to remain the same; once a slot in the table is taken by an instruction, that is its
slot for the rest of time (since that instruction could already exist in a module on-chain).
2. The initialization of the module will publish the instruction table to the genesis
address, and will preload the vector with the gas schedule for instructions. The VM will then
load this into memory at the startup of each block.


<pre><code><b>struct</b> <a href="VMConfig.md#0x1_VMConfig_GasSchedule">GasSchedule</a> has <b>copy</b>, drop, store
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
<code>gas_constants: <a href="VMConfig.md#0x1_VMConfig_GasConstants">VMConfig::GasConstants</a></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_VMConfig_GasConstants"></a>

## Struct `GasConstants`

The gas constants contains all kind of constants used in gas calculation.


<pre><code><b>struct</b> <a href="VMConfig.md#0x1_VMConfig_GasConstants">GasConstants</a> has <b>copy</b>, drop, store
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
 Any transaction over this size will be charged <code>INTRINSIC_GAS_PER_BYTE</code> per byte
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
 The max transaction size in bytes that a transaction can have.
</dd>
<dt>
<code>gas_unit_scaling_factor: u64</code>
</dt>
<dd>
 gas unit scaling factor.
</dd>
<dt>
<code>default_account_size: u64</code>
</dt>
<dd>
 default account size.
</dd>
</dl>


</details>

<a name="0x1_VMConfig_GasCost"></a>

## Struct `GasCost`

The  <code><a href="VMConfig.md#0x1_VMConfig_GasCost">GasCost</a></code> tracks:
- instruction cost: how much time/computational power is needed to perform the instruction
- memory cost: how much memory is required for the instruction, and storage overhead


<pre><code><b>struct</b> <a href="VMConfig.md#0x1_VMConfig_GasCost">GasCost</a> has <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>instruction_gas: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>memory_gas: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_VMConfig_instruction_schedule"></a>

## Function `instruction_schedule`



<pre><code><b>public</b> <b>fun</b> <a href="VMConfig.md#0x1_VMConfig_instruction_schedule">instruction_schedule</a>(): vector&lt;<a href="VMConfig.md#0x1_VMConfig_GasCost">VMConfig::GasCost</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="VMConfig.md#0x1_VMConfig_instruction_schedule">instruction_schedule</a>(): vector&lt;<a href="VMConfig.md#0x1_VMConfig_GasCost">GasCost</a>&gt; {
    <b>let</b> table = <a href="Vector.md#0x1_Vector_empty">Vector::empty</a>();

    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(1, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(638, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(1, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(1, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(1, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(1, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(1, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(1, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(1, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(1, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(1, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(1, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(2, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(1, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(1, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(1, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(1132, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(2, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(2, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(1, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(1, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(1, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(1, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(1, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(1, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(3, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(2, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(2, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(1, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(2, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(1, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(1, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(1, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(1, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(1, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(1, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(2, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(1, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(1, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(1, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(41, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(21, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(23, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(459, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(13, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(1, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(2, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(1, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(1, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(1, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(2, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(1, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(1, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(1, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(1, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(582, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(2, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(2, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(34, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(15, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(14, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(13, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(27, 1));

    table
}
</code></pre>



</details>

<a name="0x1_VMConfig_native_schedule"></a>

## Function `native_schedule`



<pre><code><b>public</b> <b>fun</b> <a href="VMConfig.md#0x1_VMConfig_native_schedule">native_schedule</a>(): vector&lt;<a href="VMConfig.md#0x1_VMConfig_GasCost">VMConfig::GasCost</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="VMConfig.md#0x1_VMConfig_native_schedule">native_schedule</a>(): vector&lt;<a href="VMConfig.md#0x1_VMConfig_GasCost">GasCost</a>&gt; {
    <b>let</b> table = <a href="Vector.md#0x1_Vector_empty">Vector::empty</a>();
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(21, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(64, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(61, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(3351, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(181, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(98, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(84, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(1334, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(1902, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(53, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(227, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(572, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(1436, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(26, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(353, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(24, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(212, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(52, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(26, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(2002, 1));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> table, <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(64, 1));
    table
}
</code></pre>



</details>

<a name="0x1_VMConfig_gas_constants"></a>

## Function `gas_constants`



<pre><code><b>public</b> <b>fun</b> <a href="VMConfig.md#0x1_VMConfig_gas_constants">gas_constants</a>(): <a href="VMConfig.md#0x1_VMConfig_GasConstants">VMConfig::GasConstants</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="VMConfig.md#0x1_VMConfig_gas_constants">gas_constants</a>(): <a href="VMConfig.md#0x1_VMConfig_GasConstants">GasConstants</a> {
    <b>let</b> min_price_per_gas_unit: u64 = <b>if</b> (<a href="ChainId.md#0x1_ChainId_is_test">ChainId::is_test</a>()) { 0 }  <b>else</b> { 1 };
    <a href="VMConfig.md#0x1_VMConfig_GasConstants">GasConstants</a> {
        global_memory_per_byte_cost: 4,
        global_memory_per_byte_write_cost: 9,
        min_transaction_gas_units: 600,
        large_transaction_cutoff: 600,
        instrinsic_gas_per_byte: 8,
        maximum_number_of_gas_units: 40000000, //must less than base_block_gas_limit
        min_price_per_gas_unit,
        max_price_per_gas_unit: 10000,
        max_transaction_size_in_bytes: 1024 * 128,
        gas_unit_scaling_factor: 1,
        default_account_size: 800,
    }
}
</code></pre>



</details>

<a name="0x1_VMConfig_new_gas_cost"></a>

## Function `new_gas_cost`



<pre><code><b>fun</b> <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(instr_gas: u64, mem_gas: u64): <a href="VMConfig.md#0x1_VMConfig_GasCost">VMConfig::GasCost</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="VMConfig.md#0x1_VMConfig_new_gas_cost">new_gas_cost</a>(instr_gas: u64, mem_gas: u64): <a href="VMConfig.md#0x1_VMConfig_GasCost">GasCost</a> {
    <a href="VMConfig.md#0x1_VMConfig_GasCost">GasCost</a> {
        instruction_gas: instr_gas,
        memory_gas: mem_gas,
    }
}
</code></pre>



</details>

<a name="0x1_VMConfig_new_vm_config"></a>

## Function `new_vm_config`

Create a new vm config, mainly used in DAO.


<pre><code><b>public</b> <b>fun</b> <a href="VMConfig.md#0x1_VMConfig_new_vm_config">new_vm_config</a>(instruction_schedule: vector&lt;u8&gt;, native_schedule: vector&lt;u8&gt;, global_memory_per_byte_cost: u64, global_memory_per_byte_write_cost: u64, min_transaction_gas_units: u64, large_transaction_cutoff: u64, instrinsic_gas_per_byte: u64, maximum_number_of_gas_units: u64, min_price_per_gas_unit: u64, max_price_per_gas_unit: u64, max_transaction_size_in_bytes: u64, gas_unit_scaling_factor: u64, default_account_size: u64): <a href="VMConfig.md#0x1_VMConfig_VMConfig">VMConfig::VMConfig</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="VMConfig.md#0x1_VMConfig_new_vm_config">new_vm_config</a>(
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
): <a href="VMConfig.md#0x1_VMConfig">VMConfig</a> {
    <b>let</b> gas_constants = <a href="VMConfig.md#0x1_VMConfig_GasConstants">GasConstants</a> {
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
        default_account_size,
    };
    <a href="VMConfig.md#0x1_VMConfig">VMConfig</a> {
        gas_schedule: <a href="VMConfig.md#0x1_VMConfig_GasSchedule">GasSchedule</a> { instruction_schedule, native_schedule, gas_constants },
    }
}
</code></pre>



</details>

<a name="0x1_VMConfig_initialize"></a>

## Function `initialize`

Initialize the table under the genesis account


<pre><code><b>public</b> <b>fun</b> <a href="VMConfig.md#0x1_VMConfig_initialize">initialize</a>(account: &signer, instruction_schedule: vector&lt;u8&gt;, native_schedule: vector&lt;u8&gt;, global_memory_per_byte_cost: u64, global_memory_per_byte_write_cost: u64, min_transaction_gas_units: u64, large_transaction_cutoff: u64, instrinsic_gas_per_byte: u64, maximum_number_of_gas_units: u64, min_price_per_gas_unit: u64, max_price_per_gas_unit: u64, max_transaction_size_in_bytes: u64, gas_unit_scaling_factor: u64, default_account_size: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="VMConfig.md#0x1_VMConfig_initialize">initialize</a>(
    account: &signer,
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
) {
    <a href="CoreAddresses.md#0x1_CoreAddresses_assert_genesis_address">CoreAddresses::assert_genesis_address</a>(account);
    <a href="Config.md#0x1_Config_publish_new_config">Config::publish_new_config</a>&lt;<a href="VMConfig.md#0x1_VMConfig">VMConfig</a>&gt;(
        account,
        <a href="VMConfig.md#0x1_VMConfig_new_vm_config">new_vm_config</a>(
            instruction_schedule,
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
            default_account_size,
        ),
    );
}
</code></pre>



</details>

<a name="@Specification_0"></a>

## Specification



<pre><code><b>pragma</b> verify = <b>false</b>;
<b>pragma</b> aborts_if_is_strict;
</code></pre>



<a name="@Specification_0_initialize"></a>

### Function `initialize`


<pre><code><b>public</b> <b>fun</b> <a href="VMConfig.md#0x1_VMConfig_initialize">initialize</a>(account: &signer, instruction_schedule: vector&lt;u8&gt;, native_schedule: vector&lt;u8&gt;, global_memory_per_byte_cost: u64, global_memory_per_byte_write_cost: u64, min_transaction_gas_units: u64, large_transaction_cutoff: u64, instrinsic_gas_per_byte: u64, maximum_number_of_gas_units: u64, min_price_per_gas_unit: u64, max_price_per_gas_unit: u64, max_transaction_size_in_bytes: u64, gas_unit_scaling_factor: u64, default_account_size: u64)
</code></pre>




<pre><code><b>aborts_if</b> <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account) != <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>();
<b>aborts_if</b> <b>exists</b>&lt;<a href="Config.md#0x1_Config_Config">Config::Config</a>&lt;<a href="VMConfig.md#0x1_VMConfig">VMConfig</a>&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
<b>aborts_if</b>
    <b>exists</b>&lt;<a href="Config.md#0x1_Config_ModifyConfigCapabilityHolder">Config::ModifyConfigCapabilityHolder</a>&lt;<a href="VMConfig.md#0x1_VMConfig">VMConfig</a>&gt;&gt;(
        <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account),
    );
<b>ensures</b> <b>exists</b>&lt;<a href="Config.md#0x1_Config_Config">Config::Config</a>&lt;<a href="VMConfig.md#0x1_VMConfig">VMConfig</a>&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
<b>ensures</b>
    <b>exists</b>&lt;<a href="Config.md#0x1_Config_ModifyConfigCapabilityHolder">Config::ModifyConfigCapabilityHolder</a>&lt;<a href="VMConfig.md#0x1_VMConfig">VMConfig</a>&gt;&gt;(
        <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account),
    );
</code></pre>
