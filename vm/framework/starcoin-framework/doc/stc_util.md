
<a id="0x1_stc_util"></a>

# Module `0x1::stc_util`



-  [Function `is_stc`](#0x1_stc_util_is_stc)


<pre><code><b>use</b> <a href="../../move-stdlib/doc/string.md#0x1_string">0x1::string</a>;
<b>use</b> <a href="../../starcoin-stdlib/doc/type_info.md#0x1_type_info">0x1::type_info</a>;
</code></pre>



<a id="0x1_stc_util_is_stc"></a>

## Function `is_stc`



<pre><code>#[view]
<b>public</b> <b>fun</b> <a href="stc_util.md#0x1_stc_util_is_stc">is_stc</a>&lt;Coin&gt;(): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_util.md#0x1_stc_util_is_stc">is_stc</a>&lt;Coin&gt;(): bool {
    <a href="../../starcoin-stdlib/doc/type_info.md#0x1_type_info_type_name">type_info::type_name</a>&lt;Coin&gt;() == <a href="../../move-stdlib/doc/string.md#0x1_string_utf8">string::utf8</a>(b"<a href="starcoin_coin.md#0x1_starcoin_coin_STC">0x1::starcoin_coin::STC</a>")
}
</code></pre>



</details>


[move-book]: https://starcoin.dev/move/book/SUMMARY
