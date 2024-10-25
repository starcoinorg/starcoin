
<a id="0x1_on_chain_config"></a>

# Module `0x1::on_chain_config`

The module provides a general implmentation of configuration for onchain contracts.


-  [Resource `Config`](#0x1_on_chain_config_Config)
-  [Struct `ModifyConfigCapability`](#0x1_on_chain_config_ModifyConfigCapability)
-  [Resource `ModifyConfigCapabilityHolder`](#0x1_on_chain_config_ModifyConfigCapabilityHolder)
-  [Struct `ConfigChangeEvent`](#0x1_on_chain_config_ConfigChangeEvent)
-  [Constants](#@Constants_0)
-  [Function `get_by_address`](#0x1_on_chain_config_get_by_address)
-  [Function `config_exist_by_address`](#0x1_on_chain_config_config_exist_by_address)
-  [Function `set`](#0x1_on_chain_config_set)
-  [Function `set_with_capability`](#0x1_on_chain_config_set_with_capability)
-  [Function `publish_new_config_with_capability`](#0x1_on_chain_config_publish_new_config_with_capability)
-  [Function `publish_new_config`](#0x1_on_chain_config_publish_new_config)
-  [Function `extract_modify_config_capability`](#0x1_on_chain_config_extract_modify_config_capability)
-  [Function `restore_modify_config_capability`](#0x1_on_chain_config_restore_modify_config_capability)
-  [Function `destroy_modify_config_capability`](#0x1_on_chain_config_destroy_modify_config_capability)
-  [Function `account_address`](#0x1_on_chain_config_account_address)
-  [Function `emit_config_change_event`](#0x1_on_chain_config_emit_config_change_event)
-  [Specification](#@Specification_1)
    -  [Function `get_by_address`](#@Specification_1_get_by_address)
    -  [Function `config_exist_by_address`](#@Specification_1_config_exist_by_address)
    -  [Function `set`](#@Specification_1_set)
    -  [Function `set_with_capability`](#@Specification_1_set_with_capability)
    -  [Function `publish_new_config_with_capability`](#@Specification_1_publish_new_config_with_capability)
    -  [Function `publish_new_config`](#@Specification_1_publish_new_config)
    -  [Function `extract_modify_config_capability`](#@Specification_1_extract_modify_config_capability)
    -  [Function `restore_modify_config_capability`](#@Specification_1_restore_modify_config_capability)
    -  [Function `destroy_modify_config_capability`](#@Specification_1_destroy_modify_config_capability)
    -  [Function `account_address`](#@Specification_1_account_address)
    -  [Function `emit_config_change_event`](#@Specification_1_emit_config_change_event)


<pre><code><b>use</b> <a href="account.md#0x1_account">0x1::account</a>;
<b>use</b> <a href="../../move-stdlib/doc/error.md#0x1_error">0x1::error</a>;
<b>use</b> <a href="event.md#0x1_event">0x1::event</a>;
<b>use</b> <a href="../../move-stdlib/doc/option.md#0x1_option">0x1::option</a>;
<b>use</b> <a href="../../move-stdlib/doc/signer.md#0x1_signer">0x1::signer</a>;
</code></pre>



<a id="0x1_on_chain_config_Config"></a>

## Resource `Config`

A generic singleton resource that holds a value of a specific type.


<pre><code><b>struct</b> <a href="on_chain_config.md#0x1_on_chain_config_Config">Config</a>&lt;ConfigValue: <b>copy</b>, drop, store&gt; <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>payload: ConfigValue</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="0x1_on_chain_config_ModifyConfigCapability"></a>

## Struct `ModifyConfigCapability`

Accounts with this privilege can modify config of type ConfigValue under account_address


<pre><code><b>struct</b> <a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapability">ModifyConfigCapability</a>&lt;ConfigValue: <b>copy</b>, drop, store&gt; <b>has</b> store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>account_address: <b>address</b></code>
</dt>
<dd>

</dd>
<dt>
<code>events: <a href="event.md#0x1_event_EventHandle">event::EventHandle</a>&lt;<a href="on_chain_config.md#0x1_on_chain_config_ConfigChangeEvent">on_chain_config::ConfigChangeEvent</a>&lt;ConfigValue&gt;&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="0x1_on_chain_config_ModifyConfigCapabilityHolder"></a>

## Resource `ModifyConfigCapabilityHolder`

A holder for ModifyConfigCapability, for extraction and restoration of ModifyConfigCapability.


<pre><code><b>struct</b> <a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a>&lt;ConfigValue: <b>copy</b>, drop, store&gt; <b>has</b> store, key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>cap: <a href="../../move-stdlib/doc/option.md#0x1_option_Option">option::Option</a>&lt;<a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapability">on_chain_config::ModifyConfigCapability</a>&lt;ConfigValue&gt;&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="0x1_on_chain_config_ConfigChangeEvent"></a>

## Struct `ConfigChangeEvent`

Event emitted when config value is changed.


<pre><code><b>struct</b> <a href="on_chain_config.md#0x1_on_chain_config_ConfigChangeEvent">ConfigChangeEvent</a>&lt;ConfigValue: <b>copy</b>, drop, store&gt; <b>has</b> drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>account_address: <b>address</b></code>
</dt>
<dd>

</dd>
<dt>
<code>value: ConfigValue</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="@Constants_0"></a>

## Constants


<a id="0x1_on_chain_config_ECAPABILITY_HOLDER_NOT_EXISTS"></a>



<pre><code><b>const</b> <a href="on_chain_config.md#0x1_on_chain_config_ECAPABILITY_HOLDER_NOT_EXISTS">ECAPABILITY_HOLDER_NOT_EXISTS</a>: u64 = 101;
</code></pre>



<a id="0x1_on_chain_config_ECONFIG_VALUE_DOES_NOT_EXIST"></a>



<pre><code><b>const</b> <a href="on_chain_config.md#0x1_on_chain_config_ECONFIG_VALUE_DOES_NOT_EXIST">ECONFIG_VALUE_DOES_NOT_EXIST</a>: u64 = 13;
</code></pre>



<a id="0x1_on_chain_config_get_by_address"></a>

## Function `get_by_address`

Get a copy of <code>ConfigValue</code> value stored under <code>addr</code>.


<pre><code><b>public</b> <b>fun</b> <a href="on_chain_config.md#0x1_on_chain_config_get_by_address">get_by_address</a>&lt;ConfigValue: <b>copy</b>, drop, store&gt;(addr: <b>address</b>): ConfigValue
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="on_chain_config.md#0x1_on_chain_config_get_by_address">get_by_address</a>&lt;ConfigValue: <b>copy</b> + drop + store&gt;(addr: <b>address</b>): ConfigValue <b>acquires</b> <a href="on_chain_config.md#0x1_on_chain_config_Config">Config</a> {
    <b>assert</b>!(<b>exists</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_Config">Config</a>&lt;ConfigValue&gt;&gt;(addr), <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_state">error::invalid_state</a>(<a href="on_chain_config.md#0x1_on_chain_config_ECONFIG_VALUE_DOES_NOT_EXIST">ECONFIG_VALUE_DOES_NOT_EXIST</a>));
    *&<b>borrow_global</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_Config">Config</a>&lt;ConfigValue&gt;&gt;(addr).payload
}
</code></pre>



</details>

<a id="0x1_on_chain_config_config_exist_by_address"></a>

## Function `config_exist_by_address`

Check whether the config of <code>ConfigValue</code> type exists under <code>addr</code>.


<pre><code><b>public</b> <b>fun</b> <a href="on_chain_config.md#0x1_on_chain_config_config_exist_by_address">config_exist_by_address</a>&lt;ConfigValue: <b>copy</b>, drop, store&gt;(addr: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="on_chain_config.md#0x1_on_chain_config_config_exist_by_address">config_exist_by_address</a>&lt;ConfigValue: <b>copy</b> + drop + store&gt;(addr: <b>address</b>): bool {
    <b>exists</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_Config">Config</a>&lt;ConfigValue&gt;&gt;(addr)
}
</code></pre>



</details>

<a id="0x1_on_chain_config_set"></a>

## Function `set`

Set a config item to a new value with capability stored under signer


<pre><code><b>public</b> <b>fun</b> <a href="on_chain_config.md#0x1_on_chain_config_set">set</a>&lt;ConfigValue: <b>copy</b>, drop, store&gt;(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, payload: ConfigValue)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="on_chain_config.md#0x1_on_chain_config_set">set</a>&lt;ConfigValue: <b>copy</b> + drop + store&gt;(
    <a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    payload: ConfigValue,
) <b>acquires</b> <a href="on_chain_config.md#0x1_on_chain_config_Config">Config</a>, <a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a> {
    <b>let</b> signer_address = <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>);
    <b>assert</b>!(
        <b>exists</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a>&lt;ConfigValue&gt;&gt;(signer_address),
        <a href="../../move-stdlib/doc/error.md#0x1_error_resource_exhausted">error::resource_exhausted</a>(<a href="on_chain_config.md#0x1_on_chain_config_ECAPABILITY_HOLDER_NOT_EXISTS">ECAPABILITY_HOLDER_NOT_EXISTS</a>),
    );
    <b>let</b> cap_holder = <b>borrow_global_mut</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a>&lt;ConfigValue&gt;&gt;(signer_address);
    <b>assert</b>!(<a href="../../move-stdlib/doc/option.md#0x1_option_is_some">option::is_some</a>(&cap_holder.cap), <a href="../../move-stdlib/doc/error.md#0x1_error_resource_exhausted">error::resource_exhausted</a>(<a href="on_chain_config.md#0x1_on_chain_config_ECAPABILITY_HOLDER_NOT_EXISTS">ECAPABILITY_HOLDER_NOT_EXISTS</a>));
    <a href="on_chain_config.md#0x1_on_chain_config_set_with_capability">set_with_capability</a>(<a href="../../move-stdlib/doc/option.md#0x1_option_borrow_mut">option::borrow_mut</a>(&<b>mut</b> cap_holder.cap), payload);
}
</code></pre>



</details>

<a id="0x1_on_chain_config_set_with_capability"></a>

## Function `set_with_capability`

Set a config item to a new value with cap.


<pre><code><b>public</b> <b>fun</b> <a href="on_chain_config.md#0x1_on_chain_config_set_with_capability">set_with_capability</a>&lt;ConfigValue: <b>copy</b>, drop, store&gt;(cap: &<b>mut</b> <a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapability">on_chain_config::ModifyConfigCapability</a>&lt;ConfigValue&gt;, payload: ConfigValue)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="on_chain_config.md#0x1_on_chain_config_set_with_capability">set_with_capability</a>&lt;ConfigValue: <b>copy</b> + drop + store&gt;(
    cap: &<b>mut</b> <a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapability">ModifyConfigCapability</a>&lt;ConfigValue&gt;,
    payload: ConfigValue,
) <b>acquires</b> <a href="on_chain_config.md#0x1_on_chain_config_Config">Config</a> {
    <b>let</b> addr = cap.account_address;
    <b>assert</b>!(<b>exists</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_Config">Config</a>&lt;ConfigValue&gt;&gt;(addr), <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_state">error::invalid_state</a>(<a href="on_chain_config.md#0x1_on_chain_config_ECONFIG_VALUE_DOES_NOT_EXIST">ECONFIG_VALUE_DOES_NOT_EXIST</a>));
    <b>let</b> config = <b>borrow_global_mut</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_Config">Config</a>&lt;ConfigValue&gt;&gt;(addr);
    config.payload = <b>copy</b> payload;
    <a href="on_chain_config.md#0x1_on_chain_config_emit_config_change_event">emit_config_change_event</a>(cap, payload);
}
</code></pre>



</details>

<a id="0x1_on_chain_config_publish_new_config_with_capability"></a>

## Function `publish_new_config_with_capability`

Publish a new config item. The caller will use the returned ModifyConfigCapability to specify the access control
policy for who can modify the config.


<pre><code><b>public</b> <b>fun</b> <a href="on_chain_config.md#0x1_on_chain_config_publish_new_config_with_capability">publish_new_config_with_capability</a>&lt;ConfigValue: <b>copy</b>, drop, store&gt;(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, payload: ConfigValue): <a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapability">on_chain_config::ModifyConfigCapability</a>&lt;ConfigValue&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="on_chain_config.md#0x1_on_chain_config_publish_new_config_with_capability">publish_new_config_with_capability</a>&lt;ConfigValue: <b>copy</b> + drop + store&gt;(
    <a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    payload: ConfigValue,
): <a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapability">ModifyConfigCapability</a>&lt;ConfigValue&gt; <b>acquires</b> <a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a> {
    <a href="on_chain_config.md#0x1_on_chain_config_publish_new_config">publish_new_config</a>&lt;ConfigValue&gt;(<a href="account.md#0x1_account">account</a>, payload);
    <a href="on_chain_config.md#0x1_on_chain_config_extract_modify_config_capability">extract_modify_config_capability</a>&lt;ConfigValue&gt;(<a href="account.md#0x1_account">account</a>)
}
</code></pre>



</details>

<a id="0x1_on_chain_config_publish_new_config"></a>

## Function `publish_new_config`

Publish a new config item under account address.


<pre><code><b>public</b> <b>fun</b> <a href="on_chain_config.md#0x1_on_chain_config_publish_new_config">publish_new_config</a>&lt;ConfigValue: <b>copy</b>, drop, store&gt;(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, payload: ConfigValue)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="on_chain_config.md#0x1_on_chain_config_publish_new_config">publish_new_config</a>&lt;ConfigValue: <b>copy</b> + drop + store&gt;(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, payload: ConfigValue) {
    <b>move_to</b>(<a href="account.md#0x1_account">account</a>, <a href="on_chain_config.md#0x1_on_chain_config_Config">Config</a>&lt;ConfigValue&gt; { payload });
    <b>let</b> cap = <a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapability">ModifyConfigCapability</a>&lt;ConfigValue&gt; {
        account_address: <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>),
        events: <a href="account.md#0x1_account_new_event_handle">account::new_event_handle</a>&lt;<a href="on_chain_config.md#0x1_on_chain_config_ConfigChangeEvent">ConfigChangeEvent</a>&lt;ConfigValue&gt;&gt;(<a href="account.md#0x1_account">account</a>),
    };
    <b>move_to</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a>&lt;ConfigValue&gt;&gt;(
        <a href="account.md#0x1_account">account</a>,
        <a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a> { cap: <a href="../../move-stdlib/doc/option.md#0x1_option_some">option::some</a>(cap) }
    );
}
</code></pre>



</details>

<a id="0x1_on_chain_config_extract_modify_config_capability"></a>

## Function `extract_modify_config_capability`

Extract account's ModifyConfigCapability for ConfigValue type


<pre><code><b>public</b> <b>fun</b> <a href="on_chain_config.md#0x1_on_chain_config_extract_modify_config_capability">extract_modify_config_capability</a>&lt;ConfigValue: <b>copy</b>, drop, store&gt;(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>): <a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapability">on_chain_config::ModifyConfigCapability</a>&lt;ConfigValue&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="on_chain_config.md#0x1_on_chain_config_extract_modify_config_capability">extract_modify_config_capability</a>&lt;ConfigValue: <b>copy</b> + drop + store&gt;(
    <a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
): <a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapability">ModifyConfigCapability</a>&lt;ConfigValue&gt; <b>acquires</b> <a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a> {
    <b>let</b> signer_address = <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>);
    <b>assert</b>!(
        <b>exists</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a>&lt;ConfigValue&gt;&gt;(signer_address),
        <a href="../../move-stdlib/doc/error.md#0x1_error_permission_denied">error::permission_denied</a>(<a href="on_chain_config.md#0x1_on_chain_config_ECAPABILITY_HOLDER_NOT_EXISTS">ECAPABILITY_HOLDER_NOT_EXISTS</a>)
    );
    <b>let</b> cap_holder = <b>borrow_global_mut</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a>&lt;ConfigValue&gt;&gt;(signer_address);
    <a href="../../move-stdlib/doc/option.md#0x1_option_extract">option::extract</a>(&<b>mut</b> cap_holder.cap)
}
</code></pre>



</details>

<a id="0x1_on_chain_config_restore_modify_config_capability"></a>

## Function `restore_modify_config_capability`

Restore account's ModifyConfigCapability


<pre><code><b>public</b> <b>fun</b> <a href="on_chain_config.md#0x1_on_chain_config_restore_modify_config_capability">restore_modify_config_capability</a>&lt;ConfigValue: <b>copy</b>, drop, store&gt;(cap: <a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapability">on_chain_config::ModifyConfigCapability</a>&lt;ConfigValue&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="on_chain_config.md#0x1_on_chain_config_restore_modify_config_capability">restore_modify_config_capability</a>&lt;ConfigValue: <b>copy</b> + drop + store&gt;(
    cap: <a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapability">ModifyConfigCapability</a>&lt;ConfigValue&gt;,
) <b>acquires</b> <a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a> {
    <b>let</b> cap_holder = <b>borrow_global_mut</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a>&lt;ConfigValue&gt;&gt;(cap.account_address);
    <a href="../../move-stdlib/doc/option.md#0x1_option_fill">option::fill</a>(&<b>mut</b> cap_holder.cap, cap);
}
</code></pre>



</details>

<a id="0x1_on_chain_config_destroy_modify_config_capability"></a>

## Function `destroy_modify_config_capability`

Destroy the given ModifyConfigCapability


<pre><code><b>public</b> <b>fun</b> <a href="on_chain_config.md#0x1_on_chain_config_destroy_modify_config_capability">destroy_modify_config_capability</a>&lt;ConfigValue: <b>copy</b>, drop, store&gt;(cap: <a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapability">on_chain_config::ModifyConfigCapability</a>&lt;ConfigValue&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="on_chain_config.md#0x1_on_chain_config_destroy_modify_config_capability">destroy_modify_config_capability</a>&lt;ConfigValue: <b>copy</b> + drop + store&gt;(
    cap: <a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapability">ModifyConfigCapability</a>&lt;ConfigValue&gt;,
) {
    <b>let</b> <a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapability">ModifyConfigCapability</a> { account_address: _, events } = cap;
    <a href="event.md#0x1_event_destroy_handle">event::destroy_handle</a>(events)
}
</code></pre>



</details>

<a id="0x1_on_chain_config_account_address"></a>

## Function `account_address`

Return the address of the given ModifyConfigCapability


<pre><code><b>public</b> <b>fun</b> <a href="on_chain_config.md#0x1_on_chain_config_account_address">account_address</a>&lt;ConfigValue: <b>copy</b>, drop, store&gt;(cap: &<a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapability">on_chain_config::ModifyConfigCapability</a>&lt;ConfigValue&gt;): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="on_chain_config.md#0x1_on_chain_config_account_address">account_address</a>&lt;ConfigValue: <b>copy</b> + drop + store&gt;(cap: &<a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapability">ModifyConfigCapability</a>&lt;ConfigValue&gt;): <b>address</b> {
    cap.account_address
}
</code></pre>



</details>

<a id="0x1_on_chain_config_emit_config_change_event"></a>

## Function `emit_config_change_event`

Emit a config change event.


<pre><code><b>fun</b> <a href="on_chain_config.md#0x1_on_chain_config_emit_config_change_event">emit_config_change_event</a>&lt;ConfigValue: <b>copy</b>, drop, store&gt;(cap: &<b>mut</b> <a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapability">on_chain_config::ModifyConfigCapability</a>&lt;ConfigValue&gt;, value: ConfigValue)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="on_chain_config.md#0x1_on_chain_config_emit_config_change_event">emit_config_change_event</a>&lt;ConfigValue: <b>copy</b> + drop + store&gt;(
    cap: &<b>mut</b> <a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapability">ModifyConfigCapability</a>&lt;ConfigValue&gt;,
    value: ConfigValue,
) {
    <a href="event.md#0x1_event_emit_event">event::emit_event</a>&lt;<a href="on_chain_config.md#0x1_on_chain_config_ConfigChangeEvent">ConfigChangeEvent</a>&lt;ConfigValue&gt;&gt;(
        &<b>mut</b> cap.events,
        <a href="on_chain_config.md#0x1_on_chain_config_ConfigChangeEvent">ConfigChangeEvent</a> {
            account_address: cap.account_address,
            value,
        },
    );
}
</code></pre>



</details>

<a id="@Specification_1"></a>

## Specification



<pre><code><b>pragma</b> verify;
<b>pragma</b> aborts_if_is_strict;
</code></pre>




<a id="0x1_on_chain_config_spec_get"></a>


<pre><code><b>fun</b> <a href="on_chain_config.md#0x1_on_chain_config_spec_get">spec_get</a>&lt;ConfigValue&gt;(addr: <b>address</b>): ConfigValue {
   <b>global</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_Config">Config</a>&lt;ConfigValue&gt;&gt;(addr).payload
}
</code></pre>



<a id="@Specification_1_get_by_address"></a>

### Function `get_by_address`


<pre><code><b>public</b> <b>fun</b> <a href="on_chain_config.md#0x1_on_chain_config_get_by_address">get_by_address</a>&lt;ConfigValue: <b>copy</b>, drop, store&gt;(addr: <b>address</b>): ConfigValue
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_Config">Config</a>&lt;ConfigValue&gt;&gt;(addr);
<b>ensures</b> <b>exists</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_Config">Config</a>&lt;ConfigValue&gt;&gt;(addr);
<b>ensures</b> result == <a href="on_chain_config.md#0x1_on_chain_config_spec_get">spec_get</a>&lt;ConfigValue&gt;(addr);
</code></pre>



<a id="@Specification_1_config_exist_by_address"></a>

### Function `config_exist_by_address`


<pre><code><b>public</b> <b>fun</b> <a href="on_chain_config.md#0x1_on_chain_config_config_exist_by_address">config_exist_by_address</a>&lt;ConfigValue: <b>copy</b>, drop, store&gt;(addr: <b>address</b>): bool
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
<b>ensures</b> result == <b>exists</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_Config">Config</a>&lt;ConfigValue&gt;&gt;(addr);
</code></pre>



<a id="@Specification_1_set"></a>

### Function `set`


<pre><code><b>public</b> <b>fun</b> <a href="on_chain_config.md#0x1_on_chain_config_set">set</a>&lt;ConfigValue: <b>copy</b>, drop, store&gt;(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, payload: ConfigValue)
</code></pre>




<pre><code><b>let</b> addr = <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>);
<b>let</b> cap_opt = <a href="on_chain_config.md#0x1_on_chain_config_spec_cap">spec_cap</a>&lt;ConfigValue&gt;(addr);
<b>aborts_if</b> !<b>exists</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a>&lt;ConfigValue&gt;&gt;(addr);
<b>aborts_if</b> <a href="../../move-stdlib/doc/option.md#0x1_option_is_none">option::is_none</a>&lt;<a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapability">ModifyConfigCapability</a>&lt;ConfigValue&gt;&gt;(cap_opt);
<b>ensures</b> <b>exists</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a>&lt;ConfigValue&gt;&gt;(addr);
<b>pragma</b> aborts_if_is_partial;
<b>ensures</b> <b>exists</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_Config">Config</a>&lt;ConfigValue&gt;&gt;(
    <a href="../../move-stdlib/doc/option.md#0x1_option_borrow">option::borrow</a>&lt;<a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapability">ModifyConfigCapability</a>&lt;ConfigValue&gt;&gt;(
        <a href="on_chain_config.md#0x1_on_chain_config_spec_cap">spec_cap</a>&lt;ConfigValue&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>))
    ).account_address,
);
<b>ensures</b> <b>global</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_Config">Config</a>&lt;ConfigValue&gt;&gt;(
    <a href="../../move-stdlib/doc/option.md#0x1_option_borrow">option::borrow</a>&lt;<a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapability">ModifyConfigCapability</a>&lt;ConfigValue&gt;&gt;(
        <a href="on_chain_config.md#0x1_on_chain_config_spec_cap">spec_cap</a>&lt;ConfigValue&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>))
    ).account_address,
).payload == payload;
</code></pre>




<a id="0x1_on_chain_config_spec_cap"></a>


<pre><code><b>fun</b> <a href="on_chain_config.md#0x1_on_chain_config_spec_cap">spec_cap</a>&lt;ConfigValue&gt;(addr: <b>address</b>): <a href="../../move-stdlib/doc/option.md#0x1_option_Option">option::Option</a>&lt;<a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapability">ModifyConfigCapability</a>&lt;ConfigValue&gt;&gt; {
   <b>global</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a>&lt;ConfigValue&gt;&gt;(addr).cap
}
</code></pre>



<a id="@Specification_1_set_with_capability"></a>

### Function `set_with_capability`


<pre><code><b>public</b> <b>fun</b> <a href="on_chain_config.md#0x1_on_chain_config_set_with_capability">set_with_capability</a>&lt;ConfigValue: <b>copy</b>, drop, store&gt;(cap: &<b>mut</b> <a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapability">on_chain_config::ModifyConfigCapability</a>&lt;ConfigValue&gt;, payload: ConfigValue)
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_Config">Config</a>&lt;ConfigValue&gt;&gt;(cap.account_address);
<b>ensures</b> <b>exists</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_Config">Config</a>&lt;ConfigValue&gt;&gt;(cap.account_address);
<b>ensures</b> <b>global</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_Config">Config</a>&lt;ConfigValue&gt;&gt;(cap.account_address).payload == payload;
</code></pre>



<a id="@Specification_1_publish_new_config_with_capability"></a>

### Function `publish_new_config_with_capability`


<pre><code><b>public</b> <b>fun</b> <a href="on_chain_config.md#0x1_on_chain_config_publish_new_config_with_capability">publish_new_config_with_capability</a>&lt;ConfigValue: <b>copy</b>, drop, store&gt;(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, payload: ConfigValue): <a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapability">on_chain_config::ModifyConfigCapability</a>&lt;ConfigValue&gt;
</code></pre>




<pre><code><b>include</b> <a href="on_chain_config.md#0x1_on_chain_config_PublishNewConfigAbortsIf">PublishNewConfigAbortsIf</a>&lt;ConfigValue&gt;;
<b>ensures</b> <b>exists</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_Config">Config</a>&lt;ConfigValue&gt;&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>));
<b>ensures</b> <b>global</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_Config">Config</a>&lt;ConfigValue&gt;&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>)).payload == payload;
<b>ensures</b> <b>exists</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a>&lt;ConfigValue&gt;&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>));
<b>ensures</b> <a href="../../move-stdlib/doc/option.md#0x1_option_is_none">option::is_none</a>(<b>global</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a>&lt;ConfigValue&gt;&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>)).cap);
</code></pre>



<a id="@Specification_1_publish_new_config"></a>

### Function `publish_new_config`


<pre><code><b>public</b> <b>fun</b> <a href="on_chain_config.md#0x1_on_chain_config_publish_new_config">publish_new_config</a>&lt;ConfigValue: <b>copy</b>, drop, store&gt;(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, payload: ConfigValue)
</code></pre>




<pre><code><b>include</b> <a href="on_chain_config.md#0x1_on_chain_config_PublishNewConfigAbortsIf">PublishNewConfigAbortsIf</a>&lt;ConfigValue&gt;;
<b>ensures</b> <b>exists</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_Config">Config</a>&lt;ConfigValue&gt;&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>));
<b>ensures</b> <b>global</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_Config">Config</a>&lt;ConfigValue&gt;&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>)).payload == payload;
<b>ensures</b> <b>exists</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a>&lt;ConfigValue&gt;&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>));
<b>ensures</b> <a href="../../move-stdlib/doc/option.md#0x1_option_is_some">option::is_some</a>(<b>global</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a>&lt;ConfigValue&gt;&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>)).cap);
</code></pre>




<a id="0x1_on_chain_config_PublishNewConfigAbortsIf"></a>


<pre><code><b>schema</b> <a href="on_chain_config.md#0x1_on_chain_config_PublishNewConfigAbortsIf">PublishNewConfigAbortsIf</a>&lt;ConfigValue&gt; {
    <a href="account.md#0x1_account">account</a>: <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>;
    <b>aborts_if</b> <b>exists</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_Config">Config</a>&lt;ConfigValue&gt;&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>));
    <b>aborts_if</b> <b>exists</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a>&lt;ConfigValue&gt;&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>));
}
</code></pre>




<a id="0x1_on_chain_config_AbortsIfConfigNotExist"></a>


<pre><code><b>schema</b> <a href="on_chain_config.md#0x1_on_chain_config_AbortsIfConfigNotExist">AbortsIfConfigNotExist</a>&lt;ConfigValue&gt; {
    addr: <b>address</b>;
    <b>aborts_if</b> !<b>exists</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_Config">Config</a>&lt;ConfigValue&gt;&gt;(addr);
}
</code></pre>




<a id="0x1_on_chain_config_AbortsIfConfigOrCapabilityNotExist"></a>


<pre><code><b>schema</b> <a href="on_chain_config.md#0x1_on_chain_config_AbortsIfConfigOrCapabilityNotExist">AbortsIfConfigOrCapabilityNotExist</a>&lt;ConfigValue&gt; {
    addr: <b>address</b>;
    <b>aborts_if</b> !<b>exists</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_Config">Config</a>&lt;ConfigValue&gt;&gt;(addr);
    <b>aborts_if</b> !<b>exists</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a>&lt;ConfigValue&gt;&gt;(addr);
}
</code></pre>




<a id="0x1_on_chain_config_PublishNewConfigEnsures"></a>


<pre><code><b>schema</b> <a href="on_chain_config.md#0x1_on_chain_config_PublishNewConfigEnsures">PublishNewConfigEnsures</a>&lt;ConfigValue&gt; {
    <a href="account.md#0x1_account">account</a>: <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>;
    <b>ensures</b> <b>exists</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_Config">Config</a>&lt;ConfigValue&gt;&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>));
    <b>ensures</b> <b>exists</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a>&lt;ConfigValue&gt;&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>));
}
</code></pre>




<a id="0x1_on_chain_config_AbortsIfCapNotExist"></a>


<pre><code><b>schema</b> <a href="on_chain_config.md#0x1_on_chain_config_AbortsIfCapNotExist">AbortsIfCapNotExist</a>&lt;ConfigValue&gt; {
    <b>address</b>: <b>address</b>;
    <b>aborts_if</b> !<b>exists</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a>&lt;ConfigValue&gt;&gt;(<b>address</b>);
    <b>aborts_if</b> <a href="../../move-stdlib/doc/option.md#0x1_option_is_none">option::is_none</a>&lt;<a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapability">ModifyConfigCapability</a>&lt;ConfigValue&gt;&gt;(
        <b>global</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a>&lt;ConfigValue&gt;&gt;(<b>address</b>).cap,
    );
}
</code></pre>



<a id="@Specification_1_extract_modify_config_capability"></a>

### Function `extract_modify_config_capability`


<pre><code><b>public</b> <b>fun</b> <a href="on_chain_config.md#0x1_on_chain_config_extract_modify_config_capability">extract_modify_config_capability</a>&lt;ConfigValue: <b>copy</b>, drop, store&gt;(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>): <a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapability">on_chain_config::ModifyConfigCapability</a>&lt;ConfigValue&gt;
</code></pre>




<pre><code><b>let</b> <b>address</b> = <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>);
<b>include</b> <a href="on_chain_config.md#0x1_on_chain_config_AbortsIfCapNotExist">AbortsIfCapNotExist</a>&lt;ConfigValue&gt;;
<b>ensures</b> <b>exists</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a>&lt;ConfigValue&gt;&gt;(<b>address</b>);
<b>ensures</b> <a href="../../move-stdlib/doc/option.md#0x1_option_is_none">option::is_none</a>&lt;<a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapability">ModifyConfigCapability</a>&lt;ConfigValue&gt;&gt;(
    <b>global</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a>&lt;ConfigValue&gt;&gt;(<b>address</b>).cap
);
<b>ensures</b> result == <b>old</b>(<a href="../../move-stdlib/doc/option.md#0x1_option_borrow">option::borrow</a>(<b>global</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a>&lt;ConfigValue&gt;&gt;(<b>address</b>).cap));
</code></pre>



<a id="@Specification_1_restore_modify_config_capability"></a>

### Function `restore_modify_config_capability`


<pre><code><b>public</b> <b>fun</b> <a href="on_chain_config.md#0x1_on_chain_config_restore_modify_config_capability">restore_modify_config_capability</a>&lt;ConfigValue: <b>copy</b>, drop, store&gt;(cap: <a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapability">on_chain_config::ModifyConfigCapability</a>&lt;ConfigValue&gt;)
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a>&lt;ConfigValue&gt;&gt;(cap.account_address);
<b>aborts_if</b> <a href="../../move-stdlib/doc/option.md#0x1_option_is_some">option::is_some</a>(<b>global</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a>&lt;ConfigValue&gt;&gt;(cap.account_address).cap);
<b>ensures</b> <b>exists</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a>&lt;ConfigValue&gt;&gt;(cap.account_address);
<b>ensures</b> <a href="../../move-stdlib/doc/option.md#0x1_option_is_some">option::is_some</a>(<b>global</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a>&lt;ConfigValue&gt;&gt;(cap.account_address).cap);
<b>ensures</b> <a href="../../move-stdlib/doc/option.md#0x1_option_borrow">option::borrow</a>(<b>global</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a>&lt;ConfigValue&gt;&gt;(cap.account_address).cap) == cap;
</code></pre>



<a id="@Specification_1_destroy_modify_config_capability"></a>

### Function `destroy_modify_config_capability`


<pre><code><b>public</b> <b>fun</b> <a href="on_chain_config.md#0x1_on_chain_config_destroy_modify_config_capability">destroy_modify_config_capability</a>&lt;ConfigValue: <b>copy</b>, drop, store&gt;(cap: <a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapability">on_chain_config::ModifyConfigCapability</a>&lt;ConfigValue&gt;)
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a id="@Specification_1_account_address"></a>

### Function `account_address`


<pre><code><b>public</b> <b>fun</b> <a href="on_chain_config.md#0x1_on_chain_config_account_address">account_address</a>&lt;ConfigValue: <b>copy</b>, drop, store&gt;(cap: &<a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapability">on_chain_config::ModifyConfigCapability</a>&lt;ConfigValue&gt;): <b>address</b>
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
<b>ensures</b> result == cap.account_address;
</code></pre>



<a id="@Specification_1_emit_config_change_event"></a>

### Function `emit_config_change_event`


<pre><code><b>fun</b> <a href="on_chain_config.md#0x1_on_chain_config_emit_config_change_event">emit_config_change_event</a>&lt;ConfigValue: <b>copy</b>, drop, store&gt;(cap: &<b>mut</b> <a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapability">on_chain_config::ModifyConfigCapability</a>&lt;ConfigValue&gt;, value: ConfigValue)
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>


[move-book]: https://starcoin.dev/move/book/SUMMARY
