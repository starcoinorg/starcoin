
<a id="0x1_stdlib_upgrade_scripts"></a>

# Module `0x1::stdlib_upgrade_scripts`

The module for stdlib upgrade init scripts


-  [Function `dummy_upgrade`](#0x1_stdlib_upgrade_scripts_dummy_upgrade)
-  [Function `do_dummy_upgrade`](#0x1_stdlib_upgrade_scripts_do_dummy_upgrade)
-  [Specification](#@Specification_0)


<pre><code><b>use</b> <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug">0x1::debug</a>;
<b>use</b> <a href="../../move-stdlib/doc/string.md#0x1_string">0x1::string</a>;
</code></pre>



<a id="0x1_stdlib_upgrade_scripts_dummy_upgrade"></a>

## Function `dummy_upgrade`



<pre><code><b>public</b> entry <b>fun</b> <a href="stdlib_upgrade_scripts.md#0x1_stdlib_upgrade_scripts_dummy_upgrade">dummy_upgrade</a>(sender: <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="stdlib_upgrade_scripts.md#0x1_stdlib_upgrade_scripts_dummy_upgrade">dummy_upgrade</a>(
    sender: <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
) {
    <a href="stdlib_upgrade_scripts.md#0x1_stdlib_upgrade_scripts_do_dummy_upgrade">do_dummy_upgrade</a>(&sender);
}
</code></pre>



</details>

<a id="0x1_stdlib_upgrade_scripts_do_dummy_upgrade"></a>

## Function `do_dummy_upgrade`



<pre><code><b>public</b> <b>fun</b> <a href="stdlib_upgrade_scripts.md#0x1_stdlib_upgrade_scripts_do_dummy_upgrade">do_dummy_upgrade</a>(_sender: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stdlib_upgrade_scripts.md#0x1_stdlib_upgrade_scripts_do_dummy_upgrade">do_dummy_upgrade</a>(
    _sender: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
) {
    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"do_dummy_upgrade"));
}
</code></pre>



</details>

<a id="@Specification_0"></a>

## Specification



<pre><code><b>pragma</b> verify = <b>false</b>;
<b>pragma</b> aborts_if_is_strict = <b>true</b>;
</code></pre>


[move-book]: https://starcoin.dev/move/book/SUMMARY
