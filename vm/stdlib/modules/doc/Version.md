
<a name="0x1_Version"></a>

# Module `0x1::Version`

### Table of Contents

-  [Struct `Version`](#0x1_Version_Version)
-  [Function `initialize`](#0x1_Version_initialize)
-  [Function `get`](#0x1_Version_get)
-  [Function `set`](#0x1_Version_set)



<a name="0x1_Version_Version"></a>

## Struct `Version`



<pre><code><b>struct</b> <a href="#0x1_Version">Version</a>
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



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Version_initialize">initialize</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Version_initialize">initialize</a>(account: &signer) {
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>(), <a href="ErrorCode.md#0x1_ErrorCode_ENOT_GENESIS_ACCOUNT">ErrorCode::ENOT_GENESIS_ACCOUNT</a>());

    <a href="Config.md#0x1_Config_publish_new_config">Config::publish_new_config</a>&lt;<a href="#0x1_Version_Version">Self::Version</a>&gt;(
        account,
        <a href="#0x1_Version">Version</a> { major: 1 },
    );
}
</code></pre>



</details>

<a name="0x1_Version_get"></a>

## Function `get`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Version_get">get</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Version_get">get</a>():u64{
    <b>let</b> version = <a href="Config.md#0x1_Config_get_by_address">Config::get_by_address</a>&lt;<a href="#0x1_Version_Version">Self::Version</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
    version.major
}
</code></pre>



</details>

<a name="0x1_Version_set"></a>

## Function `set`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Version_set">set</a>(account: &signer, major: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Version_set">set</a>(account: &signer, major: u64) {
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>(), <a href="ErrorCode.md#0x1_ErrorCode_ENOT_GENESIS_ACCOUNT">ErrorCode::ENOT_GENESIS_ACCOUNT</a>());
    <b>let</b> old_config = <a href="Config.md#0x1_Config_get">Config::get</a>&lt;<a href="#0x1_Version_Version">Self::Version</a>&gt;(account);

    <b>assert</b>(
        old_config.major &lt; major,
        25
    );

    <a href="Config.md#0x1_Config_set">Config::set</a>&lt;<a href="#0x1_Version_Version">Self::Version</a>&gt;(
        account,
        <a href="#0x1_Version">Version</a> { major }
    );
}
</code></pre>



</details>
