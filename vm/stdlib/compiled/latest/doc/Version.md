
<a name="0x1_Version"></a>

# Module `0x1::Version`

<code><a href="Version.md#0x1_Version">Version</a></code> tracks version of something, like current VM version.


-  [Struct `Version`](#0x1_Version_Version)
-  [Constants](#@Constants_0)
-  [Function `new_version`](#0x1_Version_new_version)
-  [Function `get`](#0x1_Version_get)
-  [Specification](#@Specification_1)
    -  [Function `new_version`](#@Specification_1_new_version)
    -  [Function `get`](#@Specification_1_get)


<pre><code><b>use</b> <a href="Config.md#0x1_Config">0x1::Config</a>;
</code></pre>



<a name="0x1_Version_Version"></a>

## Struct `Version`

Version.


<pre><code><b>struct</b> <a href="Version.md#0x1_Version">Version</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>major: u64</code>
</dt>
<dd>
 major number.
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

Create a new version.


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

Get version under <code>addr</code>.


<pre><code><b>public</b> <b>fun</b> <a href="Version.md#0x1_Version_get">get</a>(addr: <b>address</b>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Version.md#0x1_Version_get">get</a>(addr: <b>address</b>): u64 {
    <b>let</b> version = <a href="Config.md#0x1_Config_get_by_address">Config::get_by_address</a>&lt;<a href="Version.md#0x1_Version_Version">Self::Version</a>&gt;(addr);
    version.major
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


<pre><code><b>public</b> <b>fun</b> <a href="Version.md#0x1_Version_get">get</a>(addr: <b>address</b>): u64
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Config.md#0x1_Config_Config">Config::Config</a>&lt;<a href="Version.md#0x1_Version">Version</a>&gt;&gt;(addr);
</code></pre>
