
<a id="0x1_oracle_price_script"></a>

# Module `0x1::oracle_price_script`



-  [Function `register_oracle`](#0x1_oracle_price_script_register_oracle)
-  [Function `init_data_source`](#0x1_oracle_price_script_init_data_source)
-  [Function `update`](#0x1_oracle_price_script_update)


<pre><code><b>use</b> <a href="oracle_price.md#0x1_oracle_price">0x1::oracle_price</a>;
</code></pre>



<a id="0x1_oracle_price_script_register_oracle"></a>

## Function `register_oracle`



<pre><code><b>public</b> entry <b>fun</b> <a href="oracle_price_script.md#0x1_oracle_price_script_register_oracle">register_oracle</a>&lt;OracleT: <b>copy</b>, drop, store&gt;(sender: <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, precision: u8)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="oracle_price_script.md#0x1_oracle_price_script_register_oracle">register_oracle</a>&lt;OracleT: <b>copy</b>+store+drop&gt;(sender: <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, precision: u8) {
    <a href="oracle_price.md#0x1_oracle_price_register_oracle_entry">oracle_price::register_oracle_entry</a>&lt;OracleT&gt;(sender, precision);
}
</code></pre>



</details>

<a id="0x1_oracle_price_script_init_data_source"></a>

## Function `init_data_source`



<pre><code><b>public</b> entry <b>fun</b> <a href="oracle_price_script.md#0x1_oracle_price_script_init_data_source">init_data_source</a>&lt;OracleT: <b>copy</b>, drop, store&gt;(sender: <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, init_value: u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="oracle_price_script.md#0x1_oracle_price_script_init_data_source">init_data_source</a>&lt;OracleT: <b>copy</b>+store+drop&gt;(sender: <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, init_value: u128) {
    <a href="oracle_price.md#0x1_oracle_price_init_data_source_entry">oracle_price::init_data_source_entry</a>&lt;OracleT&gt;(sender, init_value);
}
</code></pre>



</details>

<a id="0x1_oracle_price_script_update"></a>

## Function `update`



<pre><code><b>public</b> entry <b>fun</b> <b>update</b>&lt;OracleT: <b>copy</b>, drop, store&gt;(sender: <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, value: u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <b>update</b>&lt;OracleT: <b>copy</b>+store+drop&gt;(sender: <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, value: u128) {
    <a href="oracle_price.md#0x1_oracle_price_update_entry">oracle_price::update_entry</a>&lt;OracleT&gt;(sender, value);
}
</code></pre>



</details>


[move-book]: https://starcoin.dev/move/book/SUMMARY
