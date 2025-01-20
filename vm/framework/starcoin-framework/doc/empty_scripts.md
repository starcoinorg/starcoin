
<a id="0x1_empty_scripts"></a>

# Module `0x1::empty_scripts`



-  [Function `empty_script`](#0x1_empty_scripts_empty_script)
-  [Function `test_metadata`](#0x1_empty_scripts_test_metadata)
-  [Specification](#@Specification_0)


<pre><code><b>use</b> <a href="coin.md#0x1_coin">0x1::coin</a>;
<b>use</b> <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug">0x1::debug</a>;
<b>use</b> <a href="fungible_asset.md#0x1_fungible_asset">0x1::fungible_asset</a>;
<b>use</b> <a href="object.md#0x1_object">0x1::object</a>;
<b>use</b> <a href="../../move-stdlib/doc/option.md#0x1_option">0x1::option</a>;
<b>use</b> <a href="starcoin_coin.md#0x1_starcoin_coin">0x1::starcoin_coin</a>;
<b>use</b> <a href="../../move-stdlib/doc/string.md#0x1_string">0x1::string</a>;
</code></pre>



<a id="0x1_empty_scripts_empty_script"></a>

## Function `empty_script`



<pre><code><b>public</b> entry <b>fun</b> <a href="empty_scripts.md#0x1_empty_scripts_empty_script">empty_script</a>()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="empty_scripts.md#0x1_empty_scripts_empty_script">empty_script</a>() {}
</code></pre>



</details>

<a id="0x1_empty_scripts_test_metadata"></a>

## Function `test_metadata`



<pre><code><b>public</b> entry <b>fun</b> <a href="empty_scripts.md#0x1_empty_scripts_test_metadata">test_metadata</a>(_account: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="empty_scripts.md#0x1_empty_scripts_test_metadata">test_metadata</a>(_account: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>) {
    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&<a href="../../move-stdlib/doc/string.md#0x1_string_utf8">string::utf8</a>(b"test_metadata | entered"));
    <b>let</b> metadata = <a href="coin.md#0x1_coin_paired_metadata">coin::paired_metadata</a>&lt;STC&gt;();
    <b>assert</b>!(<a href="../../move-stdlib/doc/option.md#0x1_option_is_some">option::is_some</a>(&metadata), 10000);
    <b>let</b> metdata_obj = <a href="../../move-stdlib/doc/option.md#0x1_option_destroy_some">option::destroy_some</a>(metadata);
    <b>assert</b>!(<a href="object.md#0x1_object_is_object">object::is_object</a>(<a href="object.md#0x1_object_object_address">object::object_address</a>(&metdata_obj)), 10001);
    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&<a href="../../move-stdlib/doc/string.md#0x1_string_utf8">string::utf8</a>(b"test_metadata | exited"));

}
</code></pre>



</details>

<a id="@Specification_0"></a>

## Specification



<pre><code><b>pragma</b> verify = <b>false</b>;
<b>pragma</b> aborts_if_is_partial = <b>false</b>;
<b>pragma</b> aborts_if_is_strict = <b>false</b>;
</code></pre>


[move-book]: https://starcoin.dev/move/book/SUMMARY
