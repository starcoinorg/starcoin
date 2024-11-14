
<a id="0x1_vm_config"></a>

# Module `0x1::vm_config`

<code><a href="vm_config.md#0x1_vm_config_VMConfig">VMConfig</a></code> keep track of VM related configuration, like gas schedule.


-  [Struct `VMConfig`](#0x1_vm_config_VMConfig)
-  [Function `initialize`](#0x1_vm_config_initialize)
-  [Specification](#@Specification_0)
    -  [Function `initialize`](#@Specification_0_initialize)


<pre><code><b>use</b> <a href="gas_schedule.md#0x1_gas_schedule">0x1::gas_schedule</a>;
<b>use</b> <a href="on_chain_config.md#0x1_on_chain_config">0x1::on_chain_config</a>;
<b>use</b> <a href="system_addresses.md#0x1_system_addresses">0x1::system_addresses</a>;
<b>use</b> <a href="util.md#0x1_util">0x1::util</a>;
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
<code><a href="gas_schedule.md#0x1_gas_schedule">gas_schedule</a>: <a href="gas_schedule.md#0x1_gas_schedule_GasScheduleV2">gas_schedule::GasScheduleV2</a></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="0x1_vm_config_initialize"></a>

## Function `initialize`

Initialize the table under the genesis account


<pre><code><b>public</b> <b>fun</b> <a href="vm_config.md#0x1_vm_config_initialize">initialize</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, gas_schedule_blob: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="vm_config.md#0x1_vm_config_initialize">initialize</a>(
    <a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    gas_schedule_blob: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
) {
    // CoreAddresses::assert_genesis_address(<a href="account.md#0x1_account">account</a>);
    <a href="system_addresses.md#0x1_system_addresses_assert_starcoin_framework">system_addresses::assert_starcoin_framework</a>(<a href="account.md#0x1_account">account</a>);
    <b>let</b> <a href="gas_schedule.md#0x1_gas_schedule">gas_schedule</a>: GasScheduleV2 = <a href="util.md#0x1_util_from_bytes">util::from_bytes</a>(gas_schedule_blob);
    <a href="on_chain_config.md#0x1_on_chain_config_publish_new_config">on_chain_config::publish_new_config</a>&lt;<a href="vm_config.md#0x1_vm_config_VMConfig">VMConfig</a>&gt;(
        <a href="account.md#0x1_account">account</a>,
        <a href="vm_config.md#0x1_vm_config_VMConfig">VMConfig</a> {
            <a href="gas_schedule.md#0x1_gas_schedule">gas_schedule</a>,
        },
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


<pre><code><b>public</b> <b>fun</b> <a href="vm_config.md#0x1_vm_config_initialize">initialize</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, gas_schedule_blob: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;)
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
