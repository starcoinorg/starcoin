
<a id="0x1_stc_util"></a>

# Module `0x1::stc_util`



-  [Constants](#@Constants_0)
-  [Function `is_stc`](#0x1_stc_util_is_stc)
-  [Function `token_issuer`](#0x1_stc_util_token_issuer)
-  [Function `is_net_dev`](#0x1_stc_util_is_net_dev)
-  [Function `is_net_test`](#0x1_stc_util_is_net_test)
-  [Function `is_net_halley`](#0x1_stc_util_is_net_halley)
-  [Function `is_net_barnard`](#0x1_stc_util_is_net_barnard)
-  [Function `is_net_main`](#0x1_stc_util_is_net_main)
-  [Function `is_net_vega`](#0x1_stc_util_is_net_vega)


<pre><code><b>use</b> <a href="chain_id.md#0x1_chain_id">0x1::chain_id</a>;
<b>use</b> <a href="../../move-stdlib/doc/string.md#0x1_string">0x1::string</a>;
<b>use</b> <a href="../../starcoin-stdlib/doc/type_info.md#0x1_type_info">0x1::type_info</a>;
</code></pre>



<a id="@Constants_0"></a>

## Constants


<a id="0x1_stc_util_CHAIN_ID_BARNARD"></a>



<pre><code><b>const</b> <a href="stc_util.md#0x1_stc_util_CHAIN_ID_BARNARD">CHAIN_ID_BARNARD</a>: u8 = 251;
</code></pre>



<a id="0x1_stc_util_CHAIN_ID_DEV"></a>



<pre><code><b>const</b> <a href="stc_util.md#0x1_stc_util_CHAIN_ID_DEV">CHAIN_ID_DEV</a>: u8 = 254;
</code></pre>



<a id="0x1_stc_util_CHAIN_ID_HALLEY"></a>



<pre><code><b>const</b> <a href="stc_util.md#0x1_stc_util_CHAIN_ID_HALLEY">CHAIN_ID_HALLEY</a>: u8 = 253;
</code></pre>



<a id="0x1_stc_util_CHAIN_ID_MAIN"></a>



<pre><code><b>const</b> <a href="stc_util.md#0x1_stc_util_CHAIN_ID_MAIN">CHAIN_ID_MAIN</a>: u8 = 1;
</code></pre>



<a id="0x1_stc_util_CHAIN_ID_PROXIMA"></a>



<pre><code><b>const</b> <a href="stc_util.md#0x1_stc_util_CHAIN_ID_PROXIMA">CHAIN_ID_PROXIMA</a>: u8 = 252;
</code></pre>



<a id="0x1_stc_util_CHAIN_ID_TEST"></a>



<pre><code><b>const</b> <a href="stc_util.md#0x1_stc_util_CHAIN_ID_TEST">CHAIN_ID_TEST</a>: u8 = 255;
</code></pre>



<a id="0x1_stc_util_CHAIN_ID_VEGA"></a>



<pre><code><b>const</b> <a href="stc_util.md#0x1_stc_util_CHAIN_ID_VEGA">CHAIN_ID_VEGA</a>: u8 = 2;
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

<a id="0x1_stc_util_token_issuer"></a>

## Function `token_issuer`



<pre><code>#[view]
<b>public</b> <b>fun</b> <a href="stc_util.md#0x1_stc_util_token_issuer">token_issuer</a>&lt;Coin&gt;(): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_util.md#0x1_stc_util_token_issuer">token_issuer</a>&lt;Coin&gt;(): <b>address</b> {
    <a href="../../starcoin-stdlib/doc/type_info.md#0x1_type_info_account_address">type_info::account_address</a>(&<a href="../../starcoin-stdlib/doc/type_info.md#0x1_type_info_type_of">type_info::type_of</a>&lt;Coin&gt;())
}
</code></pre>



</details>

<a id="0x1_stc_util_is_net_dev"></a>

## Function `is_net_dev`



<pre><code>#[view]
<b>public</b> <b>fun</b> <a href="stc_util.md#0x1_stc_util_is_net_dev">is_net_dev</a>(): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_util.md#0x1_stc_util_is_net_dev">is_net_dev</a>(): bool {
    <a href="chain_id.md#0x1_chain_id_get">chain_id::get</a>() == <a href="stc_util.md#0x1_stc_util_CHAIN_ID_DEV">CHAIN_ID_DEV</a>
}
</code></pre>



</details>

<a id="0x1_stc_util_is_net_test"></a>

## Function `is_net_test`



<pre><code>#[view]
<b>public</b> <b>fun</b> <a href="stc_util.md#0x1_stc_util_is_net_test">is_net_test</a>(): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_util.md#0x1_stc_util_is_net_test">is_net_test</a>(): bool {
    <a href="chain_id.md#0x1_chain_id_get">chain_id::get</a>() == <a href="stc_util.md#0x1_stc_util_CHAIN_ID_TEST">CHAIN_ID_TEST</a>
}
</code></pre>



</details>

<a id="0x1_stc_util_is_net_halley"></a>

## Function `is_net_halley`



<pre><code>#[view]
<b>public</b> <b>fun</b> <a href="stc_util.md#0x1_stc_util_is_net_halley">is_net_halley</a>(): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_util.md#0x1_stc_util_is_net_halley">is_net_halley</a>(): bool {
    <a href="chain_id.md#0x1_chain_id_get">chain_id::get</a>() == <a href="stc_util.md#0x1_stc_util_CHAIN_ID_HALLEY">CHAIN_ID_HALLEY</a>
}
</code></pre>



</details>

<a id="0x1_stc_util_is_net_barnard"></a>

## Function `is_net_barnard`



<pre><code>#[view]
<b>public</b> <b>fun</b> <a href="stc_util.md#0x1_stc_util_is_net_barnard">is_net_barnard</a>(): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_util.md#0x1_stc_util_is_net_barnard">is_net_barnard</a>(): bool {
    <a href="chain_id.md#0x1_chain_id_get">chain_id::get</a>() == <a href="stc_util.md#0x1_stc_util_CHAIN_ID_BARNARD">CHAIN_ID_BARNARD</a>
}
</code></pre>



</details>

<a id="0x1_stc_util_is_net_main"></a>

## Function `is_net_main`



<pre><code>#[view]
<b>public</b> <b>fun</b> <a href="stc_util.md#0x1_stc_util_is_net_main">is_net_main</a>(): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_util.md#0x1_stc_util_is_net_main">is_net_main</a>(): bool {
    <a href="chain_id.md#0x1_chain_id_get">chain_id::get</a>() == <a href="stc_util.md#0x1_stc_util_CHAIN_ID_MAIN">CHAIN_ID_MAIN</a>
}
</code></pre>



</details>

<a id="0x1_stc_util_is_net_vega"></a>

## Function `is_net_vega`



<pre><code>#[view]
<b>public</b> <b>fun</b> <a href="stc_util.md#0x1_stc_util_is_net_vega">is_net_vega</a>(): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_util.md#0x1_stc_util_is_net_vega">is_net_vega</a>(): bool {
    <a href="chain_id.md#0x1_chain_id_get">chain_id::get</a>() == <a href="stc_util.md#0x1_stc_util_CHAIN_ID_VEGA">CHAIN_ID_VEGA</a>
}
</code></pre>



</details>


[move-book]: https://starcoin.dev/move/book/SUMMARY
