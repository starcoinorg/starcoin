
<a name="0x1_Version"></a>

# Module `0x1::Version`



-  [Struct `Version`](#0x1_Version_Version)
-  [Function `initialize`](#0x1_Version_initialize)
-  [Function `new_version`](#0x1_Version_new_version)
-  [Function `get`](#0x1_Version_get)
-  [Function `set`](#0x1_Version_set)
-  [Specification](#@Specification_0)
    -  [Function `initialize`](#@Specification_0_initialize)
    -  [Function `new_version`](#@Specification_0_new_version)
    -  [Function `get`](#@Specification_0_get)
    -  [Function `set`](#@Specification_0_set)


<pre><code><b>use</b> <a href="Config.md#0x1_Config">0x1::Config</a>;
<b>use</b> <a href="CoreAddresses.md#0x1_CoreAddresses">0x1::CoreAddresses</a>;
<b>use</b> <a href="Signer.md#0x1_Signer">0x1::Signer</a>;
</code></pre>



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
    <a href="CoreAddresses.md#0x1_CoreAddresses_assert_genesis_address">CoreAddresses::assert_genesis_address</a>(account);
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
    <b>assert</b>(<a href="Version.md#0x1_Version_get">Self::get</a>() &lt; major, 25);
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
    <a href="CoreAddresses.md#0x1_CoreAddresses_assert_genesis_address">CoreAddresses::assert_genesis_address</a>(account);
    <b>let</b> old_config = <a href="Config.md#0x1_Config_get_by_address">Config::get_by_address</a>&lt;<a href="Version.md#0x1_Version_Version">Self::Version</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
    <b>assert</b>(old_config.major &lt; major, 25);  //todo
    <a href="Config.md#0x1_Config_set">Config::set</a>&lt;<a href="Version.md#0x1_Version_Version">Self::Version</a>&gt;(account, <a href="Version.md#0x1_Version">Version</a> { major });
}
</code></pre>



</details>

<a name="@Specification_0"></a>

## Specification



<pre><code><b>pragma</b> verify;
<b>pragma</b> aborts_if_is_strict;
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



<a name="@Specification_0_new_version"></a>

### Function `new_version`


<pre><code><b>public</b> <b>fun</b> <a href="Version.md#0x1_Version_new_version">new_version</a>(major: u64): <a href="Version.md#0x1_Version_Version">Version::Version</a>
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Config.md#0x1_Config_Config">Config::Config</a>&lt;<a href="Version.md#0x1_Version">Version</a>&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
<b>aborts_if</b> <a href="Version.md#0x1_Version_get">Self::get</a>() &gt;= major;
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




<pre><code><b>pragma</b> verify = <b>false</b>;
<b>aborts_if</b> <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account) != <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>();
<b>aborts_if</b> <a href="Config.md#0x1_Config_spec_get">Config::spec_get</a>&lt;<a href="Version.md#0x1_Version">Version</a>&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account)).major &gt;= major;
</code></pre>
