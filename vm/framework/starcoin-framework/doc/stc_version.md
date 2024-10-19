
<a id="0x1_stc_version"></a>

# Module `0x1::stc_version`

<code><a href="stc_version.md#0x1_stc_version_Version">Version</a></code> tracks version of something, like current VM version.


-  [Struct `Version`](#0x1_stc_version_Version)
-  [Constants](#@Constants_0)
-  [Function `new_version`](#0x1_stc_version_new_version)
-  [Function `get`](#0x1_stc_version_get)
-  [Specification](#@Specification_1)
    -  [Function `new_version`](#@Specification_1_new_version)
    -  [Function `get`](#@Specification_1_get)


<pre><code><b>use</b> <a href="on_chain_config.md#0x1_on_chain_config">0x1::on_chain_config</a>;
</code></pre>



<a id="0x1_stc_version_Version"></a>

## Struct `Version`

Version.


<pre><code><b>struct</b> <a href="stc_version.md#0x1_stc_version_Version">Version</a> <b>has</b> <b>copy</b>, drop, store
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

<a id="@Constants_0"></a>

## Constants


<a id="0x1_stc_version_EMAJOR_TO_OLD"></a>



<pre><code><b>const</b> <a href="stc_version.md#0x1_stc_version_EMAJOR_TO_OLD">EMAJOR_TO_OLD</a>: u64 = 101;
</code></pre>



<a id="0x1_stc_version_new_version"></a>

## Function `new_version`

Create a new version.


<pre><code><b>public</b> <b>fun</b> <a href="stc_version.md#0x1_stc_version_new_version">new_version</a>(major: u64): <a href="stc_version.md#0x1_stc_version_Version">stc_version::Version</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_version.md#0x1_stc_version_new_version">new_version</a>(major: u64): <a href="stc_version.md#0x1_stc_version_Version">Version</a> {
    <a href="stc_version.md#0x1_stc_version_Version">Version</a> { major }
}
</code></pre>



</details>

<a id="0x1_stc_version_get"></a>

## Function `get`

Get version under <code>addr</code>.


<pre><code><b>public</b> <b>fun</b> <a href="stc_version.md#0x1_stc_version_get">get</a>(addr: <b>address</b>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_version.md#0x1_stc_version_get">get</a>(addr: <b>address</b>): u64 {
    <b>let</b> <a href="version.md#0x1_version">version</a> = <a href="on_chain_config.md#0x1_on_chain_config_get_by_address">on_chain_config::get_by_address</a>&lt;<a href="stc_version.md#0x1_stc_version_Version">Self::Version</a>&gt;(addr);
    <a href="version.md#0x1_version">version</a>.major
}
</code></pre>



</details>

<a id="@Specification_1"></a>

## Specification



<pre><code><b>pragma</b> verify;
<b>pragma</b> aborts_if_is_strict;
</code></pre>



<a id="@Specification_1_new_version"></a>

### Function `new_version`


<pre><code><b>public</b> <b>fun</b> <a href="stc_version.md#0x1_stc_version_new_version">new_version</a>(major: u64): <a href="stc_version.md#0x1_stc_version_Version">stc_version::Version</a>
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a id="@Specification_1_get"></a>

### Function `get`


<pre><code><b>public</b> <b>fun</b> <a href="stc_version.md#0x1_stc_version_get">get</a>(addr: <b>address</b>): u64
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_Config">on_chain_config::Config</a>&lt;<a href="stc_version.md#0x1_stc_version_Version">Version</a>&gt;&gt;(addr);
</code></pre>


[move-book]: https://starcoin.dev/move/book/SUMMARY
