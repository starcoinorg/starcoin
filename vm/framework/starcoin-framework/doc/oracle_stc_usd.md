
<a id="0x1_oracle_stc_usd"></a>

# Module `0x1::oracle_stc_usd`



-  [Struct `STCUSD`](#0x1_oracle_stc_usd_STCUSD)
-  [Function `register`](#0x1_oracle_stc_usd_register)
-  [Function `read`](#0x1_oracle_stc_usd_read)
-  [Function `read_record`](#0x1_oracle_stc_usd_read_record)
-  [Function `read_records`](#0x1_oracle_stc_usd_read_records)


<pre><code><b>use</b> <a href="oracle.md#0x1_oracle">0x1::oracle</a>;
<b>use</b> <a href="oracle_price.md#0x1_oracle_price">0x1::oracle_price</a>;
</code></pre>



<a id="0x1_oracle_stc_usd_STCUSD"></a>

## Struct `STCUSD`

The STC to USD price oracle


<pre><code><b>struct</b> <a href="oracle_stc_usd.md#0x1_oracle_stc_usd_STCUSD">STCUSD</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>dummy_field: bool</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="0x1_oracle_stc_usd_register"></a>

## Function `register`



<pre><code><b>public</b> <b>fun</b> <a href="oracle_stc_usd.md#0x1_oracle_stc_usd_register">register</a>(sender: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="oracle_stc_usd.md#0x1_oracle_stc_usd_register">register</a>(sender: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>) {
    <a href="oracle_price.md#0x1_oracle_price_register_oracle">oracle_price::register_oracle</a>&lt;<a href="oracle_stc_usd.md#0x1_oracle_stc_usd_STCUSD">STCUSD</a>&gt;(sender, 6);
}
</code></pre>



</details>

<a id="0x1_oracle_stc_usd_read"></a>

## Function `read`



<pre><code><b>public</b> <b>fun</b> <a href="oracle_stc_usd.md#0x1_oracle_stc_usd_read">read</a>(ds_addr: <b>address</b>): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="oracle_stc_usd.md#0x1_oracle_stc_usd_read">read</a>(ds_addr: <b>address</b>): u128 {
    <a href="oracle_price.md#0x1_oracle_price_read">oracle_price::read</a>&lt;<a href="oracle_stc_usd.md#0x1_oracle_stc_usd_STCUSD">STCUSD</a>&gt;(ds_addr)
}
</code></pre>



</details>

<a id="0x1_oracle_stc_usd_read_record"></a>

## Function `read_record`



<pre><code><b>public</b> <b>fun</b> <a href="oracle_stc_usd.md#0x1_oracle_stc_usd_read_record">read_record</a>(ds_addr: <b>address</b>): <a href="oracle.md#0x1_oracle_DataRecord">oracle::DataRecord</a>&lt;u128&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="oracle_stc_usd.md#0x1_oracle_stc_usd_read_record">read_record</a>(ds_addr: <b>address</b>): DataRecord&lt;u128&gt; {
    <a href="oracle_price.md#0x1_oracle_price_read_record">oracle_price::read_record</a>&lt;<a href="oracle_stc_usd.md#0x1_oracle_stc_usd_STCUSD">STCUSD</a>&gt;(ds_addr)
}
</code></pre>



</details>

<a id="0x1_oracle_stc_usd_read_records"></a>

## Function `read_records`



<pre><code><b>public</b> <b>fun</b> <a href="oracle_stc_usd.md#0x1_oracle_stc_usd_read_records">read_records</a>(ds_addrs: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<b>address</b>&gt;): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="oracle.md#0x1_oracle_DataRecord">oracle::DataRecord</a>&lt;u128&gt;&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="oracle_stc_usd.md#0x1_oracle_stc_usd_read_records">read_records</a>(ds_addrs: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<b>address</b>&gt;): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;DataRecord&lt;u128&gt;&gt; {
    <a href="oracle_price.md#0x1_oracle_price_read_records">oracle_price::read_records</a>&lt;<a href="oracle_stc_usd.md#0x1_oracle_stc_usd_STCUSD">STCUSD</a>&gt;(ds_addrs)
}
</code></pre>



</details>


[move-book]: https://starcoin.dev/move/book/SUMMARY
