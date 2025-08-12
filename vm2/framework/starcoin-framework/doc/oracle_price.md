
<a id="0x1_oracle_price"></a>

# Module `0x1::oracle_price`



-  [Struct `PriceOracleInfo`](#0x1_oracle_price_PriceOracleInfo)
-  [Function `register_oracle_entry`](#0x1_oracle_price_register_oracle_entry)
-  [Function `register_oracle`](#0x1_oracle_price_register_oracle)
-  [Function `init_data_source_entry`](#0x1_oracle_price_init_data_source_entry)
-  [Function `init_data_source`](#0x1_oracle_price_init_data_source)
-  [Function `is_data_source_initialized`](#0x1_oracle_price_is_data_source_initialized)
-  [Function `get_scaling_factor`](#0x1_oracle_price_get_scaling_factor)
-  [Function `update_entry`](#0x1_oracle_price_update_entry)
-  [Function `update`](#0x1_oracle_price_update)
-  [Function `update_with_cap`](#0x1_oracle_price_update_with_cap)
-  [Function `read`](#0x1_oracle_price_read)
-  [Function `read_record`](#0x1_oracle_price_read_record)
-  [Function `read_records`](#0x1_oracle_price_read_records)


<pre><code><b>use</b> <a href="../../starcoin-stdlib/doc/math128.md#0x1_math128">0x1::math128</a>;
<b>use</b> <a href="oracle.md#0x1_oracle">0x1::oracle</a>;
</code></pre>



<a id="0x1_oracle_price_PriceOracleInfo"></a>

## Struct `PriceOracleInfo`



<pre><code><b>struct</b> <a href="oracle_price.md#0x1_oracle_price_PriceOracleInfo">PriceOracleInfo</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>scaling_factor: u128</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="0x1_oracle_price_register_oracle_entry"></a>

## Function `register_oracle_entry`



<pre><code><b>public</b> entry <b>fun</b> <a href="oracle_price.md#0x1_oracle_price_register_oracle_entry">register_oracle_entry</a>&lt;OracleT: <b>copy</b>, drop, store&gt;(sender: <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, precision: u8)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="oracle_price.md#0x1_oracle_price_register_oracle_entry">register_oracle_entry</a>&lt;OracleT: <b>copy</b>+store+drop&gt;(sender: <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, precision: u8) {
    <a href="oracle_price.md#0x1_oracle_price_register_oracle">register_oracle</a>&lt;OracleT&gt;(&sender, precision);
}
</code></pre>



</details>

<a id="0x1_oracle_price_register_oracle"></a>

## Function `register_oracle`



<pre><code><b>public</b> <b>fun</b> <a href="oracle_price.md#0x1_oracle_price_register_oracle">register_oracle</a>&lt;OracleT: <b>copy</b>, drop, store&gt;(sender: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, precision: u8)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="oracle_price.md#0x1_oracle_price_register_oracle">register_oracle</a>&lt;OracleT: <b>copy</b> + store + drop&gt;(sender: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, precision: u8) {
    <b>let</b> scaling_factor = <a href="../../starcoin-stdlib/doc/math128.md#0x1_math128_pow">math128::pow</a>(10, (precision <b>as</b> u128));
    <a href="oracle.md#0x1_oracle_register_oracle">oracle::register_oracle</a>&lt;OracleT, <a href="oracle_price.md#0x1_oracle_price_PriceOracleInfo">PriceOracleInfo</a>&gt;(sender, <a href="oracle_price.md#0x1_oracle_price_PriceOracleInfo">PriceOracleInfo</a> {
        scaling_factor,
    });
}
</code></pre>



</details>

<a id="0x1_oracle_price_init_data_source_entry"></a>

## Function `init_data_source_entry`



<pre><code><b>public</b> entry <b>fun</b> <a href="oracle_price.md#0x1_oracle_price_init_data_source_entry">init_data_source_entry</a>&lt;OracleT: <b>copy</b>, drop, store&gt;(sender: <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, init_value: u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="oracle_price.md#0x1_oracle_price_init_data_source_entry">init_data_source_entry</a>&lt;OracleT: <b>copy</b>+store+drop&gt;(sender: <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, init_value: u128) {
    <a href="oracle_price.md#0x1_oracle_price_init_data_source">init_data_source</a>&lt;OracleT&gt;(&sender, init_value);
}
</code></pre>



</details>

<a id="0x1_oracle_price_init_data_source"></a>

## Function `init_data_source`



<pre><code><b>public</b> <b>fun</b> <a href="oracle_price.md#0x1_oracle_price_init_data_source">init_data_source</a>&lt;OracleT: <b>copy</b>, drop, store&gt;(sender: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, init_value: u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="oracle_price.md#0x1_oracle_price_init_data_source">init_data_source</a>&lt;OracleT: <b>copy</b>+store+drop&gt;(sender: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, init_value: u128) {
    <a href="oracle.md#0x1_oracle_init_data_source">oracle::init_data_source</a>&lt;OracleT, <a href="oracle_price.md#0x1_oracle_price_PriceOracleInfo">PriceOracleInfo</a>, u128&gt;(sender, init_value);
}
</code></pre>



</details>

<a id="0x1_oracle_price_is_data_source_initialized"></a>

## Function `is_data_source_initialized`



<pre><code><b>public</b> <b>fun</b> <a href="oracle_price.md#0x1_oracle_price_is_data_source_initialized">is_data_source_initialized</a>&lt;OracleT: <b>copy</b>, drop, store&gt;(ds_addr: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="oracle_price.md#0x1_oracle_price_is_data_source_initialized">is_data_source_initialized</a>&lt;OracleT: <b>copy</b>+store+drop&gt;(ds_addr: <b>address</b>): bool {
    <a href="oracle.md#0x1_oracle_is_data_source_initialized">oracle::is_data_source_initialized</a>&lt;OracleT, u128&gt;(ds_addr)
}
</code></pre>



</details>

<a id="0x1_oracle_price_get_scaling_factor"></a>

## Function `get_scaling_factor`



<pre><code><b>public</b> <b>fun</b> <a href="oracle_price.md#0x1_oracle_price_get_scaling_factor">get_scaling_factor</a>&lt;OracleT: <b>copy</b>, drop, store&gt;(): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="oracle_price.md#0x1_oracle_price_get_scaling_factor">get_scaling_factor</a>&lt;OracleT: <b>copy</b> + store + drop&gt;(): u128 {
    <b>let</b> info = <a href="oracle.md#0x1_oracle_get_oracle_info">oracle::get_oracle_info</a>&lt;OracleT, <a href="oracle_price.md#0x1_oracle_price_PriceOracleInfo">PriceOracleInfo</a>&gt;();
    info.scaling_factor
}
</code></pre>



</details>

<a id="0x1_oracle_price_update_entry"></a>

## Function `update_entry`



<pre><code><b>public</b> entry <b>fun</b> <a href="oracle_price.md#0x1_oracle_price_update_entry">update_entry</a>&lt;OracleT: <b>copy</b>, drop, store&gt;(sender: <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, value: u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="oracle_price.md#0x1_oracle_price_update_entry">update_entry</a>&lt;OracleT: <b>copy</b>+store+drop&gt;(sender: <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, value: u128) {
    <b>update</b>&lt;OracleT&gt;(&sender, value);
}
</code></pre>



</details>

<a id="0x1_oracle_price_update"></a>

## Function `update`



<pre><code><b>public</b> <b>fun</b> <b>update</b>&lt;OracleT: <b>copy</b>, drop, store&gt;(sender: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, value: u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <b>update</b>&lt;OracleT: <b>copy</b>+store+drop&gt;(sender: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, value: u128) {
    <a href="oracle.md#0x1_oracle_update">oracle::update</a>&lt;OracleT, u128&gt;(sender, value);
}
</code></pre>



</details>

<a id="0x1_oracle_price_update_with_cap"></a>

## Function `update_with_cap`



<pre><code><b>public</b> <b>fun</b> <a href="oracle_price.md#0x1_oracle_price_update_with_cap">update_with_cap</a>&lt;OracleT: <b>copy</b>, drop, store&gt;(cap: &<b>mut</b> <a href="oracle.md#0x1_oracle_UpdateCapability">oracle::UpdateCapability</a>&lt;OracleT&gt;, value: u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="oracle_price.md#0x1_oracle_price_update_with_cap">update_with_cap</a>&lt;OracleT: <b>copy</b>+store+drop&gt;(cap: &<b>mut</b> UpdateCapability&lt;OracleT&gt;, value: u128) {
    <a href="oracle.md#0x1_oracle_update_with_cap">oracle::update_with_cap</a>&lt;OracleT, u128&gt;(cap, value);
}
</code></pre>



</details>

<a id="0x1_oracle_price_read"></a>

## Function `read`



<pre><code><b>public</b> <b>fun</b> <a href="oracle_price.md#0x1_oracle_price_read">read</a>&lt;OracleT: <b>copy</b>, drop, store&gt;(addr: <b>address</b>): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="oracle_price.md#0x1_oracle_price_read">read</a>&lt;OracleT: <b>copy</b>+store+drop&gt;(addr: <b>address</b>): u128 {
    <a href="oracle.md#0x1_oracle_read">oracle::read</a>&lt;OracleT, u128&gt;(addr)
}
</code></pre>



</details>

<a id="0x1_oracle_price_read_record"></a>

## Function `read_record`



<pre><code><b>public</b> <b>fun</b> <a href="oracle_price.md#0x1_oracle_price_read_record">read_record</a>&lt;OracleT: <b>copy</b>, drop, store&gt;(addr: <b>address</b>): <a href="oracle.md#0x1_oracle_DataRecord">oracle::DataRecord</a>&lt;u128&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="oracle_price.md#0x1_oracle_price_read_record">read_record</a>&lt;OracleT: <b>copy</b>+store+drop&gt;(addr: <b>address</b>): DataRecord&lt;u128&gt; {
    <a href="oracle.md#0x1_oracle_read_record">oracle::read_record</a>&lt;OracleT, u128&gt;(addr)
}
</code></pre>



</details>

<a id="0x1_oracle_price_read_records"></a>

## Function `read_records`



<pre><code><b>public</b> <b>fun</b> <a href="oracle_price.md#0x1_oracle_price_read_records">read_records</a>&lt;OracleT: <b>copy</b>, drop, store&gt;(addrs: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<b>address</b>&gt;): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="oracle.md#0x1_oracle_DataRecord">oracle::DataRecord</a>&lt;u128&gt;&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="oracle_price.md#0x1_oracle_price_read_records">read_records</a>&lt;OracleT: <b>copy</b>+store+drop&gt;(addrs: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<b>address</b>&gt;): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;DataRecord&lt;u128&gt;&gt; {
    <a href="oracle.md#0x1_oracle_read_records">oracle::read_records</a>&lt;OracleT, u128&gt;(addrs)
}
</code></pre>



</details>


[move-book]: https://starcoin.dev/move/book/SUMMARY
