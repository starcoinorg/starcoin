
<a name="0x1_Config"></a>

# Module `0x1::Config`

### Table of Contents

-  [Resource `Config`](#0x1_Config_Config)
-  [Resource `ModifyConfigCapability`](#0x1_Config_ModifyConfigCapability)
-  [Resource `ModifyConfigCapabilityHolder`](#0x1_Config_ModifyConfigCapabilityHolder)
-  [Struct `ConfigChangeEvent`](#0x1_Config_ConfigChangeEvent)
-  [Function `get`](#0x1_Config_get)
-  [Function `get_by_address`](#0x1_Config_get_by_address)
-  [Function `set`](#0x1_Config_set)
-  [Function `set_with_capability`](#0x1_Config_set_with_capability)
-  [Function `publish_new_config_with_capability`](#0x1_Config_publish_new_config_with_capability)
-  [Function `publish_new_config`](#0x1_Config_publish_new_config)
-  [Function `extract_modify_config_capability`](#0x1_Config_extract_modify_config_capability)
-  [Function `restore_modify_config_capability`](#0x1_Config_restore_modify_config_capability)
-  [Function `emit_config_change_event`](#0x1_Config_emit_config_change_event)
-  [Specification](#0x1_Config_Specification)



<a name="0x1_Config_Config"></a>

## Resource `Config`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_Config">Config</a>&lt;ConfigValue: <b>copyable</b>&gt;
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

## Resource `ModifyConfigCapability`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_Config_ModifyConfigCapability">ModifyConfigCapability</a>&lt;ConfigValue: <b>copyable</b>&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>account_address: address</code>
</dt>
<dd>

</dd>
<dt>

<code>events: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="#0x1_Config_ConfigChangeEvent">Config::ConfigChangeEvent</a>&lt;ConfigValue&gt;&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_Config_ModifyConfigCapabilityHolder"></a>

## Resource `ModifyConfigCapabilityHolder`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_Config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a>&lt;ConfigValue: <b>copyable</b>&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>cap: <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;<a href="#0x1_Config_ModifyConfigCapability">Config::ModifyConfigCapability</a>&lt;ConfigValue&gt;&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_Config_ConfigChangeEvent"></a>

## Struct `ConfigChangeEvent`



<pre><code><b>struct</b> <a href="#0x1_Config_ConfigChangeEvent">ConfigChangeEvent</a>&lt;ConfigValue: <b>copyable</b>&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>account_address: address</code>
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

<a name="0x1_Config_get"></a>

## Function `get`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Config_get">get</a>&lt;ConfigValue: <b>copyable</b>&gt;(account: &signer): ConfigValue
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Config_get">get</a>&lt;ConfigValue: <b>copyable</b>&gt;(account: &signer): ConfigValue <b>acquires</b> <a href="#0x1_Config">Config</a> {
    <b>let</b> addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>assert</b>(exists&lt;<a href="#0x1_Config">Config</a>&lt;ConfigValue&gt;&gt;(addr), 24);
    *&borrow_global&lt;<a href="#0x1_Config">Config</a>&lt;ConfigValue&gt;&gt;(addr).payload
}
</code></pre>



</details>

<a name="0x1_Config_get_by_address"></a>

## Function `get_by_address`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Config_get_by_address">get_by_address</a>&lt;ConfigValue: <b>copyable</b>&gt;(addr: address): ConfigValue
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Config_get_by_address">get_by_address</a>&lt;ConfigValue: <b>copyable</b>&gt;(addr: address): ConfigValue <b>acquires</b> <a href="#0x1_Config">Config</a> {
    <b>assert</b>(exists&lt;<a href="#0x1_Config">Config</a>&lt;ConfigValue&gt;&gt;(addr), 24);
    *&borrow_global&lt;<a href="#0x1_Config">Config</a>&lt;ConfigValue&gt;&gt;(addr).payload
}
</code></pre>



</details>

<a name="0x1_Config_set"></a>

## Function `set`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Config_set">set</a>&lt;ConfigValue: <b>copyable</b>&gt;(account: &signer, payload: ConfigValue)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Config_set">set</a>&lt;ConfigValue: <b>copyable</b>&gt;(account: &signer, payload: ConfigValue) <b>acquires</b> <a href="#0x1_Config">Config</a>,<a href="#0x1_Config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a>{
    <b>let</b> signer_address = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    //TODO <b>define</b> no capability error code.
    <b>assert</b>(exists&lt;<a href="#0x1_Config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a>&lt;ConfigValue&gt;&gt;(signer_address), 24);
    <b>let</b> cap_holder = borrow_global_mut&lt;<a href="#0x1_Config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a>&lt;ConfigValue&gt;&gt;(signer_address);
    <b>assert</b>(<a href="Option.md#0x1_Option_is_some">Option::is_some</a>(&cap_holder.cap), 24);
    <a href="#0x1_Config_set_with_capability">set_with_capability</a>(<a href="Option.md#0x1_Option_borrow_mut">Option::borrow_mut</a>(&<b>mut</b> cap_holder.cap), payload)
}
</code></pre>



</details>

<a name="0x1_Config_set_with_capability"></a>

## Function `set_with_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Config_set_with_capability">set_with_capability</a>&lt;ConfigValue: <b>copyable</b>&gt;(cap: &<b>mut</b> <a href="#0x1_Config_ModifyConfigCapability">Config::ModifyConfigCapability</a>&lt;ConfigValue&gt;, payload: ConfigValue)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Config_set_with_capability">set_with_capability</a>&lt;ConfigValue: <b>copyable</b>&gt;(cap: &<b>mut</b> <a href="#0x1_Config_ModifyConfigCapability">ModifyConfigCapability</a>&lt;ConfigValue&gt;, payload: ConfigValue) <b>acquires</b> <a href="#0x1_Config">Config</a>{
    <b>let</b> addr = cap.account_address;
    <b>assert</b>(exists&lt;<a href="#0x1_Config">Config</a>&lt;ConfigValue&gt;&gt;(addr), 24);
    <b>let</b> config = borrow_global_mut&lt;<a href="#0x1_Config">Config</a>&lt;ConfigValue&gt;&gt;(addr);
    config.payload = <b>copy</b> payload;
    <a href="#0x1_Config_emit_config_change_event">emit_config_change_event</a>(cap, payload);
}
</code></pre>



</details>

<a name="0x1_Config_publish_new_config_with_capability"></a>

## Function `publish_new_config_with_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Config_publish_new_config_with_capability">publish_new_config_with_capability</a>&lt;ConfigValue: <b>copyable</b>&gt;(account: &signer, payload: ConfigValue): <a href="#0x1_Config_ModifyConfigCapability">Config::ModifyConfigCapability</a>&lt;ConfigValue&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Config_publish_new_config_with_capability">publish_new_config_with_capability</a>&lt;ConfigValue: <b>copyable</b>&gt;(
    account: &signer,
    payload: ConfigValue,
): <a href="#0x1_Config_ModifyConfigCapability">ModifyConfigCapability</a>&lt;ConfigValue&gt; <b>acquires</b> <a href="#0x1_Config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a>{
    <a href="#0x1_Config_publish_new_config">publish_new_config</a>&lt;ConfigValue&gt;(account, payload);
    <a href="#0x1_Config_extract_modify_config_capability">extract_modify_config_capability</a>&lt;ConfigValue&gt;(account)
}
</code></pre>



</details>

<a name="0x1_Config_publish_new_config"></a>

## Function `publish_new_config`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Config_publish_new_config">publish_new_config</a>&lt;ConfigValue: <b>copyable</b>&gt;(account: &signer, payload: ConfigValue)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Config_publish_new_config">publish_new_config</a>&lt;ConfigValue: <b>copyable</b>&gt;(account: &signer, payload: ConfigValue) {
    move_to(account, <a href="#0x1_Config">Config</a>{ payload });
    <b>let</b> cap = <a href="#0x1_Config_ModifyConfigCapability">ModifyConfigCapability</a>&lt;ConfigValue&gt; {account_address: <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account), events: <a href="Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="#0x1_Config_ConfigChangeEvent">ConfigChangeEvent</a>&lt;ConfigValue&gt;&gt;(account)};
    move_to(account, <a href="#0x1_Config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a>{cap: <a href="Option.md#0x1_Option_some">Option::some</a>(cap)});
}
</code></pre>



</details>

<a name="0x1_Config_extract_modify_config_capability"></a>

## Function `extract_modify_config_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Config_extract_modify_config_capability">extract_modify_config_capability</a>&lt;ConfigValue: <b>copyable</b>&gt;(account: &signer): <a href="#0x1_Config_ModifyConfigCapability">Config::ModifyConfigCapability</a>&lt;ConfigValue&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Config_extract_modify_config_capability">extract_modify_config_capability</a>&lt;ConfigValue: <b>copyable</b>&gt;(account: &signer): <a href="#0x1_Config_ModifyConfigCapability">ModifyConfigCapability</a>&lt;ConfigValue&gt; <b>acquires</b> <a href="#0x1_Config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a>{
    <b>let</b> signer_address = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>let</b> cap_holder = borrow_global_mut&lt;<a href="#0x1_Config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a>&lt;ConfigValue&gt;&gt;(signer_address);
    <a href="Option.md#0x1_Option_extract">Option::extract</a>(&<b>mut</b> cap_holder.cap)
}
</code></pre>



</details>

<a name="0x1_Config_restore_modify_config_capability"></a>

## Function `restore_modify_config_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Config_restore_modify_config_capability">restore_modify_config_capability</a>&lt;ConfigValue: <b>copyable</b>&gt;(cap: <a href="#0x1_Config_ModifyConfigCapability">Config::ModifyConfigCapability</a>&lt;ConfigValue&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Config_restore_modify_config_capability">restore_modify_config_capability</a>&lt;ConfigValue: <b>copyable</b>&gt;(cap: <a href="#0x1_Config_ModifyConfigCapability">ModifyConfigCapability</a>&lt;ConfigValue&gt;) <b>acquires</b> <a href="#0x1_Config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a>{
    <b>let</b> cap_holder = borrow_global_mut&lt;<a href="#0x1_Config_ModifyConfigCapabilityHolder">ModifyConfigCapabilityHolder</a>&lt;ConfigValue&gt;&gt;(cap.account_address);
    <a href="Option.md#0x1_Option_fill">Option::fill</a>(&<b>mut</b> cap_holder.cap, cap);
}
</code></pre>



</details>

<a name="0x1_Config_emit_config_change_event"></a>

## Function `emit_config_change_event`



<pre><code><b>fun</b> <a href="#0x1_Config_emit_config_change_event">emit_config_change_event</a>&lt;ConfigValue: <b>copyable</b>&gt;(cap: &<b>mut</b> <a href="#0x1_Config_ModifyConfigCapability">Config::ModifyConfigCapability</a>&lt;ConfigValue&gt;, value: ConfigValue)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_Config_emit_config_change_event">emit_config_change_event</a>&lt;ConfigValue: <b>copyable</b>&gt;(cap: &<b>mut</b> <a href="#0x1_Config_ModifyConfigCapability">ModifyConfigCapability</a>&lt;ConfigValue&gt;, value: ConfigValue) {
    <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>&lt;<a href="#0x1_Config_ConfigChangeEvent">ConfigChangeEvent</a>&lt;ConfigValue&gt;&gt;(
        &<b>mut</b> cap.events,
        <a href="#0x1_Config_ConfigChangeEvent">ConfigChangeEvent</a> {
            account_address: cap.account_address,
            value: value,
        },
    );
}
</code></pre>



</details>

<a name="0x1_Config_Specification"></a>

## Specification



<pre><code>pragma verify = <b>false</b>;
</code></pre>
