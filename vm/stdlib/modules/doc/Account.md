
<a name="0x1_Account"></a>

# Module `0x1::Account`



-  [Resource <code><a href="Account.md#0x1_Account">Account</a></code>](#0x1_Account_Account)
-  [Resource <code><a href="Account.md#0x1_Account_Balance">Balance</a></code>](#0x1_Account_Balance)
-  [Resource <code><a href="Account.md#0x1_Account_WithdrawCapability">WithdrawCapability</a></code>](#0x1_Account_WithdrawCapability)
-  [Resource <code><a href="Account.md#0x1_Account_KeyRotationCapability">KeyRotationCapability</a></code>](#0x1_Account_KeyRotationCapability)
-  [Struct <code><a href="Account.md#0x1_Account_SentPaymentEvent">SentPaymentEvent</a></code>](#0x1_Account_SentPaymentEvent)
-  [Struct <code><a href="Account.md#0x1_Account_ReceivedPaymentEvent">ReceivedPaymentEvent</a></code>](#0x1_Account_ReceivedPaymentEvent)
-  [Struct <code><a href="Account.md#0x1_Account_AcceptTokenEvent">AcceptTokenEvent</a></code>](#0x1_Account_AcceptTokenEvent)
-  [Const <code><a href="Account.md#0x1_Account_DUMMY_AUTH_KEY">DUMMY_AUTH_KEY</a></code>](#0x1_Account_DUMMY_AUTH_KEY)
-  [Function <code>EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED</code>](#0x1_Account_EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED)
-  [Function <code>EMALFORMED_AUTHENTICATION_KEY</code>](#0x1_Account_EMALFORMED_AUTHENTICATION_KEY)
-  [Function <code>EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED</code>](#0x1_Account_EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED)
-  [Function <code>ADDRESS_PUBLIC_KEY_INCONSISTENT</code>](#0x1_Account_ADDRESS_PUBLIC_KEY_INCONSISTENT)
-  [Function <code>create_genesis_account</code>](#0x1_Account_create_genesis_account)
-  [Function <code>release_genesis_signer</code>](#0x1_Account_release_genesis_signer)
-  [Function <code>create_account</code>](#0x1_Account_create_account)
-  [Function <code>make_account</code>](#0x1_Account_make_account)
-  [Function <code>create_signer</code>](#0x1_Account_create_signer)
-  [Function <code>destroy_signer</code>](#0x1_Account_destroy_signer)
-  [Function <code>deposit_to</code>](#0x1_Account_deposit_to)
-  [Function <code>deposit</code>](#0x1_Account_deposit)
-  [Function <code>deposit_with_metadata</code>](#0x1_Account_deposit_with_metadata)
-  [Function <code>deposit_with_payer_and_metadata</code>](#0x1_Account_deposit_with_payer_and_metadata)
-  [Function <code>withdraw_from_balance</code>](#0x1_Account_withdraw_from_balance)
-  [Function <code>withdraw</code>](#0x1_Account_withdraw)
-  [Function <code>withdraw_with_capability</code>](#0x1_Account_withdraw_with_capability)
-  [Function <code>extract_withdraw_capability</code>](#0x1_Account_extract_withdraw_capability)
-  [Function <code>restore_withdraw_capability</code>](#0x1_Account_restore_withdraw_capability)
-  [Function <code>pay_from_capability</code>](#0x1_Account_pay_from_capability)
-  [Function <code>pay_from_with_metadata</code>](#0x1_Account_pay_from_with_metadata)
-  [Function <code>pay_from</code>](#0x1_Account_pay_from)
-  [Function <code>rotate_authentication_key</code>](#0x1_Account_rotate_authentication_key)
-  [Function <code>extract_key_rotation_capability</code>](#0x1_Account_extract_key_rotation_capability)
-  [Function <code>restore_key_rotation_capability</code>](#0x1_Account_restore_key_rotation_capability)
-  [Function <code>balance_for</code>](#0x1_Account_balance_for)
-  [Function <code>balance</code>](#0x1_Account_balance)
-  [Function <code>accept_token</code>](#0x1_Account_accept_token)
-  [Function <code>is_accepts_token</code>](#0x1_Account_is_accepts_token)
-  [Function <code>sequence_number_for_account</code>](#0x1_Account_sequence_number_for_account)
-  [Function <code>sequence_number</code>](#0x1_Account_sequence_number)
-  [Function <code>authentication_key</code>](#0x1_Account_authentication_key)
-  [Function <code>delegated_key_rotation_capability</code>](#0x1_Account_delegated_key_rotation_capability)
-  [Function <code>delegated_withdraw_capability</code>](#0x1_Account_delegated_withdraw_capability)
-  [Function <code>withdraw_capability_address</code>](#0x1_Account_withdraw_capability_address)
-  [Function <code>key_rotation_capability_address</code>](#0x1_Account_key_rotation_capability_address)
-  [Function <code>exists_at</code>](#0x1_Account_exists_at)
-  [Function <code>txn_prologue</code>](#0x1_Account_txn_prologue)
-  [Function <code>txn_epilogue</code>](#0x1_Account_txn_epilogue)
-  [Specification](#@Specification_0)
    -  [Function <code>create_genesis_account</code>](#@Specification_0_create_genesis_account)
    -  [Function <code>release_genesis_signer</code>](#@Specification_0_release_genesis_signer)
    -  [Function <code>create_account</code>](#@Specification_0_create_account)
    -  [Function <code>make_account</code>](#@Specification_0_make_account)
    -  [Function <code>deposit_to</code>](#@Specification_0_deposit_to)
    -  [Function <code>deposit</code>](#@Specification_0_deposit)
    -  [Function <code>deposit_with_metadata</code>](#@Specification_0_deposit_with_metadata)
    -  [Function <code>deposit_with_payer_and_metadata</code>](#@Specification_0_deposit_with_payer_and_metadata)
    -  [Function <code>withdraw_from_balance</code>](#@Specification_0_withdraw_from_balance)
    -  [Function <code>withdraw</code>](#@Specification_0_withdraw)
    -  [Function <code>withdraw_with_capability</code>](#@Specification_0_withdraw_with_capability)
    -  [Function <code>extract_withdraw_capability</code>](#@Specification_0_extract_withdraw_capability)
    -  [Function <code>restore_withdraw_capability</code>](#@Specification_0_restore_withdraw_capability)
    -  [Function <code>pay_from_capability</code>](#@Specification_0_pay_from_capability)
    -  [Function <code>pay_from_with_metadata</code>](#@Specification_0_pay_from_with_metadata)
    -  [Function <code>pay_from</code>](#@Specification_0_pay_from)
    -  [Function <code>rotate_authentication_key</code>](#@Specification_0_rotate_authentication_key)
    -  [Function <code>extract_key_rotation_capability</code>](#@Specification_0_extract_key_rotation_capability)
    -  [Function <code>restore_key_rotation_capability</code>](#@Specification_0_restore_key_rotation_capability)
    -  [Function <code>balance_for</code>](#@Specification_0_balance_for)
    -  [Function <code>balance</code>](#@Specification_0_balance)
    -  [Function <code>accept_token</code>](#@Specification_0_accept_token)
    -  [Function <code>is_accepts_token</code>](#@Specification_0_is_accepts_token)
    -  [Function <code>sequence_number</code>](#@Specification_0_sequence_number)
    -  [Function <code>authentication_key</code>](#@Specification_0_authentication_key)
    -  [Function <code>delegated_key_rotation_capability</code>](#@Specification_0_delegated_key_rotation_capability)
    -  [Function <code>delegated_withdraw_capability</code>](#@Specification_0_delegated_withdraw_capability)
    -  [Function <code>withdraw_capability_address</code>](#@Specification_0_withdraw_capability_address)
    -  [Function <code>key_rotation_capability_address</code>](#@Specification_0_key_rotation_capability_address)
    -  [Function <code>exists_at</code>](#@Specification_0_exists_at)
    -  [Function <code>txn_prologue</code>](#@Specification_0_txn_prologue)
    -  [Function <code>txn_epilogue</code>](#@Specification_0_txn_epilogue)


<a name="0x1_Account_Account"></a>

## Resource `Account`



<pre><code><b>resource</b> <b>struct</b> <a href="Account.md#0x1_Account">Account</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>authentication_key: vector&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>withdrawal_capability: <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;<a href="Account.md#0x1_Account_WithdrawCapability">Account::WithdrawCapability</a>&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>key_rotation_capability: <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;<a href="Account.md#0x1_Account_KeyRotationCapability">Account::KeyRotationCapability</a>&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>received_events: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="Account.md#0x1_Account_ReceivedPaymentEvent">Account::ReceivedPaymentEvent</a>&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>sent_events: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="Account.md#0x1_Account_SentPaymentEvent">Account::SentPaymentEvent</a>&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>accept_token_events: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="Account.md#0x1_Account_AcceptTokenEvent">Account::AcceptTokenEvent</a>&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>sequence_number: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_Account_Balance"></a>

## Resource `Balance`



<pre><code><b>resource</b> <b>struct</b> <a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;
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

<a name="0x1_Account_WithdrawCapability"></a>

## Resource `WithdrawCapability`



<pre><code><b>resource</b> <b>struct</b> <a href="Account.md#0x1_Account_WithdrawCapability">WithdrawCapability</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>account_address: address</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_Account_KeyRotationCapability"></a>

## Resource `KeyRotationCapability`



<pre><code><b>resource</b> <b>struct</b> <a href="Account.md#0x1_Account_KeyRotationCapability">KeyRotationCapability</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>account_address: address</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_Account_SentPaymentEvent"></a>

## Struct `SentPaymentEvent`



<pre><code><b>struct</b> <a href="Account.md#0x1_Account_SentPaymentEvent">SentPaymentEvent</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>amount: u128</code>
</dt>
<dd>

</dd>
<dt>
<code>token_code: vector&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>payee: address</code>
</dt>
<dd>

</dd>
<dt>
<code>metadata: vector&lt;u8&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_Account_ReceivedPaymentEvent"></a>

## Struct `ReceivedPaymentEvent`



<pre><code><b>struct</b> <a href="Account.md#0x1_Account_ReceivedPaymentEvent">ReceivedPaymentEvent</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>amount: u128</code>
</dt>
<dd>

</dd>
<dt>
<code>token_code: vector&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>payer: address</code>
</dt>
<dd>

</dd>
<dt>
<code>metadata: vector&lt;u8&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_Account_AcceptTokenEvent"></a>

## Struct `AcceptTokenEvent`

Message for accept token events


<pre><code><b>struct</b> <a href="Account.md#0x1_Account_AcceptTokenEvent">AcceptTokenEvent</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>token_code: vector&lt;u8&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_Account_DUMMY_AUTH_KEY"></a>

## Const `DUMMY_AUTH_KEY`



<pre><code><b>const</b> <a href="Account.md#0x1_Account_DUMMY_AUTH_KEY">DUMMY_AUTH_KEY</a>: vector&lt;u8&gt; = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
</code></pre>



<a name="0x1_Account_EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED"></a>

## Function `EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED`



<pre><code><b>fun</b> <a href="Account.md#0x1_Account_EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED">EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="Account.md#0x1_Account_EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED">EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED</a>(): u64 { <a href="ErrorCode.md#0x1_ErrorCode_ECODE_BASE">ErrorCode::ECODE_BASE</a>() + 1}
</code></pre>



</details>

<a name="0x1_Account_EMALFORMED_AUTHENTICATION_KEY"></a>

## Function `EMALFORMED_AUTHENTICATION_KEY`



<pre><code><b>fun</b> <a href="Account.md#0x1_Account_EMALFORMED_AUTHENTICATION_KEY">EMALFORMED_AUTHENTICATION_KEY</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="Account.md#0x1_Account_EMALFORMED_AUTHENTICATION_KEY">EMALFORMED_AUTHENTICATION_KEY</a>(): u64 { <a href="ErrorCode.md#0x1_ErrorCode_ECODE_BASE">ErrorCode::ECODE_BASE</a>() + 2}
</code></pre>



</details>

<a name="0x1_Account_EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED"></a>

## Function `EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED`



<pre><code><b>fun</b> <a href="Account.md#0x1_Account_EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED">EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="Account.md#0x1_Account_EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED">EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED</a>(): u64 { <a href="ErrorCode.md#0x1_ErrorCode_ECODE_BASE">ErrorCode::ECODE_BASE</a>() + 3}
</code></pre>



</details>

<a name="0x1_Account_ADDRESS_PUBLIC_KEY_INCONSISTENT"></a>

## Function `ADDRESS_PUBLIC_KEY_INCONSISTENT`



<pre><code><b>fun</b> <a href="Account.md#0x1_Account_ADDRESS_PUBLIC_KEY_INCONSISTENT">ADDRESS_PUBLIC_KEY_INCONSISTENT</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="Account.md#0x1_Account_ADDRESS_PUBLIC_KEY_INCONSISTENT">ADDRESS_PUBLIC_KEY_INCONSISTENT</a>(): u64 { <a href="ErrorCode.md#0x1_ErrorCode_ECODE_BASE">ErrorCode::ECODE_BASE</a>() + 4}
</code></pre>



</details>

<a name="0x1_Account_create_genesis_account"></a>

## Function `create_genesis_account`



<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_create_genesis_account">create_genesis_account</a>(new_account_address: address): signer
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_create_genesis_account">create_genesis_account</a>(
    new_account_address: address,
) :signer {
    <b>assert</b>(<a href="Timestamp.md#0x1_Timestamp_is_genesis">Timestamp::is_genesis</a>(), <a href="ErrorCode.md#0x1_ErrorCode_ENOT_GENESIS">ErrorCode::ENOT_GENESIS</a>());
    <b>let</b> new_account = <a href="Account.md#0x1_Account_create_signer">create_signer</a>(new_account_address);
    <a href="Account.md#0x1_Account_make_account">make_account</a>(&new_account, <a href="Account.md#0x1_Account_DUMMY_AUTH_KEY">DUMMY_AUTH_KEY</a>);
    new_account
}
</code></pre>



</details>

<a name="0x1_Account_release_genesis_signer"></a>

## Function `release_genesis_signer`



<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_release_genesis_signer">release_genesis_signer</a>(genesis_account: signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_release_genesis_signer">release_genesis_signer</a>(genesis_account: signer){
    <a href="Account.md#0x1_Account_destroy_signer">destroy_signer</a>(genesis_account);
}
</code></pre>



</details>

<a name="0x1_Account_create_account"></a>

## Function `create_account`



<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_create_account">create_account</a>&lt;TokenType&gt;(fresh_address: address, public_key_vec: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_create_account">create_account</a>&lt;TokenType&gt;(fresh_address: address, public_key_vec: vector&lt;u8&gt;) <b>acquires</b> <a href="Account.md#0x1_Account">Account</a> {
    <b>let</b> authentication_key = <a href="Authenticator.md#0x1_Authenticator_ed25519_authentication_key">Authenticator::ed25519_authentication_key</a>(public_key_vec);
    <b>let</b> new_address = <a href="Authenticator.md#0x1_Authenticator_derived_address">Authenticator::derived_address</a>(<b>copy</b> authentication_key);
    <b>assert</b>(new_address == fresh_address, <a href="Account.md#0x1_Account_ADDRESS_PUBLIC_KEY_INCONSISTENT">ADDRESS_PUBLIC_KEY_INCONSISTENT</a>());

    <b>let</b> new_account = <a href="Account.md#0x1_Account_create_signer">create_signer</a>(new_address);
    <a href="Account.md#0x1_Account_make_account">make_account</a>(&new_account, authentication_key);
    // Make sure all account accept <a href="STC.md#0x1_STC">STC</a>.
    <b>if</b> (!<a href="STC.md#0x1_STC_is_stc">STC::is_stc</a>&lt;TokenType&gt;()){
        <a href="Account.md#0x1_Account_accept_token">accept_token</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;(&new_account);
    };
    <a href="Account.md#0x1_Account_accept_token">accept_token</a>&lt;TokenType&gt;(&new_account);
    <a href="Account.md#0x1_Account_destroy_signer">destroy_signer</a>(new_account);
}
</code></pre>



</details>

<a name="0x1_Account_make_account"></a>

## Function `make_account`



<pre><code><b>fun</b> <a href="Account.md#0x1_Account_make_account">make_account</a>(new_account: &signer, authentication_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="Account.md#0x1_Account_make_account">make_account</a>(
    new_account: &signer,
    authentication_key: vector&lt;u8&gt;,
) {
    <b>assert</b>(<a href="Vector.md#0x1_Vector_length">Vector::length</a>(&authentication_key) == 32, <a href="Account.md#0x1_Account_EMALFORMED_AUTHENTICATION_KEY">EMALFORMED_AUTHENTICATION_KEY</a>());
    <b>let</b> new_account_addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(new_account);
    <a href="Event.md#0x1_Event_publish_generator">Event::publish_generator</a>(new_account);
    move_to(new_account, <a href="Account.md#0x1_Account">Account</a> {
          authentication_key,
          withdrawal_capability: <a href="Option.md#0x1_Option_some">Option::some</a>(
              <a href="Account.md#0x1_Account_WithdrawCapability">WithdrawCapability</a> {
                  account_address: new_account_addr
          }),
          key_rotation_capability: <a href="Option.md#0x1_Option_some">Option::some</a>(
              <a href="Account.md#0x1_Account_KeyRotationCapability">KeyRotationCapability</a> {
                  account_address: new_account_addr
          }),
          received_events: <a href="Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="Account.md#0x1_Account_ReceivedPaymentEvent">ReceivedPaymentEvent</a>&gt;(new_account),
          sent_events: <a href="Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="Account.md#0x1_Account_SentPaymentEvent">SentPaymentEvent</a>&gt;(new_account),
          accept_token_events: <a href="Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="Account.md#0x1_Account_AcceptTokenEvent">AcceptTokenEvent</a>&gt;(new_account),
          sequence_number: 0,
    });
}
</code></pre>



</details>

<a name="0x1_Account_create_signer"></a>

## Function `create_signer`



<pre><code><b>fun</b> <a href="Account.md#0x1_Account_create_signer">create_signer</a>(addr: address): signer
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>fun</b> <a href="Account.md#0x1_Account_create_signer">create_signer</a>(addr: address): signer;
</code></pre>



</details>

<a name="0x1_Account_destroy_signer"></a>

## Function `destroy_signer`



<pre><code><b>fun</b> <a href="Account.md#0x1_Account_destroy_signer">destroy_signer</a>(sig: signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>fun</b> <a href="Account.md#0x1_Account_destroy_signer">destroy_signer</a>(sig: signer);
</code></pre>



</details>

<a name="0x1_Account_deposit_to"></a>

## Function `deposit_to`



<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_deposit_to">deposit_to</a>&lt;TokenType&gt;(account: &signer, payee: address, to_deposit: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_deposit_to">deposit_to</a>&lt;TokenType&gt;(account: &signer, payee: address, to_deposit: <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt;)
<b>acquires</b> <a href="Account.md#0x1_Account">Account</a>, <a href="Account.md#0x1_Account_Balance">Balance</a> {
    <a href="Account.md#0x1_Account_deposit_with_metadata">deposit_with_metadata</a>(account, payee, to_deposit, x"")
}
</code></pre>



</details>

<a name="0x1_Account_deposit"></a>

## Function `deposit`



<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_deposit">deposit</a>&lt;TokenType&gt;(account: &signer, to_deposit: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_deposit">deposit</a>&lt;TokenType&gt;(account: &signer, to_deposit: <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt;)
<b>acquires</b> <a href="Account.md#0x1_Account">Account</a>, <a href="Account.md#0x1_Account_Balance">Balance</a> {
    <b>let</b> account_address = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>if</b> (!<a href="Account.md#0x1_Account_is_accepts_token">is_accepts_token</a>&lt;TokenType&gt;(account_address)){
        <a href="Account.md#0x1_Account_accept_token">accept_token</a>&lt;TokenType&gt;(account);
    };
    <a href="Account.md#0x1_Account_deposit_to">deposit_to</a>(account, account_address, to_deposit)
}
</code></pre>



</details>

<a name="0x1_Account_deposit_with_metadata"></a>

## Function `deposit_with_metadata`



<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_deposit_with_metadata">deposit_with_metadata</a>&lt;TokenType&gt;(account: &signer, payee: address, to_deposit: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;, metadata: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_deposit_with_metadata">deposit_with_metadata</a>&lt;TokenType&gt;(account: &signer,
    payee: address,
    to_deposit: <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt;,
    metadata: vector&lt;u8&gt;,
) <b>acquires</b> <a href="Account.md#0x1_Account">Account</a>, <a href="Account.md#0x1_Account_Balance">Balance</a> {
    <a href="Account.md#0x1_Account_deposit_with_payer_and_metadata">deposit_with_payer_and_metadata</a>(
        <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account),
        payee,
        to_deposit,
        metadata,
    );
}
</code></pre>



</details>

<a name="0x1_Account_deposit_with_payer_and_metadata"></a>

## Function `deposit_with_payer_and_metadata`



<pre><code><b>fun</b> <a href="Account.md#0x1_Account_deposit_with_payer_and_metadata">deposit_with_payer_and_metadata</a>&lt;TokenType&gt;(payer: address, payee: address, to_deposit: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;, metadata: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="Account.md#0x1_Account_deposit_with_payer_and_metadata">deposit_with_payer_and_metadata</a>&lt;TokenType&gt;(
    payer: address,
    payee: address,
    to_deposit: <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt;,
    metadata: vector&lt;u8&gt;,
) <b>acquires</b> <a href="Account.md#0x1_Account">Account</a>, <a href="Account.md#0x1_Account_Balance">Balance</a> {
    // Check that the `to_deposit` token is non-zero
    <b>let</b> deposit_value = <a href="Token.md#0x1_Token_value">Token::value</a>(&to_deposit);
    <b>assert</b>(deposit_value &gt; 0, <a href="ErrorCode.md#0x1_ErrorCode_ECOIN_DEPOSIT_IS_ZERO">ErrorCode::ECOIN_DEPOSIT_IS_ZERO</a>());

    <b>let</b> token_code = <a href="Token.md#0x1_Token_token_code">Token::token_code</a>&lt;TokenType&gt;();

    // Load the payer's account
    <b>let</b> payer_account_ref = borrow_global_mut&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(payer);
    // Log a sent event
    <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>&lt;<a href="Account.md#0x1_Account_SentPaymentEvent">SentPaymentEvent</a>&gt;(
        &<b>mut</b> payer_account_ref.sent_events,
        <a href="Account.md#0x1_Account_SentPaymentEvent">SentPaymentEvent</a> {
            amount: deposit_value,
            token_code: (<b>copy</b> token_code),
            payee: payee,
            metadata: *&metadata
        },
    );

    // Load the payee's account
    <b>let</b> payee_account_ref = borrow_global_mut&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(payee);
    <b>let</b> payee_balance = borrow_global_mut&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(payee);
    // Deposit the `to_deposit` token
    <a href="Token.md#0x1_Token_deposit">Token::deposit</a>(&<b>mut</b> payee_balance.token, to_deposit);
    // Log a received event
    <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>&lt;<a href="Account.md#0x1_Account_ReceivedPaymentEvent">ReceivedPaymentEvent</a>&gt;(
        &<b>mut</b> payee_account_ref.received_events,
        <a href="Account.md#0x1_Account_ReceivedPaymentEvent">ReceivedPaymentEvent</a> {
            amount: deposit_value,
            token_code: token_code,
            payer: payer,
            metadata: metadata
        }
    );
}
</code></pre>



</details>

<a name="0x1_Account_withdraw_from_balance"></a>

## Function `withdraw_from_balance`



<pre><code><b>fun</b> <a href="Account.md#0x1_Account_withdraw_from_balance">withdraw_from_balance</a>&lt;TokenType&gt;(_addr: address, balance: &<b>mut</b> <a href="Account.md#0x1_Account_Balance">Account::Balance</a>&lt;TokenType&gt;, amount: u128): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="Account.md#0x1_Account_withdraw_from_balance">withdraw_from_balance</a>&lt;TokenType&gt;(_addr: address, balance: &<b>mut</b> <a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;, amount: u128): <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt;{
    <a href="Token.md#0x1_Token_withdraw">Token::withdraw</a>(&<b>mut</b> balance.token, amount)
}
</code></pre>



</details>

<a name="0x1_Account_withdraw"></a>

## Function `withdraw`



<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_withdraw">withdraw</a>&lt;TokenType&gt;(account: &signer, amount: u128): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_withdraw">withdraw</a>&lt;TokenType&gt;(account: &signer, amount: u128): <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt;
<b>acquires</b> <a href="Account.md#0x1_Account">Account</a>, <a href="Account.md#0x1_Account_Balance">Balance</a> {
    <b>let</b> sender_addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>let</b> sender_balance = borrow_global_mut&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(sender_addr);
    // The sender_addr has delegated the privilege <b>to</b> withdraw from her account elsewhere--<b>abort</b>.
    <b>assert</b>(!<a href="Account.md#0x1_Account_delegated_withdraw_capability">delegated_withdraw_capability</a>(sender_addr), <a href="Account.md#0x1_Account_EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED">EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED</a>());
    // The sender_addr has retained her withdrawal privileges--proceed.
    <a href="Account.md#0x1_Account_withdraw_from_balance">withdraw_from_balance</a>&lt;TokenType&gt;(sender_addr, sender_balance, amount)
}
</code></pre>



</details>

<a name="0x1_Account_withdraw_with_capability"></a>

## Function `withdraw_with_capability`



<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_withdraw_with_capability">withdraw_with_capability</a>&lt;TokenType&gt;(cap: &<a href="Account.md#0x1_Account_WithdrawCapability">Account::WithdrawCapability</a>, amount: u128): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_withdraw_with_capability">withdraw_with_capability</a>&lt;TokenType&gt;(
    cap: &<a href="Account.md#0x1_Account_WithdrawCapability">WithdrawCapability</a>, amount: u128
): <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt; <b>acquires</b> <a href="Account.md#0x1_Account_Balance">Balance</a> {
    <b>let</b> balance = borrow_global_mut&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(cap.account_address);
    <a href="Account.md#0x1_Account_withdraw_from_balance">withdraw_from_balance</a>&lt;TokenType&gt;(cap.account_address, balance , amount)
}
</code></pre>



</details>

<a name="0x1_Account_extract_withdraw_capability"></a>

## Function `extract_withdraw_capability`



<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_extract_withdraw_capability">extract_withdraw_capability</a>(sender: &signer): <a href="Account.md#0x1_Account_WithdrawCapability">Account::WithdrawCapability</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_extract_withdraw_capability">extract_withdraw_capability</a>(
    sender: &signer
): <a href="Account.md#0x1_Account_WithdrawCapability">WithdrawCapability</a> <b>acquires</b> <a href="Account.md#0x1_Account">Account</a> {
    <b>let</b> sender_addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(sender);
    // Abort <b>if</b> we already extracted the unique withdraw capability for this account.
    <b>assert</b>(!<a href="Account.md#0x1_Account_delegated_withdraw_capability">delegated_withdraw_capability</a>(sender_addr), <a href="Account.md#0x1_Account_EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED">EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED</a>());
    <b>let</b> account = borrow_global_mut&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(sender_addr);
    <a href="Option.md#0x1_Option_extract">Option::extract</a>(&<b>mut</b> account.withdrawal_capability)
}
</code></pre>



</details>

<a name="0x1_Account_restore_withdraw_capability"></a>

## Function `restore_withdraw_capability`



<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_restore_withdraw_capability">restore_withdraw_capability</a>(cap: <a href="Account.md#0x1_Account_WithdrawCapability">Account::WithdrawCapability</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_restore_withdraw_capability">restore_withdraw_capability</a>(cap: <a href="Account.md#0x1_Account_WithdrawCapability">WithdrawCapability</a>)
   <b>acquires</b> <a href="Account.md#0x1_Account">Account</a> {
       <b>let</b> account = borrow_global_mut&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(cap.account_address);
       <a href="Option.md#0x1_Option_fill">Option::fill</a>(&<b>mut</b> account.withdrawal_capability, cap)
}
</code></pre>



</details>

<a name="0x1_Account_pay_from_capability"></a>

## Function `pay_from_capability`



<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_pay_from_capability">pay_from_capability</a>&lt;TokenType&gt;(cap: &<a href="Account.md#0x1_Account_WithdrawCapability">Account::WithdrawCapability</a>, payee: address, amount: u128, metadata: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_pay_from_capability">pay_from_capability</a>&lt;TokenType&gt;(
    cap: &<a href="Account.md#0x1_Account_WithdrawCapability">WithdrawCapability</a>,
    payee: address,
    amount: u128,
    metadata: vector&lt;u8&gt;,
) <b>acquires</b> <a href="Account.md#0x1_Account">Account</a>, <a href="Account.md#0x1_Account_Balance">Balance</a> {
    <a href="Account.md#0x1_Account_deposit_with_payer_and_metadata">deposit_with_payer_and_metadata</a>&lt;TokenType&gt;(
        *&cap.account_address,
        payee,
        <a href="Account.md#0x1_Account_withdraw_with_capability">withdraw_with_capability</a>(cap, amount),
        metadata,
    );
}
</code></pre>



</details>

<a name="0x1_Account_pay_from_with_metadata"></a>

## Function `pay_from_with_metadata`



<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_pay_from_with_metadata">pay_from_with_metadata</a>&lt;TokenType&gt;(account: &signer, payee: address, amount: u128, metadata: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_pay_from_with_metadata">pay_from_with_metadata</a>&lt;TokenType&gt;(
    account: &signer,
    payee: address,
    amount: u128,
    metadata: vector&lt;u8&gt;,
) <b>acquires</b> <a href="Account.md#0x1_Account">Account</a>, <a href="Account.md#0x1_Account_Balance">Balance</a> {
    <a href="Account.md#0x1_Account_deposit_with_metadata">deposit_with_metadata</a>&lt;TokenType&gt;(
        account,
        payee,
        <a href="Account.md#0x1_Account_withdraw">withdraw</a>(account, amount),
        metadata,
    );
}
</code></pre>



</details>

<a name="0x1_Account_pay_from"></a>

## Function `pay_from`



<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_pay_from">pay_from</a>&lt;TokenType&gt;(account: &signer, payee: address, amount: u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_pay_from">pay_from</a>&lt;TokenType&gt;(
    account: &signer,
    payee: address,
    amount: u128
) <b>acquires</b> <a href="Account.md#0x1_Account">Account</a>, <a href="Account.md#0x1_Account_Balance">Balance</a> {
    <a href="Account.md#0x1_Account_pay_from_with_metadata">pay_from_with_metadata</a>&lt;TokenType&gt;(account, payee, amount, x"");
}
</code></pre>



</details>

<a name="0x1_Account_rotate_authentication_key"></a>

## Function `rotate_authentication_key`



<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_rotate_authentication_key">rotate_authentication_key</a>(cap: &<a href="Account.md#0x1_Account_KeyRotationCapability">Account::KeyRotationCapability</a>, new_authentication_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_rotate_authentication_key">rotate_authentication_key</a>(
    cap: &<a href="Account.md#0x1_Account_KeyRotationCapability">KeyRotationCapability</a>,
    new_authentication_key: vector&lt;u8&gt;,
) <b>acquires</b> <a href="Account.md#0x1_Account">Account</a>  {
    <b>let</b> sender_account_resource = borrow_global_mut&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(cap.account_address);
    // Don't allow rotating <b>to</b> clearly invalid key
    <b>assert</b>(<a href="Vector.md#0x1_Vector_length">Vector::length</a>(&new_authentication_key) == 32, <a href="Account.md#0x1_Account_EMALFORMED_AUTHENTICATION_KEY">EMALFORMED_AUTHENTICATION_KEY</a>());
    sender_account_resource.authentication_key = new_authentication_key;
}
</code></pre>



</details>

<a name="0x1_Account_extract_key_rotation_capability"></a>

## Function `extract_key_rotation_capability`



<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_extract_key_rotation_capability">extract_key_rotation_capability</a>(account: &signer): <a href="Account.md#0x1_Account_KeyRotationCapability">Account::KeyRotationCapability</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_extract_key_rotation_capability">extract_key_rotation_capability</a>(account: &signer): <a href="Account.md#0x1_Account_KeyRotationCapability">KeyRotationCapability</a>
<b>acquires</b> <a href="Account.md#0x1_Account">Account</a> {
    <b>let</b> account_address = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    // Abort <b>if</b> we already extracted the unique key rotation capability for this account.
    <b>assert</b>(!<a href="Account.md#0x1_Account_delegated_key_rotation_capability">delegated_key_rotation_capability</a>(account_address), <a href="Account.md#0x1_Account_EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED">EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED</a>());
    <b>let</b> account = borrow_global_mut&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(account_address);
    <a href="Option.md#0x1_Option_extract">Option::extract</a>(&<b>mut</b> account.key_rotation_capability)
}
</code></pre>



</details>

<a name="0x1_Account_restore_key_rotation_capability"></a>

## Function `restore_key_rotation_capability`



<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_restore_key_rotation_capability">restore_key_rotation_capability</a>(cap: <a href="Account.md#0x1_Account_KeyRotationCapability">Account::KeyRotationCapability</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_restore_key_rotation_capability">restore_key_rotation_capability</a>(cap: <a href="Account.md#0x1_Account_KeyRotationCapability">KeyRotationCapability</a>)
<b>acquires</b> <a href="Account.md#0x1_Account">Account</a> {
    <b>let</b> account = borrow_global_mut&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(cap.account_address);
    <a href="Option.md#0x1_Option_fill">Option::fill</a>(&<b>mut</b> account.key_rotation_capability, cap)
}
</code></pre>



</details>

<a name="0x1_Account_balance_for"></a>

## Function `balance_for`



<pre><code><b>fun</b> <a href="Account.md#0x1_Account_balance_for">balance_for</a>&lt;TokenType&gt;(balance: &<a href="Account.md#0x1_Account_Balance">Account::Balance</a>&lt;TokenType&gt;): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="Account.md#0x1_Account_balance_for">balance_for</a>&lt;TokenType&gt;(balance: &<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;): u128 {
    <a href="Token.md#0x1_Token_value">Token::value</a>&lt;TokenType&gt;(&balance.token)
}
</code></pre>



</details>

<a name="0x1_Account_balance"></a>

## Function `balance`



<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_balance">balance</a>&lt;TokenType&gt;(addr: address): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_balance">balance</a>&lt;TokenType&gt;(addr: address): u128 <b>acquires</b> <a href="Account.md#0x1_Account_Balance">Balance</a> {
    <a href="Account.md#0x1_Account_balance_for">balance_for</a>(borrow_global&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(addr))
}
</code></pre>



</details>

<a name="0x1_Account_accept_token"></a>

## Function `accept_token`



<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_accept_token">accept_token</a>&lt;TokenType&gt;(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_accept_token">accept_token</a>&lt;TokenType&gt;(account: &signer) <b>acquires</b> <a href="Account.md#0x1_Account">Account</a> {
    move_to(account, <a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;{ token: <a href="Token.md#0x1_Token_zero">Token::zero</a>&lt;TokenType&gt;() });
    <b>let</b> token_code = <a href="Token.md#0x1_Token_token_code">Token::token_code</a>&lt;TokenType&gt;();
    // Load the sender's account
    <b>let</b> sender_account_ref = borrow_global_mut&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
    // Log a sent event
    <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>&lt;<a href="Account.md#0x1_Account_AcceptTokenEvent">AcceptTokenEvent</a>&gt;(
        &<b>mut</b> sender_account_ref.accept_token_events,
        <a href="Account.md#0x1_Account_AcceptTokenEvent">AcceptTokenEvent</a> {
            token_code:  token_code,
        },
    );
}
</code></pre>



</details>

<a name="0x1_Account_is_accepts_token"></a>

## Function `is_accepts_token`



<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_is_accepts_token">is_accepts_token</a>&lt;TokenType&gt;(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_is_accepts_token">is_accepts_token</a>&lt;TokenType&gt;(addr: address): bool {
    <b>exists</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(addr)
}
</code></pre>



</details>

<a name="0x1_Account_sequence_number_for_account"></a>

## Function `sequence_number_for_account`



<pre><code><b>fun</b> <a href="Account.md#0x1_Account_sequence_number_for_account">sequence_number_for_account</a>(account: &<a href="Account.md#0x1_Account_Account">Account::Account</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="Account.md#0x1_Account_sequence_number_for_account">sequence_number_for_account</a>(account: &<a href="Account.md#0x1_Account">Account</a>): u64 {
    account.sequence_number
}
</code></pre>



</details>

<a name="0x1_Account_sequence_number"></a>

## Function `sequence_number`



<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_sequence_number">sequence_number</a>(addr: address): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_sequence_number">sequence_number</a>(addr: address): u64 <b>acquires</b> <a href="Account.md#0x1_Account">Account</a> {
    <a href="Account.md#0x1_Account_sequence_number_for_account">sequence_number_for_account</a>(borrow_global&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(addr))
}
</code></pre>



</details>

<a name="0x1_Account_authentication_key"></a>

## Function `authentication_key`



<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_authentication_key">authentication_key</a>(addr: address): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_authentication_key">authentication_key</a>(addr: address): vector&lt;u8&gt; <b>acquires</b> <a href="Account.md#0x1_Account">Account</a> {
    *&borrow_global&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(addr).authentication_key
}
</code></pre>



</details>

<a name="0x1_Account_delegated_key_rotation_capability"></a>

## Function `delegated_key_rotation_capability`



<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_delegated_key_rotation_capability">delegated_key_rotation_capability</a>(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_delegated_key_rotation_capability">delegated_key_rotation_capability</a>(addr: address): bool
<b>acquires</b> <a href="Account.md#0x1_Account">Account</a> {
    <a href="Option.md#0x1_Option_is_none">Option::is_none</a>(&borrow_global&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(addr).key_rotation_capability)
}
</code></pre>



</details>

<a name="0x1_Account_delegated_withdraw_capability"></a>

## Function `delegated_withdraw_capability`



<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_delegated_withdraw_capability">delegated_withdraw_capability</a>(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_delegated_withdraw_capability">delegated_withdraw_capability</a>(addr: address): bool
<b>acquires</b> <a href="Account.md#0x1_Account">Account</a> {
    <a href="Option.md#0x1_Option_is_none">Option::is_none</a>(&borrow_global&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(addr).withdrawal_capability)
}
</code></pre>



</details>

<a name="0x1_Account_withdraw_capability_address"></a>

## Function `withdraw_capability_address`



<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_withdraw_capability_address">withdraw_capability_address</a>(cap: &<a href="Account.md#0x1_Account_WithdrawCapability">Account::WithdrawCapability</a>): &address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_withdraw_capability_address">withdraw_capability_address</a>(cap: &<a href="Account.md#0x1_Account_WithdrawCapability">WithdrawCapability</a>): &address {
    &cap.account_address
}
</code></pre>



</details>

<a name="0x1_Account_key_rotation_capability_address"></a>

## Function `key_rotation_capability_address`



<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_key_rotation_capability_address">key_rotation_capability_address</a>(cap: &<a href="Account.md#0x1_Account_KeyRotationCapability">Account::KeyRotationCapability</a>): &address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_key_rotation_capability_address">key_rotation_capability_address</a>(cap: &<a href="Account.md#0x1_Account_KeyRotationCapability">KeyRotationCapability</a>): &address {
    &cap.account_address
}
</code></pre>



</details>

<a name="0x1_Account_exists_at"></a>

## Function `exists_at`



<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_exists_at">exists_at</a>(check_addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_exists_at">exists_at</a>(check_addr: address): bool {
    <b>exists</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(check_addr)
}
</code></pre>



</details>

<a name="0x1_Account_txn_prologue"></a>

## Function `txn_prologue`



<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_txn_prologue">txn_prologue</a>&lt;TokenType&gt;(account: &signer, txn_sender: address, txn_sequence_number: u64, txn_public_key: vector&lt;u8&gt;, txn_gas_price: u64, txn_max_gas_units: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_txn_prologue">txn_prologue</a>&lt;TokenType&gt;(
    account: &signer,
    txn_sender: address,
    txn_sequence_number: u64,
    txn_public_key: vector&lt;u8&gt;,
    txn_gas_price: u64,
    txn_max_gas_units: u64,
) <b>acquires</b> <a href="Account.md#0x1_Account">Account</a>, <a href="Account.md#0x1_Account_Balance">Balance</a> {
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>(), <a href="ErrorCode.md#0x1_ErrorCode_PROLOGUE_ACCOUNT_DOES_NOT_EXIST">ErrorCode::PROLOGUE_ACCOUNT_DOES_NOT_EXIST</a>());

    // FUTURE: Make these error codes sequential
    // Verify that the transaction sender's account <b>exists</b>
    <b>assert</b>(<a href="Account.md#0x1_Account_exists_at">exists_at</a>(txn_sender), <a href="ErrorCode.md#0x1_ErrorCode_PROLOGUE_ACCOUNT_DOES_NOT_EXIST">ErrorCode::PROLOGUE_ACCOUNT_DOES_NOT_EXIST</a>());

    // Load the transaction sender's account
    <b>let</b> sender_account = borrow_global_mut&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(txn_sender);

    // Check that the hash of the transaction's <b>public</b> key matches the account's auth key
    <b>assert</b>(
        <a href="Hash.md#0x1_Hash_sha3_256">Hash::sha3_256</a>(txn_public_key) == *&sender_account.authentication_key,
        <a href="ErrorCode.md#0x1_ErrorCode_PROLOGUE_INVALID_ACCOUNT_AUTH_KEY">ErrorCode::PROLOGUE_INVALID_ACCOUNT_AUTH_KEY</a>()
    );

    // Check that the account has enough balance for all of the gas
    <b>let</b> max_transaction_fee = txn_gas_price * txn_max_gas_units;
    <b>let</b> balance_amount = <a href="Account.md#0x1_Account_balance">balance</a>&lt;TokenType&gt;(txn_sender);
    <b>assert</b>(balance_amount &gt;= (max_transaction_fee <b>as</b> u128), <a href="ErrorCode.md#0x1_ErrorCode_PROLOGUE_CANT_PAY_GAS_DEPOSIT">ErrorCode::PROLOGUE_CANT_PAY_GAS_DEPOSIT</a>());

    // Check that the transaction sequence number matches the sequence number of the account
    <b>assert</b>(txn_sequence_number &gt;= sender_account.sequence_number, <a href="ErrorCode.md#0x1_ErrorCode_PROLOGUE_SEQUENCE_NUMBER_TOO_OLD">ErrorCode::PROLOGUE_SEQUENCE_NUMBER_TOO_OLD</a>());
    <b>assert</b>(txn_sequence_number == sender_account.sequence_number, <a href="ErrorCode.md#0x1_ErrorCode_PROLOGUE_SEQUENCE_NUMBER_TOO_NEW">ErrorCode::PROLOGUE_SEQUENCE_NUMBER_TOO_NEW</a>());
}
</code></pre>



</details>

<a name="0x1_Account_txn_epilogue"></a>

## Function `txn_epilogue`



<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_txn_epilogue">txn_epilogue</a>&lt;TokenType&gt;(account: &signer, txn_sender: address, txn_sequence_number: u64, txn_gas_price: u64, txn_max_gas_units: u64, gas_units_remaining: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_txn_epilogue">txn_epilogue</a>&lt;TokenType&gt;(
    account: &signer,
    txn_sender: address,
    txn_sequence_number: u64,
    txn_gas_price: u64,
    txn_max_gas_units: u64,
    gas_units_remaining: u64,
) <b>acquires</b> <a href="Account.md#0x1_Account">Account</a>, <a href="Account.md#0x1_Account_Balance">Balance</a> {
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>(), <a href="ErrorCode.md#0x1_ErrorCode_ENOT_GENESIS_ACCOUNT">ErrorCode::ENOT_GENESIS_ACCOUNT</a>());

    // Load the transaction sender's account and balance resources
    <b>let</b> sender_account = borrow_global_mut&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(txn_sender);
    <b>let</b> sender_balance = borrow_global_mut&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(txn_sender);

    // Charge for gas
    <b>let</b> transaction_fee_amount =(txn_gas_price * (txn_max_gas_units - gas_units_remaining) <b>as</b> u128);
    <b>assert</b>(
        <a href="Account.md#0x1_Account_balance_for">balance_for</a>(sender_balance) &gt;= transaction_fee_amount,
        <a href="ErrorCode.md#0x1_ErrorCode_EINSUFFICIENT_BALANCE">ErrorCode::EINSUFFICIENT_BALANCE</a>()
    );

    // Bump the sequence number
    sender_account.sequence_number = txn_sequence_number + 1;

    <b>if</b> (transaction_fee_amount &gt; 0) {
        <b>let</b> transaction_fee = <a href="Account.md#0x1_Account_withdraw_from_balance">withdraw_from_balance</a>(
                <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account),
                sender_balance,
                transaction_fee_amount
        );
        <a href="TransactionFee.md#0x1_TransactionFee_pay_fee">TransactionFee::pay_fee</a>(transaction_fee);
    };
}
</code></pre>



</details>

<a name="@Specification_0"></a>

## Specification



<pre><code>pragma verify;
pragma aborts_if_is_strict = <b>true</b>;
</code></pre>



<a name="@Specification_0_create_genesis_account"></a>

### Function `create_genesis_account`


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_create_genesis_account">create_genesis_account</a>(new_account_address: address): signer
</code></pre>




<pre><code><b>aborts_if</b> !<a href="Timestamp.md#0x1_Timestamp_is_genesis">Timestamp::is_genesis</a>();
<b>aborts_if</b> len(<a href="Account.md#0x1_Account_DUMMY_AUTH_KEY">DUMMY_AUTH_KEY</a>) != 32;
<b>aborts_if</b> <b>exists</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(new_account_address);
</code></pre>



<a name="@Specification_0_release_genesis_signer"></a>

### Function `release_genesis_signer`


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_release_genesis_signer">release_genesis_signer</a>(genesis_account: signer)
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="@Specification_0_create_account"></a>

### Function `create_account`


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_create_account">create_account</a>&lt;TokenType&gt;(fresh_address: address, public_key_vec: vector&lt;u8&gt;)
</code></pre>




<pre><code><b>aborts_if</b> len(<a href="Authenticator.md#0x1_Authenticator_spec_ed25519_authentication_key">Authenticator::spec_ed25519_authentication_key</a>(public_key_vec)) != 32;
<b>aborts_if</b> <a href="Authenticator.md#0x1_Authenticator_spec_derived_address">Authenticator::spec_derived_address</a>(<a href="Authenticator.md#0x1_Authenticator_spec_ed25519_authentication_key">Authenticator::spec_ed25519_authentication_key</a>(public_key_vec)) != fresh_address;
<b>aborts_if</b> <b>exists</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(fresh_address);
<b>aborts_if</b> <a href="Token.md#0x1_Token_spec_token_code">Token::spec_token_code</a>&lt;TokenType&gt;() != <a href="Token.md#0x1_Token_spec_token_code">Token::spec_token_code</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;() && <b>exists</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;&gt;(fresh_address);
<b>aborts_if</b> <b>exists</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(fresh_address);
<b>ensures</b> <a href="Account.md#0x1_Account_exists_at">exists_at</a>(fresh_address);
<b>ensures</b> <b>exists</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(fresh_address);
</code></pre>



<a name="@Specification_0_make_account"></a>

### Function `make_account`


<pre><code><b>fun</b> <a href="Account.md#0x1_Account_make_account">make_account</a>(new_account: &signer, authentication_key: vector&lt;u8&gt;)
</code></pre>




<pre><code><b>aborts_if</b> len(authentication_key) != 32;
<b>aborts_if</b> <b>exists</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(new_account));
<b>ensures</b> <a href="Account.md#0x1_Account_exists_at">exists_at</a>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(new_account));
</code></pre>



<a name="@Specification_0_deposit_to"></a>

### Function `deposit_to`


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_deposit_to">deposit_to</a>&lt;TokenType&gt;(account: &signer, payee: address, to_deposit: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;)
</code></pre>




<pre><code><b>include</b> <a href="Account.md#0x1_Account_DepositWithPayerAndMetadataAbortsIf">DepositWithPayerAndMetadataAbortsIf</a>&lt;TokenType&gt;{payer: <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account)};
</code></pre>



<a name="@Specification_0_deposit"></a>

### Function `deposit`


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_deposit">deposit</a>&lt;TokenType&gt;(account: &signer, to_deposit: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;)
</code></pre>




<pre><code><b>aborts_if</b> to_deposit.value == 0;
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
<b>aborts_if</b> <b>exists</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)) && <b>global</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)).token.value + to_deposit.value &gt; max_u128();
<b>ensures</b> <b>exists</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
</code></pre>



<a name="@Specification_0_deposit_with_metadata"></a>

### Function `deposit_with_metadata`


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_deposit_with_metadata">deposit_with_metadata</a>&lt;TokenType&gt;(account: &signer, payee: address, to_deposit: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;, metadata: vector&lt;u8&gt;)
</code></pre>




<pre><code><b>include</b> <a href="Account.md#0x1_Account_DepositWithPayerAndMetadataAbortsIf">DepositWithPayerAndMetadataAbortsIf</a>&lt;TokenType&gt;{payer: <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account)};
</code></pre>



<a name="@Specification_0_deposit_with_payer_and_metadata"></a>

### Function `deposit_with_payer_and_metadata`


<pre><code><b>fun</b> <a href="Account.md#0x1_Account_deposit_with_payer_and_metadata">deposit_with_payer_and_metadata</a>&lt;TokenType&gt;(payer: address, payee: address, to_deposit: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;, metadata: vector&lt;u8&gt;)
</code></pre>




<pre><code><b>include</b> <a href="Account.md#0x1_Account_DepositWithPayerAndMetadataAbortsIf">DepositWithPayerAndMetadataAbortsIf</a>&lt;TokenType&gt;;
</code></pre>




<a name="0x1_Account_DepositWithPayerAndMetadataAbortsIf"></a>


<pre><code><b>schema</b> <a href="Account.md#0x1_Account_DepositWithPayerAndMetadataAbortsIf">DepositWithPayerAndMetadataAbortsIf</a>&lt;TokenType&gt; {
    payer: address;
    payee: address;
    to_deposit: <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt;;
    <b>aborts_if</b> to_deposit.value == 0;
    <b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(payer);
    <b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(payee);
    <b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(payee);
    <b>aborts_if</b> <b>global</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(payee).token.value + to_deposit.value &gt; max_u128();
}
</code></pre>



<a name="@Specification_0_withdraw_from_balance"></a>

### Function `withdraw_from_balance`


<pre><code><b>fun</b> <a href="Account.md#0x1_Account_withdraw_from_balance">withdraw_from_balance</a>&lt;TokenType&gt;(_addr: address, balance: &<b>mut</b> <a href="Account.md#0x1_Account_Balance">Account::Balance</a>&lt;TokenType&gt;, amount: u128): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;
</code></pre>




<pre><code><b>aborts_if</b> balance.token.value &lt; amount;
</code></pre>



<a name="@Specification_0_withdraw"></a>

### Function `withdraw`


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_withdraw">withdraw</a>&lt;TokenType&gt;(account: &signer, amount: u128): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;
</code></pre>




<pre><code>pragma opaque = <b>true</b>;
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
<b>aborts_if</b> <b>global</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account)).token.value &lt; amount;
<b>aborts_if</b> <a href="Option.md#0x1_Option_spec_is_none">Option::spec_is_none</a>(<b>global</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account)).withdrawal_capability);
<b>ensures</b> [abstract] result == <a href="Account.md#0x1_Account_spec_withdraw">spec_withdraw</a>&lt;TokenType&gt;(account, amount);
</code></pre>




<a name="0x1_Account_spec_withdraw"></a>


<pre><code><b>define</b> <a href="Account.md#0x1_Account_spec_withdraw">spec_withdraw</a>&lt;TokenType&gt;(account: signer, amount: u128): <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt; {
<a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt; { value: amount }
}
</code></pre>



<a name="@Specification_0_withdraw_with_capability"></a>

### Function `withdraw_with_capability`


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_withdraw_with_capability">withdraw_with_capability</a>&lt;TokenType&gt;(cap: &<a href="Account.md#0x1_Account_WithdrawCapability">Account::WithdrawCapability</a>, amount: u128): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(cap.account_address);
<b>aborts_if</b> <b>global</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(cap.account_address).token.value &lt; amount;
</code></pre>



<a name="@Specification_0_extract_withdraw_capability"></a>

### Function `extract_withdraw_capability`


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_extract_withdraw_capability">extract_withdraw_capability</a>(sender: &signer): <a href="Account.md#0x1_Account_WithdrawCapability">Account::WithdrawCapability</a>
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(sender));
<b>aborts_if</b> <a href="Option.md#0x1_Option_spec_is_none">Option::spec_is_none</a>(<b>global</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;( <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(sender)).withdrawal_capability);
</code></pre>



<a name="@Specification_0_restore_withdraw_capability"></a>

### Function `restore_withdraw_capability`


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_restore_withdraw_capability">restore_withdraw_capability</a>(cap: <a href="Account.md#0x1_Account_WithdrawCapability">Account::WithdrawCapability</a>)
</code></pre>




<pre><code><b>aborts_if</b> <a href="Option.md#0x1_Option_spec_is_some">Option::spec_is_some</a>(<b>global</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(cap.account_address).withdrawal_capability);
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(cap.account_address);
</code></pre>



<a name="@Specification_0_pay_from_capability"></a>

### Function `pay_from_capability`


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_pay_from_capability">pay_from_capability</a>&lt;TokenType&gt;(cap: &<a href="Account.md#0x1_Account_WithdrawCapability">Account::WithdrawCapability</a>, payee: address, amount: u128, metadata: vector&lt;u8&gt;)
</code></pre>




<pre><code>pragma verify = <b>false</b>;
<b>aborts_if</b> amount == 0;
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(cap.account_address);
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(cap.account_address);
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(payee);
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(payee);
<b>aborts_if</b> <b>global</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(cap.account_address).token.value &lt; amount;
<b>aborts_if</b> <b>global</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(payee).token.value + amount &gt; max_u128();
<b>ensures</b> <b>global</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(payee).token.value == <b>old</b>(<b>global</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(payee).token.value) + amount;
</code></pre>



<a name="@Specification_0_pay_from_with_metadata"></a>

### Function `pay_from_with_metadata`


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_pay_from_with_metadata">pay_from_with_metadata</a>&lt;TokenType&gt;(account: &signer, payee: address, amount: u128, metadata: vector&lt;u8&gt;)
</code></pre>




<pre><code>pragma verify = <b>false</b>;
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
<b>aborts_if</b> <b>global</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account)).token.value &lt; amount;
<b>aborts_if</b> <a href="Option.md#0x1_Option_spec_is_none">Option::spec_is_none</a>(<b>global</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account)).withdrawal_capability);
<b>include</b> <a href="Account.md#0x1_Account_DepositWithPayerAndMetadataAbortsIf">DepositWithPayerAndMetadataAbortsIf</a>&lt;TokenType&gt;{
    payer: <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account),
    payee: payee,
    to_deposit: <a href="Account.md#0x1_Account_spec_withdraw">spec_withdraw</a>&lt;TokenType&gt;(account, amount)
};
</code></pre>



<a name="@Specification_0_pay_from"></a>

### Function `pay_from`


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_pay_from">pay_from</a>&lt;TokenType&gt;(account: &signer, payee: address, amount: u128)
</code></pre>




<pre><code>pragma verify = <b>false</b>;
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
<b>aborts_if</b> <b>global</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account)).token.value &lt; amount;
<b>aborts_if</b> <a href="Option.md#0x1_Option_spec_is_none">Option::spec_is_none</a>(<b>global</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account)).withdrawal_capability);
<b>include</b> <a href="Account.md#0x1_Account_DepositWithPayerAndMetadataAbortsIf">DepositWithPayerAndMetadataAbortsIf</a>&lt;TokenType&gt;{
    payer: <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account),
    to_deposit: <a href="Account.md#0x1_Account_spec_withdraw">spec_withdraw</a>&lt;TokenType&gt;(account, amount)
};
</code></pre>



<a name="@Specification_0_rotate_authentication_key"></a>

### Function `rotate_authentication_key`


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_rotate_authentication_key">rotate_authentication_key</a>(cap: &<a href="Account.md#0x1_Account_KeyRotationCapability">Account::KeyRotationCapability</a>, new_authentication_key: vector&lt;u8&gt;)
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(cap.account_address);
<b>aborts_if</b> len(new_authentication_key) != 32;
<b>ensures</b> <b>global</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(cap.account_address).authentication_key == new_authentication_key;
</code></pre>




<a name="0x1_Account_spec_rotate_authentication_key"></a>


<pre><code><b>define</b> <a href="Account.md#0x1_Account_spec_rotate_authentication_key">spec_rotate_authentication_key</a>(addr: address, new_authentication_key: vector&lt;u8&gt;): bool {
    <b>global</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(addr).authentication_key == new_authentication_key
}
</code></pre>



<a name="@Specification_0_extract_key_rotation_capability"></a>

### Function `extract_key_rotation_capability`


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_extract_key_rotation_capability">extract_key_rotation_capability</a>(account: &signer): <a href="Account.md#0x1_Account_KeyRotationCapability">Account::KeyRotationCapability</a>
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
<b>aborts_if</b> <a href="Option.md#0x1_Option_spec_is_none">Option::spec_is_none</a>(<b>global</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account)).key_rotation_capability);
</code></pre>



<a name="@Specification_0_restore_key_rotation_capability"></a>

### Function `restore_key_rotation_capability`


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_restore_key_rotation_capability">restore_key_rotation_capability</a>(cap: <a href="Account.md#0x1_Account_KeyRotationCapability">Account::KeyRotationCapability</a>)
</code></pre>




<pre><code><b>aborts_if</b> <a href="Option.md#0x1_Option_spec_is_some">Option::spec_is_some</a>(<b>global</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(cap.account_address).key_rotation_capability);
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(cap.account_address);
</code></pre>



<a name="@Specification_0_balance_for"></a>

### Function `balance_for`


<pre><code><b>fun</b> <a href="Account.md#0x1_Account_balance_for">balance_for</a>&lt;TokenType&gt;(balance: &<a href="Account.md#0x1_Account_Balance">Account::Balance</a>&lt;TokenType&gt;): u128
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="@Specification_0_balance"></a>

### Function `balance`


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_balance">balance</a>&lt;TokenType&gt;(addr: address): u128
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(addr);
</code></pre>



<a name="@Specification_0_accept_token"></a>

### Function `accept_token`


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_accept_token">accept_token</a>&lt;TokenType&gt;(account: &signer)
</code></pre>




<pre><code><b>aborts_if</b> <b>exists</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
</code></pre>



<a name="@Specification_0_is_accepts_token"></a>

### Function `is_accepts_token`


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_is_accepts_token">is_accepts_token</a>&lt;TokenType&gt;(addr: address): bool
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="@Specification_0_sequence_number"></a>

### Function `sequence_number`


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_sequence_number">sequence_number</a>(addr: address): u64
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(addr);
</code></pre>



<a name="@Specification_0_authentication_key"></a>

### Function `authentication_key`


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_authentication_key">authentication_key</a>(addr: address): vector&lt;u8&gt;
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(addr);
</code></pre>



<a name="@Specification_0_delegated_key_rotation_capability"></a>

### Function `delegated_key_rotation_capability`


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_delegated_key_rotation_capability">delegated_key_rotation_capability</a>(addr: address): bool
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(addr);
</code></pre>



<a name="@Specification_0_delegated_withdraw_capability"></a>

### Function `delegated_withdraw_capability`


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_delegated_withdraw_capability">delegated_withdraw_capability</a>(addr: address): bool
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(addr);
</code></pre>



<a name="@Specification_0_withdraw_capability_address"></a>

### Function `withdraw_capability_address`


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_withdraw_capability_address">withdraw_capability_address</a>(cap: &<a href="Account.md#0x1_Account_WithdrawCapability">Account::WithdrawCapability</a>): &address
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="@Specification_0_key_rotation_capability_address"></a>

### Function `key_rotation_capability_address`


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_key_rotation_capability_address">key_rotation_capability_address</a>(cap: &<a href="Account.md#0x1_Account_KeyRotationCapability">Account::KeyRotationCapability</a>): &address
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="@Specification_0_exists_at"></a>

### Function `exists_at`


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_exists_at">exists_at</a>(check_addr: address): bool
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="@Specification_0_txn_prologue"></a>

### Function `txn_prologue`


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_txn_prologue">txn_prologue</a>&lt;TokenType&gt;(account: &signer, txn_sender: address, txn_sequence_number: u64, txn_public_key: vector&lt;u8&gt;, txn_gas_price: u64, txn_max_gas_units: u64)
</code></pre>




<pre><code><b>aborts_if</b> <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) != <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>();
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(txn_sender);
<b>aborts_if</b> <a href="Hash.md#0x1_Hash_sha3_256">Hash::sha3_256</a>(txn_public_key) != <b>global</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(txn_sender).authentication_key;
<b>aborts_if</b> txn_gas_price * txn_max_gas_units &gt; max_u64();
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(txn_sender);
<b>aborts_if</b> <b>global</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(txn_sender).token.value &lt; txn_gas_price * txn_max_gas_units;
<b>aborts_if</b> txn_sequence_number &lt; <b>global</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(txn_sender).sequence_number;
<b>aborts_if</b> txn_sequence_number != <b>global</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(txn_sender).sequence_number;
</code></pre>



<a name="@Specification_0_txn_epilogue"></a>

### Function `txn_epilogue`


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_txn_epilogue">txn_epilogue</a>&lt;TokenType&gt;(account: &signer, txn_sender: address, txn_sequence_number: u64, txn_gas_price: u64, txn_max_gas_units: u64, gas_units_remaining: u64)
</code></pre>




<pre><code>pragma verify = <b>false</b>;
<b>aborts_if</b> <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) != <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>();
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(txn_sender);
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(txn_sender);
<b>aborts_if</b> txn_max_gas_units &lt; gas_units_remaining;
<b>aborts_if</b> txn_gas_price * (txn_max_gas_units - gas_units_remaining) &gt; max_u64();
<b>aborts_if</b> <b>global</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(txn_sender).token.value &lt; txn_gas_price * (txn_max_gas_units - gas_units_remaining);
<b>aborts_if</b> txn_sequence_number + 1 &gt; max_u64();
<b>aborts_if</b> txn_gas_price * (txn_max_gas_units - gas_units_remaining) &gt; 0 &&
           !<b>exists</b>&lt;<a href="TransactionFee.md#0x1_TransactionFee_TransactionFee">TransactionFee::TransactionFee</a>&lt;TokenType&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
<b>aborts_if</b> <b>global</b>&lt;<a href="TransactionFee.md#0x1_TransactionFee_TransactionFee">TransactionFee::TransactionFee</a>&lt;TokenType&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>()).fee.value + txn_gas_price * (txn_max_gas_units - gas_units_remaining) &gt; max_u128();
</code></pre>
