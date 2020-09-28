
<a name="0x1_TokenLockPool"></a>

# Module `0x1::TokenLockPool`

### Table of Contents

-  [Resource `TokenPool`](#0x1_TokenLockPool_TokenPool)
-  [Resource `FixedTimeLockKey`](#0x1_TokenLockPool_FixedTimeLockKey)
-  [Resource `LinearTimeLockKey`](#0x1_TokenLockPool_LinearTimeLockKey)
-  [Function `EDESTROY_KEY_NOT_EMPTY`](#0x1_TokenLockPool_EDESTROY_KEY_NOT_EMPTY)
-  [Function `ETIMELOCK_NOT_UNLOCKED`](#0x1_TokenLockPool_ETIMELOCK_NOT_UNLOCKED)
-  [Function `EAMOUNT_TOO_BIG`](#0x1_TokenLockPool_EAMOUNT_TOO_BIG)
-  [Function `EPEROID_IS_ZERO`](#0x1_TokenLockPool_EPEROID_IS_ZERO)
-  [Function `initialize`](#0x1_TokenLockPool_initialize)
-  [Function `create_linear_lock`](#0x1_TokenLockPool_create_linear_lock)
-  [Function `create_fixed_lock`](#0x1_TokenLockPool_create_fixed_lock)
-  [Function `unlock_by_linear`](#0x1_TokenLockPool_unlock_by_linear)
-  [Function `unlock_by_fixed`](#0x1_TokenLockPool_unlock_by_fixed)
-  [Function `unlocked_value_of`](#0x1_TokenLockPool_unlocked_value_of)
-  [Function `time_lock_of`](#0x1_TokenLockPool_time_lock_of)
-  [Function `destroy_empty`](#0x1_TokenLockPool_destroy_empty)
-  [Function `save_linear_key`](#0x1_TokenLockPool_save_linear_key)
-  [Function `take_linear_key`](#0x1_TokenLockPool_take_linear_key)
-  [Function `exists_linear_key_at`](#0x1_TokenLockPool_exists_linear_key_at)
-  [Function `save_fixed_key`](#0x1_TokenLockPool_save_fixed_key)
-  [Function `take_fixed_key`](#0x1_TokenLockPool_take_fixed_key)
-  [Function `exists_fixed_key_at`](#0x1_TokenLockPool_exists_fixed_key_at)



<a name="0x1_TokenLockPool_TokenPool"></a>

## Resource `TokenPool`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_TokenLockPool_TokenPool">TokenPool</a>&lt;TokenType&gt;
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



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_TokenLockPool_FixedTimeLockKey">FixedTimeLockKey</a>&lt;TokenType&gt;
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
<code>time_lock: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_TokenLockPool_LinearTimeLockKey"></a>

## Resource `LinearTimeLockKey`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_TokenLockPool_LinearTimeLockKey">LinearTimeLockKey</a>&lt;TokenType&gt;
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
<code>taked: u128</code>
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

<a name="0x1_TokenLockPool_EDESTROY_KEY_NOT_EMPTY"></a>

## Function `EDESTROY_KEY_NOT_EMPTY`



<pre><code><b>fun</b> <a href="#0x1_TokenLockPool_EDESTROY_KEY_NOT_EMPTY">EDESTROY_KEY_NOT_EMPTY</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_TokenLockPool_EDESTROY_KEY_NOT_EMPTY">EDESTROY_KEY_NOT_EMPTY</a>(): u64 {
    <a href="ErrorCode.md#0x1_ErrorCode_ECODE_BASE">ErrorCode::ECODE_BASE</a>() + 1
}
</code></pre>



</details>

<a name="0x1_TokenLockPool_ETIMELOCK_NOT_UNLOCKED"></a>

## Function `ETIMELOCK_NOT_UNLOCKED`



<pre><code><b>fun</b> <a href="#0x1_TokenLockPool_ETIMELOCK_NOT_UNLOCKED">ETIMELOCK_NOT_UNLOCKED</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_TokenLockPool_ETIMELOCK_NOT_UNLOCKED">ETIMELOCK_NOT_UNLOCKED</a>(): u64 {
    <a href="ErrorCode.md#0x1_ErrorCode_ECODE_BASE">ErrorCode::ECODE_BASE</a>() + 2
}
</code></pre>



</details>

<a name="0x1_TokenLockPool_EAMOUNT_TOO_BIG"></a>

## Function `EAMOUNT_TOO_BIG`



<pre><code><b>fun</b> <a href="#0x1_TokenLockPool_EAMOUNT_TOO_BIG">EAMOUNT_TOO_BIG</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_TokenLockPool_EAMOUNT_TOO_BIG">EAMOUNT_TOO_BIG</a>(): u64 {
    <a href="ErrorCode.md#0x1_ErrorCode_ECODE_BASE">ErrorCode::ECODE_BASE</a>() + 3
}
</code></pre>



</details>

<a name="0x1_TokenLockPool_EPEROID_IS_ZERO"></a>

## Function `EPEROID_IS_ZERO`



<pre><code><b>fun</b> <a href="#0x1_TokenLockPool_EPEROID_IS_ZERO">EPEROID_IS_ZERO</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_TokenLockPool_EPEROID_IS_ZERO">EPEROID_IS_ZERO</a>(): u64 {
    <a href="ErrorCode.md#0x1_ErrorCode_ECODE_BASE">ErrorCode::ECODE_BASE</a>() + 4
}
</code></pre>



</details>

<a name="0x1_TokenLockPool_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TokenLockPool_initialize">initialize</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TokenLockPool_initialize">initialize</a>(account: &signer) {
    <b>assert</b>(<a href="Timestamp.md#0x1_Timestamp_is_genesis">Timestamp::is_genesis</a>(), <a href="ErrorCode.md#0x1_ErrorCode_ENOT_GENESIS">ErrorCode::ENOT_GENESIS</a>());
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>(), <a href="ErrorCode.md#0x1_ErrorCode_ENOT_GENESIS_ACCOUNT">ErrorCode::ENOT_GENESIS_ACCOUNT</a>());
    <b>let</b> token_pool = <a href="#0x1_TokenLockPool_TokenPool">TokenPool</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt; { token: <a href="Token.md#0x1_Token_zero">Token::zero</a>() };
    move_to(account, token_pool);
    //TODO how <b>to</b> init other token's pool.
}
</code></pre>



</details>

<a name="0x1_TokenLockPool_create_linear_lock"></a>

## Function `create_linear_lock`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TokenLockPool_create_linear_lock">create_linear_lock</a>&lt;TokenType&gt;(token: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;, lock_peroid: u64): <a href="#0x1_TokenLockPool_LinearTimeLockKey">TokenLockPool::LinearTimeLockKey</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TokenLockPool_create_linear_lock">create_linear_lock</a>&lt;TokenType&gt;(token: <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt;, lock_peroid: u64): <a href="#0x1_TokenLockPool_LinearTimeLockKey">LinearTimeLockKey</a>&lt;TokenType&gt; <b>acquires</b> <a href="#0x1_TokenLockPool_TokenPool">TokenPool</a> {
    <b>assert</b>(lock_peroid &gt; 0, <a href="#0x1_TokenLockPool_EPEROID_IS_ZERO">EPEROID_IS_ZERO</a>());
    <b>let</b> lock_time = <a href="Timestamp.md#0x1_Timestamp_now_seconds">Timestamp::now_seconds</a>();
    <b>let</b> origin = <a href="Token.md#0x1_Token_share">Token::share</a>(&token);
    <b>let</b> token_pool = borrow_global_mut&lt;<a href="#0x1_TokenLockPool_TokenPool">TokenPool</a>&lt;TokenType&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
    <a href="Token.md#0x1_Token_deposit">Token::deposit</a>(&<b>mut</b> token_pool.token, token);
    <a href="#0x1_TokenLockPool_LinearTimeLockKey">LinearTimeLockKey</a>&lt;TokenType&gt; {
        origin,
        taked: 0,
        lock_time,
        lock_peroid
    }
}
</code></pre>



</details>

<a name="0x1_TokenLockPool_create_fixed_lock"></a>

## Function `create_fixed_lock`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TokenLockPool_create_fixed_lock">create_fixed_lock</a>&lt;TokenType&gt;(token: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;, lock_peroid: u64): <a href="#0x1_TokenLockPool_FixedTimeLockKey">TokenLockPool::FixedTimeLockKey</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TokenLockPool_create_fixed_lock">create_fixed_lock</a>&lt;TokenType&gt;(token: <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt;, lock_peroid: u64): <a href="#0x1_TokenLockPool_FixedTimeLockKey">FixedTimeLockKey</a>&lt;TokenType&gt; <b>acquires</b> <a href="#0x1_TokenLockPool_TokenPool">TokenPool</a> {
    <b>assert</b>(lock_peroid &gt; 0, <a href="#0x1_TokenLockPool_EPEROID_IS_ZERO">EPEROID_IS_ZERO</a>());
    <b>let</b> now = <a href="Timestamp.md#0x1_Timestamp_now_seconds">Timestamp::now_seconds</a>();
    <b>let</b> origin = <a href="Token.md#0x1_Token_share">Token::share</a>(&token);
    <b>let</b> time_lock = now + lock_peroid;
    <b>let</b> token_pool = borrow_global_mut&lt;<a href="#0x1_TokenLockPool_TokenPool">TokenPool</a>&lt;TokenType&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
    <a href="Token.md#0x1_Token_deposit">Token::deposit</a>(&<b>mut</b> token_pool.token, token);
    <a href="#0x1_TokenLockPool_FixedTimeLockKey">FixedTimeLockKey</a>&lt;TokenType&gt; {
        origin,
        time_lock,
    }
}
</code></pre>



</details>

<a name="0x1_TokenLockPool_unlock_by_linear"></a>

## Function `unlock_by_linear`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TokenLockPool_unlock_by_linear">unlock_by_linear</a>&lt;TokenType&gt;(key: &<b>mut</b> <a href="#0x1_TokenLockPool_LinearTimeLockKey">TokenLockPool::LinearTimeLockKey</a>&lt;TokenType&gt;): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TokenLockPool_unlock_by_linear">unlock_by_linear</a>&lt;TokenType&gt;(key: &<b>mut</b> <a href="#0x1_TokenLockPool_LinearTimeLockKey">LinearTimeLockKey</a>&lt;TokenType&gt;): <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt; <b>acquires</b> <a href="#0x1_TokenLockPool_TokenPool">TokenPool</a> {
    <b>let</b> value = <a href="#0x1_TokenLockPool_unlocked_value_of">unlocked_value_of</a>(key);
    <b>let</b> token_pool = borrow_global_mut&lt;<a href="#0x1_TokenLockPool_TokenPool">TokenPool</a>&lt;TokenType&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
    <b>let</b> token = <a href="Token.md#0x1_Token_withdraw_share">Token::withdraw_share</a>(&<b>mut</b> token_pool.token, value);
    key.taked = key.taked + value;
    token
}
</code></pre>



</details>

<a name="0x1_TokenLockPool_unlock_by_fixed"></a>

## Function `unlock_by_fixed`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TokenLockPool_unlock_by_fixed">unlock_by_fixed</a>&lt;TokenType&gt;(key: <a href="#0x1_TokenLockPool_FixedTimeLockKey">TokenLockPool::FixedTimeLockKey</a>&lt;TokenType&gt;): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TokenLockPool_unlock_by_fixed">unlock_by_fixed</a>&lt;TokenType&gt;(key: <a href="#0x1_TokenLockPool_FixedTimeLockKey">FixedTimeLockKey</a>&lt;TokenType&gt;): <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt;  <b>acquires</b> <a href="#0x1_TokenLockPool_TokenPool">TokenPool</a> {
    <b>let</b> now = <a href="Timestamp.md#0x1_Timestamp_now_seconds">Timestamp::now_seconds</a>();
    <b>assert</b>(now &gt;= key.time_lock, <a href="#0x1_TokenLockPool_ETIMELOCK_NOT_UNLOCKED">ETIMELOCK_NOT_UNLOCKED</a>());
    <b>let</b> token_pool = borrow_global_mut&lt;<a href="#0x1_TokenLockPool_TokenPool">TokenPool</a>&lt;TokenType&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
    <b>let</b> token = <a href="Token.md#0x1_Token_withdraw_share">Token::withdraw_share</a>(&<b>mut</b> token_pool.token, key.origin);
    <b>let</b> <a href="#0x1_TokenLockPool_FixedTimeLockKey">FixedTimeLockKey</a> { origin: _, time_lock: _ } = key;
    token
}
</code></pre>



</details>

<a name="0x1_TokenLockPool_unlocked_value_of"></a>

## Function `unlocked_value_of`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TokenLockPool_unlocked_value_of">unlocked_value_of</a>&lt;TokenType&gt;(key: &<a href="#0x1_TokenLockPool_LinearTimeLockKey">TokenLockPool::LinearTimeLockKey</a>&lt;TokenType&gt;): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TokenLockPool_unlocked_value_of">unlocked_value_of</a>&lt;TokenType&gt;(key: &<a href="#0x1_TokenLockPool_LinearTimeLockKey">LinearTimeLockKey</a>&lt;TokenType&gt;): u128 {
    <b>let</b> now = <a href="Timestamp.md#0x1_Timestamp_now_seconds">Timestamp::now_seconds</a>();
    <b>let</b> elapsed_time = now - key.lock_time;
    <b>if</b> (elapsed_time &gt;= key.lock_peroid) {
        <b>return</b> key.origin - key.taked
    }<b>else</b> {
        //for avoid overflow
        <b>if</b> (key.origin &gt; (key.lock_peroid <b>as</b> u128)) {
            key.origin / (key.lock_peroid <b>as</b> u128) * (elapsed_time <b>as</b> u128) - key.taked
        }<b>else</b> {
            key.origin * (elapsed_time <b>as</b> u128) / (key.lock_peroid <b>as</b> u128) - key.taked
        }
    }
}
</code></pre>



</details>

<a name="0x1_TokenLockPool_time_lock_of"></a>

## Function `time_lock_of`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TokenLockPool_time_lock_of">time_lock_of</a>&lt;TokenType&gt;(key: &<a href="#0x1_TokenLockPool_FixedTimeLockKey">TokenLockPool::FixedTimeLockKey</a>&lt;TokenType&gt;): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TokenLockPool_time_lock_of">time_lock_of</a>&lt;TokenType&gt;(key: &<a href="#0x1_TokenLockPool_FixedTimeLockKey">FixedTimeLockKey</a>&lt;TokenType&gt;): u64 {
    key.time_lock
}
</code></pre>



</details>

<a name="0x1_TokenLockPool_destroy_empty"></a>

## Function `destroy_empty`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TokenLockPool_destroy_empty">destroy_empty</a>&lt;TokenType&gt;(key: <a href="#0x1_TokenLockPool_LinearTimeLockKey">TokenLockPool::LinearTimeLockKey</a>&lt;TokenType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TokenLockPool_destroy_empty">destroy_empty</a>&lt;TokenType&gt;(key: <a href="#0x1_TokenLockPool_LinearTimeLockKey">LinearTimeLockKey</a>&lt;TokenType&gt;) {
    <b>let</b> <a href="#0x1_TokenLockPool_LinearTimeLockKey">LinearTimeLockKey</a>&lt;TokenType&gt; { origin, taked, lock_time: _, lock_peroid: _ } = key;
    <b>assert</b>(origin == taked, <a href="#0x1_TokenLockPool_EDESTROY_KEY_NOT_EMPTY">EDESTROY_KEY_NOT_EMPTY</a>());
}
</code></pre>



</details>

<a name="0x1_TokenLockPool_save_linear_key"></a>

## Function `save_linear_key`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TokenLockPool_save_linear_key">save_linear_key</a>&lt;TokenType&gt;(account: &signer, key: <a href="#0x1_TokenLockPool_LinearTimeLockKey">TokenLockPool::LinearTimeLockKey</a>&lt;TokenType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TokenLockPool_save_linear_key">save_linear_key</a>&lt;TokenType&gt;(account: &signer, key: <a href="#0x1_TokenLockPool_LinearTimeLockKey">LinearTimeLockKey</a>&lt;TokenType&gt;) {
    move_to(account, key);
}
</code></pre>



</details>

<a name="0x1_TokenLockPool_take_linear_key"></a>

## Function `take_linear_key`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TokenLockPool_take_linear_key">take_linear_key</a>&lt;TokenType&gt;(account: &signer): <a href="#0x1_TokenLockPool_LinearTimeLockKey">TokenLockPool::LinearTimeLockKey</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TokenLockPool_take_linear_key">take_linear_key</a>&lt;TokenType&gt;(account: &signer): <a href="#0x1_TokenLockPool_LinearTimeLockKey">LinearTimeLockKey</a>&lt;TokenType&gt; <b>acquires</b> <a href="#0x1_TokenLockPool_LinearTimeLockKey">LinearTimeLockKey</a> {
    move_from&lt;<a href="#0x1_TokenLockPool_LinearTimeLockKey">LinearTimeLockKey</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account))
}
</code></pre>



</details>

<a name="0x1_TokenLockPool_exists_linear_key_at"></a>

## Function `exists_linear_key_at`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TokenLockPool_exists_linear_key_at">exists_linear_key_at</a>&lt;TokenType&gt;(address: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TokenLockPool_exists_linear_key_at">exists_linear_key_at</a>&lt;TokenType&gt;(address: address): bool {
    exists&lt;<a href="#0x1_TokenLockPool_LinearTimeLockKey">LinearTimeLockKey</a>&lt;TokenType&gt;&gt;(address)
}
</code></pre>



</details>

<a name="0x1_TokenLockPool_save_fixed_key"></a>

## Function `save_fixed_key`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TokenLockPool_save_fixed_key">save_fixed_key</a>&lt;TokenType&gt;(account: &signer, key: <a href="#0x1_TokenLockPool_FixedTimeLockKey">TokenLockPool::FixedTimeLockKey</a>&lt;TokenType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TokenLockPool_save_fixed_key">save_fixed_key</a>&lt;TokenType&gt;(account: &signer, key: <a href="#0x1_TokenLockPool_FixedTimeLockKey">FixedTimeLockKey</a>&lt;TokenType&gt;) {
    move_to(account, key);
}
</code></pre>



</details>

<a name="0x1_TokenLockPool_take_fixed_key"></a>

## Function `take_fixed_key`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TokenLockPool_take_fixed_key">take_fixed_key</a>&lt;TokenType&gt;(account: &signer): <a href="#0x1_TokenLockPool_FixedTimeLockKey">TokenLockPool::FixedTimeLockKey</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TokenLockPool_take_fixed_key">take_fixed_key</a>&lt;TokenType&gt;(account: &signer): <a href="#0x1_TokenLockPool_FixedTimeLockKey">FixedTimeLockKey</a>&lt;TokenType&gt; <b>acquires</b> <a href="#0x1_TokenLockPool_FixedTimeLockKey">FixedTimeLockKey</a> {
    move_from&lt;<a href="#0x1_TokenLockPool_FixedTimeLockKey">FixedTimeLockKey</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account))
}
</code></pre>



</details>

<a name="0x1_TokenLockPool_exists_fixed_key_at"></a>

## Function `exists_fixed_key_at`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TokenLockPool_exists_fixed_key_at">exists_fixed_key_at</a>&lt;TokenType&gt;(address: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TokenLockPool_exists_fixed_key_at">exists_fixed_key_at</a>&lt;TokenType&gt;(address: address): bool {
    exists&lt;<a href="#0x1_TokenLockPool_FixedTimeLockKey">FixedTimeLockKey</a>&lt;TokenType&gt;&gt;(address)
}
</code></pre>



</details>
