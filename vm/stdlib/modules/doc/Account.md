
<a name="0x1_Account"></a>

# Module `0x1::Account`

### Table of Contents

-  [Resource `Account`](#0x1_Account_Account)
-  [Resource `Balance`](#0x1_Account_Balance)
-  [Resource `WithdrawCapability`](#0x1_Account_WithdrawCapability)
-  [Resource `KeyRotationCapability`](#0x1_Account_KeyRotationCapability)
-  [Struct `SentPaymentEvent`](#0x1_Account_SentPaymentEvent)
-  [Struct `ReceivedPaymentEvent`](#0x1_Account_ReceivedPaymentEvent)
-  [Struct `AcceptTokenEvent`](#0x1_Account_AcceptTokenEvent)
-  [Const `DUMMY_AUTH_KEY`](#0x1_Account_DUMMY_AUTH_KEY)
-  [Function `EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED`](#0x1_Account_EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED)
-  [Function `EMALFORMED_AUTHENTICATION_KEY`](#0x1_Account_EMALFORMED_AUTHENTICATION_KEY)
-  [Function `EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED`](#0x1_Account_EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED)
-  [Function `ADDRESS_PUBLIC_KEY_INCONSISTENT`](#0x1_Account_ADDRESS_PUBLIC_KEY_INCONSISTENT)
-  [Function `create_genesis_account`](#0x1_Account_create_genesis_account)
-  [Function `release_genesis_signer`](#0x1_Account_release_genesis_signer)
-  [Function `create_account`](#0x1_Account_create_account)
-  [Function `make_account`](#0x1_Account_make_account)
-  [Function `create_signer`](#0x1_Account_create_signer)
-  [Function `destroy_signer`](#0x1_Account_destroy_signer)
-  [Function `deposit_to`](#0x1_Account_deposit_to)
-  [Function `deposit`](#0x1_Account_deposit)
-  [Function `deposit_with_metadata`](#0x1_Account_deposit_with_metadata)
-  [Function `deposit_with_payer_and_metadata`](#0x1_Account_deposit_with_payer_and_metadata)
-  [Function `withdraw_from_balance`](#0x1_Account_withdraw_from_balance)
-  [Function `withdraw`](#0x1_Account_withdraw)
-  [Function `withdraw_with_capability`](#0x1_Account_withdraw_with_capability)
-  [Function `extract_withdraw_capability`](#0x1_Account_extract_withdraw_capability)
-  [Function `restore_withdraw_capability`](#0x1_Account_restore_withdraw_capability)
-  [Function `pay_from_capability`](#0x1_Account_pay_from_capability)
-  [Function `pay_from_with_metadata`](#0x1_Account_pay_from_with_metadata)
-  [Function `pay_from`](#0x1_Account_pay_from)
-  [Function `rotate_authentication_key_for_account`](#0x1_Account_rotate_authentication_key_for_account)
-  [Function `rotate_authentication_key`](#0x1_Account_rotate_authentication_key)
-  [Function `extract_key_rotation_capability`](#0x1_Account_extract_key_rotation_capability)
-  [Function `restore_key_rotation_capability`](#0x1_Account_restore_key_rotation_capability)
-  [Function `balance_for`](#0x1_Account_balance_for)
-  [Function `balance`](#0x1_Account_balance)
-  [Function `accept_token`](#0x1_Account_accept_token)
-  [Function `is_accepts_token`](#0x1_Account_is_accepts_token)
-  [Function `sequence_number_for_account`](#0x1_Account_sequence_number_for_account)
-  [Function `sequence_number`](#0x1_Account_sequence_number)
-  [Function `authentication_key`](#0x1_Account_authentication_key)
-  [Function `delegated_key_rotation_capability`](#0x1_Account_delegated_key_rotation_capability)
-  [Function `delegated_withdraw_capability`](#0x1_Account_delegated_withdraw_capability)
-  [Function `withdraw_capability_address`](#0x1_Account_withdraw_capability_address)
-  [Function `key_rotation_capability_address`](#0x1_Account_key_rotation_capability_address)
-  [Function `exists_at`](#0x1_Account_exists_at)
-  [Function `txn_prologue`](#0x1_Account_txn_prologue)
-  [Function `txn_epilogue`](#0x1_Account_txn_epilogue)
-  [Specification](#0x1_Account_Specification)
    -  [Function `create_genesis_account`](#0x1_Account_Specification_create_genesis_account)
    -  [Function `release_genesis_signer`](#0x1_Account_Specification_release_genesis_signer)
    -  [Function `create_account`](#0x1_Account_Specification_create_account)
    -  [Function `make_account`](#0x1_Account_Specification_make_account)
    -  [Function `deposit_to`](#0x1_Account_Specification_deposit_to)
    -  [Function `deposit`](#0x1_Account_Specification_deposit)
    -  [Function `deposit_with_metadata`](#0x1_Account_Specification_deposit_with_metadata)
    -  [Function `deposit_with_payer_and_metadata`](#0x1_Account_Specification_deposit_with_payer_and_metadata)
    -  [Function `withdraw_from_balance`](#0x1_Account_Specification_withdraw_from_balance)
    -  [Function `withdraw`](#0x1_Account_Specification_withdraw)
    -  [Function `withdraw_with_capability`](#0x1_Account_Specification_withdraw_with_capability)
    -  [Function `extract_withdraw_capability`](#0x1_Account_Specification_extract_withdraw_capability)
    -  [Function `restore_withdraw_capability`](#0x1_Account_Specification_restore_withdraw_capability)
    -  [Function `pay_from_capability`](#0x1_Account_Specification_pay_from_capability)
    -  [Function `pay_from_with_metadata`](#0x1_Account_Specification_pay_from_with_metadata)
    -  [Function `pay_from`](#0x1_Account_Specification_pay_from)
    -  [Function `rotate_authentication_key_for_account`](#0x1_Account_Specification_rotate_authentication_key_for_account)
    -  [Function `rotate_authentication_key`](#0x1_Account_Specification_rotate_authentication_key)
    -  [Function `extract_key_rotation_capability`](#0x1_Account_Specification_extract_key_rotation_capability)
    -  [Function `restore_key_rotation_capability`](#0x1_Account_Specification_restore_key_rotation_capability)
    -  [Function `balance_for`](#0x1_Account_Specification_balance_for)
    -  [Function `balance`](#0x1_Account_Specification_balance)
    -  [Function `accept_token`](#0x1_Account_Specification_accept_token)
    -  [Function `is_accepts_token`](#0x1_Account_Specification_is_accepts_token)
    -  [Function `sequence_number`](#0x1_Account_Specification_sequence_number)
    -  [Function `authentication_key`](#0x1_Account_Specification_authentication_key)
    -  [Function `delegated_key_rotation_capability`](#0x1_Account_Specification_delegated_key_rotation_capability)
    -  [Function `delegated_withdraw_capability`](#0x1_Account_Specification_delegated_withdraw_capability)
    -  [Function `withdraw_capability_address`](#0x1_Account_Specification_withdraw_capability_address)
    -  [Function `key_rotation_capability_address`](#0x1_Account_Specification_key_rotation_capability_address)
    -  [Function `exists_at`](#0x1_Account_Specification_exists_at)
    -  [Function `txn_prologue`](#0x1_Account_Specification_txn_prologue)
    -  [Function `txn_epilogue`](#0x1_Account_Specification_txn_epilogue)



<a name="0x1_Account_Account"></a>

## Resource `Account`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_Account">Account</a>
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

<code>withdrawal_capability: <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;<a href="#0x1_Account_WithdrawCapability">Account::WithdrawCapability</a>&gt;</code>
</dt>
<dd>

</dd>
<dt>

<code>key_rotation_capability: <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;<a href="#0x1_Account_KeyRotationCapability">Account::KeyRotationCapability</a>&gt;</code>
</dt>
<dd>

</dd>
<dt>

<code>received_events: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="#0x1_Account_ReceivedPaymentEvent">Account::ReceivedPaymentEvent</a>&gt;</code>
</dt>
<dd>

</dd>
<dt>

<code>sent_events: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="#0x1_Account_SentPaymentEvent">Account::SentPaymentEvent</a>&gt;</code>
</dt>
<dd>

</dd>
<dt>

<code>accept_token_events: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="#0x1_Account_AcceptTokenEvent">Account::AcceptTokenEvent</a>&gt;</code>
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



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;
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



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_Account_WithdrawCapability">WithdrawCapability</a>
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



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_Account_KeyRotationCapability">KeyRotationCapability</a>
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



<pre><code><b>struct</b> <a href="#0x1_Account_SentPaymentEvent">SentPaymentEvent</a>
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



<pre><code><b>struct</b> <a href="#0x1_Account_ReceivedPaymentEvent">ReceivedPaymentEvent</a>
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


<pre><code><b>struct</b> <a href="#0x1_Account_AcceptTokenEvent">AcceptTokenEvent</a>
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



<pre><code><b>const</b> DUMMY_AUTH_KEY: vector&lt;u8&gt; = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
</code></pre>



<a name="0x1_Account_EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED"></a>

## Function `EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED`



<pre><code><b>fun</b> <a href="#0x1_Account_EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED">EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_Account_EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED">EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED</a>(): u64 { <a href="ErrorCode.md#0x1_ErrorCode_ECODE_BASE">ErrorCode::ECODE_BASE</a>() + 1}
</code></pre>



</details>

<a name="0x1_Account_EMALFORMED_AUTHENTICATION_KEY"></a>

## Function `EMALFORMED_AUTHENTICATION_KEY`



<pre><code><b>fun</b> <a href="#0x1_Account_EMALFORMED_AUTHENTICATION_KEY">EMALFORMED_AUTHENTICATION_KEY</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_Account_EMALFORMED_AUTHENTICATION_KEY">EMALFORMED_AUTHENTICATION_KEY</a>(): u64 { <a href="ErrorCode.md#0x1_ErrorCode_ECODE_BASE">ErrorCode::ECODE_BASE</a>() + 2}
</code></pre>



</details>

<a name="0x1_Account_EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED"></a>

## Function `EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED`



<pre><code><b>fun</b> <a href="#0x1_Account_EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED">EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_Account_EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED">EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED</a>(): u64 { <a href="ErrorCode.md#0x1_ErrorCode_ECODE_BASE">ErrorCode::ECODE_BASE</a>() + 3}
</code></pre>



</details>

<a name="0x1_Account_ADDRESS_PUBLIC_KEY_INCONSISTENT"></a>

## Function `ADDRESS_PUBLIC_KEY_INCONSISTENT`



<pre><code><b>fun</b> <a href="#0x1_Account_ADDRESS_PUBLIC_KEY_INCONSISTENT">ADDRESS_PUBLIC_KEY_INCONSISTENT</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_Account_ADDRESS_PUBLIC_KEY_INCONSISTENT">ADDRESS_PUBLIC_KEY_INCONSISTENT</a>(): u64 { <a href="ErrorCode.md#0x1_ErrorCode_ECODE_BASE">ErrorCode::ECODE_BASE</a>() + 4}
</code></pre>



</details>

<a name="0x1_Account_create_genesis_account"></a>

## Function `create_genesis_account`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_create_genesis_account">create_genesis_account</a>(new_account_address: address): signer
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_create_genesis_account">create_genesis_account</a>(
    new_account_address: address,
) :signer {
    <b>assert</b>(<a href="Timestamp.md#0x1_Timestamp_is_genesis">Timestamp::is_genesis</a>(), <a href="ErrorCode.md#0x1_ErrorCode_ENOT_GENESIS">ErrorCode::ENOT_GENESIS</a>());
    <b>let</b> new_account = <a href="#0x1_Account_create_signer">create_signer</a>(new_account_address);
    <a href="#0x1_Account_make_account">make_account</a>(&new_account, DUMMY_AUTH_KEY);
    new_account
}
</code></pre>



</details>

<a name="0x1_Account_release_genesis_signer"></a>

## Function `release_genesis_signer`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_release_genesis_signer">release_genesis_signer</a>(genesis_account: signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_release_genesis_signer">release_genesis_signer</a>(genesis_account: signer){
    <a href="#0x1_Account_destroy_signer">destroy_signer</a>(genesis_account);
}
</code></pre>



</details>

<a name="0x1_Account_create_account"></a>

## Function `create_account`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_create_account">create_account</a>&lt;TokenType&gt;(fresh_address: address, public_key_vec: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_create_account">create_account</a>&lt;TokenType&gt;(fresh_address: address, public_key_vec: vector&lt;u8&gt;) <b>acquires</b> <a href="#0x1_Account">Account</a> {
    <b>let</b> authentication_key = <a href="Authenticator.md#0x1_Authenticator_ed25519_authentication_key">Authenticator::ed25519_authentication_key</a>(public_key_vec);
    <b>let</b> new_address = <a href="Authenticator.md#0x1_Authenticator_derived_address">Authenticator::derived_address</a>(<b>copy</b> authentication_key);
    <b>assert</b>(new_address == fresh_address, <a href="#0x1_Account_ADDRESS_PUBLIC_KEY_INCONSISTENT">ADDRESS_PUBLIC_KEY_INCONSISTENT</a>());

    <b>let</b> new_account = <a href="#0x1_Account_create_signer">create_signer</a>(new_address);
    <a href="#0x1_Account_make_account">make_account</a>(&new_account, authentication_key);
    // Make sure all account accept <a href="STC.md#0x1_STC">STC</a>.
    <b>if</b> (!<a href="STC.md#0x1_STC_is_stc">STC::is_stc</a>&lt;TokenType&gt;()){
        <a href="#0x1_Account_accept_token">accept_token</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;(&new_account);
    };
    <a href="#0x1_Account_accept_token">accept_token</a>&lt;TokenType&gt;(&new_account);
    <a href="#0x1_Account_destroy_signer">destroy_signer</a>(new_account);
}
</code></pre>



</details>

<a name="0x1_Account_make_account"></a>

## Function `make_account`



<pre><code><b>fun</b> <a href="#0x1_Account_make_account">make_account</a>(new_account: &signer, authentication_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_Account_make_account">make_account</a>(
    new_account: &signer,
    authentication_key: vector&lt;u8&gt;,
) {
    <b>assert</b>(<a href="Vector.md#0x1_Vector_length">Vector::length</a>(&authentication_key) == 32, 88888);
    <b>let</b> new_account_addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(new_account);
    <a href="Event.md#0x1_Event_publish_generator">Event::publish_generator</a>(new_account);
    move_to(new_account, <a href="#0x1_Account">Account</a> {
          authentication_key,
          withdrawal_capability: <a href="Option.md#0x1_Option_some">Option::some</a>(
              <a href="#0x1_Account_WithdrawCapability">WithdrawCapability</a> {
                  account_address: new_account_addr
          }),
          key_rotation_capability: <a href="Option.md#0x1_Option_some">Option::some</a>(
              <a href="#0x1_Account_KeyRotationCapability">KeyRotationCapability</a> {
                  account_address: new_account_addr
          }),
          received_events: <a href="Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="#0x1_Account_ReceivedPaymentEvent">ReceivedPaymentEvent</a>&gt;(new_account),
          sent_events: <a href="Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="#0x1_Account_SentPaymentEvent">SentPaymentEvent</a>&gt;(new_account),
          accept_token_events: <a href="Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="#0x1_Account_AcceptTokenEvent">AcceptTokenEvent</a>&gt;(new_account),
          sequence_number: 0,
    });
}
</code></pre>



</details>

<a name="0x1_Account_create_signer"></a>

## Function `create_signer`



<pre><code><b>fun</b> <a href="#0x1_Account_create_signer">create_signer</a>(addr: address): signer
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>fun</b> <a href="#0x1_Account_create_signer">create_signer</a>(addr: address): signer;
</code></pre>



</details>

<a name="0x1_Account_destroy_signer"></a>

## Function `destroy_signer`



<pre><code><b>fun</b> <a href="#0x1_Account_destroy_signer">destroy_signer</a>(sig: signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>fun</b> <a href="#0x1_Account_destroy_signer">destroy_signer</a>(sig: signer);
</code></pre>



</details>

<a name="0x1_Account_deposit_to"></a>

## Function `deposit_to`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_deposit_to">deposit_to</a>&lt;TokenType&gt;(account: &signer, payee: address, to_deposit: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_deposit_to">deposit_to</a>&lt;TokenType&gt;(account: &signer, payee: address, to_deposit: <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt;)
<b>acquires</b> <a href="#0x1_Account">Account</a>, <a href="#0x1_Account_Balance">Balance</a> {
    <a href="#0x1_Account_deposit_with_metadata">deposit_with_metadata</a>(account, payee, to_deposit, x"")
}
</code></pre>



</details>

<a name="0x1_Account_deposit"></a>

## Function `deposit`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_deposit">deposit</a>&lt;TokenType&gt;(account: &signer, to_deposit: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_deposit">deposit</a>&lt;TokenType&gt;(account: &signer, to_deposit: <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt;)
<b>acquires</b> <a href="#0x1_Account">Account</a>, <a href="#0x1_Account_Balance">Balance</a> {
    <b>let</b> account_address = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>if</b> (!<a href="#0x1_Account_is_accepts_token">is_accepts_token</a>&lt;TokenType&gt;(account_address)){
        <a href="#0x1_Account_accept_token">accept_token</a>&lt;TokenType&gt;(account);
    };
    <a href="#0x1_Account_deposit_to">deposit_to</a>(account, account_address, to_deposit)
}
</code></pre>



</details>

<a name="0x1_Account_deposit_with_metadata"></a>

## Function `deposit_with_metadata`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_deposit_with_metadata">deposit_with_metadata</a>&lt;TokenType&gt;(account: &signer, payee: address, to_deposit: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;, metadata: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_deposit_with_metadata">deposit_with_metadata</a>&lt;TokenType&gt;(account: &signer,
    payee: address,
    to_deposit: <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt;,
    metadata: vector&lt;u8&gt;,
) <b>acquires</b> <a href="#0x1_Account">Account</a>, <a href="#0x1_Account_Balance">Balance</a> {
    <a href="#0x1_Account_deposit_with_payer_and_metadata">deposit_with_payer_and_metadata</a>(
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



<pre><code><b>fun</b> <a href="#0x1_Account_deposit_with_payer_and_metadata">deposit_with_payer_and_metadata</a>&lt;TokenType&gt;(payer: address, payee: address, to_deposit: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;, metadata: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_Account_deposit_with_payer_and_metadata">deposit_with_payer_and_metadata</a>&lt;TokenType&gt;(
    payer: address,
    payee: address,
    to_deposit: <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt;,
    metadata: vector&lt;u8&gt;,
) <b>acquires</b> <a href="#0x1_Account">Account</a>, <a href="#0x1_Account_Balance">Balance</a> {
    // Check that the `to_deposit` token is non-zero
    <b>let</b> deposit_value = <a href="Token.md#0x1_Token_value">Token::value</a>(&to_deposit);
    <b>assert</b>(deposit_value &gt; 0, <a href="ErrorCode.md#0x1_ErrorCode_ECOIN_DEPOSIT_IS_ZERO">ErrorCode::ECOIN_DEPOSIT_IS_ZERO</a>());

    <b>let</b> token_code = <a href="Token.md#0x1_Token_token_code">Token::token_code</a>&lt;TokenType&gt;();

    // Load the payer's account
    <b>let</b> payer_account_ref = borrow_global_mut&lt;<a href="#0x1_Account">Account</a>&gt;(payer);
    // Log a sent event
    <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>&lt;<a href="#0x1_Account_SentPaymentEvent">SentPaymentEvent</a>&gt;(
        &<b>mut</b> payer_account_ref.sent_events,
        <a href="#0x1_Account_SentPaymentEvent">SentPaymentEvent</a> {
            amount: deposit_value,
            token_code: (<b>copy</b> token_code),
            payee: payee,
            metadata: *&metadata
        },
    );

    // Load the payee's account
    <b>let</b> payee_account_ref = borrow_global_mut&lt;<a href="#0x1_Account">Account</a>&gt;(payee);
    <b>let</b> payee_balance = borrow_global_mut&lt;<a href="#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(payee);
    // Deposit the `to_deposit` token
    <a href="Token.md#0x1_Token_deposit">Token::deposit</a>(&<b>mut</b> payee_balance.token, to_deposit);
    // Log a received event
    <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>&lt;<a href="#0x1_Account_ReceivedPaymentEvent">ReceivedPaymentEvent</a>&gt;(
        &<b>mut</b> payee_account_ref.received_events,
        <a href="#0x1_Account_ReceivedPaymentEvent">ReceivedPaymentEvent</a> {
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



<pre><code><b>fun</b> <a href="#0x1_Account_withdraw_from_balance">withdraw_from_balance</a>&lt;TokenType&gt;(_addr: address, balance: &<b>mut</b> <a href="#0x1_Account_Balance">Account::Balance</a>&lt;TokenType&gt;, amount: u128): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_Account_withdraw_from_balance">withdraw_from_balance</a>&lt;TokenType&gt;(_addr: address, balance: &<b>mut</b> <a href="#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;, amount: u128): <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt;{
    <a href="Token.md#0x1_Token_withdraw">Token::withdraw</a>(&<b>mut</b> balance.token, amount)
}
</code></pre>



</details>

<a name="0x1_Account_withdraw"></a>

## Function `withdraw`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_withdraw">withdraw</a>&lt;TokenType&gt;(account: &signer, amount: u128): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_withdraw">withdraw</a>&lt;TokenType&gt;(account: &signer, amount: u128): <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt;
<b>acquires</b> <a href="#0x1_Account">Account</a>, <a href="#0x1_Account_Balance">Balance</a> {
    <b>let</b> sender_addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>let</b> sender_balance = borrow_global_mut&lt;<a href="#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(sender_addr);
    // The sender_addr has delegated the privilege <b>to</b> withdraw from her account elsewhere--<b>abort</b>.
    <b>assert</b>(!<a href="#0x1_Account_delegated_withdraw_capability">delegated_withdraw_capability</a>(sender_addr), <a href="#0x1_Account_EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED">EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED</a>());
    // The sender_addr has retained her withdrawal privileges--proceed.
    <a href="#0x1_Account_withdraw_from_balance">withdraw_from_balance</a>&lt;TokenType&gt;(sender_addr, sender_balance, amount)
}
</code></pre>



</details>

<a name="0x1_Account_withdraw_with_capability"></a>

## Function `withdraw_with_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_withdraw_with_capability">withdraw_with_capability</a>&lt;TokenType&gt;(cap: &<a href="#0x1_Account_WithdrawCapability">Account::WithdrawCapability</a>, amount: u128): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_withdraw_with_capability">withdraw_with_capability</a>&lt;TokenType&gt;(
    cap: &<a href="#0x1_Account_WithdrawCapability">WithdrawCapability</a>, amount: u128
): <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt; <b>acquires</b> <a href="#0x1_Account_Balance">Balance</a> {
    <b>let</b> balance = borrow_global_mut&lt;<a href="#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(cap.account_address);
    <a href="#0x1_Account_withdraw_from_balance">withdraw_from_balance</a>&lt;TokenType&gt;(cap.account_address, balance , amount)
}
</code></pre>



</details>

<a name="0x1_Account_extract_withdraw_capability"></a>

## Function `extract_withdraw_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_extract_withdraw_capability">extract_withdraw_capability</a>(sender: &signer): <a href="#0x1_Account_WithdrawCapability">Account::WithdrawCapability</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_extract_withdraw_capability">extract_withdraw_capability</a>(
    sender: &signer
): <a href="#0x1_Account_WithdrawCapability">WithdrawCapability</a> <b>acquires</b> <a href="#0x1_Account">Account</a> {
    <b>let</b> sender_addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(sender);
    // Abort <b>if</b> we already extracted the unique withdraw capability for this account.
    <b>assert</b>(!<a href="#0x1_Account_delegated_withdraw_capability">delegated_withdraw_capability</a>(sender_addr), <a href="#0x1_Account_EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED">EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED</a>());
    <b>let</b> account = borrow_global_mut&lt;<a href="#0x1_Account">Account</a>&gt;(sender_addr);
    <a href="Option.md#0x1_Option_extract">Option::extract</a>(&<b>mut</b> account.withdrawal_capability)
}
</code></pre>



</details>

<a name="0x1_Account_restore_withdraw_capability"></a>

## Function `restore_withdraw_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_restore_withdraw_capability">restore_withdraw_capability</a>(cap: <a href="#0x1_Account_WithdrawCapability">Account::WithdrawCapability</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_restore_withdraw_capability">restore_withdraw_capability</a>(cap: <a href="#0x1_Account_WithdrawCapability">WithdrawCapability</a>)
   <b>acquires</b> <a href="#0x1_Account">Account</a> {
       <b>let</b> account = borrow_global_mut&lt;<a href="#0x1_Account">Account</a>&gt;(cap.account_address);
       <a href="Option.md#0x1_Option_fill">Option::fill</a>(&<b>mut</b> account.withdrawal_capability, cap)
}
</code></pre>



</details>

<a name="0x1_Account_pay_from_capability"></a>

## Function `pay_from_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_pay_from_capability">pay_from_capability</a>&lt;TokenType&gt;(cap: &<a href="#0x1_Account_WithdrawCapability">Account::WithdrawCapability</a>, payee: address, amount: u128, metadata: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_pay_from_capability">pay_from_capability</a>&lt;TokenType&gt;(
    cap: &<a href="#0x1_Account_WithdrawCapability">WithdrawCapability</a>,
    payee: address,
    amount: u128,
    metadata: vector&lt;u8&gt;,
) <b>acquires</b> <a href="#0x1_Account">Account</a>, <a href="#0x1_Account_Balance">Balance</a> {
    <a href="#0x1_Account_deposit_with_payer_and_metadata">deposit_with_payer_and_metadata</a>&lt;TokenType&gt;(
        *&cap.account_address,
        payee,
        <a href="#0x1_Account_withdraw_with_capability">withdraw_with_capability</a>(cap, amount),
        metadata,
    );
}
</code></pre>



</details>

<a name="0x1_Account_pay_from_with_metadata"></a>

## Function `pay_from_with_metadata`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_pay_from_with_metadata">pay_from_with_metadata</a>&lt;TokenType&gt;(account: &signer, payee: address, amount: u128, metadata: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_pay_from_with_metadata">pay_from_with_metadata</a>&lt;TokenType&gt;(
    account: &signer,
    payee: address,
    amount: u128,
    metadata: vector&lt;u8&gt;,
) <b>acquires</b> <a href="#0x1_Account">Account</a>, <a href="#0x1_Account_Balance">Balance</a> {
    <a href="#0x1_Account_deposit_with_metadata">deposit_with_metadata</a>&lt;TokenType&gt;(
        account,
        payee,
        <a href="#0x1_Account_withdraw">withdraw</a>(account, amount),
        metadata,
    );
}
</code></pre>



</details>

<a name="0x1_Account_pay_from"></a>

## Function `pay_from`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_pay_from">pay_from</a>&lt;TokenType&gt;(account: &signer, payee: address, amount: u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_pay_from">pay_from</a>&lt;TokenType&gt;(
    account: &signer,
    payee: address,
    amount: u128
) <b>acquires</b> <a href="#0x1_Account">Account</a>, <a href="#0x1_Account_Balance">Balance</a> {
    <a href="#0x1_Account_pay_from_with_metadata">pay_from_with_metadata</a>&lt;TokenType&gt;(account, payee, amount, x"");
}
</code></pre>



</details>

<a name="0x1_Account_rotate_authentication_key_for_account"></a>

## Function `rotate_authentication_key_for_account`



<pre><code><b>fun</b> <a href="#0x1_Account_rotate_authentication_key_for_account">rotate_authentication_key_for_account</a>(account: &<b>mut</b> <a href="#0x1_Account_Account">Account::Account</a>, new_authentication_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_Account_rotate_authentication_key_for_account">rotate_authentication_key_for_account</a>(account: &<b>mut</b> <a href="#0x1_Account">Account</a>, new_authentication_key: vector&lt;u8&gt;) {
  // Don't allow rotating <b>to</b> clearly invalid key
  <b>assert</b>(<a href="Vector.md#0x1_Vector_length">Vector::length</a>(&new_authentication_key) == 32, <a href="#0x1_Account_EMALFORMED_AUTHENTICATION_KEY">EMALFORMED_AUTHENTICATION_KEY</a>());
  account.authentication_key = new_authentication_key;
}
</code></pre>



</details>

<a name="0x1_Account_rotate_authentication_key"></a>

## Function `rotate_authentication_key`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_rotate_authentication_key">rotate_authentication_key</a>(cap: &<a href="#0x1_Account_KeyRotationCapability">Account::KeyRotationCapability</a>, new_authentication_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_rotate_authentication_key">rotate_authentication_key</a>(
    cap: &<a href="#0x1_Account_KeyRotationCapability">KeyRotationCapability</a>,
    new_authentication_key: vector&lt;u8&gt;,
) <b>acquires</b> <a href="#0x1_Account">Account</a>  {
    <b>let</b> sender_account_resource = borrow_global_mut&lt;<a href="#0x1_Account">Account</a>&gt;(cap.account_address);
    // Don't allow rotating <b>to</b> clearly invalid key
    <b>assert</b>(<a href="Vector.md#0x1_Vector_length">Vector::length</a>(&new_authentication_key) == 32, <a href="#0x1_Account_EMALFORMED_AUTHENTICATION_KEY">EMALFORMED_AUTHENTICATION_KEY</a>());
    sender_account_resource.authentication_key = new_authentication_key;
}
</code></pre>



</details>

<a name="0x1_Account_extract_key_rotation_capability"></a>

## Function `extract_key_rotation_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_extract_key_rotation_capability">extract_key_rotation_capability</a>(account: &signer): <a href="#0x1_Account_KeyRotationCapability">Account::KeyRotationCapability</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_extract_key_rotation_capability">extract_key_rotation_capability</a>(account: &signer): <a href="#0x1_Account_KeyRotationCapability">KeyRotationCapability</a>
<b>acquires</b> <a href="#0x1_Account">Account</a> {
    <b>let</b> account_address = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    // Abort <b>if</b> we already extracted the unique key rotation capability for this account.
    <b>assert</b>(!<a href="#0x1_Account_delegated_key_rotation_capability">delegated_key_rotation_capability</a>(account_address), <a href="#0x1_Account_EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED">EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED</a>());
    <b>let</b> account = borrow_global_mut&lt;<a href="#0x1_Account">Account</a>&gt;(account_address);
    <a href="Option.md#0x1_Option_extract">Option::extract</a>(&<b>mut</b> account.key_rotation_capability)
}
</code></pre>



</details>

<a name="0x1_Account_restore_key_rotation_capability"></a>

## Function `restore_key_rotation_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_restore_key_rotation_capability">restore_key_rotation_capability</a>(cap: <a href="#0x1_Account_KeyRotationCapability">Account::KeyRotationCapability</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_restore_key_rotation_capability">restore_key_rotation_capability</a>(cap: <a href="#0x1_Account_KeyRotationCapability">KeyRotationCapability</a>)
<b>acquires</b> <a href="#0x1_Account">Account</a> {
    <b>let</b> account = borrow_global_mut&lt;<a href="#0x1_Account">Account</a>&gt;(cap.account_address);
    <a href="Option.md#0x1_Option_fill">Option::fill</a>(&<b>mut</b> account.key_rotation_capability, cap)
}
</code></pre>



</details>

<a name="0x1_Account_balance_for"></a>

## Function `balance_for`



<pre><code><b>fun</b> <a href="#0x1_Account_balance_for">balance_for</a>&lt;TokenType&gt;(balance: &<a href="#0x1_Account_Balance">Account::Balance</a>&lt;TokenType&gt;): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_Account_balance_for">balance_for</a>&lt;TokenType&gt;(balance: &<a href="#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;): u128 {
    <a href="Token.md#0x1_Token_value">Token::value</a>&lt;TokenType&gt;(&balance.token)
}
</code></pre>



</details>

<a name="0x1_Account_balance"></a>

## Function `balance`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_balance">balance</a>&lt;TokenType&gt;(addr: address): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_balance">balance</a>&lt;TokenType&gt;(addr: address): u128 <b>acquires</b> <a href="#0x1_Account_Balance">Balance</a> {
    <a href="#0x1_Account_balance_for">balance_for</a>(borrow_global&lt;<a href="#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(addr))
}
</code></pre>



</details>

<a name="0x1_Account_accept_token"></a>

## Function `accept_token`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_accept_token">accept_token</a>&lt;TokenType&gt;(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_accept_token">accept_token</a>&lt;TokenType&gt;(account: &signer) <b>acquires</b> <a href="#0x1_Account">Account</a> {
    move_to(account, <a href="#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;{ token: <a href="Token.md#0x1_Token_zero">Token::zero</a>&lt;TokenType&gt;() });
    <b>let</b> token_code = <a href="Token.md#0x1_Token_token_code">Token::token_code</a>&lt;TokenType&gt;();
    // Load the sender's account
    <b>let</b> sender_account_ref = borrow_global_mut&lt;<a href="#0x1_Account">Account</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
    // Log a sent event
    <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>&lt;<a href="#0x1_Account_AcceptTokenEvent">AcceptTokenEvent</a>&gt;(
        &<b>mut</b> sender_account_ref.accept_token_events,
        <a href="#0x1_Account_AcceptTokenEvent">AcceptTokenEvent</a> {
            token_code:  token_code,
        },
    );
}
</code></pre>



</details>

<a name="0x1_Account_is_accepts_token"></a>

## Function `is_accepts_token`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_is_accepts_token">is_accepts_token</a>&lt;TokenType&gt;(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_is_accepts_token">is_accepts_token</a>&lt;TokenType&gt;(addr: address): bool {
    exists&lt;<a href="#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(addr)
}
</code></pre>



</details>

<a name="0x1_Account_sequence_number_for_account"></a>

## Function `sequence_number_for_account`



<pre><code><b>fun</b> <a href="#0x1_Account_sequence_number_for_account">sequence_number_for_account</a>(account: &<a href="#0x1_Account_Account">Account::Account</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_Account_sequence_number_for_account">sequence_number_for_account</a>(account: &<a href="#0x1_Account">Account</a>): u64 {
    account.sequence_number
}
</code></pre>



</details>

<a name="0x1_Account_sequence_number"></a>

## Function `sequence_number`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_sequence_number">sequence_number</a>(addr: address): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_sequence_number">sequence_number</a>(addr: address): u64 <b>acquires</b> <a href="#0x1_Account">Account</a> {
    <a href="#0x1_Account_sequence_number_for_account">sequence_number_for_account</a>(borrow_global&lt;<a href="#0x1_Account">Account</a>&gt;(addr))
}
</code></pre>



</details>

<a name="0x1_Account_authentication_key"></a>

## Function `authentication_key`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_authentication_key">authentication_key</a>(addr: address): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_authentication_key">authentication_key</a>(addr: address): vector&lt;u8&gt; <b>acquires</b> <a href="#0x1_Account">Account</a> {
    *&borrow_global&lt;<a href="#0x1_Account">Account</a>&gt;(addr).authentication_key
}
</code></pre>



</details>

<a name="0x1_Account_delegated_key_rotation_capability"></a>

## Function `delegated_key_rotation_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_delegated_key_rotation_capability">delegated_key_rotation_capability</a>(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_delegated_key_rotation_capability">delegated_key_rotation_capability</a>(addr: address): bool
<b>acquires</b> <a href="#0x1_Account">Account</a> {
    <a href="Option.md#0x1_Option_is_none">Option::is_none</a>(&borrow_global&lt;<a href="#0x1_Account">Account</a>&gt;(addr).key_rotation_capability)
}
</code></pre>



</details>

<a name="0x1_Account_delegated_withdraw_capability"></a>

## Function `delegated_withdraw_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_delegated_withdraw_capability">delegated_withdraw_capability</a>(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_delegated_withdraw_capability">delegated_withdraw_capability</a>(addr: address): bool
<b>acquires</b> <a href="#0x1_Account">Account</a> {
    <a href="Option.md#0x1_Option_is_none">Option::is_none</a>(&borrow_global&lt;<a href="#0x1_Account">Account</a>&gt;(addr).withdrawal_capability)
}
</code></pre>



</details>

<a name="0x1_Account_withdraw_capability_address"></a>

## Function `withdraw_capability_address`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_withdraw_capability_address">withdraw_capability_address</a>(cap: &<a href="#0x1_Account_WithdrawCapability">Account::WithdrawCapability</a>): &address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_withdraw_capability_address">withdraw_capability_address</a>(cap: &<a href="#0x1_Account_WithdrawCapability">WithdrawCapability</a>): &address {
    &cap.account_address
}
</code></pre>



</details>

<a name="0x1_Account_key_rotation_capability_address"></a>

## Function `key_rotation_capability_address`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_key_rotation_capability_address">key_rotation_capability_address</a>(cap: &<a href="#0x1_Account_KeyRotationCapability">Account::KeyRotationCapability</a>): &address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_key_rotation_capability_address">key_rotation_capability_address</a>(cap: &<a href="#0x1_Account_KeyRotationCapability">KeyRotationCapability</a>): &address {
    &cap.account_address
}
</code></pre>



</details>

<a name="0x1_Account_exists_at"></a>

## Function `exists_at`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_exists_at">exists_at</a>(check_addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_exists_at">exists_at</a>(check_addr: address): bool {
    exists&lt;<a href="#0x1_Account">Account</a>&gt;(check_addr)
}
</code></pre>



</details>

<a name="0x1_Account_txn_prologue"></a>

## Function `txn_prologue`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_txn_prologue">txn_prologue</a>&lt;TokenType&gt;(account: &signer, txn_sender: address, txn_sequence_number: u64, txn_public_key: vector&lt;u8&gt;, txn_gas_price: u64, txn_max_gas_units: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_txn_prologue">txn_prologue</a>&lt;TokenType&gt;(
    account: &signer,
    txn_sender: address,
    txn_sequence_number: u64,
    txn_public_key: vector&lt;u8&gt;,
    txn_gas_price: u64,
    txn_max_gas_units: u64,
) <b>acquires</b> <a href="#0x1_Account">Account</a>, <a href="#0x1_Account_Balance">Balance</a> {
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>(), <a href="ErrorCode.md#0x1_ErrorCode_PROLOGUE_ACCOUNT_DOES_NOT_EXIST">ErrorCode::PROLOGUE_ACCOUNT_DOES_NOT_EXIST</a>());

    // FUTURE: Make these error codes sequential
    // Verify that the transaction sender's account exists
    <b>assert</b>(<a href="#0x1_Account_exists_at">exists_at</a>(txn_sender), <a href="ErrorCode.md#0x1_ErrorCode_PROLOGUE_ACCOUNT_DOES_NOT_EXIST">ErrorCode::PROLOGUE_ACCOUNT_DOES_NOT_EXIST</a>());

    // Load the transaction sender's account
    <b>let</b> sender_account = borrow_global_mut&lt;<a href="#0x1_Account">Account</a>&gt;(txn_sender);

    // Check that the hash of the transaction's <b>public</b> key matches the account's auth key
    <b>assert</b>(
        <a href="Hash.md#0x1_Hash_sha3_256">Hash::sha3_256</a>(txn_public_key) == *&sender_account.authentication_key,
        <a href="ErrorCode.md#0x1_ErrorCode_PROLOGUE_INVALID_ACCOUNT_AUTH_KEY">ErrorCode::PROLOGUE_INVALID_ACCOUNT_AUTH_KEY</a>()
    );

    // Check that the account has enough balance for all of the gas
    <b>let</b> max_transaction_fee = txn_gas_price * txn_max_gas_units;
    <b>let</b> balance_amount = <a href="#0x1_Account_balance">balance</a>&lt;TokenType&gt;(txn_sender);
    <b>assert</b>(balance_amount &gt;= (max_transaction_fee <b>as</b> u128), <a href="ErrorCode.md#0x1_ErrorCode_PROLOGUE_CANT_PAY_GAS_DEPOSIT">ErrorCode::PROLOGUE_CANT_PAY_GAS_DEPOSIT</a>());

    // Check that the transaction sequence number matches the sequence number of the account
    <b>assert</b>(txn_sequence_number &gt;= sender_account.sequence_number, <a href="ErrorCode.md#0x1_ErrorCode_PROLOGUE_SEQUENCE_NUMBER_TOO_OLD">ErrorCode::PROLOGUE_SEQUENCE_NUMBER_TOO_OLD</a>());
    <b>assert</b>(txn_sequence_number == sender_account.sequence_number, <a href="ErrorCode.md#0x1_ErrorCode_PROLOGUE_SEQUENCE_NUMBER_TOO_NEW">ErrorCode::PROLOGUE_SEQUENCE_NUMBER_TOO_NEW</a>());
}
</code></pre>



</details>

<a name="0x1_Account_txn_epilogue"></a>

## Function `txn_epilogue`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_txn_epilogue">txn_epilogue</a>&lt;TokenType&gt;(account: &signer, txn_sender: address, txn_sequence_number: u64, txn_gas_price: u64, txn_max_gas_units: u64, gas_units_remaining: u64, _state_cost_amount: u64, _cost_is_negative: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_txn_epilogue">txn_epilogue</a>&lt;TokenType&gt;(
    account: &signer,
    txn_sender: address,
    txn_sequence_number: u64,
    txn_gas_price: u64,
    txn_max_gas_units: u64,
    gas_units_remaining: u64,
    _state_cost_amount: u64,
    _cost_is_negative: bool,
) <b>acquires</b> <a href="#0x1_Account">Account</a>, <a href="#0x1_Account_Balance">Balance</a> {
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>(), <a href="ErrorCode.md#0x1_ErrorCode_ENOT_GENESIS_ACCOUNT">ErrorCode::ENOT_GENESIS_ACCOUNT</a>());

    // Load the transaction sender's account and balance resources
    <b>let</b> sender_account = borrow_global_mut&lt;<a href="#0x1_Account">Account</a>&gt;(txn_sender);
    <b>let</b> sender_balance = borrow_global_mut&lt;<a href="#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(txn_sender);

    // Charge for gas
    <b>let</b> transaction_fee_amount =(txn_gas_price * (txn_max_gas_units - gas_units_remaining) <b>as</b> u128);
    <b>assert</b>(
        <a href="#0x1_Account_balance_for">balance_for</a>(sender_balance) &gt;= transaction_fee_amount,
        <a href="ErrorCode.md#0x1_ErrorCode_EINSUFFICIENT_BALANCE">ErrorCode::EINSUFFICIENT_BALANCE</a>()
    );

    // Todo: remove the abandoned code
    // <b>let</b> cost = <a href="SignedInteger64.md#0x1_SignedInteger64_create_from_raw_value">SignedInteger64::create_from_raw_value</a>(state_cost_amount, cost_is_negative);
    // <b>assert</b>(
    //     <a href="SignedInteger64.md#0x1_SignedInteger64_get_value">SignedInteger64::get_value</a>(cost) &gt;= 0, 7
    // );

    // Bump the sequence number
    sender_account.sequence_number = txn_sequence_number + 1;

    <b>if</b> (transaction_fee_amount &gt; 0) {
        <b>let</b> transaction_fee = <a href="#0x1_Account_withdraw_from_balance">withdraw_from_balance</a>(
                <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account),
                sender_balance,
                transaction_fee_amount
        );
        <a href="TransactionFee.md#0x1_TransactionFee_pay_fee">TransactionFee::pay_fee</a>(transaction_fee);
    };
}
</code></pre>



</details>

<a name="0x1_Account_Specification"></a>

## Specification



<pre><code>pragma verify;
pragma aborts_if_is_strict = <b>true</b>;
</code></pre>



<a name="0x1_Account_Specification_create_genesis_account"></a>

### Function `create_genesis_account`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_create_genesis_account">create_genesis_account</a>(new_account_address: address): signer
</code></pre>




<pre><code><b>aborts_if</b> !<a href="Timestamp.md#0x1_Timestamp_is_genesis">Timestamp::is_genesis</a>();
<b>aborts_if</b> len(DUMMY_AUTH_KEY) != 32;
<b>aborts_if</b> exists&lt;<a href="#0x1_Account">Account</a>&gt;(new_account_address);
</code></pre>



<a name="0x1_Account_Specification_release_genesis_signer"></a>

### Function `release_genesis_signer`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_release_genesis_signer">release_genesis_signer</a>(genesis_account: signer)
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="0x1_Account_Specification_create_account"></a>

### Function `create_account`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_create_account">create_account</a>&lt;TokenType&gt;(fresh_address: address, public_key_vec: vector&lt;u8&gt;)
</code></pre>




<pre><code>pragma verify = <b>false</b>;
<b>aborts_if</b> len(public_key_vec) != 32;
<b>aborts_if</b> exists&lt;<a href="#0x1_Account">Account</a>&gt;(fresh_address);
</code></pre>



<a name="0x1_Account_Specification_make_account"></a>

### Function `make_account`


<pre><code><b>fun</b> <a href="#0x1_Account_make_account">make_account</a>(new_account: &signer, authentication_key: vector&lt;u8&gt;)
</code></pre>




<pre><code><b>aborts_if</b> len(authentication_key) != 32;
<b>aborts_if</b> exists&lt;<a href="#0x1_Account">Account</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(new_account));
</code></pre>



<a name="0x1_Account_Specification_deposit_to"></a>

### Function `deposit_to`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_deposit_to">deposit_to</a>&lt;TokenType&gt;(account: &signer, payee: address, to_deposit: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;)
</code></pre>




<pre><code><b>include</b> <a href="#0x1_Account_Deposit_With_Payer_And_Metadata">Deposit_With_Payer_And_Metadata</a>&lt;TokenType&gt;{payer: <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account)};
</code></pre>



<a name="0x1_Account_Specification_deposit"></a>

### Function `deposit`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_deposit">deposit</a>&lt;TokenType&gt;(account: &signer, to_deposit: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;)
</code></pre>




<pre><code>pragma verify = <b>false</b>;
<b>aborts_if</b> to_deposit.value == 0;
<b>aborts_if</b> !exists&lt;<a href="#0x1_Account">Account</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
<b>aborts_if</b> <b>global</b>&lt;<a href="#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)).token.value + to_deposit.value &gt; max_u128();
<b>ensures</b> <b>global</b>&lt;<a href="#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)).token.value == <b>old</b>(<b>global</b>&lt;<a href="#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)).token.value) + to_deposit.value;
</code></pre>



<a name="0x1_Account_Specification_deposit_with_metadata"></a>

### Function `deposit_with_metadata`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_deposit_with_metadata">deposit_with_metadata</a>&lt;TokenType&gt;(account: &signer, payee: address, to_deposit: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;, metadata: vector&lt;u8&gt;)
</code></pre>




<pre><code><b>include</b> <a href="#0x1_Account_Deposit_With_Payer_And_Metadata">Deposit_With_Payer_And_Metadata</a>&lt;TokenType&gt;{payer: <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account)};
</code></pre>



<a name="0x1_Account_Specification_deposit_with_payer_and_metadata"></a>

### Function `deposit_with_payer_and_metadata`


<pre><code><b>fun</b> <a href="#0x1_Account_deposit_with_payer_and_metadata">deposit_with_payer_and_metadata</a>&lt;TokenType&gt;(payer: address, payee: address, to_deposit: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;, metadata: vector&lt;u8&gt;)
</code></pre>




<pre><code><b>include</b> <a href="#0x1_Account_Deposit_With_Payer_And_Metadata">Deposit_With_Payer_And_Metadata</a>&lt;TokenType&gt;;
</code></pre>




<a name="0x1_Account_Deposit_With_Payer_And_Metadata"></a>


<pre><code><b>schema</b> <a href="#0x1_Account_Deposit_With_Payer_And_Metadata">Deposit_With_Payer_And_Metadata</a>&lt;TokenType&gt; {
    payer: address;
    payee: address;
    to_deposit: <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt;;
    <b>aborts_if</b> to_deposit.value == 0;
    <b>aborts_if</b> !exists&lt;<a href="#0x1_Account">Account</a>&gt;(payer);
    <b>aborts_if</b> !exists&lt;<a href="#0x1_Account">Account</a>&gt;(payee);
    <b>aborts_if</b> !exists&lt;<a href="#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(payee);
    <b>aborts_if</b> <b>global</b>&lt;<a href="#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(payee).token.value + to_deposit.value &gt; max_u128();
    <b>ensures</b> <b>global</b>&lt;<a href="#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(payee).token.value == <b>old</b>(<b>global</b>&lt;<a href="#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(payee).token.value) + to_deposit.value;
}
</code></pre>



<a name="0x1_Account_Specification_withdraw_from_balance"></a>

### Function `withdraw_from_balance`


<pre><code><b>fun</b> <a href="#0x1_Account_withdraw_from_balance">withdraw_from_balance</a>&lt;TokenType&gt;(_addr: address, balance: &<b>mut</b> <a href="#0x1_Account_Balance">Account::Balance</a>&lt;TokenType&gt;, amount: u128): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;
</code></pre>




<pre><code><b>aborts_if</b> balance.token.value &lt; amount;
<b>ensures</b> balance.token.value == <b>old</b>(balance.token.value) - amount;
</code></pre>



<a name="0x1_Account_Specification_withdraw"></a>

### Function `withdraw`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_withdraw">withdraw</a>&lt;TokenType&gt;(account: &signer, amount: u128): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;
</code></pre>




<pre><code><b>aborts_if</b> <a href="Option.md#0x1_Option_spec_is_none">Option::spec_is_none</a>(<b>global</b>&lt;<a href="#0x1_Account">Account</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)).withdrawal_capability);
<b>aborts_if</b> !exists&lt;<a href="#0x1_Account">Account</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
<b>aborts_if</b> !exists&lt;<a href="#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
<b>aborts_if</b> <b>global</b>&lt;<a href="#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)).token.value &lt; amount;
<b>ensures</b> <b>global</b>&lt;<a href="#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)).token.value
        == <b>old</b>(<b>global</b>&lt;<a href="#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)).token.value) - amount;
</code></pre>



<a name="0x1_Account_Specification_withdraw_with_capability"></a>

### Function `withdraw_with_capability`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_withdraw_with_capability">withdraw_with_capability</a>&lt;TokenType&gt;(cap: &<a href="#0x1_Account_WithdrawCapability">Account::WithdrawCapability</a>, amount: u128): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;
</code></pre>




<pre><code><b>aborts_if</b> !exists&lt;<a href="#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(cap.account_address);
<b>aborts_if</b> <b>global</b>&lt;<a href="#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(cap.account_address).token.value &lt; amount;
<b>ensures</b> <b>global</b>&lt;<a href="#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(cap.account_address).token.value
        == <b>old</b>(<b>global</b>&lt;<a href="#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(cap.account_address).token.value) - amount;
</code></pre>



<a name="0x1_Account_Specification_extract_withdraw_capability"></a>

### Function `extract_withdraw_capability`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_extract_withdraw_capability">extract_withdraw_capability</a>(sender: &signer): <a href="#0x1_Account_WithdrawCapability">Account::WithdrawCapability</a>
</code></pre>




<pre><code><b>aborts_if</b> !exists&lt;<a href="#0x1_Account">Account</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(sender));
<b>aborts_if</b> <a href="Option.md#0x1_Option_spec_is_none">Option::spec_is_none</a>(<b>global</b>&lt;<a href="#0x1_Account">Account</a>&gt;( <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(sender)).withdrawal_capability);
</code></pre>



<a name="0x1_Account_Specification_restore_withdraw_capability"></a>

### Function `restore_withdraw_capability`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_restore_withdraw_capability">restore_withdraw_capability</a>(cap: <a href="#0x1_Account_WithdrawCapability">Account::WithdrawCapability</a>)
</code></pre>




<pre><code><b>aborts_if</b> <a href="Option.md#0x1_Option_spec_is_some">Option::spec_is_some</a>(<b>global</b>&lt;<a href="#0x1_Account">Account</a>&gt;(cap.account_address).withdrawal_capability);
<b>aborts_if</b> !exists&lt;<a href="#0x1_Account">Account</a>&gt;(cap.account_address);
</code></pre>



<a name="0x1_Account_Specification_pay_from_capability"></a>

### Function `pay_from_capability`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_pay_from_capability">pay_from_capability</a>&lt;TokenType&gt;(cap: &<a href="#0x1_Account_WithdrawCapability">Account::WithdrawCapability</a>, payee: address, amount: u128, metadata: vector&lt;u8&gt;)
</code></pre>




<pre><code>pragma verify = <b>false</b>;
<b>aborts_if</b> !exists&lt;<a href="#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(cap.account_address);
<b>aborts_if</b> <b>global</b>&lt;<a href="#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(cap.account_address).token.value &lt; amount;
<b>aborts_if</b> amount == 0;
<b>aborts_if</b> !exists&lt;<a href="#0x1_Account">Account</a>&gt;(cap.account_address);
<b>aborts_if</b> !exists&lt;<a href="#0x1_Account">Account</a>&gt;(payee);
<b>aborts_if</b> !exists&lt;<a href="#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(payee);
</code></pre>



<a name="0x1_Account_Specification_pay_from_with_metadata"></a>

### Function `pay_from_with_metadata`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_pay_from_with_metadata">pay_from_with_metadata</a>&lt;TokenType&gt;(account: &signer, payee: address, amount: u128, metadata: vector&lt;u8&gt;)
</code></pre>




<pre><code>pragma verify = <b>false</b>;
<b>aborts_if</b> !exists&lt;<a href="#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
<b>aborts_if</b> <b>global</b>&lt;<a href="#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)).token.value &lt; amount;
<b>ensures</b> <b>global</b>&lt;<a href="#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)).token.value
        == <b>old</b>(<b>global</b>&lt;<a href="#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)).token.value) - amount;
<b>include</b> <a href="#0x1_Account_Deposit_With_Payer_And_Metadata">Deposit_With_Payer_And_Metadata</a>&lt;TokenType&gt;{
    payer: <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account),
    to_deposit: <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt; { value: amount }
};
</code></pre>



<a name="0x1_Account_Specification_pay_from"></a>

### Function `pay_from`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_pay_from">pay_from</a>&lt;TokenType&gt;(account: &signer, payee: address, amount: u128)
</code></pre>




<pre><code>pragma verify = <b>false</b>;
<b>aborts_if</b> !exists&lt;<a href="#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
<b>aborts_if</b> <b>global</b>&lt;<a href="#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)).token.value &lt; amount;
<b>ensures</b> <b>global</b>&lt;<a href="#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)).token.value
        == <b>old</b>(<b>global</b>&lt;<a href="#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)).token.value) - amount;
<b>include</b> <a href="#0x1_Account_Deposit_With_Payer_And_Metadata">Deposit_With_Payer_And_Metadata</a>&lt;TokenType&gt;{
    payer: <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account),
    to_deposit: <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt; { value: amount }
};
</code></pre>



<a name="0x1_Account_Specification_rotate_authentication_key_for_account"></a>

### Function `rotate_authentication_key_for_account`


<pre><code><b>fun</b> <a href="#0x1_Account_rotate_authentication_key_for_account">rotate_authentication_key_for_account</a>(account: &<b>mut</b> <a href="#0x1_Account_Account">Account::Account</a>, new_authentication_key: vector&lt;u8&gt;)
</code></pre>




<pre><code><b>aborts_if</b> len(new_authentication_key) != 32;
<b>ensures</b> account.authentication_key == new_authentication_key;
</code></pre>



<a name="0x1_Account_Specification_rotate_authentication_key"></a>

### Function `rotate_authentication_key`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_rotate_authentication_key">rotate_authentication_key</a>(cap: &<a href="#0x1_Account_KeyRotationCapability">Account::KeyRotationCapability</a>, new_authentication_key: vector&lt;u8&gt;)
</code></pre>




<pre><code><b>aborts_if</b> !exists&lt;<a href="#0x1_Account">Account</a>&gt;(cap.account_address);
<b>aborts_if</b> len(new_authentication_key) != 32;
<b>ensures</b> <b>global</b>&lt;<a href="#0x1_Account">Account</a>&gt;(cap.account_address).authentication_key == new_authentication_key;
</code></pre>




<a name="0x1_Account_spec_rotate_authentication_key"></a>


<pre><code><b>define</b> <a href="#0x1_Account_spec_rotate_authentication_key">spec_rotate_authentication_key</a>(addr: address, new_authentication_key: vector&lt;u8&gt;): bool {
    <b>global</b>&lt;<a href="#0x1_Account">Account</a>&gt;(addr).authentication_key == new_authentication_key
}
</code></pre>



<a name="0x1_Account_Specification_extract_key_rotation_capability"></a>

### Function `extract_key_rotation_capability`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_extract_key_rotation_capability">extract_key_rotation_capability</a>(account: &signer): <a href="#0x1_Account_KeyRotationCapability">Account::KeyRotationCapability</a>
</code></pre>




<pre><code><b>aborts_if</b> !exists&lt;<a href="#0x1_Account">Account</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
<b>aborts_if</b> <a href="Option.md#0x1_Option_spec_is_none">Option::spec_is_none</a>(<b>global</b>&lt;<a href="#0x1_Account">Account</a>&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account)).key_rotation_capability);
</code></pre>



<a name="0x1_Account_Specification_restore_key_rotation_capability"></a>

### Function `restore_key_rotation_capability`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_restore_key_rotation_capability">restore_key_rotation_capability</a>(cap: <a href="#0x1_Account_KeyRotationCapability">Account::KeyRotationCapability</a>)
</code></pre>




<pre><code><b>aborts_if</b> <a href="Option.md#0x1_Option_spec_is_some">Option::spec_is_some</a>(<b>global</b>&lt;<a href="#0x1_Account">Account</a>&gt;(cap.account_address).key_rotation_capability);
<b>aborts_if</b> !exists&lt;<a href="#0x1_Account">Account</a>&gt;(cap.account_address);
</code></pre>



<a name="0x1_Account_Specification_balance_for"></a>

### Function `balance_for`


<pre><code><b>fun</b> <a href="#0x1_Account_balance_for">balance_for</a>&lt;TokenType&gt;(balance: &<a href="#0x1_Account_Balance">Account::Balance</a>&lt;TokenType&gt;): u128
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="0x1_Account_Specification_balance"></a>

### Function `balance`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_balance">balance</a>&lt;TokenType&gt;(addr: address): u128
</code></pre>




<pre><code><b>aborts_if</b> !exists&lt;<a href="#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(addr);
</code></pre>



<a name="0x1_Account_Specification_accept_token"></a>

### Function `accept_token`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_accept_token">accept_token</a>&lt;TokenType&gt;(account: &signer)
</code></pre>




<pre><code><b>aborts_if</b> exists&lt;<a href="#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
<b>aborts_if</b> !exists&lt;<a href="#0x1_Account">Account</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
</code></pre>



<a name="0x1_Account_Specification_is_accepts_token"></a>

### Function `is_accepts_token`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_is_accepts_token">is_accepts_token</a>&lt;TokenType&gt;(addr: address): bool
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="0x1_Account_Specification_sequence_number"></a>

### Function `sequence_number`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_sequence_number">sequence_number</a>(addr: address): u64
</code></pre>




<pre><code><b>aborts_if</b> !exists&lt;<a href="#0x1_Account">Account</a>&gt;(addr);
</code></pre>



<a name="0x1_Account_Specification_authentication_key"></a>

### Function `authentication_key`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_authentication_key">authentication_key</a>(addr: address): vector&lt;u8&gt;
</code></pre>




<pre><code><b>aborts_if</b> !exists&lt;<a href="#0x1_Account">Account</a>&gt;(addr);
</code></pre>



<a name="0x1_Account_Specification_delegated_key_rotation_capability"></a>

### Function `delegated_key_rotation_capability`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_delegated_key_rotation_capability">delegated_key_rotation_capability</a>(addr: address): bool
</code></pre>




<pre><code><b>aborts_if</b> !exists&lt;<a href="#0x1_Account">Account</a>&gt;(addr);
</code></pre>



<a name="0x1_Account_Specification_delegated_withdraw_capability"></a>

### Function `delegated_withdraw_capability`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_delegated_withdraw_capability">delegated_withdraw_capability</a>(addr: address): bool
</code></pre>




<pre><code><b>aborts_if</b> !exists&lt;<a href="#0x1_Account">Account</a>&gt;(addr);
</code></pre>



<a name="0x1_Account_Specification_withdraw_capability_address"></a>

### Function `withdraw_capability_address`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_withdraw_capability_address">withdraw_capability_address</a>(cap: &<a href="#0x1_Account_WithdrawCapability">Account::WithdrawCapability</a>): &address
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="0x1_Account_Specification_key_rotation_capability_address"></a>

### Function `key_rotation_capability_address`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_key_rotation_capability_address">key_rotation_capability_address</a>(cap: &<a href="#0x1_Account_KeyRotationCapability">Account::KeyRotationCapability</a>): &address
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="0x1_Account_Specification_exists_at"></a>

### Function `exists_at`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_exists_at">exists_at</a>(check_addr: address): bool
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="0x1_Account_Specification_txn_prologue"></a>

### Function `txn_prologue`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_txn_prologue">txn_prologue</a>&lt;TokenType&gt;(account: &signer, txn_sender: address, txn_sequence_number: u64, txn_public_key: vector&lt;u8&gt;, txn_gas_price: u64, txn_max_gas_units: u64)
</code></pre>




<pre><code><b>aborts_if</b> <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) != <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>();
<b>aborts_if</b> !exists&lt;<a href="#0x1_Account">Account</a>&gt;(txn_sender);
<b>aborts_if</b> <a href="Hash.md#0x1_Hash_sha3_256">Hash::sha3_256</a>(txn_public_key) != <b>global</b>&lt;<a href="#0x1_Account">Account</a>&gt;(txn_sender).authentication_key;
<b>aborts_if</b> txn_gas_price * txn_max_gas_units &gt; max_u64();
<b>aborts_if</b> !exists&lt;<a href="#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(txn_sender);
<b>aborts_if</b> <b>global</b>&lt;<a href="#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(txn_sender).token.value &lt; txn_gas_price * txn_max_gas_units;
<b>aborts_if</b> txn_sequence_number &lt; <b>global</b>&lt;<a href="#0x1_Account">Account</a>&gt;(txn_sender).sequence_number;
<b>aborts_if</b> txn_sequence_number != <b>global</b>&lt;<a href="#0x1_Account">Account</a>&gt;(txn_sender).sequence_number;
</code></pre>



<a name="0x1_Account_Specification_txn_epilogue"></a>

### Function `txn_epilogue`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Account_txn_epilogue">txn_epilogue</a>&lt;TokenType&gt;(account: &signer, txn_sender: address, txn_sequence_number: u64, txn_gas_price: u64, txn_max_gas_units: u64, gas_units_remaining: u64, _state_cost_amount: u64, _cost_is_negative: bool)
</code></pre>




<pre><code>pragma verify = <b>false</b>;
<b>aborts_if</b> <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) != <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>();
<b>aborts_if</b> !exists&lt;<a href="#0x1_Account">Account</a>&gt;(txn_sender);
<b>aborts_if</b> !exists&lt;<a href="#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(txn_sender);
<b>aborts_if</b> txn_gas_price * (txn_max_gas_units - gas_units_remaining) &gt; max_u64();
<b>aborts_if</b> txn_max_gas_units &lt; gas_units_remaining;
<b>aborts_if</b> <b>global</b>&lt;<a href="#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(txn_sender).token.value &lt; txn_gas_price * (txn_max_gas_units - gas_units_remaining);
<b>aborts_if</b> txn_sequence_number + 1 &gt; max_u64();
<b>aborts_if</b> txn_gas_price * (txn_max_gas_units - gas_units_remaining) &gt; 0 &&
           !exists&lt;<a href="TransactionFee.md#0x1_TransactionFee_TransactionFee">TransactionFee::TransactionFee</a>&lt;TokenType&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
<b>aborts_if</b> <b>global</b>&lt;<a href="TransactionFee.md#0x1_TransactionFee_TransactionFee">TransactionFee::TransactionFee</a>&lt;TokenType&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>()).fee.value + txn_gas_price * (txn_max_gas_units - gas_units_remaining) &gt; max_u128();
</code></pre>
