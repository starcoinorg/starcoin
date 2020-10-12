
<a name="0x1_TokenLockPool"></a>

# Module `0x1::TokenLockPool`



-  [Resource <code><a href="TokenLockPool.md#0x1_TokenLockPool_TokenPool">TokenPool</a></code>](#0x1_TokenLockPool_TokenPool)
-  [Resource <code><a href="TokenLockPool.md#0x1_TokenLockPool_FixedTimeLockKey">FixedTimeLockKey</a></code>](#0x1_TokenLockPool_FixedTimeLockKey)
-  [Resource <code><a href="TokenLockPool.md#0x1_TokenLockPool_LinearTimeLockKey">LinearTimeLockKey</a></code>](#0x1_TokenLockPool_LinearTimeLockKey)
-  [Function <code>EDESTROY_KEY_NOT_EMPTY</code>](#0x1_TokenLockPool_EDESTROY_KEY_NOT_EMPTY)
-  [Function <code>ETIMELOCK_NOT_UNLOCKED</code>](#0x1_TokenLockPool_ETIMELOCK_NOT_UNLOCKED)
-  [Function <code>EAMOUNT_TOO_BIG</code>](#0x1_TokenLockPool_EAMOUNT_TOO_BIG)
-  [Function <code>initialize</code>](#0x1_TokenLockPool_initialize)
-  [Function <code>create_linear_lock</code>](#0x1_TokenLockPool_create_linear_lock)
-  [Function <code>create_fixed_lock</code>](#0x1_TokenLockPool_create_fixed_lock)
-  [Function <code>unlock_with_linear_key</code>](#0x1_TokenLockPool_unlock_with_linear_key)
-  [Function <code>unlock_with_fixed_key</code>](#0x1_TokenLockPool_unlock_with_fixed_key)
-  [Function <code>unlocked_amount_of_linear_key</code>](#0x1_TokenLockPool_unlocked_amount_of_linear_key)
-  [Function <code>unlocked_amount_of_fixed_key</code>](#0x1_TokenLockPool_unlocked_amount_of_fixed_key)
-  [Function <code>end_time_of</code>](#0x1_TokenLockPool_end_time_of)
-  [Function <code>destroy_empty</code>](#0x1_TokenLockPool_destroy_empty)


<a name="0x1_TokenLockPool_TokenPool"></a>

## Resource `TokenPool`



<pre><code><b>resource</b> <b>struct</b> <a href="TokenLockPool.md#0x1_TokenLockPool_TokenPool">TokenPool</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>token: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_TokenLockPool_FixedTimeLockKey"></a>

## Resource `FixedTimeLockKey`



<pre><code><b>resource</b> <b>struct</b> <a href="TokenLockPool.md#0x1_TokenLockPool_FixedTimeLockKey">FixedTimeLockKey</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>total: u128</code>
</dt>
<dd>

</dd>
<dt>
<code>end_time: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_TokenLockPool_LinearTimeLockKey"></a>

## Resource `LinearTimeLockKey`



<pre><code><b>resource</b> <b>struct</b> <a href="TokenLockPool.md#0x1_TokenLockPool_LinearTimeLockKey">LinearTimeLockKey</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>total: u128</code>
</dt>
<dd>

</dd>
<dt>
<code>taked: u128</code>
</dt>
<dd>

</dd>
<dt>
<code>start_time: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>peroid: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_TokenLockPool_EDESTROY_KEY_NOT_EMPTY"></a>

## Function `EDESTROY_KEY_NOT_EMPTY`



<pre><code><b>fun</b> <a href="TokenLockPool.md#0x1_TokenLockPool_EDESTROY_KEY_NOT_EMPTY">EDESTROY_KEY_NOT_EMPTY</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="TokenLockPool.md#0x1_TokenLockPool_EDESTROY_KEY_NOT_EMPTY">EDESTROY_KEY_NOT_EMPTY</a>(): u64 {
    <a href="ErrorCode.md#0x1_ErrorCode_ECODE_BASE">ErrorCode::ECODE_BASE</a>() + 1
}
</code></pre>



</details>

<a name="0x1_TokenLockPool_ETIMELOCK_NOT_UNLOCKED"></a>

## Function `ETIMELOCK_NOT_UNLOCKED`



<pre><code><b>fun</b> <a href="TokenLockPool.md#0x1_TokenLockPool_ETIMELOCK_NOT_UNLOCKED">ETIMELOCK_NOT_UNLOCKED</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="TokenLockPool.md#0x1_TokenLockPool_ETIMELOCK_NOT_UNLOCKED">ETIMELOCK_NOT_UNLOCKED</a>(): u64 {
    <a href="ErrorCode.md#0x1_ErrorCode_ECODE_BASE">ErrorCode::ECODE_BASE</a>() + 2
}
</code></pre>



</details>

<a name="0x1_TokenLockPool_EAMOUNT_TOO_BIG"></a>

## Function `EAMOUNT_TOO_BIG`



<pre><code><b>fun</b> <a href="TokenLockPool.md#0x1_TokenLockPool_EAMOUNT_TOO_BIG">EAMOUNT_TOO_BIG</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="TokenLockPool.md#0x1_TokenLockPool_EAMOUNT_TOO_BIG">EAMOUNT_TOO_BIG</a>(): u64 {
    <a href="ErrorCode.md#0x1_ErrorCode_ECODE_BASE">ErrorCode::ECODE_BASE</a>() + 3
}
</code></pre>



</details>

<a name="0x1_TokenLockPool_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="TokenLockPool.md#0x1_TokenLockPool_initialize">initialize</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="TokenLockPool.md#0x1_TokenLockPool_initialize">initialize</a>(account: &signer) {
    <b>assert</b>(<a href="Timestamp.md#0x1_Timestamp_is_genesis">Timestamp::is_genesis</a>(), <a href="ErrorCode.md#0x1_ErrorCode_ENOT_GENESIS">ErrorCode::ENOT_GENESIS</a>());
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>(), <a href="ErrorCode.md#0x1_ErrorCode_ENOT_GENESIS_ACCOUNT">ErrorCode::ENOT_GENESIS_ACCOUNT</a>());
    <b>let</b> token_pool = <a href="TokenLockPool.md#0x1_TokenLockPool_TokenPool">TokenPool</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt; { token: <a href="Token.md#0x1_Token_zero">Token::zero</a>() };
    move_to(account, token_pool);
    //TODO how <b>to</b> init other token's pool.
}
</code></pre>



</details>

<a name="0x1_TokenLockPool_create_linear_lock"></a>

## Function `create_linear_lock`



<pre><code><b>public</b> <b>fun</b> <a href="TokenLockPool.md#0x1_TokenLockPool_create_linear_lock">create_linear_lock</a>&lt;TokenType&gt;(token: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;, peroid: u64): <a href="TokenLockPool.md#0x1_TokenLockPool_LinearTimeLockKey">TokenLockPool::LinearTimeLockKey</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="TokenLockPool.md#0x1_TokenLockPool_create_linear_lock">create_linear_lock</a>&lt;TokenType&gt;(token: <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt;, peroid: u64): <a href="TokenLockPool.md#0x1_TokenLockPool_LinearTimeLockKey">LinearTimeLockKey</a>&lt;TokenType&gt; <b>acquires</b> <a href="TokenLockPool.md#0x1_TokenLockPool_TokenPool">TokenPool</a> {
    <b>assert</b>(peroid &gt; 0, <a href="ErrorCode.md#0x1_ErrorCode_EINVALID_ARGUMENT">ErrorCode::EINVALID_ARGUMENT</a>());
    <b>let</b> start_time = <a href="Timestamp.md#0x1_Timestamp_now_seconds">Timestamp::now_seconds</a>();
    <b>let</b> total = <a href="Token.md#0x1_Token_value">Token::value</a>(&token);
    <b>let</b> token_pool = borrow_global_mut&lt;<a href="TokenLockPool.md#0x1_TokenLockPool_TokenPool">TokenPool</a>&lt;TokenType&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
    <a href="Token.md#0x1_Token_deposit">Token::deposit</a>(&<b>mut</b> token_pool.token, token);
    <a href="TokenLockPool.md#0x1_TokenLockPool_LinearTimeLockKey">LinearTimeLockKey</a>&lt;TokenType&gt; {
        total,
        taked: 0,
        start_time,
        peroid
    }
}
</code></pre>



</details>

<a name="0x1_TokenLockPool_create_fixed_lock"></a>

## Function `create_fixed_lock`



<pre><code><b>public</b> <b>fun</b> <a href="TokenLockPool.md#0x1_TokenLockPool_create_fixed_lock">create_fixed_lock</a>&lt;TokenType&gt;(token: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;, peroid: u64): <a href="TokenLockPool.md#0x1_TokenLockPool_FixedTimeLockKey">TokenLockPool::FixedTimeLockKey</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="TokenLockPool.md#0x1_TokenLockPool_create_fixed_lock">create_fixed_lock</a>&lt;TokenType&gt;(token: <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt;, peroid: u64): <a href="TokenLockPool.md#0x1_TokenLockPool_FixedTimeLockKey">FixedTimeLockKey</a>&lt;TokenType&gt; <b>acquires</b> <a href="TokenLockPool.md#0x1_TokenLockPool_TokenPool">TokenPool</a> {
    <b>assert</b>(peroid &gt; 0, <a href="ErrorCode.md#0x1_ErrorCode_EINVALID_ARGUMENT">ErrorCode::EINVALID_ARGUMENT</a>());
    <b>let</b> now = <a href="Timestamp.md#0x1_Timestamp_now_seconds">Timestamp::now_seconds</a>();
    <b>let</b> total = <a href="Token.md#0x1_Token_value">Token::value</a>(&token);
    <b>let</b> end_time = now + peroid;
    <b>let</b> token_pool = borrow_global_mut&lt;<a href="TokenLockPool.md#0x1_TokenLockPool_TokenPool">TokenPool</a>&lt;TokenType&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
    <a href="Token.md#0x1_Token_deposit">Token::deposit</a>(&<b>mut</b> token_pool.token, token);
    <a href="TokenLockPool.md#0x1_TokenLockPool_FixedTimeLockKey">FixedTimeLockKey</a>&lt;TokenType&gt; {
        total,
        end_time,
    }
}
</code></pre>



</details>

<a name="0x1_TokenLockPool_unlock_with_linear_key"></a>

## Function `unlock_with_linear_key`



<pre><code><b>public</b> <b>fun</b> <a href="TokenLockPool.md#0x1_TokenLockPool_unlock_with_linear_key">unlock_with_linear_key</a>&lt;TokenType&gt;(key: &<b>mut</b> <a href="TokenLockPool.md#0x1_TokenLockPool_LinearTimeLockKey">TokenLockPool::LinearTimeLockKey</a>&lt;TokenType&gt;): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="TokenLockPool.md#0x1_TokenLockPool_unlock_with_linear_key">unlock_with_linear_key</a>&lt;TokenType&gt;(key: &<b>mut</b> <a href="TokenLockPool.md#0x1_TokenLockPool_LinearTimeLockKey">LinearTimeLockKey</a>&lt;TokenType&gt;): <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt; <b>acquires</b> <a href="TokenLockPool.md#0x1_TokenLockPool_TokenPool">TokenPool</a> {
    <b>let</b> amount = <a href="TokenLockPool.md#0x1_TokenLockPool_unlocked_amount_of_linear_key">unlocked_amount_of_linear_key</a>(key);
    <b>assert</b>(amount &gt; 0, <a href="TokenLockPool.md#0x1_TokenLockPool_ETIMELOCK_NOT_UNLOCKED">ETIMELOCK_NOT_UNLOCKED</a>());
    <b>let</b> token_pool = borrow_global_mut&lt;<a href="TokenLockPool.md#0x1_TokenLockPool_TokenPool">TokenPool</a>&lt;TokenType&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
    <b>let</b> token = <a href="Token.md#0x1_Token_withdraw">Token::withdraw</a>(&<b>mut</b> token_pool.token, amount);
    key.taked = key.taked + amount;
    token
}
</code></pre>



</details>

<a name="0x1_TokenLockPool_unlock_with_fixed_key"></a>

## Function `unlock_with_fixed_key`



<pre><code><b>public</b> <b>fun</b> <a href="TokenLockPool.md#0x1_TokenLockPool_unlock_with_fixed_key">unlock_with_fixed_key</a>&lt;TokenType&gt;(key: <a href="TokenLockPool.md#0x1_TokenLockPool_FixedTimeLockKey">TokenLockPool::FixedTimeLockKey</a>&lt;TokenType&gt;): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="TokenLockPool.md#0x1_TokenLockPool_unlock_with_fixed_key">unlock_with_fixed_key</a>&lt;TokenType&gt;(key: <a href="TokenLockPool.md#0x1_TokenLockPool_FixedTimeLockKey">FixedTimeLockKey</a>&lt;TokenType&gt;): <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt;  <b>acquires</b> <a href="TokenLockPool.md#0x1_TokenLockPool_TokenPool">TokenPool</a> {
    <b>let</b> amount = <a href="TokenLockPool.md#0x1_TokenLockPool_unlocked_amount_of_fixed_key">unlocked_amount_of_fixed_key</a>(&key);
    <b>assert</b>(amount &gt; 0, <a href="TokenLockPool.md#0x1_TokenLockPool_ETIMELOCK_NOT_UNLOCKED">ETIMELOCK_NOT_UNLOCKED</a>());
    <b>let</b> token_pool = borrow_global_mut&lt;<a href="TokenLockPool.md#0x1_TokenLockPool_TokenPool">TokenPool</a>&lt;TokenType&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
    <b>let</b> token = <a href="Token.md#0x1_Token_withdraw">Token::withdraw</a>(&<b>mut</b> token_pool.token, key.total);
    <b>let</b> <a href="TokenLockPool.md#0x1_TokenLockPool_FixedTimeLockKey">FixedTimeLockKey</a> { total: _, end_time: _ } = key;
    token
}
</code></pre>



</details>

<a name="0x1_TokenLockPool_unlocked_amount_of_linear_key"></a>

## Function `unlocked_amount_of_linear_key`



<pre><code><b>public</b> <b>fun</b> <a href="TokenLockPool.md#0x1_TokenLockPool_unlocked_amount_of_linear_key">unlocked_amount_of_linear_key</a>&lt;TokenType&gt;(key: &<a href="TokenLockPool.md#0x1_TokenLockPool_LinearTimeLockKey">TokenLockPool::LinearTimeLockKey</a>&lt;TokenType&gt;): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="TokenLockPool.md#0x1_TokenLockPool_unlocked_amount_of_linear_key">unlocked_amount_of_linear_key</a>&lt;TokenType&gt;(key: &<a href="TokenLockPool.md#0x1_TokenLockPool_LinearTimeLockKey">LinearTimeLockKey</a>&lt;TokenType&gt;): u128 {
    <b>let</b> now = <a href="Timestamp.md#0x1_Timestamp_now_seconds">Timestamp::now_seconds</a>();
    <b>let</b> elapsed_time = now - key.start_time;
    <b>if</b> (elapsed_time &gt;= key.peroid) {
        key.total - key.taked
    }<b>else</b> {
        <a href="Math.md#0x1_Math_mul_div">Math::mul_div</a>(key.total, (elapsed_time <b>as</b> u128), (key.peroid <b>as</b> u128)) - key.taked
    }
}
</code></pre>



</details>

<a name="0x1_TokenLockPool_unlocked_amount_of_fixed_key"></a>

## Function `unlocked_amount_of_fixed_key`



<pre><code><b>public</b> <b>fun</b> <a href="TokenLockPool.md#0x1_TokenLockPool_unlocked_amount_of_fixed_key">unlocked_amount_of_fixed_key</a>&lt;TokenType&gt;(key: &<a href="TokenLockPool.md#0x1_TokenLockPool_FixedTimeLockKey">TokenLockPool::FixedTimeLockKey</a>&lt;TokenType&gt;): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="TokenLockPool.md#0x1_TokenLockPool_unlocked_amount_of_fixed_key">unlocked_amount_of_fixed_key</a>&lt;TokenType&gt;(key: &<a href="TokenLockPool.md#0x1_TokenLockPool_FixedTimeLockKey">FixedTimeLockKey</a>&lt;TokenType&gt;): u128 {
    <b>let</b> now = <a href="Timestamp.md#0x1_Timestamp_now_seconds">Timestamp::now_seconds</a>();
    <b>if</b> (now &gt;= key.end_time) {
        key.total
    }<b>else</b>{
        0
    }
}
</code></pre>



</details>

<a name="0x1_TokenLockPool_end_time_of"></a>

## Function `end_time_of`



<pre><code><b>public</b> <b>fun</b> <a href="TokenLockPool.md#0x1_TokenLockPool_end_time_of">end_time_of</a>&lt;TokenType&gt;(key: &<a href="TokenLockPool.md#0x1_TokenLockPool_FixedTimeLockKey">TokenLockPool::FixedTimeLockKey</a>&lt;TokenType&gt;): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="TokenLockPool.md#0x1_TokenLockPool_end_time_of">end_time_of</a>&lt;TokenType&gt;(key: &<a href="TokenLockPool.md#0x1_TokenLockPool_FixedTimeLockKey">FixedTimeLockKey</a>&lt;TokenType&gt;): u64 {
    key.end_time
}
</code></pre>



</details>

<a name="0x1_TokenLockPool_destroy_empty"></a>

## Function `destroy_empty`



<pre><code><b>public</b> <b>fun</b> <a href="TokenLockPool.md#0x1_TokenLockPool_destroy_empty">destroy_empty</a>&lt;TokenType&gt;(key: <a href="TokenLockPool.md#0x1_TokenLockPool_LinearTimeLockKey">TokenLockPool::LinearTimeLockKey</a>&lt;TokenType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="TokenLockPool.md#0x1_TokenLockPool_destroy_empty">destroy_empty</a>&lt;TokenType&gt;(key: <a href="TokenLockPool.md#0x1_TokenLockPool_LinearTimeLockKey">LinearTimeLockKey</a>&lt;TokenType&gt;) {
    <b>let</b> <a href="TokenLockPool.md#0x1_TokenLockPool_LinearTimeLockKey">LinearTimeLockKey</a>&lt;TokenType&gt; { total, taked, start_time: _, peroid: _ } = key;
    <b>assert</b>(total == taked, <a href="TokenLockPool.md#0x1_TokenLockPool_EDESTROY_KEY_NOT_EMPTY">EDESTROY_KEY_NOT_EMPTY</a>());
}
</code></pre>



</details>
