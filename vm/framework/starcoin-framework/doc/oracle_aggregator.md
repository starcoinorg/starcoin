
<a id="0x1_oracle_aggregator"></a>

# Module `0x1::oracle_aggregator`



-  [Constants](#@Constants_0)
-  [Function `latest_price_average_aggregator`](#0x1_oracle_aggregator_latest_price_average_aggregator)


<pre><code><b>use</b> <a href="../../move-stdlib/doc/error.md#0x1_error">0x1::error</a>;
<b>use</b> <a href="../../starcoin-stdlib/doc/math_fixed.md#0x1_math_fixed">0x1::math_fixed</a>;
<b>use</b> <a href="oracle.md#0x1_oracle">0x1::oracle</a>;
<b>use</b> <a href="oracle_price.md#0x1_oracle_price">0x1::oracle_price</a>;
<b>use</b> <a href="timestamp.md#0x1_timestamp">0x1::timestamp</a>;
<b>use</b> <a href="../../move-stdlib/doc/vector.md#0x1_vector">0x1::vector</a>;
</code></pre>



<a id="@Constants_0"></a>

## Constants


<a id="0x1_oracle_aggregator_ERR_NO_PRICE_DATA_AVIABLE"></a>

No price data match requirement condition.


<pre><code><b>const</b> <a href="oracle_aggregator.md#0x1_oracle_aggregator_ERR_NO_PRICE_DATA_AVIABLE">ERR_NO_PRICE_DATA_AVIABLE</a>: u64 = 101;
</code></pre>



<a id="0x1_oracle_aggregator_latest_price_average_aggregator"></a>

## Function `latest_price_average_aggregator`

Get latest price from datasources and calculate avg.
<code>addrs</code>: the datasource's addr, <code>updated_in</code>: the datasource should updated in x millseoconds.


<pre><code><b>public</b> <b>fun</b> <a href="oracle_aggregator.md#0x1_oracle_aggregator_latest_price_average_aggregator">latest_price_average_aggregator</a>&lt;OracleT: <b>copy</b>, drop, store&gt;(addrs: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<b>address</b>&gt;, updated_in: u64): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="oracle_aggregator.md#0x1_oracle_aggregator_latest_price_average_aggregator">latest_price_average_aggregator</a>&lt;OracleT: <b>copy</b>+store+drop&gt;(
    addrs: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<b>address</b>&gt;,
    updated_in: u64
): u128 {
    <b>let</b> len = <a href="../../move-stdlib/doc/vector.md#0x1_vector_length">vector::length</a>(addrs);
    <b>let</b> price_records = <a href="oracle_price.md#0x1_oracle_price_read_records">oracle_price::read_records</a>&lt;OracleT&gt;(addrs);
    <b>let</b> prices = <a href="../../move-stdlib/doc/vector.md#0x1_vector_empty">vector::empty</a>();
    <b>let</b> i = 0;
    <b>let</b> expect_updated_after = <a href="timestamp.md#0x1_timestamp_now_milliseconds">timestamp::now_milliseconds</a>() - updated_in;
    <b>while</b> (i &lt; len) {
        <b>let</b> record = <a href="../../move-stdlib/doc/vector.md#0x1_vector_pop_back">vector::pop_back</a>(&<b>mut</b> price_records);
        <b>let</b> (_version, price, updated_at) = <a href="oracle.md#0x1_oracle_unpack_record">oracle::unpack_record</a>(record);
        <b>if</b> (updated_at &gt;= expect_updated_after) {
            <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> prices, price);
        };
        i = i + 1;
    };
    // <b>if</b> all price data not match the update_in filter, <b>abort</b>.
    <b>assert</b>!(!<a href="../../move-stdlib/doc/vector.md#0x1_vector_is_empty">vector::is_empty</a>(&prices), <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_state">error::invalid_state</a>(<a href="oracle_aggregator.md#0x1_oracle_aggregator_ERR_NO_PRICE_DATA_AVIABLE">ERR_NO_PRICE_DATA_AVIABLE</a>));
    <a href="../../starcoin-stdlib/doc/math_fixed.md#0x1_math_fixed_avg">math_fixed::avg</a>(&prices)
}
</code></pre>



</details>


[move-book]: https://starcoin.dev/move/book/SUMMARY
