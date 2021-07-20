
<a name="0x1_OracleAggregator"></a>

# Module `0x1::OracleAggregator`



-  [Constants](#@Constants_0)
-  [Function `latest_price_average_aggregator`](#0x1_OracleAggregator_latest_price_average_aggregator)


<pre><code><b>use</b> <a href="Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="Math.md#0x1_Math">0x1::Math</a>;
<b>use</b> <a href="Oracle.md#0x1_Oracle">0x1::Oracle</a>;
<b>use</b> <a href="Timestamp.md#0x1_Timestamp">0x1::Timestamp</a>;
<b>use</b> <a href="Vector.md#0x1_Vector">0x1::Vector</a>;
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x1_OracleAggregator_ERR_NO_PRICE_DATA_AVIABLE"></a>

No price data match requirement condition.


<pre><code><b>const</b> <a href="Oracle.md#0x1_OracleAggregator_ERR_NO_PRICE_DATA_AVIABLE">ERR_NO_PRICE_DATA_AVIABLE</a>: u64 = 101;
</code></pre>



<a name="0x1_OracleAggregator_latest_price_average_aggregator"></a>

## Function `latest_price_average_aggregator`

Get latest price from datasources and calculate avg.
<code>addrs</code> the datasource's addr, <code>updated_in</code> the datasource should updated in x millseoconds.


<pre><code><b>public</b> <b>fun</b> <a href="Oracle.md#0x1_OracleAggregator_latest_price_average_aggregator">latest_price_average_aggregator</a>&lt;DataT: <b>copy</b>, drop, store&gt;(addrs: &vector&lt;address&gt;, updated_in: u64): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Oracle.md#0x1_OracleAggregator_latest_price_average_aggregator">latest_price_average_aggregator</a>&lt;DataT: <b>copy</b>+store+drop&gt;(addrs: &vector&lt;address&gt;, updated_in: u64): u128 {
    <b>let</b> len = <a href="Vector.md#0x1_Vector_length">Vector::length</a>(addrs);
    <b>let</b> price_data_vec = <a href="Oracle.md#0x1_Oracle_read_price_data_batch">Oracle::read_price_data_batch</a>&lt;DataT&gt;(addrs);
    <b>let</b> prices = <a href="Vector.md#0x1_Vector_empty">Vector::empty</a>();
    <b>let</b> i = 0;
    <b>let</b> expect_updated_after = <a href="Timestamp.md#0x1_Timestamp_now_milliseconds">Timestamp::now_milliseconds</a>() - updated_in;
    <b>while</b> (i &lt; len){
        <b>let</b> data = <a href="Vector.md#0x1_Vector_pop_back">Vector::pop_back</a>(&<b>mut</b> price_data_vec);
        <b>let</b> (_version, price, updated_at) = <a href="Oracle.md#0x1_Oracle_unpack_data">Oracle::unpack_data</a>(data);
        <b>if</b> (updated_at &gt;= expect_updated_after) {
            <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> prices, price);
        };
        i = i + 1;
    };
    // <b>if</b> all price data not match the update_in filter, <b>abort</b>.
    <b>assert</b>(!<a href="Vector.md#0x1_Vector_is_empty">Vector::is_empty</a>(&prices), <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="Oracle.md#0x1_OracleAggregator_ERR_NO_PRICE_DATA_AVIABLE">ERR_NO_PRICE_DATA_AVIABLE</a>));
    <a href="Math.md#0x1_Math_avg">Math::avg</a>(&prices)
}
</code></pre>



</details>
