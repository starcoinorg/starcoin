
<a id="0x1_vm_config"></a>

# Module `0x1::vm_config`

<code><a href="vm_config.md#0x1_vm_config_VMConfig">VMConfig</a></code> keep track of VM related configuration, like gas schedule.


-  [Struct `VMConfig`](#0x1_vm_config_VMConfig)
-  [Struct `GasSchedule`](#0x1_vm_config_GasSchedule)
-  [Struct `GasConstants`](#0x1_vm_config_GasConstants)
-  [Struct `GasCost`](#0x1_vm_config_GasCost)
-  [Function `instruction_schedule`](#0x1_vm_config_instruction_schedule)
-  [Function `native_schedule`](#0x1_vm_config_native_schedule)
-  [Function `gas_constants`](#0x1_vm_config_gas_constants)
-  [Function `new_gas_cost`](#0x1_vm_config_new_gas_cost)
-  [Function `new_vm_config`](#0x1_vm_config_new_vm_config)
-  [Function `initialize`](#0x1_vm_config_initialize)
-  [Specification](#@Specification_0)
    -  [Function `initialize`](#@Specification_0_initialize)


<pre><code><b>use</b> <a href="on_chain_config.md#0x1_on_chain_config">0x1::on_chain_config</a>;
<b>use</b> <a href="stc_util.md#0x1_stc_util">0x1::stc_util</a>;
<b>use</b> <a href="system_addresses.md#0x1_system_addresses">0x1::system_addresses</a>;
</code></pre>



<a id="0x1_vm_config_VMConfig"></a>

## Struct `VMConfig`

The struct to hold all config data needed to operate the VM.
* gas_schedule: Cost of running the VM.


<pre><code><b>struct</b> <a href="vm_config.md#0x1_vm_config_VMConfig">VMConfig</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code><a href="gas_schedule.md#0x1_gas_schedule">gas_schedule</a>: <a href="vm_config.md#0x1_vm_config_GasSchedule">vm_config::GasSchedule</a></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="0x1_vm_config_GasSchedule"></a>

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


<pre><code><b>struct</b> <a href="vm_config.md#0x1_vm_config_GasSchedule">GasSchedule</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>instruction_schedule: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>native_schedule: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>gas_constants: <a href="vm_config.md#0x1_vm_config_GasConstants">vm_config::GasConstants</a></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="0x1_vm_config_GasConstants"></a>

## Struct `GasConstants`

The gas constants contains all kind of constants used in gas calculation.


<pre><code><b>struct</b> <a href="vm_config.md#0x1_vm_config_GasConstants">GasConstants</a> <b>has</b> <b>copy</b>, drop, store
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

<a id="0x1_vm_config_GasCost"></a>

## Struct `GasCost`

The  <code><a href="vm_config.md#0x1_vm_config_GasCost">GasCost</a></code> tracks:
- instruction cost: how much time/computational power is needed to perform the instruction
- memory cost: how much memory is required for the instruction, and storage overhead


<pre><code><b>struct</b> <a href="vm_config.md#0x1_vm_config_GasCost">GasCost</a> <b>has</b> <b>copy</b>, drop, store
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

<a id="0x1_vm_config_instruction_schedule"></a>

## Function `instruction_schedule`



<pre><code><b>public</b> <b>fun</b> <a href="vm_config.md#0x1_vm_config_instruction_schedule">instruction_schedule</a>(): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="vm_config.md#0x1_vm_config_GasCost">vm_config::GasCost</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="vm_config.md#0x1_vm_config_instruction_schedule">instruction_schedule</a>(): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="vm_config.md#0x1_vm_config_GasCost">GasCost</a>&gt; {
    <b>let</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a> = <a href="../../move-stdlib/doc/vector.md#0x1_vector_empty">vector::empty</a>();

    // POP
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(1, 1));
    // RET
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(638, 1));
    // BR_TRUE
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(1, 1));
    // BR_FALSE
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(1, 1));
    // BRANCH
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(1, 1));
    // LD_U64
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(1, 1));
    // LD_CONST
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(1, 1));
    // LD_TRUE
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(1, 1));
    // LD_FALSE
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(1, 1));
    // COPY_LOC
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(1, 1));
    // MOVE_LOC
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(1, 1));
    // ST_LOC
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(1, 1));
    // MUT_BORROW_LOC
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(2, 1));
    // IMM_BORROW_LOC
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(1, 1));
    // MUT_BORROW_FIELD
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(1, 1));
    // IMM_BORROW_FIELD
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(1, 1));
    // CALL
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(1132, 1));
    // PACK
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(2, 1));
    // UNPACK
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(2, 1));
    // READ_REF
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(1, 1));
    // WRITE_REF
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(1, 1));
    // ADD
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(1, 1));
    // SUB
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(1, 1));
    // MUL
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(1, 1));
    // MOD
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(1, 1));
    // DIV
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(3, 1));
    // BIT_OR
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(2, 1));
    // BIT_AND
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(2, 1));
    // XOR
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(1, 1));
    // OR
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(2, 1));
    // AND
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(1, 1));
    // NOT
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(1, 1));
    // EQ
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(1, 1));
    // NEQ
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(1, 1));
    // LT
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(1, 1));
    // GT
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(1, 1));
    // LE
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(2, 1));
    // GE
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(1, 1));
    // ABORT
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(1, 1));
    // NOP
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(1, 1));
    // EXISTS
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(41, 1));
    // MUT_BORROW_GLOBAL
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(21, 1));
    // IML_BORROW_GLOBAL
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(23, 1));
    // MOVE_FROM
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(459, 1));
    // MOVE_TO
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(13, 1));
    // FREEZE_REF
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(1, 1));
    // SHL
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(2, 1));
    // SHR
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(1, 1));
    // LD_U8
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(1, 1));
    // LD_U128
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(1, 1));

    // CAST_U8
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(2, 1));
    // CAST_U64
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(1, 1));
    // CAST_U128
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(1, 1));
    // MUT_BORORW_FIELD_GENERIC
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(1, 1));
    // IMM_BORORW_FIELD_GENERIC
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(1, 1));
    // CALL_GENERIC
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(582, 1));
    // PACK_GENERIC
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(2, 1));
    // UNPACK_GENERIC
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(2, 1));
    // EXISTS_GENERIC
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(34, 1));
    // MUT_BORROW_GLOBAL_GENERIC
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(15, 1));
    // IMM_BORROW_GLOBAL_GENERIC
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(14, 1));
    // MOVE_FROM_GENERIC
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(13, 1));
    // MOVE_TO_GENERIC
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(27, 1));

    // VEC_PACK
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(84, 1));
    // VEC_LEN
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(98, 1));
    // VEC_IMM_BORROW
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(1334, 1));
    // VEC_MUT_BORROW
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(1902, 1));
    // VEC_PUSH_BACK
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(53, 1));
    // VEC_POP_BACK
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(227, 1));
    // VEC_UNPACK
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(572, 1));
    // VEC_SWAP
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(1436, 1));
    <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>
}
</code></pre>



</details>

<a id="0x1_vm_config_native_schedule"></a>

## Function `native_schedule`



<pre><code><b>public</b> <b>fun</b> <a href="vm_config.md#0x1_vm_config_native_schedule">native_schedule</a>(): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="vm_config.md#0x1_vm_config_GasCost">vm_config::GasCost</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="vm_config.md#0x1_vm_config_native_schedule">native_schedule</a>(): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="vm_config.md#0x1_vm_config_GasCost">GasCost</a>&gt; {
    <b>let</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a> = <a href="../../move-stdlib/doc/vector.md#0x1_vector_empty">vector::empty</a>();
    //Hash::sha2_256 0
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(21, 1));
    //Hash::sha3_256 1
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(64, 1));
    //Signature::ed25519_verify 2
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(61, 1));
    //ED25519_THRESHOLD_VERIFY 3 this <b>native</b> funciton is deprecated
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(3351, 1));
    //BSC::to_bytes 4
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(181, 1));
    //<a href="../../move-stdlib/doc/vector.md#0x1_vector_length">vector::length</a> 5
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(98, 1));
    //<a href="../../move-stdlib/doc/vector.md#0x1_vector_empty">vector::empty</a> 6
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(84, 1));
    //<a href="../../move-stdlib/doc/vector.md#0x1_vector_borrow">vector::borrow</a> 7
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(1334, 1));
    //<a href="../../move-stdlib/doc/vector.md#0x1_vector_borrow_mut">vector::borrow_mut</a> 8
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(1902, 1));
    //<a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a> 9
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(53, 1));
    //<a href="../../move-stdlib/doc/vector.md#0x1_vector_pop_back">vector::pop_back</a> 10
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(227, 1));
    //vector::destory_empty 11
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(572, 1));
    //<a href="../../move-stdlib/doc/vector.md#0x1_vector_swap">vector::swap</a> 12
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(1436, 1));
    //Signature::ed25519_validate_pubkey 13
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(26, 1));
    //Signer::borrow_address 14
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(353, 1));
    //Account::creator_signer 15
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(24, 1));
    //Account::destroy_signer 16
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(212, 1));
    //Event::emit_event 17
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(52, 1));
    //BCS::to_address 18
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(26, 1));
    //Token::name_of 19
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(2002, 1));
    //Hash::keccak_256 20
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(64, 1));
    //Hash::ripemd160 21
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(64, 1));
    //Signature::native_ecrecover 22
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(128, 1));
    //U256::from_bytes 23
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(2, 1));
    //U256::add 24
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(4, 1));
    //U256::sub 25
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(4, 1));
    //U256::mul 26
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(4, 1));
    //U256::div 27
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(10, 1));
    // U256::rem 28
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(4, 1));
    // U256::pow 29
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(8, 1));
    // TODO: settle down the gas cost
    // <a href="../../move-stdlib/doc/vector.md#0x1_vector_append">vector::append</a> 30
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(40, 1));
    // <a href="../../move-stdlib/doc/vector.md#0x1_vector_remove">vector::remove</a> 31
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(20, 1));
    // <a href="../../move-stdlib/doc/vector.md#0x1_vector_reverse">vector::reverse</a> 32
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(10, 1));

    // Table::new_table_handle 33
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(4, 1));
    // Table::add_box 34
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(4, 1));
    // Table::borrow_box 35
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(10, 1));
    // Table::remove_box 36
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(8, 1));
    // Table::contains_box 37
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(40, 1));
    // Table::destroy_empty_box 38
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(20, 1));
    // Table::drop_unchecked_box 39
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(73, 1));
    // <a href="../../move-stdlib/doc/string.md#0x1_string">string</a>.check_utf8 40
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(4, 1));
    // <a href="../../move-stdlib/doc/string.md#0x1_string">string</a>.sub_str 41
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(4, 1));
    // <a href="../../move-stdlib/doc/string.md#0x1_string">string</a>.is_char_boundary 42
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(4, 1));
    // Table::string.index_of 43
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(4, 1));
    // FromBCS::from_bytes 44
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(4, 1));
    // Secp256k1::ecdsa_recover_internal 45
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(4, 1));
    // vector::spawn_from 46
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>, <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(4, 1));

    <a href="../../starcoin-stdlib/doc/table.md#0x1_table">table</a>
}
</code></pre>



</details>

<a id="0x1_vm_config_gas_constants"></a>

## Function `gas_constants`



<pre><code><b>public</b> <b>fun</b> <a href="vm_config.md#0x1_vm_config_gas_constants">gas_constants</a>(): <a href="vm_config.md#0x1_vm_config_GasConstants">vm_config::GasConstants</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="vm_config.md#0x1_vm_config_gas_constants">gas_constants</a>(): <a href="vm_config.md#0x1_vm_config_GasConstants">GasConstants</a> {
    <b>let</b> min_price_per_gas_unit: u64 = <b>if</b> (<a href="stc_util.md#0x1_stc_util_is_net_test">stc_util::is_net_test</a>()) { 0 }  <b>else</b> { 1 };
    <b>let</b> maximum_number_of_gas_units: u64 = 40000000;//must less than base_block_gas_limit

    <b>if</b> (<a href="stc_util.md#0x1_stc_util_is_net_test">stc_util::is_net_test</a>() || <a href="stc_util.md#0x1_stc_util_is_net_dev">stc_util::is_net_dev</a>() || <a href="stc_util.md#0x1_stc_util_is_net_halley">stc_util::is_net_halley</a>()) {
        maximum_number_of_gas_units = maximum_number_of_gas_units * 10
    };
    <a href="vm_config.md#0x1_vm_config_GasConstants">GasConstants</a> {
        global_memory_per_byte_cost: 4,
        global_memory_per_byte_write_cost: 9,
        min_transaction_gas_units: 600,
        large_transaction_cutoff: 600,
        instrinsic_gas_per_byte: 8,
        maximum_number_of_gas_units,
        min_price_per_gas_unit,
        max_price_per_gas_unit: 10000,
        max_transaction_size_in_bytes: 1024 * 128,
        gas_unit_scaling_factor: 1,
        default_account_size: 800,
    }
}
</code></pre>



</details>

<a id="0x1_vm_config_new_gas_cost"></a>

## Function `new_gas_cost`



<pre><code><b>fun</b> <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(instr_gas: u64, mem_gas: u64): <a href="vm_config.md#0x1_vm_config_GasCost">vm_config::GasCost</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="vm_config.md#0x1_vm_config_new_gas_cost">new_gas_cost</a>(instr_gas: u64, mem_gas: u64): <a href="vm_config.md#0x1_vm_config_GasCost">GasCost</a> {
    <a href="vm_config.md#0x1_vm_config_GasCost">GasCost</a> {
        instruction_gas: instr_gas,
        memory_gas: mem_gas,
    }
}
</code></pre>



</details>

<a id="0x1_vm_config_new_vm_config"></a>

## Function `new_vm_config`

Create a new vm config, mainly used in DAO.


<pre><code><b>public</b> <b>fun</b> <a href="vm_config.md#0x1_vm_config_new_vm_config">new_vm_config</a>(instruction_schedule: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, native_schedule: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, global_memory_per_byte_cost: u64, global_memory_per_byte_write_cost: u64, min_transaction_gas_units: u64, large_transaction_cutoff: u64, instrinsic_gas_per_byte: u64, maximum_number_of_gas_units: u64, min_price_per_gas_unit: u64, max_price_per_gas_unit: u64, max_transaction_size_in_bytes: u64, gas_unit_scaling_factor: u64, default_account_size: u64): <a href="vm_config.md#0x1_vm_config_VMConfig">vm_config::VMConfig</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="vm_config.md#0x1_vm_config_new_vm_config">new_vm_config</a>(
    instruction_schedule: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    native_schedule: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
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
): <a href="vm_config.md#0x1_vm_config_VMConfig">VMConfig</a> {
    <b>let</b> gas_constants = <a href="vm_config.md#0x1_vm_config_GasConstants">GasConstants</a> {
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
    <a href="vm_config.md#0x1_vm_config_VMConfig">VMConfig</a> {
        <a href="gas_schedule.md#0x1_gas_schedule">gas_schedule</a>: <a href="vm_config.md#0x1_vm_config_GasSchedule">GasSchedule</a> { instruction_schedule, native_schedule, gas_constants },
    }
}
</code></pre>



</details>

<a id="0x1_vm_config_initialize"></a>

## Function `initialize`

Initialize the table under the genesis account


<pre><code><b>public</b> <b>fun</b> <a href="vm_config.md#0x1_vm_config_initialize">initialize</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, instruction_schedule: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, native_schedule: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, global_memory_per_byte_cost: u64, global_memory_per_byte_write_cost: u64, min_transaction_gas_units: u64, large_transaction_cutoff: u64, instrinsic_gas_per_byte: u64, maximum_number_of_gas_units: u64, min_price_per_gas_unit: u64, max_price_per_gas_unit: u64, max_transaction_size_in_bytes: u64, gas_unit_scaling_factor: u64, default_account_size: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="vm_config.md#0x1_vm_config_initialize">initialize</a>(
    <a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    instruction_schedule: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    native_schedule: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
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
    // CoreAddresses::assert_genesis_address(<a href="account.md#0x1_account">account</a>);
    <a href="system_addresses.md#0x1_system_addresses_assert_starcoin_framework">system_addresses::assert_starcoin_framework</a>(<a href="account.md#0x1_account">account</a>);
    <a href="on_chain_config.md#0x1_on_chain_config_publish_new_config">on_chain_config::publish_new_config</a>&lt;<a href="vm_config.md#0x1_vm_config_VMConfig">VMConfig</a>&gt;(
        <a href="account.md#0x1_account">account</a>,
        <a href="vm_config.md#0x1_vm_config_new_vm_config">new_vm_config</a>(
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

<a id="@Specification_0"></a>

## Specification



<pre><code><b>pragma</b> verify = <b>false</b>;
<b>pragma</b> aborts_if_is_strict;
</code></pre>



<a id="@Specification_0_initialize"></a>

### Function `initialize`


<pre><code><b>public</b> <b>fun</b> <a href="vm_config.md#0x1_vm_config_initialize">initialize</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, instruction_schedule: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, native_schedule: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, global_memory_per_byte_cost: u64, global_memory_per_byte_write_cost: u64, min_transaction_gas_units: u64, large_transaction_cutoff: u64, instrinsic_gas_per_byte: u64, maximum_number_of_gas_units: u64, min_price_per_gas_unit: u64, max_price_per_gas_unit: u64, max_transaction_size_in_bytes: u64, gas_unit_scaling_factor: u64, default_account_size: u64)
</code></pre>




<pre><code><b>aborts_if</b> <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>) != <a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>();
<b>aborts_if</b> <b>exists</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_Config">on_chain_config::Config</a>&lt;<a href="vm_config.md#0x1_vm_config_VMConfig">VMConfig</a>&gt;&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>));
<b>aborts_if</b>
    <b>exists</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapabilityHolder">on_chain_config::ModifyConfigCapabilityHolder</a>&lt;<a href="vm_config.md#0x1_vm_config_VMConfig">VMConfig</a>&gt;&gt;(
        <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>),
    );
<b>ensures</b> <b>exists</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_Config">on_chain_config::Config</a>&lt;<a href="vm_config.md#0x1_vm_config_VMConfig">VMConfig</a>&gt;&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>));
<b>ensures</b>
    <b>exists</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapabilityHolder">on_chain_config::ModifyConfigCapabilityHolder</a>&lt;<a href="vm_config.md#0x1_vm_config_VMConfig">VMConfig</a>&gt;&gt;(
        <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>),
    );
</code></pre>


[move-book]: https://starcoin.dev/move/book/SUMMARY
