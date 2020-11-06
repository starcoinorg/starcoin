
<a name="0x1_Version"></a>

# Module `0x1::Version`



-  [Struct `Version`](#0x1_Version_Version)
-  [Constants](#@Constants_0)
-  [Function `new_version`](#0x1_Version_new_version)
-  [Function `get`](#0x1_Version_get)
-  [Function `set`](#0x1_Version_set)
-  [Specification](#@Specification_1)
    -  [Function `new_version`](#@Specification_1_new_version)
    -  [Function `get`](#@Specification_1_get)
    -  [Function `set`](#@Specification_1_set)


<pre><code><b>use</b> <a href="Config.md#0x1_Config">0x1::Config</a>;
<b>use</b> <a href="Errors.md#0x1_Errors">0x1::Errors</a>;
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

<a name="@Constants_0"></a>

## Constants


<a name="0x1_Version_EMAJOR_TO_OLD"></a>



<pre><code><b>const</b> <a href="Version.md#0x1_Version_EMAJOR_TO_OLD">EMAJOR_TO_OLD</a>: u64 = 101;
</code></pre>



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



<pre><code><b>public</b> <b>fun</b> <a href="Version.md#0x1_Version_get">get</a>(addr: address): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Version.md#0x1_Version_get">get</a>(addr: address): u64 {
    <b>let</b> version = <a href="Config.md#0x1_Config_get_by_address">Config::get_by_address</a>&lt;<a href="Version.md#0x1_Version_Version">Self::Version</a>&gt;(addr);
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
    <b>let</b> old_config = <a href="Config.md#0x1_Config_get_by_address">Config::get_by_address</a>&lt;<a href="Version.md#0x1_Version_Version">Self::Version</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
    <b>assert</b>(old_config.major &lt; major, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Version.md#0x1_Version_EMAJOR_TO_OLD">EMAJOR_TO_OLD</a>));
    <a href="Config.md#0x1_Config_set">Config::set</a>&lt;<a href="Version.md#0x1_Version_Version">Self::Version</a>&gt;(account, <a href="Version.md#0x1_Version">Version</a> { major });
}
</code></pre>



</details>

<a name="@Specification_1"></a>

## Specification



<pre><code><b>pragma</b> verify;
<b>pragma</b> aborts_if_is_strict;
</code></pre>



<a name="@Specification_1_new_version"></a>

### Function `new_version`


<pre><code><b>public</b> <b>fun</b> <a href="Version.md#0x1_Version_new_version">new_version</a>(major: u64): <a href="Version.md#0x1_Version_Version">Version::Version</a>
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="@Specification_1_get"></a>

### Function `get`


<pre><code><b>public</b> <b>fun</b> <a href="Version.md#0x1_Version_get">get</a>(addr: address): u64
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Config.md#0x1_Config_Config">Config::Config</a>&lt;<a href="Version.md#0x1_Version">Version</a>&gt;&gt;(addr);
</code></pre>



<a name="@Specification_1_set"></a>

### Function `set`


<pre><code><b>public</b> <b>fun</b> <a href="Version.md#0x1_Version_set">set</a>(account: &signer, major: u64)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
<b>aborts_if</b> <a href="Config.md#0x1_Config_spec_get">Config::spec_get</a>&lt;<a href="Version.md#0x1_Version">Version</a>&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account)).major &gt;= major;
</code></pre>
