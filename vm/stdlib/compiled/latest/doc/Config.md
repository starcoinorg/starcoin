
<a name="0x1_Config"></a>

# Module `0x1::Config`

The module provides a general implmentation of configuration for onchain contracts.


-  [Resource `Config`](#0x1_Config_Config)
-  [Struct `ModifyConfigCapability`](#0x1_Config_ModifyConfigCapability)
-  [Resource `ModifyConfigCapabilityHolder`](#0x1_Config_ModifyConfigCapabilityHolder)
-  [Struct `ConfigChangeEvent`](#0x1_Config_ConfigChangeEvent)
-  [Constants](#@Constants_0)
-  [Function `get_by_address`](#0x1_Config_get_by_address)
-  [Function `config_exist_by_address`](#0x1_Config_config_exist_by_address)
-  [Function `set`](#0x1_Config_set)
-  [Function `set_with_capability`](#0x1_Config_set_with_capability)
-  [Function `publish_new_config_with_capability`](#0x1_Config_publish_new_config_with_capability)
-  [Function `publish_new_config`](#0x1_Config_publish_new_config)
-  [Function `extract_modify_config_capability`](#0x1_Config_extract_modify_config_capability)
-  [Function `restore_modify_config_capability`](#0x1_Config_restore_modify_config_capability)
-  [Function `destroy_modify_config_capability`](#0x1_Config_destroy_modify_config_capability)
-  [Function `account_address`](#0x1_Config_account_address)
-  [Function `emit_config_change_event`](#0x1_Config_emit_config_change_event)
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


<pre><code><b>use</b> <a href="Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="Event.md#0x1_Event">0x1::Event</a>;
<b>use</b> <a href="Option.md#0x1_Option">0x1::Option</a>;
<b>use</b> <a href="Signer.md#0x1_Signer">0x1::Signer</a>;
</code></pre>



<a name="0x1_Config_Config"></a>

## Resource `Config`

A generic singleton resource that holds a value of a specific type.


<pre><code><b>struct</b> <a href="Config.md#0x1_Config">Config</a>&lt;ConfigValue: <b>copy</b>, drop, store&gt; <b>has</b> key
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

<a name="0x1_Config_ModifyConfigCapability"></a>

## Struct `ModifyConfigCapability`

Accounts with this privilege can modify config of type ConfigValue under account_address


<pre><code><b>struct</b> <a href="Config.md#0x1_Config_ModifyConfigCapability">ModifyConfigCapability</a>&lt;ConfigValue: <b>copy</b>, drop, store&gt; <b>has</b> store
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
<code>events: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="Config.md#0x1_Config_ConfigChangeEvent">Config::ConfigChangeEvent</a>&lt;ConfigValue&gt;&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_Config_ModifyConfigCapabilityHolder"></a>

## Resource `ModifyConfigCapabilityHolder`

A holder for ModifyConfigCapability, for extract and restore ModifyConfigCapability.


<pre><code><b>struct</b> <a href="Config.md#0x1_Config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a>&lt;ConfigValue: <b>copy</b>, drop, store&gt; <b>has</b> store, key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>cap: <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;<a href="Config.md#0x1_Config_ModifyConfigCapability">Config::ModifyConfigCapability</a>&lt;ConfigValue&gt;&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_Config_ConfigChangeEvent"></a>

## Struct `ConfigChangeEvent`

Event emitted when config value is changed.


<pre><code><b>struct</b> <a href="Config.md#0x1_Config_ConfigChangeEvent">ConfigChangeEvent</a>&lt;ConfigValue: <b>copy</b>, drop, store&gt; <b>has</b> drop, store
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

<a name="@Constants_0"></a>

## Constants


<a name="0x1_Config_ECAPABILITY_HOLDER_NOT_EXISTS"></a>



<pre><code><b>const</b> <a href="Config.md#0x1_Config_ECAPABILITY_HOLDER_NOT_EXISTS">ECAPABILITY_HOLDER_NOT_EXISTS</a>: u64 = 101;
</code></pre>



<a name="0x1_Config_ECONFIG_VALUE_DOES_NOT_EXIST"></a>



<pre><code><b>const</b> <a href="Config.md#0x1_Config_ECONFIG_VALUE_DOES_NOT_EXIST">ECONFIG_VALUE_DOES_NOT_EXIST</a>: u64 = 13;
</code></pre>



<a name="0x1_Config_get_by_address"></a>

## Function `get_by_address`

Get a copy of <code>ConfigValue</code> value stored under <code>addr</code>.


<pre><code><b>public</b> <b>fun</b> <a href="Config.md#0x1_Config_get_by_address">get_by_address</a>&lt;ConfigValue: <b>copy</b>, drop, store&gt;(addr: <b>address</b>): ConfigValue
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Config.md#0x1_Config_get_by_address">get_by_address</a>&lt;ConfigValue: <b>copy</b> + drop + store&gt;(addr: <b>address</b>): ConfigValue <b>acquires</b> <a href="Config.md#0x1_Config">Config</a> {
    <b>assert</b>!(<b>exists</b>&lt;<a href="Config.md#0x1_Config">Config</a>&lt;ConfigValue&gt;&gt;(addr), <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="Config.md#0x1_Config_ECONFIG_VALUE_DOES_NOT_EXIST">ECONFIG_VALUE_DOES_NOT_EXIST</a>));
    *&<b>borrow_global</b>&lt;<a href="Config.md#0x1_Config">Config</a>&lt;ConfigValue&gt;&gt;(addr).payload
}
</code></pre>



</details>

<a name="0x1_Config_config_exist_by_address"></a>

## Function `config_exist_by_address`

Check whether the config of <code>ConfigValue</code> type exists under <code>addr</code>.


<pre><code><b>public</b> <b>fun</b> <a href="Config.md#0x1_Config_config_exist_by_address">config_exist_by_address</a>&lt;ConfigValue: <b>copy</b>, drop, store&gt;(addr: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Config.md#0x1_Config_config_exist_by_address">config_exist_by_address</a>&lt;ConfigValue: <b>copy</b> + drop + store&gt;(addr: <b>address</b>): bool {
    <b>exists</b>&lt;<a href="Config.md#0x1_Config">Config</a>&lt;ConfigValue&gt;&gt;(addr)
}
</code></pre>



</details>

<a name="0x1_Config_set"></a>

## Function `set`

Set a config item to a new value with capability stored under signer


<pre><code><b>public</b> <b>fun</b> <a href="Config.md#0x1_Config_set">set</a>&lt;ConfigValue: <b>copy</b>, drop, store&gt;(account: &signer, payload: ConfigValue)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Config.md#0x1_Config_set">set</a>&lt;ConfigValue: <b>copy</b> + drop + store&gt;(account: &signer, payload: ConfigValue) <b>acquires</b> <a href="Config.md#0x1_Config">Config</a>,<a href="Config.md#0x1_Config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a>{
    <b>let</b> signer_address = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>assert</b>!(<b>exists</b>&lt;<a href="Config.md#0x1_Config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a>&lt;ConfigValue&gt;&gt;(signer_address), <a href="Errors.md#0x1_Errors_requires_capability">Errors::requires_capability</a>(<a href="Config.md#0x1_Config_ECAPABILITY_HOLDER_NOT_EXISTS">ECAPABILITY_HOLDER_NOT_EXISTS</a>));
    <b>let</b> cap_holder = <b>borrow_global_mut</b>&lt;<a href="Config.md#0x1_Config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a>&lt;ConfigValue&gt;&gt;(signer_address);
    <b>assert</b>!(<a href="Option.md#0x1_Option_is_some">Option::is_some</a>(&cap_holder.cap), <a href="Errors.md#0x1_Errors_requires_capability">Errors::requires_capability</a>(<a href="Config.md#0x1_Config_ECAPABILITY_HOLDER_NOT_EXISTS">ECAPABILITY_HOLDER_NOT_EXISTS</a>));
    <a href="Config.md#0x1_Config_set_with_capability">set_with_capability</a>(<a href="Option.md#0x1_Option_borrow_mut">Option::borrow_mut</a>(&<b>mut</b> cap_holder.cap), payload)
}
</code></pre>



</details>

<a name="0x1_Config_set_with_capability"></a>

## Function `set_with_capability`

Set a config item to a new value with cap.


<pre><code><b>public</b> <b>fun</b> <a href="Config.md#0x1_Config_set_with_capability">set_with_capability</a>&lt;ConfigValue: <b>copy</b>, drop, store&gt;(cap: &<b>mut</b> <a href="Config.md#0x1_Config_ModifyConfigCapability">Config::ModifyConfigCapability</a>&lt;ConfigValue&gt;, payload: ConfigValue)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Config.md#0x1_Config_set_with_capability">set_with_capability</a>&lt;ConfigValue: <b>copy</b> + drop + store&gt;(cap: &<b>mut</b> <a href="Config.md#0x1_Config_ModifyConfigCapability">ModifyConfigCapability</a>&lt;ConfigValue&gt;, payload: ConfigValue) <b>acquires</b> <a href="Config.md#0x1_Config">Config</a>{
    <b>let</b> addr = cap.account_address;
    <b>assert</b>!(<b>exists</b>&lt;<a href="Config.md#0x1_Config">Config</a>&lt;ConfigValue&gt;&gt;(addr), <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="Config.md#0x1_Config_ECONFIG_VALUE_DOES_NOT_EXIST">ECONFIG_VALUE_DOES_NOT_EXIST</a>));
    <b>let</b> config = <b>borrow_global_mut</b>&lt;<a href="Config.md#0x1_Config">Config</a>&lt;ConfigValue&gt;&gt;(addr);
    config.payload = <b>copy</b> payload;
    <a href="Config.md#0x1_Config_emit_config_change_event">emit_config_change_event</a>(cap, payload);
}
</code></pre>



</details>

<a name="0x1_Config_publish_new_config_with_capability"></a>

## Function `publish_new_config_with_capability`

Publish a new config item. The caller will use the returned ModifyConfigCapability to specify the access control
policy for who can modify the config.


<pre><code><b>public</b> <b>fun</b> <a href="Config.md#0x1_Config_publish_new_config_with_capability">publish_new_config_with_capability</a>&lt;ConfigValue: <b>copy</b>, drop, store&gt;(account: &signer, payload: ConfigValue): <a href="Config.md#0x1_Config_ModifyConfigCapability">Config::ModifyConfigCapability</a>&lt;ConfigValue&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Config.md#0x1_Config_publish_new_config_with_capability">publish_new_config_with_capability</a>&lt;ConfigValue: <b>copy</b> + drop + store&gt;(
    account: &signer,
    payload: ConfigValue,
): <a href="Config.md#0x1_Config_ModifyConfigCapability">ModifyConfigCapability</a>&lt;ConfigValue&gt; <b>acquires</b> <a href="Config.md#0x1_Config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a>{
    <a href="Config.md#0x1_Config_publish_new_config">publish_new_config</a>&lt;ConfigValue&gt;(account, payload);
    <a href="Config.md#0x1_Config_extract_modify_config_capability">extract_modify_config_capability</a>&lt;ConfigValue&gt;(account)
}
</code></pre>



</details>

<a name="0x1_Config_publish_new_config"></a>

## Function `publish_new_config`

Publish a new config item under account address.


<pre><code><b>public</b> <b>fun</b> <a href="Config.md#0x1_Config_publish_new_config">publish_new_config</a>&lt;ConfigValue: <b>copy</b>, drop, store&gt;(account: &signer, payload: ConfigValue)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Config.md#0x1_Config_publish_new_config">publish_new_config</a>&lt;ConfigValue: <b>copy</b> + drop + store&gt;(account: &signer, payload: ConfigValue) {
    <b>move_to</b>(account, <a href="Config.md#0x1_Config">Config</a>&lt;ConfigValue&gt;{ payload });
    <b>let</b> cap = <a href="Config.md#0x1_Config_ModifyConfigCapability">ModifyConfigCapability</a>&lt;ConfigValue&gt; {account_address: <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account), events: <a href="Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="Config.md#0x1_Config_ConfigChangeEvent">ConfigChangeEvent</a>&lt;ConfigValue&gt;&gt;(account)};
    <b>move_to</b>(account, <a href="Config.md#0x1_Config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a>{cap: <a href="Option.md#0x1_Option_some">Option::some</a>(cap)});
}
</code></pre>



</details>

<a name="0x1_Config_extract_modify_config_capability"></a>

## Function `extract_modify_config_capability`

Extract account's ModifyConfigCapability for ConfigValue type


<pre><code><b>public</b> <b>fun</b> <a href="Config.md#0x1_Config_extract_modify_config_capability">extract_modify_config_capability</a>&lt;ConfigValue: <b>copy</b>, drop, store&gt;(account: &signer): <a href="Config.md#0x1_Config_ModifyConfigCapability">Config::ModifyConfigCapability</a>&lt;ConfigValue&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Config.md#0x1_Config_extract_modify_config_capability">extract_modify_config_capability</a>&lt;ConfigValue: <b>copy</b> + drop + store&gt;(account: &signer): <a href="Config.md#0x1_Config_ModifyConfigCapability">ModifyConfigCapability</a>&lt;ConfigValue&gt; <b>acquires</b> <a href="Config.md#0x1_Config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a>{
    <b>let</b> signer_address = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>assert</b>!(<b>exists</b>&lt;<a href="Config.md#0x1_Config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a>&lt;ConfigValue&gt;&gt;(signer_address), <a href="Errors.md#0x1_Errors_requires_capability">Errors::requires_capability</a>(<a href="Config.md#0x1_Config_ECAPABILITY_HOLDER_NOT_EXISTS">ECAPABILITY_HOLDER_NOT_EXISTS</a>));
    <b>let</b> cap_holder = <b>borrow_global_mut</b>&lt;<a href="Config.md#0x1_Config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a>&lt;ConfigValue&gt;&gt;(signer_address);
    <a href="Option.md#0x1_Option_extract">Option::extract</a>(&<b>mut</b> cap_holder.cap)
}
</code></pre>



</details>

<a name="0x1_Config_restore_modify_config_capability"></a>

## Function `restore_modify_config_capability`

Restore account's ModifyConfigCapability


<pre><code><b>public</b> <b>fun</b> <a href="Config.md#0x1_Config_restore_modify_config_capability">restore_modify_config_capability</a>&lt;ConfigValue: <b>copy</b>, drop, store&gt;(cap: <a href="Config.md#0x1_Config_ModifyConfigCapability">Config::ModifyConfigCapability</a>&lt;ConfigValue&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Config.md#0x1_Config_restore_modify_config_capability">restore_modify_config_capability</a>&lt;ConfigValue: <b>copy</b> + drop + store&gt;(cap: <a href="Config.md#0x1_Config_ModifyConfigCapability">ModifyConfigCapability</a>&lt;ConfigValue&gt;) <b>acquires</b> <a href="Config.md#0x1_Config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a>{
    <b>let</b> cap_holder = <b>borrow_global_mut</b>&lt;<a href="Config.md#0x1_Config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a>&lt;ConfigValue&gt;&gt;(cap.account_address);
    <a href="Option.md#0x1_Option_fill">Option::fill</a>(&<b>mut</b> cap_holder.cap, cap);
}
</code></pre>



</details>

<a name="0x1_Config_destroy_modify_config_capability"></a>

## Function `destroy_modify_config_capability`

Destroy the given ModifyConfigCapability


<pre><code><b>public</b> <b>fun</b> <a href="Config.md#0x1_Config_destroy_modify_config_capability">destroy_modify_config_capability</a>&lt;ConfigValue: <b>copy</b>, drop, store&gt;(cap: <a href="Config.md#0x1_Config_ModifyConfigCapability">Config::ModifyConfigCapability</a>&lt;ConfigValue&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Config.md#0x1_Config_destroy_modify_config_capability">destroy_modify_config_capability</a>&lt;ConfigValue: <b>copy</b> + drop + store&gt;(cap: <a href="Config.md#0x1_Config_ModifyConfigCapability">ModifyConfigCapability</a>&lt;ConfigValue&gt;) {
    <b>let</b> <a href="Config.md#0x1_Config_ModifyConfigCapability">ModifyConfigCapability</a>{account_address:_, events} = cap;
    <a href="Event.md#0x1_Event_destroy_handle">Event::destroy_handle</a>(events)
}
</code></pre>



</details>

<a name="0x1_Config_account_address"></a>

## Function `account_address`

Return the address of the given ModifyConfigCapability


<pre><code><b>public</b> <b>fun</b> <a href="Config.md#0x1_Config_account_address">account_address</a>&lt;ConfigValue: <b>copy</b>, drop, store&gt;(cap: &<a href="Config.md#0x1_Config_ModifyConfigCapability">Config::ModifyConfigCapability</a>&lt;ConfigValue&gt;): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Config.md#0x1_Config_account_address">account_address</a>&lt;ConfigValue: <b>copy</b> + drop + store&gt;(cap: &<a href="Config.md#0x1_Config_ModifyConfigCapability">ModifyConfigCapability</a>&lt;ConfigValue&gt;): <b>address</b> {
    cap.account_address
}
</code></pre>



</details>

<a name="0x1_Config_emit_config_change_event"></a>

## Function `emit_config_change_event`

Emit a config change event.


<pre><code><b>fun</b> <a href="Config.md#0x1_Config_emit_config_change_event">emit_config_change_event</a>&lt;ConfigValue: <b>copy</b>, drop, store&gt;(cap: &<b>mut</b> <a href="Config.md#0x1_Config_ModifyConfigCapability">Config::ModifyConfigCapability</a>&lt;ConfigValue&gt;, value: ConfigValue)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="Config.md#0x1_Config_emit_config_change_event">emit_config_change_event</a>&lt;ConfigValue: <b>copy</b> + drop + store&gt;(cap: &<b>mut</b> <a href="Config.md#0x1_Config_ModifyConfigCapability">ModifyConfigCapability</a>&lt;ConfigValue&gt;, value: ConfigValue) {
    <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>&lt;<a href="Config.md#0x1_Config_ConfigChangeEvent">ConfigChangeEvent</a>&lt;ConfigValue&gt;&gt;(
        &<b>mut</b> cap.events,
        <a href="Config.md#0x1_Config_ConfigChangeEvent">ConfigChangeEvent</a> {
            account_address: cap.account_address,
            value: value,
        },
    );
}
</code></pre>



</details>

<a name="@Specification_1"></a>

## Specification



<pre><code><b>pragma</b> verify = <b>false</b>;
<b>pragma</b> aborts_if_is_strict;
</code></pre>




<a name="0x1_Config_spec_get"></a>


<pre><code><b>fun</b> <a href="Config.md#0x1_Config_spec_get">spec_get</a>&lt;ConfigValue&gt;(addr: <b>address</b>): ConfigValue {
   <b>global</b>&lt;<a href="Config.md#0x1_Config">Config</a>&lt;ConfigValue&gt;&gt;(addr).payload
}
</code></pre>



<a name="@Specification_1_get_by_address"></a>

### Function `get_by_address`


<pre><code><b>public</b> <b>fun</b> <a href="Config.md#0x1_Config_get_by_address">get_by_address</a>&lt;ConfigValue: <b>copy</b>, drop, store&gt;(addr: <b>address</b>): ConfigValue
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Config.md#0x1_Config">Config</a>&lt;ConfigValue&gt;&gt;(addr);
<b>ensures</b> <b>exists</b>&lt;<a href="Config.md#0x1_Config">Config</a>&lt;ConfigValue&gt;&gt;(addr);
</code></pre>



<a name="@Specification_1_config_exist_by_address"></a>

### Function `config_exist_by_address`


<pre><code><b>public</b> <b>fun</b> <a href="Config.md#0x1_Config_config_exist_by_address">config_exist_by_address</a>&lt;ConfigValue: <b>copy</b>, drop, store&gt;(addr: <b>address</b>): bool
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="@Specification_1_set"></a>

### Function `set`


<pre><code><b>public</b> <b>fun</b> <a href="Config.md#0x1_Config_set">set</a>&lt;ConfigValue: <b>copy</b>, drop, store&gt;(account: &signer, payload: ConfigValue)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Config.md#0x1_Config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a>&lt;ConfigValue&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
<b>aborts_if</b> !<a href="Option.md#0x1_Option_is_some">Option::is_some</a>&lt;<a href="Config.md#0x1_Config_ModifyConfigCapability">ModifyConfigCapability</a>&lt;ConfigValue&gt;&gt;(<a href="Config.md#0x1_Config_spec_cap">spec_cap</a>&lt;ConfigValue&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)));
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Config.md#0x1_Config">Config</a>&lt;ConfigValue&gt;&gt;(<a href="Option.md#0x1_Option_borrow">Option::borrow</a>&lt;<a href="Config.md#0x1_Config_ModifyConfigCapability">ModifyConfigCapability</a>&lt;ConfigValue&gt;&gt;(<a href="Config.md#0x1_Config_spec_cap">spec_cap</a>&lt;ConfigValue&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account))).account_address);
<b>ensures</b> <b>exists</b>&lt;<a href="Config.md#0x1_Config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a>&lt;ConfigValue&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
</code></pre>




<a name="0x1_Config_spec_cap"></a>


<pre><code><b>fun</b> <a href="Config.md#0x1_Config_spec_cap">spec_cap</a>&lt;ConfigValue&gt;(addr: <b>address</b>): <a href="Option.md#0x1_Option">Option</a>&lt;<a href="Config.md#0x1_Config_ModifyConfigCapability">ModifyConfigCapability</a>&lt;ConfigValue&gt;&gt; {
   <b>global</b>&lt;<a href="Config.md#0x1_Config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a>&lt;ConfigValue&gt;&gt;(addr).cap
}
</code></pre>



<a name="@Specification_1_set_with_capability"></a>

### Function `set_with_capability`


<pre><code><b>public</b> <b>fun</b> <a href="Config.md#0x1_Config_set_with_capability">set_with_capability</a>&lt;ConfigValue: <b>copy</b>, drop, store&gt;(cap: &<b>mut</b> <a href="Config.md#0x1_Config_ModifyConfigCapability">Config::ModifyConfigCapability</a>&lt;ConfigValue&gt;, payload: ConfigValue)
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Config.md#0x1_Config">Config</a>&lt;ConfigValue&gt;&gt;(cap.account_address);
<b>ensures</b> <b>exists</b>&lt;<a href="Config.md#0x1_Config">Config</a>&lt;ConfigValue&gt;&gt;(cap.account_address);
</code></pre>



<a name="@Specification_1_publish_new_config_with_capability"></a>

### Function `publish_new_config_with_capability`


<pre><code><b>public</b> <b>fun</b> <a href="Config.md#0x1_Config_publish_new_config_with_capability">publish_new_config_with_capability</a>&lt;ConfigValue: <b>copy</b>, drop, store&gt;(account: &signer, payload: ConfigValue): <a href="Config.md#0x1_Config_ModifyConfigCapability">Config::ModifyConfigCapability</a>&lt;ConfigValue&gt;
</code></pre>




<pre><code><b>aborts_if</b> <b>exists</b>&lt;<a href="Config.md#0x1_Config">Config</a>&lt;ConfigValue&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
<b>aborts_if</b> <b>exists</b>&lt;<a href="Config.md#0x1_Config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a>&lt;ConfigValue&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
<b>ensures</b> <b>exists</b>&lt;<a href="Config.md#0x1_Config">Config</a>&lt;ConfigValue&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
<b>ensures</b> <b>exists</b>&lt;<a href="Config.md#0x1_Config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a>&lt;ConfigValue&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
</code></pre>



<a name="@Specification_1_publish_new_config"></a>

### Function `publish_new_config`


<pre><code><b>public</b> <b>fun</b> <a href="Config.md#0x1_Config_publish_new_config">publish_new_config</a>&lt;ConfigValue: <b>copy</b>, drop, store&gt;(account: &signer, payload: ConfigValue)
</code></pre>




<pre><code><b>aborts_if</b> <b>exists</b>&lt;<a href="Config.md#0x1_Config">Config</a>&lt;ConfigValue&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
<b>aborts_if</b> <b>exists</b>&lt;<a href="Config.md#0x1_Config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a>&lt;ConfigValue&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
<b>ensures</b> <b>exists</b>&lt;<a href="Config.md#0x1_Config">Config</a>&lt;ConfigValue&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
<b>ensures</b> <b>exists</b>&lt;<a href="Config.md#0x1_Config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a>&lt;ConfigValue&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
</code></pre>




<a name="0x1_Config_PublishNewConfigAbortsIf"></a>


<pre><code><b>schema</b> <a href="Config.md#0x1_Config_PublishNewConfigAbortsIf">PublishNewConfigAbortsIf</a>&lt;ConfigValue&gt; {
    account: signer;
    <b>aborts_if</b> <b>exists</b>&lt;<a href="Config.md#0x1_Config">Config</a>&lt;ConfigValue&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
    <b>aborts_if</b> <b>exists</b>&lt;<a href="Config.md#0x1_Config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a>&lt;ConfigValue&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
}
</code></pre>




<a name="0x1_Config_AbortsIfConfigNotExist"></a>


<pre><code><b>schema</b> <a href="Config.md#0x1_Config_AbortsIfConfigNotExist">AbortsIfConfigNotExist</a>&lt;ConfigValue&gt; {
    addr: <b>address</b>;
    <b>aborts_if</b> !<b>exists</b>&lt;<a href="Config.md#0x1_Config">Config</a>&lt;ConfigValue&gt;&gt;(addr);
}
</code></pre>




<a name="0x1_Config_AbortsIfConfigOrCapabilityNotExist"></a>


<pre><code><b>schema</b> <a href="Config.md#0x1_Config_AbortsIfConfigOrCapabilityNotExist">AbortsIfConfigOrCapabilityNotExist</a>&lt;ConfigValue&gt; {
    addr: <b>address</b>;
    <b>aborts_if</b> !<b>exists</b>&lt;<a href="Config.md#0x1_Config">Config</a>&lt;ConfigValue&gt;&gt;(addr);
    <b>aborts_if</b> !<b>exists</b>&lt;<a href="Config.md#0x1_Config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a>&lt;ConfigValue&gt;&gt;(addr);
}
</code></pre>




<a name="0x1_Config_PublishNewConfigEnsures"></a>


<pre><code><b>schema</b> <a href="Config.md#0x1_Config_PublishNewConfigEnsures">PublishNewConfigEnsures</a>&lt;ConfigValue&gt; {
    account: signer;
    <b>ensures</b> <b>exists</b>&lt;<a href="Config.md#0x1_Config">Config</a>&lt;ConfigValue&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
    <b>ensures</b> <b>exists</b>&lt;<a href="Config.md#0x1_Config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a>&lt;ConfigValue&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
}
</code></pre>



<a name="@Specification_1_extract_modify_config_capability"></a>

### Function `extract_modify_config_capability`


<pre><code><b>public</b> <b>fun</b> <a href="Config.md#0x1_Config_extract_modify_config_capability">extract_modify_config_capability</a>&lt;ConfigValue: <b>copy</b>, drop, store&gt;(account: &signer): <a href="Config.md#0x1_Config_ModifyConfigCapability">Config::ModifyConfigCapability</a>&lt;ConfigValue&gt;
</code></pre>




<pre><code><b>include</b> <a href="Config.md#0x1_Config_AbortsIfCapNotExist">AbortsIfCapNotExist</a>&lt;ConfigValue&gt;{account: <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)};
<b>ensures</b> <b>exists</b>&lt;<a href="Config.md#0x1_Config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a>&lt;ConfigValue&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
<b>ensures</b> <a href="Option.md#0x1_Option_is_none">Option::is_none</a>&lt;<a href="Config.md#0x1_Config_ModifyConfigCapability">ModifyConfigCapability</a>&lt;ConfigValue&gt;&gt;(<b>global</b>&lt;<a href="Config.md#0x1_Config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a>&lt;ConfigValue&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)).cap);
</code></pre>



<a name="@Specification_1_restore_modify_config_capability"></a>

### Function `restore_modify_config_capability`


<pre><code><b>public</b> <b>fun</b> <a href="Config.md#0x1_Config_restore_modify_config_capability">restore_modify_config_capability</a>&lt;ConfigValue: <b>copy</b>, drop, store&gt;(cap: <a href="Config.md#0x1_Config_ModifyConfigCapability">Config::ModifyConfigCapability</a>&lt;ConfigValue&gt;)
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Config.md#0x1_Config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a>&lt;ConfigValue&gt;&gt;(cap.account_address);
<b>aborts_if</b> <a href="Option.md#0x1_Option_is_some">Option::is_some</a>&lt;<a href="Config.md#0x1_Config_ModifyConfigCapability">ModifyConfigCapability</a>&lt;ConfigValue&gt;&gt;(<b>global</b>&lt;<a href="Config.md#0x1_Config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a>&lt;ConfigValue&gt;&gt;(cap.account_address).cap);
<b>ensures</b> <b>exists</b>&lt;<a href="Config.md#0x1_Config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a>&lt;ConfigValue&gt;&gt;(cap.account_address);
<b>ensures</b> <a href="Option.md#0x1_Option_is_some">Option::is_some</a>&lt;<a href="Config.md#0x1_Config_ModifyConfigCapability">ModifyConfigCapability</a>&lt;ConfigValue&gt;&gt;(<b>global</b>&lt;<a href="Config.md#0x1_Config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a>&lt;ConfigValue&gt;&gt;(cap.account_address).cap);
</code></pre>



<a name="@Specification_1_destroy_modify_config_capability"></a>

### Function `destroy_modify_config_capability`


<pre><code><b>public</b> <b>fun</b> <a href="Config.md#0x1_Config_destroy_modify_config_capability">destroy_modify_config_capability</a>&lt;ConfigValue: <b>copy</b>, drop, store&gt;(cap: <a href="Config.md#0x1_Config_ModifyConfigCapability">Config::ModifyConfigCapability</a>&lt;ConfigValue&gt;)
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="@Specification_1_account_address"></a>

### Function `account_address`


<pre><code><b>public</b> <b>fun</b> <a href="Config.md#0x1_Config_account_address">account_address</a>&lt;ConfigValue: <b>copy</b>, drop, store&gt;(cap: &<a href="Config.md#0x1_Config_ModifyConfigCapability">Config::ModifyConfigCapability</a>&lt;ConfigValue&gt;): <b>address</b>
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="@Specification_1_emit_config_change_event"></a>

### Function `emit_config_change_event`


<pre><code><b>fun</b> <a href="Config.md#0x1_Config_emit_config_change_event">emit_config_change_event</a>&lt;ConfigValue: <b>copy</b>, drop, store&gt;(cap: &<b>mut</b> <a href="Config.md#0x1_Config_ModifyConfigCapability">Config::ModifyConfigCapability</a>&lt;ConfigValue&gt;, value: ConfigValue)
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>
