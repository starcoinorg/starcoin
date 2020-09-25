
<a name="0x1_TokenBox"></a>

# Module `0x1::TokenBox`

### Table of Contents

-  [Resource `TokenBox`](#0x1_TokenBox_TokenBox)
-  [Function `create`](#0x1_TokenBox_create)
-  [Function `withdraw`](#0x1_TokenBox_withdraw)
-  [Function `split`](#0x1_TokenBox_split)
-  [Function `unlocked_value_of`](#0x1_TokenBox_unlocked_value_of)
-  [Function `destroy_empty`](#0x1_TokenBox_destroy_empty)
-  [Function `save`](#0x1_TokenBox_save)
-  [Function `take`](#0x1_TokenBox_take)
-  [Function `exists_at`](#0x1_TokenBox_exists_at)



<a name="0x1_TokenBox_TokenBox"></a>

## Resource `TokenBox`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_TokenBox">TokenBox</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>origin: u128</code>
</dt>
<dd>

</dd>
<dt>
<code>token: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>lock_time: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>lock_peroid: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_TokenBox_create"></a>

## Function `create`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TokenBox_create">create</a>&lt;TokenType&gt;(token: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;, lock_peroid: u64): <a href="#0x1_TokenBox_TokenBox">TokenBox::TokenBox</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TokenBox_create">create</a>&lt;TokenType&gt;(token: <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt;, lock_peroid: u64): <a href="#0x1_TokenBox">TokenBox</a>&lt;TokenType&gt; {
    <b>let</b> lock_time = <a href="Timestamp.md#0x1_Timestamp_now_seconds">Timestamp::now_seconds</a>();
    <b>let</b> origin = <a href="Token.md#0x1_Token_share">Token::share</a>(&token);
    <a href="#0x1_TokenBox">TokenBox</a>&lt;TokenType&gt; {
        origin,
        token,
        lock_time,
        lock_peroid
    }
}
</code></pre>



</details>

<a name="0x1_TokenBox_withdraw"></a>

## Function `withdraw`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TokenBox_withdraw">withdraw</a>&lt;TokenType&gt;(token_box: &<b>mut</b> <a href="#0x1_TokenBox_TokenBox">TokenBox::TokenBox</a>&lt;TokenType&gt;): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TokenBox_withdraw">withdraw</a>&lt;TokenType&gt;(token_box: &<b>mut</b> <a href="#0x1_TokenBox">TokenBox</a>&lt;TokenType&gt;): <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt; {
    <b>let</b> value = <a href="#0x1_TokenBox_unlocked_value_of">unlocked_value_of</a>(token_box);
    <a href="Token.md#0x1_Token_withdraw_share">Token::withdraw_share</a>(&<b>mut</b> token_box.token, value)
}
</code></pre>



</details>

<a name="0x1_TokenBox_split"></a>

## Function `split`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TokenBox_split">split</a>&lt;TokenType&gt;(token_box: <a href="#0x1_TokenBox_TokenBox">TokenBox::TokenBox</a>&lt;TokenType&gt;, amount: u128): (<a href="#0x1_TokenBox_TokenBox">TokenBox::TokenBox</a>&lt;TokenType&gt;, <a href="#0x1_TokenBox_TokenBox">TokenBox::TokenBox</a>&lt;TokenType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TokenBox_split">split</a>&lt;TokenType&gt;(token_box: <a href="#0x1_TokenBox">TokenBox</a>&lt;TokenType&gt;, amount: u128): (<a href="#0x1_TokenBox">TokenBox</a>&lt;TokenType&gt;, <a href="#0x1_TokenBox">TokenBox</a>&lt;TokenType&gt;) {
    <b>let</b> <a href="#0x1_TokenBox">TokenBox</a>&lt;TokenType&gt; { origin, token, lock_time, lock_peroid } = token_box;
    <b>let</b> (t1, t2) = <a href="Token.md#0x1_Token_split_share">Token::split_share</a>(token, amount);
    (<a href="#0x1_TokenBox">TokenBox</a>&lt;TokenType&gt; { origin, token: t1, lock_time, lock_peroid }, <a href="#0x1_TokenBox">TokenBox</a>&lt;TokenType&gt; { origin, token: t2, lock_time, lock_peroid })
}
</code></pre>



</details>

<a name="0x1_TokenBox_unlocked_value_of"></a>

## Function `unlocked_value_of`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TokenBox_unlocked_value_of">unlocked_value_of</a>&lt;TokenType&gt;(token_box: &<a href="#0x1_TokenBox_TokenBox">TokenBox::TokenBox</a>&lt;TokenType&gt;): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TokenBox_unlocked_value_of">unlocked_value_of</a>&lt;TokenType&gt;(token_box: &<a href="#0x1_TokenBox">TokenBox</a>&lt;TokenType&gt;): u128 {
    <b>let</b> now = <a href="Timestamp.md#0x1_Timestamp_now_seconds">Timestamp::now_seconds</a>();
    <b>let</b> elapsed_time = now - token_box.lock_time;
    <b>if</b> (elapsed_time &gt;= token_box.lock_peroid) {
        <b>return</b> <a href="Token.md#0x1_Token_share">Token::share</a>(&token_box.token)
    }<b>else</b> {
        token_box.origin * (elapsed_time <b>as</b> u128) / (token_box.lock_peroid <b>as</b> u128)
    }
}
</code></pre>



</details>

<a name="0x1_TokenBox_destroy_empty"></a>

## Function `destroy_empty`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TokenBox_destroy_empty">destroy_empty</a>&lt;TokenType&gt;(token_box: <a href="#0x1_TokenBox_TokenBox">TokenBox::TokenBox</a>&lt;TokenType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TokenBox_destroy_empty">destroy_empty</a>&lt;TokenType&gt;(token_box: <a href="#0x1_TokenBox">TokenBox</a>&lt;TokenType&gt;) {
    <b>let</b> <a href="#0x1_TokenBox">TokenBox</a>&lt;TokenType&gt; { origin: _, token, lock_time: _, lock_peroid: _ } = token_box;
    <a href="Token.md#0x1_Token_destroy_zero">Token::destroy_zero</a>(token);
}
</code></pre>



</details>

<a name="0x1_TokenBox_save"></a>

## Function `save`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TokenBox_save">save</a>&lt;TokenType&gt;(account: &signer, token_box: <a href="#0x1_TokenBox_TokenBox">TokenBox::TokenBox</a>&lt;TokenType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TokenBox_save">save</a>&lt;TokenType&gt;(account: &signer, token_box: <a href="#0x1_TokenBox">TokenBox</a>&lt;TokenType&gt;) {
    move_to(account, token_box);
}
</code></pre>



</details>

<a name="0x1_TokenBox_take"></a>

## Function `take`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TokenBox_take">take</a>&lt;TokenType&gt;(account: &signer): <a href="#0x1_TokenBox_TokenBox">TokenBox::TokenBox</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TokenBox_take">take</a>&lt;TokenType&gt;(account: &signer): <a href="#0x1_TokenBox">TokenBox</a>&lt;TokenType&gt; <b>acquires</b> <a href="#0x1_TokenBox">TokenBox</a> {
    move_from&lt;<a href="#0x1_TokenBox">TokenBox</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account))
}
</code></pre>



</details>

<a name="0x1_TokenBox_exists_at"></a>

## Function `exists_at`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TokenBox_exists_at">exists_at</a>&lt;TokenType&gt;(address: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TokenBox_exists_at">exists_at</a>&lt;TokenType&gt;(address: address): bool {
    exists&lt;<a href="#0x1_TokenBox">TokenBox</a>&lt;TokenType&gt;&gt;(address)
}
</code></pre>



</details>
