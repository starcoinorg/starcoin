
<a name="0x1_Version"></a>

# Module `0x1::Version`



-  [Struct <code><a href="Version.md#0x1_Version">Version</a></code>](#0x1_Version_Version)
-  [Function <code>initialize</code>](#0x1_Version_initialize)
-  [Function <code>new_version</code>](#0x1_Version_new_version)
-  [Function <code>get</code>](#0x1_Version_get)
-  [Function <code>set</code>](#0x1_Version_set)
-  [Specification](#@Specification_0)
    -  [Function <code>initialize</code>](#@Specification_0_initialize)
    -  [Function <code>get</code>](#@Specification_0_get)
    -  [Function <code>set</code>](#@Specification_0_set)


<a name="0x1_Version_Version"></a>

## Struct `Version`



<pre><code><b>struct</b> <a href="Version.md#0x1_Version">Version</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>major: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_Version_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="Version.md#0x1_Version_initialize">initialize</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Version.md#0x1_Version_initialize">initialize</a>(account: &signer) {
    <b>assert</b>(
        <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>(),
        <a href="ErrorCode.md#0x1_ErrorCode_ENOT_GENESIS_ACCOUNT">ErrorCode::ENOT_GENESIS_ACCOUNT</a>(),
    );
    <a href="Config.md#0x1_Config_publish_new_config">Config::publish_new_config</a>&lt;<a href="Version.md#0x1_Version_Version">Self::Version</a>&gt;(account, <a href="Version.md#0x1_Version">Version</a> { major: 1 });
}
</code></pre>



</details>

<a name="0x1_Version_new_version"></a>

## Function `new_version`



<pre><code><b>public</b> <b>fun</b> <a href="Version.md#0x1_Version_new_version">new_version</a>(major: u64): <a href="Version.md#0x1_Version_Version">Version::Version</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Version.md#0x1_Version_new_version">new_version</a>(major: u64): <a href="Version.md#0x1_Version">Version</a> {
    <a href="Version.md#0x1_Version">Version</a> { major }
}
</code></pre>



</details>

<a name="0x1_Version_get"></a>

## Function `get`



<pre><code><b>public</b> <b>fun</b> <a href="Version.md#0x1_Version_get">get</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Version.md#0x1_Version_get">get</a>(): u64 {
    <b>let</b> version = <a href="Config.md#0x1_Config_get_by_address">Config::get_by_address</a>&lt;<a href="Version.md#0x1_Version_Version">Self::Version</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
    version.major
}
</code></pre>



</details>

<a name="0x1_Version_set"></a>

## Function `set`



<pre><code><b>public</b> <b>fun</b> <a href="Version.md#0x1_Version_set">set</a>(account: &signer, major: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Version.md#0x1_Version_set">set</a>(account: &signer, major: u64) {
    <b>assert</b>(
        <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>(),
        <a href="ErrorCode.md#0x1_ErrorCode_ENOT_GENESIS_ACCOUNT">ErrorCode::ENOT_GENESIS_ACCOUNT</a>(),
    );
    <b>let</b> old_config = <a href="Config.md#0x1_Config_get_by_address">Config::get_by_address</a>&lt;<a href="Version.md#0x1_Version_Version">Self::Version</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
    <b>assert</b>(old_config.major &lt; major, 25);
    <a href="Config.md#0x1_Config_set">Config::set</a>&lt;<a href="Version.md#0x1_Version_Version">Self::Version</a>&gt;(account, <a href="Version.md#0x1_Version">Version</a> { major });
}
</code></pre>



</details>

<a name="@Specification_0"></a>

## Specification



<pre><code>pragma verify;
pragma aborts_if_is_strict;
</code></pre>



<a name="@Specification_0_initialize"></a>

### Function `initialize`


<pre><code><b>public</b> <b>fun</b> <a href="Version.md#0x1_Version_initialize">initialize</a>(account: &signer)
</code></pre>




<pre><code><b>aborts_if</b> <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account) != <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>();
<b>aborts_if</b>
    <b>exists</b>&lt;<a href="Config.md#0x1_Config_ModifyConfigCapabilityHolder">Config::ModifyConfigCapabilityHolder</a>&lt;<a href="Version.md#0x1_Version">Version</a>&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
<b>aborts_if</b> <b>exists</b>&lt;<a href="Config.md#0x1_Config_Config">Config::Config</a>&lt;<a href="Version.md#0x1_Version">Version</a>&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
<b>ensures</b>
    <b>exists</b>&lt;<a href="Config.md#0x1_Config_ModifyConfigCapabilityHolder">Config::ModifyConfigCapabilityHolder</a>&lt;<a href="Version.md#0x1_Version">Version</a>&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
<b>ensures</b> <b>exists</b>&lt;<a href="Config.md#0x1_Config_Config">Config::Config</a>&lt;<a href="Version.md#0x1_Version">Version</a>&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
</code></pre>



<a name="@Specification_0_get"></a>

### Function `get`


<pre><code><b>public</b> <b>fun</b> <a href="Version.md#0x1_Version_get">get</a>(): u64
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Config.md#0x1_Config_Config">Config::Config</a>&lt;<a href="Version.md#0x1_Version">Version</a>&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
</code></pre>



<a name="@Specification_0_set"></a>

### Function `set`


<pre><code><b>public</b> <b>fun</b> <a href="Version.md#0x1_Version_set">set</a>(account: &signer, major: u64)
</code></pre>




<pre><code>pragma verify = <b>false</b>;
<b>aborts_if</b> <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account) != <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>();
<b>aborts_if</b> <a href="Config.md#0x1_Config_spec_get">Config::spec_get</a>&lt;<a href="Version.md#0x1_Version">Version</a>&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account)).major &gt;= major;
</code></pre>
