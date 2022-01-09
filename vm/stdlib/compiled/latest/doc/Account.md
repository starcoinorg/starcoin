
<a name="0x1_Account"></a>

# Module `0x1::Account`

The module for the account resource that governs every account


-  [Resource `Account`](#0x1_Account_Account)
-  [Resource `Balance`](#0x1_Account_Balance)
-  [Struct `WithdrawCapability`](#0x1_Account_WithdrawCapability)
-  [Struct `KeyRotationCapability`](#0x1_Account_KeyRotationCapability)
-  [Struct `WithdrawEvent`](#0x1_Account_WithdrawEvent)
-  [Struct `DepositEvent`](#0x1_Account_DepositEvent)
-  [Struct `AcceptTokenEvent`](#0x1_Account_AcceptTokenEvent)
-  [Resource `SignerDelegated`](#0x1_Account_SignerDelegated)
-  [Struct `SignerCapability`](#0x1_Account_SignerCapability)
-  [Resource `AutoAcceptToken`](#0x1_Account_AutoAcceptToken)
-  [Constants](#@Constants_0)
-  [Function `remove_signer_capability`](#0x1_Account_remove_signer_capability)
-  [Function `create_signer_with_cap`](#0x1_Account_create_signer_with_cap)
-  [Function `destroy_signer_cap`](#0x1_Account_destroy_signer_cap)
-  [Function `signer_address`](#0x1_Account_signer_address)
-  [Function `is_signer_delegated`](#0x1_Account_is_signer_delegated)
-  [Function `create_genesis_account`](#0x1_Account_create_genesis_account)
-  [Function `release_genesis_signer`](#0x1_Account_release_genesis_signer)
-  [Function `create_account`](#0x1_Account_create_account)
-  [Function `create_account_with_address`](#0x1_Account_create_account_with_address)
-  [Function `make_account`](#0x1_Account_make_account)
-  [Function `create_signer`](#0x1_Account_create_signer)
-  [Function `create_account_with_initial_amount`](#0x1_Account_create_account_with_initial_amount)
-  [Function `create_account_with_initial_amount_v2`](#0x1_Account_create_account_with_initial_amount_v2)
-  [Function `deposit_to_self`](#0x1_Account_deposit_to_self)
-  [Function `deposit`](#0x1_Account_deposit)
-  [Function `deposit_with_metadata`](#0x1_Account_deposit_with_metadata)
-  [Function `deposit_to_balance`](#0x1_Account_deposit_to_balance)
-  [Function `withdraw_from_balance`](#0x1_Account_withdraw_from_balance)
-  [Function `withdraw`](#0x1_Account_withdraw)
-  [Function `withdraw_with_metadata`](#0x1_Account_withdraw_with_metadata)
-  [Function `withdraw_with_capability`](#0x1_Account_withdraw_with_capability)
-  [Function `withdraw_with_capability_and_metadata`](#0x1_Account_withdraw_with_capability_and_metadata)
-  [Function `extract_withdraw_capability`](#0x1_Account_extract_withdraw_capability)
-  [Function `restore_withdraw_capability`](#0x1_Account_restore_withdraw_capability)
-  [Function `emit_account_withdraw_event`](#0x1_Account_emit_account_withdraw_event)
-  [Function `emit_account_deposit_event`](#0x1_Account_emit_account_deposit_event)
-  [Function `pay_from_capability`](#0x1_Account_pay_from_capability)
-  [Function `pay_from_with_metadata`](#0x1_Account_pay_from_with_metadata)
-  [Function `pay_from`](#0x1_Account_pay_from)
-  [Function `rotate_authentication_key_with_capability`](#0x1_Account_rotate_authentication_key_with_capability)
-  [Function `extract_key_rotation_capability`](#0x1_Account_extract_key_rotation_capability)
-  [Function `restore_key_rotation_capability`](#0x1_Account_restore_key_rotation_capability)
-  [Function `destroy_key_rotation_capability`](#0x1_Account_destroy_key_rotation_capability)
-  [Function `rotate_authentication_key`](#0x1_Account_rotate_authentication_key)
-  [Function `balance_for`](#0x1_Account_balance_for)
-  [Function `balance`](#0x1_Account_balance)
-  [Function `do_accept_token`](#0x1_Account_do_accept_token)
-  [Function `accept_token`](#0x1_Account_accept_token)
-  [Function `is_accepts_token`](#0x1_Account_is_accepts_token)
-  [Function `is_accept_token`](#0x1_Account_is_accept_token)
-  [Function `can_auto_accept_token`](#0x1_Account_can_auto_accept_token)
-  [Function `set_auto_accept_token`](#0x1_Account_set_auto_accept_token)
-  [Function `try_accept_token`](#0x1_Account_try_accept_token)
-  [Function `sequence_number_for_account`](#0x1_Account_sequence_number_for_account)
-  [Function `sequence_number`](#0x1_Account_sequence_number)
-  [Function `authentication_key`](#0x1_Account_authentication_key)
-  [Function `delegated_key_rotation_capability`](#0x1_Account_delegated_key_rotation_capability)
-  [Function `delegated_withdraw_capability`](#0x1_Account_delegated_withdraw_capability)
-  [Function `withdraw_capability_address`](#0x1_Account_withdraw_capability_address)
-  [Function `key_rotation_capability_address`](#0x1_Account_key_rotation_capability_address)
-  [Function `exists_at`](#0x1_Account_exists_at)
-  [Function `is_dummy_auth_key`](#0x1_Account_is_dummy_auth_key)
-  [Function `txn_prologue`](#0x1_Account_txn_prologue)
-  [Function `txn_epilogue`](#0x1_Account_txn_epilogue)
-  [Function `txn_epilogue_v2`](#0x1_Account_txn_epilogue_v2)
-  [Specification](#@Specification_1)
    -  [Function `create_genesis_account`](#@Specification_1_create_genesis_account)
    -  [Function `release_genesis_signer`](#@Specification_1_release_genesis_signer)
    -  [Function `create_account`](#@Specification_1_create_account)
    -  [Function `create_account_with_address`](#@Specification_1_create_account_with_address)
    -  [Function `make_account`](#@Specification_1_make_account)
    -  [Function `create_account_with_initial_amount`](#@Specification_1_create_account_with_initial_amount)
    -  [Function `create_account_with_initial_amount_v2`](#@Specification_1_create_account_with_initial_amount_v2)
    -  [Function `deposit_to_self`](#@Specification_1_deposit_to_self)
    -  [Function `deposit`](#@Specification_1_deposit)
    -  [Function `deposit_with_metadata`](#@Specification_1_deposit_with_metadata)
    -  [Function `deposit_to_balance`](#@Specification_1_deposit_to_balance)
    -  [Function `withdraw_from_balance`](#@Specification_1_withdraw_from_balance)
    -  [Function `withdraw`](#@Specification_1_withdraw)
    -  [Function `withdraw_with_metadata`](#@Specification_1_withdraw_with_metadata)
    -  [Function `withdraw_with_capability`](#@Specification_1_withdraw_with_capability)
    -  [Function `withdraw_with_capability_and_metadata`](#@Specification_1_withdraw_with_capability_and_metadata)
    -  [Function `extract_withdraw_capability`](#@Specification_1_extract_withdraw_capability)
    -  [Function `restore_withdraw_capability`](#@Specification_1_restore_withdraw_capability)
    -  [Function `emit_account_withdraw_event`](#@Specification_1_emit_account_withdraw_event)
    -  [Function `emit_account_deposit_event`](#@Specification_1_emit_account_deposit_event)
    -  [Function `pay_from_capability`](#@Specification_1_pay_from_capability)
    -  [Function `pay_from_with_metadata`](#@Specification_1_pay_from_with_metadata)
    -  [Function `pay_from`](#@Specification_1_pay_from)
    -  [Function `rotate_authentication_key_with_capability`](#@Specification_1_rotate_authentication_key_with_capability)
    -  [Function `extract_key_rotation_capability`](#@Specification_1_extract_key_rotation_capability)
    -  [Function `restore_key_rotation_capability`](#@Specification_1_restore_key_rotation_capability)
    -  [Function `rotate_authentication_key`](#@Specification_1_rotate_authentication_key)
    -  [Function `balance_for`](#@Specification_1_balance_for)
    -  [Function `do_accept_token`](#@Specification_1_do_accept_token)
    -  [Function `accept_token`](#@Specification_1_accept_token)
    -  [Function `is_accepts_token`](#@Specification_1_is_accepts_token)
    -  [Function `is_accept_token`](#@Specification_1_is_accept_token)
    -  [Function `set_auto_accept_token`](#@Specification_1_set_auto_accept_token)
    -  [Function `try_accept_token`](#@Specification_1_try_accept_token)
    -  [Function `sequence_number`](#@Specification_1_sequence_number)
    -  [Function `authentication_key`](#@Specification_1_authentication_key)
    -  [Function `delegated_key_rotation_capability`](#@Specification_1_delegated_key_rotation_capability)
    -  [Function `delegated_withdraw_capability`](#@Specification_1_delegated_withdraw_capability)
    -  [Function `withdraw_capability_address`](#@Specification_1_withdraw_capability_address)
    -  [Function `key_rotation_capability_address`](#@Specification_1_key_rotation_capability_address)
    -  [Function `exists_at`](#@Specification_1_exists_at)
    -  [Function `txn_prologue`](#@Specification_1_txn_prologue)
    -  [Function `txn_epilogue`](#@Specification_1_txn_epilogue)
    -  [Function `txn_epilogue_v2`](#@Specification_1_txn_epilogue_v2)


<pre><code><b>use</b> <a href="Authenticator.md#0x1_Authenticator">0x1::Authenticator</a>;
<b>use</b> <a href="CoreAddresses.md#0x1_CoreAddresses">0x1::CoreAddresses</a>;
<b>use</b> <a href="Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="Event.md#0x1_Event">0x1::Event</a>;
<b>use</b> <a href="Hash.md#0x1_Hash">0x1::Hash</a>;
<b>use</b> <a href="Option.md#0x1_Option">0x1::Option</a>;
<b>use</b> <a href="STC.md#0x1_STC">0x1::STC</a>;
<b>use</b> <a href="Signer.md#0x1_Signer">0x1::Signer</a>;
<b>use</b> <a href="Timestamp.md#0x1_Timestamp">0x1::Timestamp</a>;
<b>use</b> <a href="Token.md#0x1_Token">0x1::Token</a>;
<b>use</b> <a href="TransactionFee.md#0x1_TransactionFee">0x1::TransactionFee</a>;
<b>use</b> <a href="Vector.md#0x1_Vector">0x1::Vector</a>;
</code></pre>



<a name="0x1_Account_Account"></a>

## Resource `Account`

Every account has a Account::Account resource


<pre><code><b>struct</b> <a href="Account.md#0x1_Account">Account</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>authentication_key: vector&lt;u8&gt;</code>
</dt>
<dd>
 The current authentication key.
 This can be different than the key used to create the account
</dd>
<dt>
<code>withdrawal_capability: <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;<a href="Account.md#0x1_Account_WithdrawCapability">Account::WithdrawCapability</a>&gt;</code>
</dt>
<dd>
 A <code>withdrawal_capability</code> allows whoever holds this capability
 to withdraw from the account. At the time of account creation
 this capability is stored in this option. It can later be
 "extracted" from this field via <code>extract_withdraw_capability</code>,
 and can also be restored via <code>restore_withdraw_capability</code>.
</dd>
<dt>
<code>key_rotation_capability: <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;<a href="Account.md#0x1_Account_KeyRotationCapability">Account::KeyRotationCapability</a>&gt;</code>
</dt>
<dd>
 A <code>key_rotation_capability</code> allows whoever holds this capability
 the ability to rotate the authentication key for the account. At
 the time of account creation this capability is stored in this
 option. It can later be "extracted" from this field via
 <code>extract_key_rotation_capability</code>, and can also be restored via
 <code>restore_key_rotation_capability</code>.
</dd>
<dt>
<code>withdraw_events: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="Account.md#0x1_Account_WithdrawEvent">Account::WithdrawEvent</a>&gt;</code>
</dt>
<dd>
 event handle for account balance withdraw event
</dd>
<dt>
<code>deposit_events: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="Account.md#0x1_Account_DepositEvent">Account::DepositEvent</a>&gt;</code>
</dt>
<dd>
 event handle for account balance deposit event
</dd>
<dt>
<code>accept_token_events: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="Account.md#0x1_Account_AcceptTokenEvent">Account::AcceptTokenEvent</a>&gt;</code>
</dt>
<dd>
 Event handle for accept_token event
</dd>
<dt>
<code>sequence_number: u64</code>
</dt>
<dd>
 The current sequence number.
 Incremented by one each time a transaction is submitted
</dd>
</dl>


</details>

<a name="0x1_Account_Balance"></a>

## Resource `Balance`

A resource that holds the tokens stored in this account


<pre><code><b>struct</b> <a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt; <b>has</b> key
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

## Struct `WithdrawCapability`

The holder of WithdrawCapability for account_address can withdraw Token from
account_address/Account::Account/balance.
There is at most one WithdrawCapability in existence for a given address.


<pre><code><b>struct</b> <a href="Account.md#0x1_Account_WithdrawCapability">WithdrawCapability</a> <b>has</b> store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>account_address: <b>address</b></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_Account_KeyRotationCapability"></a>

## Struct `KeyRotationCapability`

The holder of KeyRotationCapability for account_address can rotate the authentication key for
account_address (i.e., write to account_address/Account::Account/authentication_key).
There is at most one KeyRotationCapability in existence for a given address.


<pre><code><b>struct</b> <a href="Account.md#0x1_Account_KeyRotationCapability">KeyRotationCapability</a> <b>has</b> store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>account_address: <b>address</b></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_Account_WithdrawEvent"></a>

## Struct `WithdrawEvent`

Message for balance withdraw event.


<pre><code><b>struct</b> <a href="Account.md#0x1_Account_WithdrawEvent">WithdrawEvent</a> <b>has</b> drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>amount: u128</code>
</dt>
<dd>
 The amount of Token<TokenType> sent
</dd>
<dt>
<code>token_code: <a href="Token.md#0x1_Token_TokenCode">Token::TokenCode</a></code>
</dt>
<dd>
 The code symbol for the token that was sent
</dd>
<dt>
<code>metadata: vector&lt;u8&gt;</code>
</dt>
<dd>
 Metadata associated with the withdraw
</dd>
</dl>


</details>

<a name="0x1_Account_DepositEvent"></a>

## Struct `DepositEvent`

Message for balance deposit event.


<pre><code><b>struct</b> <a href="Account.md#0x1_Account_DepositEvent">DepositEvent</a> <b>has</b> drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>amount: u128</code>
</dt>
<dd>
 The amount of Token<TokenType> sent
</dd>
<dt>
<code>token_code: <a href="Token.md#0x1_Token_TokenCode">Token::TokenCode</a></code>
</dt>
<dd>
 The code symbol for the token that was sent
</dd>
<dt>
<code>metadata: vector&lt;u8&gt;</code>
</dt>
<dd>
 Metadata associated with the deposit
</dd>
</dl>


</details>

<a name="0x1_Account_AcceptTokenEvent"></a>

## Struct `AcceptTokenEvent`

Message for accept token events


<pre><code><b>struct</b> <a href="Account.md#0x1_Account_AcceptTokenEvent">AcceptTokenEvent</a> <b>has</b> drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>token_code: <a href="Token.md#0x1_Token_TokenCode">Token::TokenCode</a></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_Account_SignerDelegated"></a>

## Resource `SignerDelegated`



<pre><code><b>struct</b> <a href="Account.md#0x1_Account_SignerDelegated">SignerDelegated</a> <b>has</b> key
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

<a name="0x1_Account_SignerCapability"></a>

## Struct `SignerCapability`



<pre><code><b>struct</b> <a href="Account.md#0x1_Account_SignerCapability">SignerCapability</a> <b>has</b> store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>addr: <b>address</b></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_Account_AutoAcceptToken"></a>

## Resource `AutoAcceptToken`



<pre><code><b>struct</b> <a href="Account.md#0x1_Account_AutoAcceptToken">AutoAcceptToken</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>enable: bool</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_Account_MAX_U64"></a>



<pre><code><b>const</b> <a href="Account.md#0x1_Account_MAX_U64">MAX_U64</a>: u128 = 18446744073709551615;
</code></pre>



<a name="0x1_Account_EDEPRECATED_FUNCTION"></a>



<pre><code><b>const</b> <a href="Account.md#0x1_Account_EDEPRECATED_FUNCTION">EDEPRECATED_FUNCTION</a>: u64 = 19;
</code></pre>



<a name="0x1_Account_EPROLOGUE_ACCOUNT_DOES_NOT_EXIST"></a>



<pre><code><b>const</b> <a href="Account.md#0x1_Account_EPROLOGUE_ACCOUNT_DOES_NOT_EXIST">EPROLOGUE_ACCOUNT_DOES_NOT_EXIST</a>: u64 = 0;
</code></pre>



<a name="0x1_Account_CONTRACT_ACCOUNT_AUTH_KEY_PLACEHOLDER"></a>



<pre><code><b>const</b> <a href="Account.md#0x1_Account_CONTRACT_ACCOUNT_AUTH_KEY_PLACEHOLDER">CONTRACT_ACCOUNT_AUTH_KEY_PLACEHOLDER</a>: vector&lt;u8&gt; = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1];
</code></pre>



<a name="0x1_Account_DUMMY_AUTH_KEY"></a>



<pre><code><b>const</b> <a href="Account.md#0x1_Account_DUMMY_AUTH_KEY">DUMMY_AUTH_KEY</a>: vector&lt;u8&gt; = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
</code></pre>



<a name="0x1_Account_EADDRESS_AND_AUTH_KEY_MISMATCH"></a>



<pre><code><b>const</b> <a href="Account.md#0x1_Account_EADDRESS_AND_AUTH_KEY_MISMATCH">EADDRESS_AND_AUTH_KEY_MISMATCH</a>: u64 = 105;
</code></pre>



<a name="0x1_Account_EADDRESS_PUBLIC_KEY_INCONSISTENT"></a>



<pre><code><b>const</b> <a href="Account.md#0x1_Account_EADDRESS_PUBLIC_KEY_INCONSISTENT">EADDRESS_PUBLIC_KEY_INCONSISTENT</a>: u64 = 104;
</code></pre>



<a name="0x1_Account_EBAD_TRANSACTION_FEE_TOKEN"></a>



<pre><code><b>const</b> <a href="Account.md#0x1_Account_EBAD_TRANSACTION_FEE_TOKEN">EBAD_TRANSACTION_FEE_TOKEN</a>: u64 = 18;
</code></pre>



<a name="0x1_Account_ECOIN_DEPOSIT_IS_ZERO"></a>



<pre><code><b>const</b> <a href="Account.md#0x1_Account_ECOIN_DEPOSIT_IS_ZERO">ECOIN_DEPOSIT_IS_ZERO</a>: u64 = 15;
</code></pre>



<a name="0x1_Account_EINSUFFICIENT_BALANCE"></a>



<pre><code><b>const</b> <a href="Account.md#0x1_Account_EINSUFFICIENT_BALANCE">EINSUFFICIENT_BALANCE</a>: u64 = 10;
</code></pre>



<a name="0x1_Account_EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED"></a>



<pre><code><b>const</b> <a href="Account.md#0x1_Account_EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED">EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED</a>: u64 = 103;
</code></pre>



<a name="0x1_Account_EMALFORMED_AUTHENTICATION_KEY"></a>



<pre><code><b>const</b> <a href="Account.md#0x1_Account_EMALFORMED_AUTHENTICATION_KEY">EMALFORMED_AUTHENTICATION_KEY</a>: u64 = 102;
</code></pre>



<a name="0x1_Account_EPROLOGUE_CANT_PAY_GAS_DEPOSIT"></a>



<pre><code><b>const</b> <a href="Account.md#0x1_Account_EPROLOGUE_CANT_PAY_GAS_DEPOSIT">EPROLOGUE_CANT_PAY_GAS_DEPOSIT</a>: u64 = 4;
</code></pre>



<a name="0x1_Account_EPROLOGUE_INVALID_ACCOUNT_AUTH_KEY"></a>



<pre><code><b>const</b> <a href="Account.md#0x1_Account_EPROLOGUE_INVALID_ACCOUNT_AUTH_KEY">EPROLOGUE_INVALID_ACCOUNT_AUTH_KEY</a>: u64 = 1;
</code></pre>



<a name="0x1_Account_EPROLOGUE_SEQUENCE_NUMBER_TOO_BIG"></a>



<pre><code><b>const</b> <a href="Account.md#0x1_Account_EPROLOGUE_SEQUENCE_NUMBER_TOO_BIG">EPROLOGUE_SEQUENCE_NUMBER_TOO_BIG</a>: u64 = 9;
</code></pre>



<a name="0x1_Account_EPROLOGUE_SEQUENCE_NUMBER_TOO_NEW"></a>



<pre><code><b>const</b> <a href="Account.md#0x1_Account_EPROLOGUE_SEQUENCE_NUMBER_TOO_NEW">EPROLOGUE_SEQUENCE_NUMBER_TOO_NEW</a>: u64 = 3;
</code></pre>



<a name="0x1_Account_EPROLOGUE_SEQUENCE_NUMBER_TOO_OLD"></a>



<pre><code><b>const</b> <a href="Account.md#0x1_Account_EPROLOGUE_SEQUENCE_NUMBER_TOO_OLD">EPROLOGUE_SEQUENCE_NUMBER_TOO_OLD</a>: u64 = 2;
</code></pre>



<a name="0x1_Account_EPROLOGUE_SIGNER_ALREADY_DELEGATED"></a>



<pre><code><b>const</b> <a href="Account.md#0x1_Account_EPROLOGUE_SIGNER_ALREADY_DELEGATED">EPROLOGUE_SIGNER_ALREADY_DELEGATED</a>: u64 = 200;
</code></pre>



<a name="0x1_Account_ERR_SIGNER_ALREADY_DELEGATED"></a>



<pre><code><b>const</b> <a href="Account.md#0x1_Account_ERR_SIGNER_ALREADY_DELEGATED">ERR_SIGNER_ALREADY_DELEGATED</a>: u64 = 107;
</code></pre>



<a name="0x1_Account_ERR_TOKEN_NOT_ACCEPT"></a>



<pre><code><b>const</b> <a href="Account.md#0x1_Account_ERR_TOKEN_NOT_ACCEPT">ERR_TOKEN_NOT_ACCEPT</a>: u64 = 106;
</code></pre>



<a name="0x1_Account_EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED"></a>



<pre><code><b>const</b> <a href="Account.md#0x1_Account_EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED">EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED</a>: u64 = 101;
</code></pre>



<a name="0x1_Account_remove_signer_capability"></a>

## Function `remove_signer_capability`

A one-way action, once SignerCapability is removed from signer, the address cannot send txns anymore.
This function can only called once by signer.


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_remove_signer_capability">remove_signer_capability</a>(signer: &signer): <a href="Account.md#0x1_Account_SignerCapability">Account::SignerCapability</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_remove_signer_capability">remove_signer_capability</a>(signer: &signer): <a href="Account.md#0x1_Account_SignerCapability">SignerCapability</a>
<b>acquires</b> <a href="Account.md#0x1_Account">Account</a> {
    <b>let</b> signer_addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer);
    <b>assert</b>!(!<a href="Account.md#0x1_Account_is_signer_delegated">is_signer_delegated</a>(signer_addr), <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="Account.md#0x1_Account_ERR_SIGNER_ALREADY_DELEGATED">ERR_SIGNER_ALREADY_DELEGATED</a>));

    // set <b>to</b> account auth key <b>to</b> noop.
    {
        <b>let</b> key_rotation_capability = <a href="Account.md#0x1_Account_extract_key_rotation_capability">extract_key_rotation_capability</a>(signer);
        <a href="Account.md#0x1_Account_rotate_authentication_key_with_capability">rotate_authentication_key_with_capability</a>(&key_rotation_capability, <a href="Account.md#0x1_Account_CONTRACT_ACCOUNT_AUTH_KEY_PLACEHOLDER">CONTRACT_ACCOUNT_AUTH_KEY_PLACEHOLDER</a>);
        <a href="Account.md#0x1_Account_destroy_key_rotation_capability">destroy_key_rotation_capability</a>(key_rotation_capability);
        <b>move_to</b>(signer, <a href="Account.md#0x1_Account_SignerDelegated">SignerDelegated</a> {});
    };

    <b>let</b> signer_cap = <a href="Account.md#0x1_Account_SignerCapability">SignerCapability</a> {addr: signer_addr };
    signer_cap
}
</code></pre>



</details>

<a name="0x1_Account_create_signer_with_cap"></a>

## Function `create_signer_with_cap`



<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_create_signer_with_cap">create_signer_with_cap</a>(cap: &<a href="Account.md#0x1_Account_SignerCapability">Account::SignerCapability</a>): signer
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_create_signer_with_cap">create_signer_with_cap</a>(cap: &<a href="Account.md#0x1_Account_SignerCapability">SignerCapability</a>): signer {
    <a href="Account.md#0x1_Account_create_signer">create_signer</a>(cap.addr)
}
</code></pre>



</details>

<a name="0x1_Account_destroy_signer_cap"></a>

## Function `destroy_signer_cap`



<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_destroy_signer_cap">destroy_signer_cap</a>(cap: <a href="Account.md#0x1_Account_SignerCapability">Account::SignerCapability</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_destroy_signer_cap">destroy_signer_cap</a>(cap: <a href="Account.md#0x1_Account_SignerCapability">SignerCapability</a>) {
    <b>let</b> <a href="Account.md#0x1_Account_SignerCapability">SignerCapability</a> {addr: _} = cap;
}
</code></pre>



</details>

<a name="0x1_Account_signer_address"></a>

## Function `signer_address`



<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_signer_address">signer_address</a>(cap: &<a href="Account.md#0x1_Account_SignerCapability">Account::SignerCapability</a>): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_signer_address">signer_address</a>(cap: &<a href="Account.md#0x1_Account_SignerCapability">SignerCapability</a>): <b>address</b> {
    cap.addr
}
</code></pre>



</details>

<a name="0x1_Account_is_signer_delegated"></a>

## Function `is_signer_delegated`



<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_is_signer_delegated">is_signer_delegated</a>(addr: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_is_signer_delegated">is_signer_delegated</a>(addr: <b>address</b>): bool {
    <b>exists</b>&lt;<a href="Account.md#0x1_Account_SignerDelegated">SignerDelegated</a>&gt;(addr)
}
</code></pre>



</details>

<a name="0x1_Account_create_genesis_account"></a>

## Function `create_genesis_account`

Create an genesis account at <code>new_account_address</code> and return signer.
Genesis authentication_key is zero bytes.


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_create_genesis_account">create_genesis_account</a>(new_account_address: <b>address</b>): signer
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_create_genesis_account">create_genesis_account</a>(
    new_account_address: <b>address</b>,
) :signer {
    <a href="Timestamp.md#0x1_Timestamp_assert_genesis">Timestamp::assert_genesis</a>();
    <b>let</b> new_account = <a href="Account.md#0x1_Account_create_signer">create_signer</a>(new_account_address);
    <a href="Account.md#0x1_Account_make_account">make_account</a>(&new_account, <a href="Account.md#0x1_Account_DUMMY_AUTH_KEY">DUMMY_AUTH_KEY</a>);
    new_account
}
</code></pre>



</details>

<a name="0x1_Account_release_genesis_signer"></a>

## Function `release_genesis_signer`

Release genesis account signer


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_release_genesis_signer">release_genesis_signer</a>(_genesis_account: signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_release_genesis_signer">release_genesis_signer</a>(_genesis_account: signer){
}
</code></pre>



</details>

<a name="0x1_Account_create_account"></a>

## Function `create_account`

Deprecated since @v5


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_create_account">create_account</a>&lt;TokenType: store&gt;(_authentication_key: vector&lt;u8&gt;): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_create_account">create_account</a>&lt;TokenType: store&gt;(_authentication_key: vector&lt;u8&gt;): <b>address</b> {
    <b>abort</b> <a href="Errors.md#0x1_Errors_deprecated">Errors::deprecated</a>(<a href="Account.md#0x1_Account_EDEPRECATED_FUNCTION">EDEPRECATED_FUNCTION</a>)
}
</code></pre>



</details>

<a name="0x1_Account_create_account_with_address"></a>

## Function `create_account_with_address`

Creates a new account at <code>fresh_address</code> with a balance of zero and empty auth key, the address as init auth key for check transaction.
Creating an account at address StarcoinFramework will cause runtime failure as it is a
reserved address for the MoveVM.


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_create_account_with_address">create_account_with_address</a>&lt;TokenType: store&gt;(fresh_address: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_create_account_with_address">create_account_with_address</a>&lt;TokenType: store&gt;(fresh_address: <b>address</b>) <b>acquires</b> <a href="Account.md#0x1_Account">Account</a> {
    <b>let</b> new_account = <a href="Account.md#0x1_Account_create_signer">create_signer</a>(fresh_address);
    <a href="Account.md#0x1_Account_make_account">make_account</a>(&new_account, <a href="Account.md#0x1_Account_DUMMY_AUTH_KEY">DUMMY_AUTH_KEY</a>);
    // Make sure all account accept <a href="STC.md#0x1_STC">STC</a>.
    <b>if</b> (!<a href="STC.md#0x1_STC_is_stc">STC::is_stc</a>&lt;TokenType&gt;()){
        <a href="Account.md#0x1_Account_do_accept_token">do_accept_token</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;(&new_account);
    };
    <a href="Account.md#0x1_Account_do_accept_token">do_accept_token</a>&lt;TokenType&gt;(&new_account);
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
    <b>assert</b>!(<a href="Vector.md#0x1_Vector_length">Vector::length</a>(&authentication_key) == 32, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Account.md#0x1_Account_EMALFORMED_AUTHENTICATION_KEY">EMALFORMED_AUTHENTICATION_KEY</a>));
    <b>let</b> new_account_addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(new_account);
    <a href="Event.md#0x1_Event_publish_generator">Event::publish_generator</a>(new_account);
    <b>move_to</b>(new_account, <a href="Account.md#0x1_Account">Account</a> {
          authentication_key,
          withdrawal_capability: <a href="Option.md#0x1_Option_some">Option::some</a>(
              <a href="Account.md#0x1_Account_WithdrawCapability">WithdrawCapability</a> {
                  account_address: new_account_addr
          }),
          key_rotation_capability: <a href="Option.md#0x1_Option_some">Option::some</a>(
              <a href="Account.md#0x1_Account_KeyRotationCapability">KeyRotationCapability</a> {
                  account_address: new_account_addr
          }),
          withdraw_events: <a href="Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="Account.md#0x1_Account_WithdrawEvent">WithdrawEvent</a>&gt;(new_account),
          deposit_events: <a href="Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="Account.md#0x1_Account_DepositEvent">DepositEvent</a>&gt;(new_account),
          accept_token_events: <a href="Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="Account.md#0x1_Account_AcceptTokenEvent">AcceptTokenEvent</a>&gt;(new_account),
          sequence_number: 0,
    });
    <b>move_to</b>(new_account, <a href="Account.md#0x1_Account_AutoAcceptToken">AutoAcceptToken</a>{enable: <b>true</b>});
}
</code></pre>



</details>

<a name="0x1_Account_create_signer"></a>

## Function `create_signer`



<pre><code><b>fun</b> <a href="Account.md#0x1_Account_create_signer">create_signer</a>(addr: <b>address</b>): signer
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>fun</b> <a href="Account.md#0x1_Account_create_signer">create_signer</a>(addr: <b>address</b>): signer;
</code></pre>



</details>

<a name="0x1_Account_create_account_with_initial_amount"></a>

## Function `create_account_with_initial_amount`



<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="Account.md#0x1_Account_create_account_with_initial_amount">create_account_with_initial_amount</a>&lt;TokenType: store&gt;(account: signer, fresh_address: <b>address</b>, _auth_key: vector&lt;u8&gt;, initial_amount: u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="Account.md#0x1_Account_create_account_with_initial_amount">create_account_with_initial_amount</a>&lt;TokenType: store&gt;(account: signer, fresh_address: <b>address</b>, _auth_key: vector&lt;u8&gt;, initial_amount: u128)
<b>acquires</b> <a href="Account.md#0x1_Account">Account</a>, <a href="Account.md#0x1_Account_Balance">Balance</a>, <a href="Account.md#0x1_Account_AutoAcceptToken">AutoAcceptToken</a> {
     <a href="Account.md#0x1_Account_create_account_with_initial_amount_v2">create_account_with_initial_amount_v2</a>&lt;TokenType&gt;(account, fresh_address, initial_amount)
}
</code></pre>



</details>

<a name="0x1_Account_create_account_with_initial_amount_v2"></a>

## Function `create_account_with_initial_amount_v2`



<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="Account.md#0x1_Account_create_account_with_initial_amount_v2">create_account_with_initial_amount_v2</a>&lt;TokenType: store&gt;(account: signer, fresh_address: <b>address</b>, initial_amount: u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="Account.md#0x1_Account_create_account_with_initial_amount_v2">create_account_with_initial_amount_v2</a>&lt;TokenType: store&gt;(account: signer, fresh_address: <b>address</b>, initial_amount: u128)
<b>acquires</b> <a href="Account.md#0x1_Account">Account</a>, <a href="Account.md#0x1_Account_Balance">Balance</a>, <a href="Account.md#0x1_Account_AutoAcceptToken">AutoAcceptToken</a> {
    <a href="Account.md#0x1_Account_create_account_with_address">create_account_with_address</a>&lt;TokenType&gt;(fresh_address);
    <b>if</b> (initial_amount &gt; 0) {
        <a href="Account.md#0x1_Account_pay_from">pay_from</a>&lt;TokenType&gt;(&account, fresh_address, initial_amount);
    };
}
</code></pre>



</details>

<a name="0x1_Account_deposit_to_self"></a>

## Function `deposit_to_self`

Deposits the <code>to_deposit</code> token into the self's account balance


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_deposit_to_self">deposit_to_self</a>&lt;TokenType: store&gt;(account: &signer, to_deposit: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_deposit_to_self">deposit_to_self</a>&lt;TokenType: store&gt;(account: &signer, to_deposit: <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt;)
<b>acquires</b> <a href="Account.md#0x1_Account">Account</a>, <a href="Account.md#0x1_Account_Balance">Balance</a>, <a href="Account.md#0x1_Account_AutoAcceptToken">AutoAcceptToken</a> {
    <b>let</b> account_address = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>if</b> (!<a href="Account.md#0x1_Account_is_accepts_token">is_accepts_token</a>&lt;TokenType&gt;(account_address)){
        <a href="Account.md#0x1_Account_do_accept_token">do_accept_token</a>&lt;TokenType&gt;(account);
    };
    <a href="Account.md#0x1_Account_deposit">deposit</a>(account_address, to_deposit);
}
</code></pre>



</details>

<a name="0x1_Account_deposit"></a>

## Function `deposit`

Deposits the <code>to_deposit</code> token into the <code>receiver</code>'s account balance with the no metadata
It's a reverse operation of <code>withdraw</code>.


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_deposit">deposit</a>&lt;TokenType: store&gt;(receiver: <b>address</b>, to_deposit: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_deposit">deposit</a>&lt;TokenType: store&gt;(
    receiver: <b>address</b>,
    to_deposit: <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt;,
) <b>acquires</b> <a href="Account.md#0x1_Account">Account</a>, <a href="Account.md#0x1_Account_Balance">Balance</a>, <a href="Account.md#0x1_Account_AutoAcceptToken">AutoAcceptToken</a> {
    <a href="Account.md#0x1_Account_deposit_with_metadata">deposit_with_metadata</a>&lt;TokenType&gt;(receiver, to_deposit, x"")
}
</code></pre>



</details>

<a name="0x1_Account_deposit_with_metadata"></a>

## Function `deposit_with_metadata`

Deposits the <code>to_deposit</code> token into the <code>receiver</code>'s account balance with the attached <code>metadata</code>
It's a reverse operation of <code>withdraw_with_metadata</code>.


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_deposit_with_metadata">deposit_with_metadata</a>&lt;TokenType: store&gt;(receiver: <b>address</b>, to_deposit: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;, metadata: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_deposit_with_metadata">deposit_with_metadata</a>&lt;TokenType: store&gt;(
    receiver: <b>address</b>,
    to_deposit: <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt;,
    metadata: vector&lt;u8&gt;,
) <b>acquires</b> <a href="Account.md#0x1_Account">Account</a>, <a href="Account.md#0x1_Account_Balance">Balance</a>, <a href="Account.md#0x1_Account_AutoAcceptToken">AutoAcceptToken</a> {
    <a href="Account.md#0x1_Account_try_accept_token">try_accept_token</a>&lt;TokenType&gt;(receiver);

    <b>let</b> deposit_value = <a href="Token.md#0x1_Token_value">Token::value</a>(&to_deposit);
    <b>if</b> (deposit_value &gt; 0u128) {
        // Deposit the `to_deposit` token
        <a href="Account.md#0x1_Account_deposit_to_balance">deposit_to_balance</a>&lt;TokenType&gt;(<b>borrow_global_mut</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(receiver), to_deposit);

        // emit deposit event
        <a href="Account.md#0x1_Account_emit_account_deposit_event">emit_account_deposit_event</a>&lt;TokenType&gt;(receiver, deposit_value, metadata);
    } <b>else</b> {
        <a href="Token.md#0x1_Token_destroy_zero">Token::destroy_zero</a>(to_deposit);
    };
}
</code></pre>



</details>

<a name="0x1_Account_deposit_to_balance"></a>

## Function `deposit_to_balance`

Helper to deposit <code>amount</code> to the given account balance


<pre><code><b>fun</b> <a href="Account.md#0x1_Account_deposit_to_balance">deposit_to_balance</a>&lt;TokenType: store&gt;(balance: &<b>mut</b> <a href="Account.md#0x1_Account_Balance">Account::Balance</a>&lt;TokenType&gt;, token: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="Account.md#0x1_Account_deposit_to_balance">deposit_to_balance</a>&lt;TokenType: store&gt;(balance: &<b>mut</b> <a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;, token: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;) {
    <a href="Token.md#0x1_Token_deposit">Token::deposit</a>(&<b>mut</b> balance.token, token)
}
</code></pre>



</details>

<a name="0x1_Account_withdraw_from_balance"></a>

## Function `withdraw_from_balance`

Helper to withdraw <code>amount</code> from the given account balance and return the withdrawn Token<TokenType>


<pre><code><b>fun</b> <a href="Account.md#0x1_Account_withdraw_from_balance">withdraw_from_balance</a>&lt;TokenType: store&gt;(balance: &<b>mut</b> <a href="Account.md#0x1_Account_Balance">Account::Balance</a>&lt;TokenType&gt;, amount: u128): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="Account.md#0x1_Account_withdraw_from_balance">withdraw_from_balance</a>&lt;TokenType: store&gt;(balance: &<b>mut</b> <a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;, amount: u128): <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt;{
    <a href="Token.md#0x1_Token_withdraw">Token::withdraw</a>(&<b>mut</b> balance.token, amount)
}
</code></pre>



</details>

<a name="0x1_Account_withdraw"></a>

## Function `withdraw`

Withdraw <code>amount</code> Token<TokenType> from the account balance


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_withdraw">withdraw</a>&lt;TokenType: store&gt;(account: &signer, amount: u128): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_withdraw">withdraw</a>&lt;TokenType: store&gt;(account: &signer, amount: u128): <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt;
<b>acquires</b> <a href="Account.md#0x1_Account">Account</a>, <a href="Account.md#0x1_Account_Balance">Balance</a> {
    <a href="Account.md#0x1_Account_withdraw_with_metadata">withdraw_with_metadata</a>&lt;TokenType&gt;(account, amount, x"")
}
</code></pre>



</details>

<a name="0x1_Account_withdraw_with_metadata"></a>

## Function `withdraw_with_metadata`

Withdraw <code>amount</code> tokens from <code>signer</code> with given <code>metadata</code>.


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_withdraw_with_metadata">withdraw_with_metadata</a>&lt;TokenType: store&gt;(account: &signer, amount: u128, metadata: vector&lt;u8&gt;): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_withdraw_with_metadata">withdraw_with_metadata</a>&lt;TokenType: store&gt;(account: &signer, amount: u128, metadata: vector&lt;u8&gt;): <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt;
<b>acquires</b> <a href="Account.md#0x1_Account">Account</a>, <a href="Account.md#0x1_Account_Balance">Balance</a> {
    <b>let</b> sender_addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>let</b> sender_balance = <b>borrow_global_mut</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(sender_addr);
    // The sender_addr <b>has</b> delegated the privilege <b>to</b> withdraw from her account elsewhere--<b>abort</b>.
    <b>assert</b>!(!<a href="Account.md#0x1_Account_delegated_withdraw_capability">delegated_withdraw_capability</a>(sender_addr), <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="Account.md#0x1_Account_EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED">EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED</a>));
    <b>if</b> (amount == 0){
        <b>return</b> <a href="Token.md#0x1_Token_zero">Token::zero</a>()
    };
    <a href="Account.md#0x1_Account_emit_account_withdraw_event">emit_account_withdraw_event</a>&lt;TokenType&gt;(sender_addr, amount, metadata);
    // The sender_addr <b>has</b> retained her withdrawal privileges--proceed.
    <a href="Account.md#0x1_Account_withdraw_from_balance">withdraw_from_balance</a>&lt;TokenType&gt;(sender_balance, amount)
}
</code></pre>



</details>

<a name="0x1_Account_withdraw_with_capability"></a>

## Function `withdraw_with_capability`

Withdraw <code>amount</code> Token<TokenType> from the account under cap.account_address with no metadata


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_withdraw_with_capability">withdraw_with_capability</a>&lt;TokenType: store&gt;(cap: &<a href="Account.md#0x1_Account_WithdrawCapability">Account::WithdrawCapability</a>, amount: u128): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_withdraw_with_capability">withdraw_with_capability</a>&lt;TokenType: store&gt;(
    cap: &<a href="Account.md#0x1_Account_WithdrawCapability">WithdrawCapability</a>, amount: u128
): <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt; <b>acquires</b> <a href="Account.md#0x1_Account_Balance">Balance</a>, <a href="Account.md#0x1_Account">Account</a> {
    <a href="Account.md#0x1_Account_withdraw_with_capability_and_metadata">withdraw_with_capability_and_metadata</a>&lt;TokenType&gt;(cap, amount, x"")
}
</code></pre>



</details>

<a name="0x1_Account_withdraw_with_capability_and_metadata"></a>

## Function `withdraw_with_capability_and_metadata`

Withdraw <code>amount</code> Token<TokenType> from the account under cap.account_address with metadata


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_withdraw_with_capability_and_metadata">withdraw_with_capability_and_metadata</a>&lt;TokenType: store&gt;(cap: &<a href="Account.md#0x1_Account_WithdrawCapability">Account::WithdrawCapability</a>, amount: u128, metadata: vector&lt;u8&gt;): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_withdraw_with_capability_and_metadata">withdraw_with_capability_and_metadata</a>&lt;TokenType: store&gt;(
    cap: &<a href="Account.md#0x1_Account_WithdrawCapability">WithdrawCapability</a>, amount: u128, metadata: vector&lt;u8&gt;
): <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt; <b>acquires</b> <a href="Account.md#0x1_Account_Balance">Balance</a>, <a href="Account.md#0x1_Account">Account</a> {
    <b>let</b> balance = <b>borrow_global_mut</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(cap.account_address);
    <a href="Account.md#0x1_Account_emit_account_withdraw_event">emit_account_withdraw_event</a>&lt;TokenType&gt;(cap.account_address, amount, metadata);
    <a href="Account.md#0x1_Account_withdraw_from_balance">withdraw_from_balance</a>&lt;TokenType&gt;(balance , amount)
}
</code></pre>



</details>

<a name="0x1_Account_extract_withdraw_capability"></a>

## Function `extract_withdraw_capability`

Return a unique capability granting permission to withdraw from the sender's account balance.


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_extract_withdraw_capability">extract_withdraw_capability</a>(sender: &signer): <a href="Account.md#0x1_Account_WithdrawCapability">Account::WithdrawCapability</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_extract_withdraw_capability">extract_withdraw_capability</a>(
    sender: &signer
): <a href="Account.md#0x1_Account_WithdrawCapability">WithdrawCapability</a> <b>acquires</b> <a href="Account.md#0x1_Account">Account</a> {
    <b>let</b> sender_addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(sender);
    // Abort <b>if</b> we already extracted the unique withdraw capability for this account.
    <b>assert</b>!(!<a href="Account.md#0x1_Account_delegated_withdraw_capability">delegated_withdraw_capability</a>(sender_addr), <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="Account.md#0x1_Account_EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED">EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED</a>));
    <b>let</b> account = <b>borrow_global_mut</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(sender_addr);
    <a href="Option.md#0x1_Option_extract">Option::extract</a>(&<b>mut</b> account.withdrawal_capability)
}
</code></pre>



</details>

<a name="0x1_Account_restore_withdraw_capability"></a>

## Function `restore_withdraw_capability`

Return the withdraw capability to the account it originally came from


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_restore_withdraw_capability">restore_withdraw_capability</a>(cap: <a href="Account.md#0x1_Account_WithdrawCapability">Account::WithdrawCapability</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_restore_withdraw_capability">restore_withdraw_capability</a>(cap: <a href="Account.md#0x1_Account_WithdrawCapability">WithdrawCapability</a>)
   <b>acquires</b> <a href="Account.md#0x1_Account">Account</a> {
       <b>let</b> account = <b>borrow_global_mut</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(cap.account_address);
       <a href="Option.md#0x1_Option_fill">Option::fill</a>(&<b>mut</b> account.withdrawal_capability, cap)
}
</code></pre>



</details>

<a name="0x1_Account_emit_account_withdraw_event"></a>

## Function `emit_account_withdraw_event`



<pre><code><b>fun</b> <a href="Account.md#0x1_Account_emit_account_withdraw_event">emit_account_withdraw_event</a>&lt;TokenType: store&gt;(account: <b>address</b>, amount: u128, metadata: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="Account.md#0x1_Account_emit_account_withdraw_event">emit_account_withdraw_event</a>&lt;TokenType: store&gt;(account: <b>address</b>, amount: u128, metadata: vector&lt;u8&gt;)
<b>acquires</b> <a href="Account.md#0x1_Account">Account</a> {
    // emit withdraw event
    <b>let</b> account = <b>borrow_global_mut</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(account);

    <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>&lt;<a href="Account.md#0x1_Account_WithdrawEvent">WithdrawEvent</a>&gt;(&<b>mut</b> account.withdraw_events, <a href="Account.md#0x1_Account_WithdrawEvent">WithdrawEvent</a> {
        amount,
        token_code: <a href="Token.md#0x1_Token_token_code">Token::token_code</a>&lt;TokenType&gt;(),
        metadata,
    });
}
</code></pre>



</details>

<a name="0x1_Account_emit_account_deposit_event"></a>

## Function `emit_account_deposit_event`



<pre><code><b>fun</b> <a href="Account.md#0x1_Account_emit_account_deposit_event">emit_account_deposit_event</a>&lt;TokenType: store&gt;(account: <b>address</b>, amount: u128, metadata: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="Account.md#0x1_Account_emit_account_deposit_event">emit_account_deposit_event</a>&lt;TokenType: store&gt;(account: <b>address</b>, amount: u128, metadata: vector&lt;u8&gt;)
<b>acquires</b> <a href="Account.md#0x1_Account">Account</a> {
    // emit withdraw event
    <b>let</b> account = <b>borrow_global_mut</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(account);

    <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>&lt;<a href="Account.md#0x1_Account_DepositEvent">DepositEvent</a>&gt;(&<b>mut</b> account.deposit_events, <a href="Account.md#0x1_Account_DepositEvent">DepositEvent</a> {
        amount,
        token_code: <a href="Token.md#0x1_Token_token_code">Token::token_code</a>&lt;TokenType&gt;(),
        metadata,
    });
}
</code></pre>



</details>

<a name="0x1_Account_pay_from_capability"></a>

## Function `pay_from_capability`

Withdraws <code>amount</code> Token<TokenType> using the passed in WithdrawCapability, and deposits it
into the <code>payee</code>'s account balance. Creates the <code>payee</code> account if it doesn't exist.


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_pay_from_capability">pay_from_capability</a>&lt;TokenType: store&gt;(cap: &<a href="Account.md#0x1_Account_WithdrawCapability">Account::WithdrawCapability</a>, payee: <b>address</b>, amount: u128, metadata: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_pay_from_capability">pay_from_capability</a>&lt;TokenType: store&gt;(
    cap: &<a href="Account.md#0x1_Account_WithdrawCapability">WithdrawCapability</a>,
    payee: <b>address</b>,
    amount: u128,
    metadata: vector&lt;u8&gt;,
) <b>acquires</b> <a href="Account.md#0x1_Account">Account</a>, <a href="Account.md#0x1_Account_Balance">Balance</a>, <a href="Account.md#0x1_Account_AutoAcceptToken">AutoAcceptToken</a> {
    <b>let</b> tokens = <a href="Account.md#0x1_Account_withdraw_with_capability_and_metadata">withdraw_with_capability_and_metadata</a>&lt;TokenType&gt;(cap, amount, *&metadata);
    <a href="Account.md#0x1_Account_deposit_with_metadata">deposit_with_metadata</a>&lt;TokenType&gt;(
        payee,
        tokens,
        metadata,
    );
}
</code></pre>



</details>

<a name="0x1_Account_pay_from_with_metadata"></a>

## Function `pay_from_with_metadata`

Withdraw <code>amount</code> Token<TokenType> from the transaction sender's
account balance and send the token to the <code>payee</code> address with the
attached <code>metadata</code> Creates the <code>payee</code> account if it does not exist


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_pay_from_with_metadata">pay_from_with_metadata</a>&lt;TokenType: store&gt;(account: &signer, payee: <b>address</b>, amount: u128, metadata: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_pay_from_with_metadata">pay_from_with_metadata</a>&lt;TokenType: store&gt;(
    account: &signer,
    payee: <b>address</b>,
    amount: u128,
    metadata: vector&lt;u8&gt;,
) <b>acquires</b> <a href="Account.md#0x1_Account">Account</a>, <a href="Account.md#0x1_Account_Balance">Balance</a>, <a href="Account.md#0x1_Account_AutoAcceptToken">AutoAcceptToken</a> {
    <b>let</b> tokens = <a href="Account.md#0x1_Account_withdraw_with_metadata">withdraw_with_metadata</a>&lt;TokenType&gt;(account, amount, *&metadata);
    <a href="Account.md#0x1_Account_deposit_with_metadata">deposit_with_metadata</a>&lt;TokenType&gt;(
        payee,
        tokens,
        metadata,
    );
}
</code></pre>



</details>

<a name="0x1_Account_pay_from"></a>

## Function `pay_from`

Withdraw <code>amount</code> Token<TokenType> from the transaction sender's
account balance  and send the token to the <code>payee</code> address
Creates the <code>payee</code> account if it does not exist


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_pay_from">pay_from</a>&lt;TokenType: store&gt;(account: &signer, payee: <b>address</b>, amount: u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_pay_from">pay_from</a>&lt;TokenType: store&gt;(
    account: &signer,
    payee: <b>address</b>,
    amount: u128
) <b>acquires</b> <a href="Account.md#0x1_Account">Account</a>, <a href="Account.md#0x1_Account_Balance">Balance</a>, <a href="Account.md#0x1_Account_AutoAcceptToken">AutoAcceptToken</a> {
    <a href="Account.md#0x1_Account_pay_from_with_metadata">pay_from_with_metadata</a>&lt;TokenType&gt;(account, payee, amount, x"");
}
</code></pre>



</details>

<a name="0x1_Account_rotate_authentication_key_with_capability"></a>

## Function `rotate_authentication_key_with_capability`

Rotate the authentication key for the account under cap.account_address


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_rotate_authentication_key_with_capability">rotate_authentication_key_with_capability</a>(cap: &<a href="Account.md#0x1_Account_KeyRotationCapability">Account::KeyRotationCapability</a>, new_authentication_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_rotate_authentication_key_with_capability">rotate_authentication_key_with_capability</a>(
    cap: &<a href="Account.md#0x1_Account_KeyRotationCapability">KeyRotationCapability</a>,
    new_authentication_key: vector&lt;u8&gt;,
) <b>acquires</b> <a href="Account.md#0x1_Account">Account</a>  {
    <b>let</b> sender_account_resource = <b>borrow_global_mut</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(cap.account_address);
    // Don't allow rotating <b>to</b> clearly invalid key
    <b>assert</b>!(<a href="Vector.md#0x1_Vector_length">Vector::length</a>(&new_authentication_key) == 32, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Account.md#0x1_Account_EMALFORMED_AUTHENTICATION_KEY">EMALFORMED_AUTHENTICATION_KEY</a>));
    sender_account_resource.authentication_key = new_authentication_key;
}
</code></pre>



</details>

<a name="0x1_Account_extract_key_rotation_capability"></a>

## Function `extract_key_rotation_capability`

Return a unique capability granting permission to rotate the sender's authentication key


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_extract_key_rotation_capability">extract_key_rotation_capability</a>(account: &signer): <a href="Account.md#0x1_Account_KeyRotationCapability">Account::KeyRotationCapability</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_extract_key_rotation_capability">extract_key_rotation_capability</a>(account: &signer): <a href="Account.md#0x1_Account_KeyRotationCapability">KeyRotationCapability</a>
<b>acquires</b> <a href="Account.md#0x1_Account">Account</a> {
    <b>let</b> account_address = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    // Abort <b>if</b> we already extracted the unique key rotation capability for this account.
    <b>assert</b>!(!<a href="Account.md#0x1_Account_delegated_key_rotation_capability">delegated_key_rotation_capability</a>(account_address), <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="Account.md#0x1_Account_EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED">EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED</a>));
    <b>let</b> account = <b>borrow_global_mut</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(account_address);
    <a href="Option.md#0x1_Option_extract">Option::extract</a>(&<b>mut</b> account.key_rotation_capability)
}
</code></pre>



</details>

<a name="0x1_Account_restore_key_rotation_capability"></a>

## Function `restore_key_rotation_capability`

Return the key rotation capability to the account it originally came from


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_restore_key_rotation_capability">restore_key_rotation_capability</a>(cap: <a href="Account.md#0x1_Account_KeyRotationCapability">Account::KeyRotationCapability</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_restore_key_rotation_capability">restore_key_rotation_capability</a>(cap: <a href="Account.md#0x1_Account_KeyRotationCapability">KeyRotationCapability</a>)
<b>acquires</b> <a href="Account.md#0x1_Account">Account</a> {
    <b>let</b> account = <b>borrow_global_mut</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(cap.account_address);
    <a href="Option.md#0x1_Option_fill">Option::fill</a>(&<b>mut</b> account.key_rotation_capability, cap)
}
</code></pre>



</details>

<a name="0x1_Account_destroy_key_rotation_capability"></a>

## Function `destroy_key_rotation_capability`



<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_destroy_key_rotation_capability">destroy_key_rotation_capability</a>(cap: <a href="Account.md#0x1_Account_KeyRotationCapability">Account::KeyRotationCapability</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_destroy_key_rotation_capability">destroy_key_rotation_capability</a>(cap: <a href="Account.md#0x1_Account_KeyRotationCapability">KeyRotationCapability</a>) {
    <b>let</b> <a href="Account.md#0x1_Account_KeyRotationCapability">KeyRotationCapability</a> {account_address: _} = cap;
}
</code></pre>



</details>

<a name="0x1_Account_rotate_authentication_key"></a>

## Function `rotate_authentication_key`



<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="Account.md#0x1_Account_rotate_authentication_key">rotate_authentication_key</a>(account: signer, new_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="Account.md#0x1_Account_rotate_authentication_key">rotate_authentication_key</a>(account: signer, new_key: vector&lt;u8&gt;) <b>acquires</b> <a href="Account.md#0x1_Account">Account</a> {
    <b>let</b> key_rotation_capability = <a href="Account.md#0x1_Account_extract_key_rotation_capability">extract_key_rotation_capability</a>(&account);
    <a href="Account.md#0x1_Account_rotate_authentication_key_with_capability">rotate_authentication_key_with_capability</a>(&key_rotation_capability, new_key);
    <a href="Account.md#0x1_Account_restore_key_rotation_capability">restore_key_rotation_capability</a>(key_rotation_capability);
}
</code></pre>



</details>

<a name="0x1_Account_balance_for"></a>

## Function `balance_for`

Helper to return the u128 value of the <code>balance</code> for <code>account</code>


<pre><code><b>fun</b> <a href="Account.md#0x1_Account_balance_for">balance_for</a>&lt;TokenType: store&gt;(balance: &<a href="Account.md#0x1_Account_Balance">Account::Balance</a>&lt;TokenType&gt;): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="Account.md#0x1_Account_balance_for">balance_for</a>&lt;TokenType: store&gt;(balance: &<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;): u128 {
    <a href="Token.md#0x1_Token_value">Token::value</a>&lt;TokenType&gt;(&balance.token)
}
</code></pre>



</details>

<a name="0x1_Account_balance"></a>

## Function `balance`

Return the current TokenType balance of the account at <code>addr</code>.


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_balance">balance</a>&lt;TokenType: store&gt;(addr: <b>address</b>): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_balance">balance</a>&lt;TokenType: store&gt;(addr: <b>address</b>): u128 <b>acquires</b> <a href="Account.md#0x1_Account_Balance">Balance</a> {
    <b>if</b> (<b>exists</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(addr)) {
        <a href="Account.md#0x1_Account_balance_for">balance_for</a>(<b>borrow_global</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(addr))
    } <b>else</b> {
        0u128
    }
}
</code></pre>



</details>

<a name="0x1_Account_do_accept_token"></a>

## Function `do_accept_token`

Add a balance of <code><a href="Token.md#0x1_Token">Token</a></code> type to the sending account.


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_do_accept_token">do_accept_token</a>&lt;TokenType: store&gt;(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_do_accept_token">do_accept_token</a>&lt;TokenType: store&gt;(account: &signer) <b>acquires</b> <a href="Account.md#0x1_Account">Account</a> {
    <b>move_to</b>(account, <a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;{ token: <a href="Token.md#0x1_Token_zero">Token::zero</a>&lt;TokenType&gt;() });
    <b>let</b> token_code = <a href="Token.md#0x1_Token_token_code">Token::token_code</a>&lt;TokenType&gt;();
    // Load the sender's account
    <b>let</b> sender_account_ref = <b>borrow_global_mut</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
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

<a name="0x1_Account_accept_token"></a>

## Function `accept_token`



<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="Account.md#0x1_Account_accept_token">accept_token</a>&lt;TokenType: store&gt;(account: signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="Account.md#0x1_Account_accept_token">accept_token</a>&lt;TokenType: store&gt;(account: signer) <b>acquires</b> <a href="Account.md#0x1_Account">Account</a> {
    <a href="Account.md#0x1_Account_do_accept_token">do_accept_token</a>&lt;TokenType&gt;(&account);
}
</code></pre>



</details>

<a name="0x1_Account_is_accepts_token"></a>

## Function `is_accepts_token`

This is a alias of is_accept_token


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_is_accepts_token">is_accepts_token</a>&lt;TokenType: store&gt;(addr: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_is_accepts_token">is_accepts_token</a>&lt;TokenType: store&gt;(addr: <b>address</b>): bool <b>acquires</b> <a href="Account.md#0x1_Account_AutoAcceptToken">AutoAcceptToken</a> {
    <a href="Account.md#0x1_Account_is_accept_token">Self::is_accept_token</a>&lt;TokenType&gt;(addr)
}
</code></pre>



</details>

<a name="0x1_Account_is_accept_token"></a>

## Function `is_accept_token`

Return whether the account at <code>addr</code> accept <code><a href="Token.md#0x1_Token">Token</a></code> type tokens


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_is_accept_token">is_accept_token</a>&lt;TokenType: store&gt;(addr: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_is_accept_token">is_accept_token</a>&lt;TokenType: store&gt;(addr: <b>address</b>): bool <b>acquires</b> <a href="Account.md#0x1_Account_AutoAcceptToken">AutoAcceptToken</a> {
    <b>if</b> (<a href="Account.md#0x1_Account_can_auto_accept_token">can_auto_accept_token</a>(addr)) {
        <b>true</b>
    } <b>else</b> {
        <b>exists</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(addr)
    }
}
</code></pre>



</details>

<a name="0x1_Account_can_auto_accept_token"></a>

## Function `can_auto_accept_token`

Check whether the address can auto accept token.


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_can_auto_accept_token">can_auto_accept_token</a>(addr: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_can_auto_accept_token">can_auto_accept_token</a>(addr: <b>address</b>): bool <b>acquires</b> <a href="Account.md#0x1_Account_AutoAcceptToken">AutoAcceptToken</a> {
    <b>if</b> (<b>exists</b>&lt;<a href="Account.md#0x1_Account_AutoAcceptToken">AutoAcceptToken</a>&gt;(addr)) {
        <b>borrow_global</b>&lt;<a href="Account.md#0x1_Account_AutoAcceptToken">AutoAcceptToken</a>&gt;(addr).enable
    } <b>else</b> {
        <b>false</b>
    }
}
</code></pre>



</details>

<a name="0x1_Account_set_auto_accept_token"></a>

## Function `set_auto_accept_token`

Configure whether auto-accept tokens.


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_set_auto_accept_token">set_auto_accept_token</a>(account: &signer, enable: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_set_auto_accept_token">set_auto_accept_token</a>(account: &signer, enable: bool) <b>acquires</b> <a href="Account.md#0x1_Account_AutoAcceptToken">AutoAcceptToken</a> {
    <b>let</b> addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>if</b> (<b>exists</b>&lt;<a href="Account.md#0x1_Account_AutoAcceptToken">AutoAcceptToken</a>&gt;(addr)) {
        <b>let</b> config = <b>borrow_global_mut</b>&lt;<a href="Account.md#0x1_Account_AutoAcceptToken">AutoAcceptToken</a>&gt;(addr);
        config.enable = enable;
    } <b>else</b> {
        <b>move_to</b>(account, <a href="Account.md#0x1_Account_AutoAcceptToken">AutoAcceptToken</a>{enable});
    };
}
</code></pre>



</details>

<a name="0x1_Account_try_accept_token"></a>

## Function `try_accept_token`

try to accept token for <code>addr</code>.


<pre><code><b>fun</b> <a href="Account.md#0x1_Account_try_accept_token">try_accept_token</a>&lt;TokenType: store&gt;(addr: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="Account.md#0x1_Account_try_accept_token">try_accept_token</a>&lt;TokenType: store&gt;(addr: <b>address</b>) <b>acquires</b> <a href="Account.md#0x1_Account_AutoAcceptToken">AutoAcceptToken</a>, <a href="Account.md#0x1_Account">Account</a> {
    <b>if</b> (!<b>exists</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(addr)) {
        <b>if</b> (<a href="Account.md#0x1_Account_can_auto_accept_token">can_auto_accept_token</a>(addr)) {
            <b>let</b> signer = <a href="Account.md#0x1_Account_create_signer">create_signer</a>(addr);
            <a href="Account.md#0x1_Account_do_accept_token">do_accept_token</a>&lt;TokenType&gt;(&signer);
        }<b>else</b>{
            <b>abort</b> <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="Account.md#0x1_Account_ERR_TOKEN_NOT_ACCEPT">ERR_TOKEN_NOT_ACCEPT</a>)
        }
    };
}
</code></pre>



</details>

<a name="0x1_Account_sequence_number_for_account"></a>

## Function `sequence_number_for_account`

Helper to return the sequence number field for given <code>account</code>


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

Return the current sequence number at <code>addr</code>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_sequence_number">sequence_number</a>(addr: <b>address</b>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_sequence_number">sequence_number</a>(addr: <b>address</b>): u64 <b>acquires</b> <a href="Account.md#0x1_Account">Account</a> {
    <a href="Account.md#0x1_Account_sequence_number_for_account">sequence_number_for_account</a>(<b>borrow_global</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(addr))
}
</code></pre>



</details>

<a name="0x1_Account_authentication_key"></a>

## Function `authentication_key`

Return the authentication key for this account


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_authentication_key">authentication_key</a>(addr: <b>address</b>): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_authentication_key">authentication_key</a>(addr: <b>address</b>): vector&lt;u8&gt; <b>acquires</b> <a href="Account.md#0x1_Account">Account</a> {
    *&<b>borrow_global</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(addr).authentication_key
}
</code></pre>



</details>

<a name="0x1_Account_delegated_key_rotation_capability"></a>

## Function `delegated_key_rotation_capability`

Return true if the account at <code>addr</code> has delegated its key rotation capability


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_delegated_key_rotation_capability">delegated_key_rotation_capability</a>(addr: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_delegated_key_rotation_capability">delegated_key_rotation_capability</a>(addr: <b>address</b>): bool
<b>acquires</b> <a href="Account.md#0x1_Account">Account</a> {
    <a href="Option.md#0x1_Option_is_none">Option::is_none</a>(&<b>borrow_global</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(addr).key_rotation_capability)
}
</code></pre>



</details>

<a name="0x1_Account_delegated_withdraw_capability"></a>

## Function `delegated_withdraw_capability`

Return true if the account at <code>addr</code> has delegated its withdraw capability


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_delegated_withdraw_capability">delegated_withdraw_capability</a>(addr: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_delegated_withdraw_capability">delegated_withdraw_capability</a>(addr: <b>address</b>): bool
<b>acquires</b> <a href="Account.md#0x1_Account">Account</a> {
    <a href="Option.md#0x1_Option_is_none">Option::is_none</a>(&<b>borrow_global</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(addr).withdrawal_capability)
}
</code></pre>



</details>

<a name="0x1_Account_withdraw_capability_address"></a>

## Function `withdraw_capability_address`

Return a reference to the address associated with the given withdraw capability


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_withdraw_capability_address">withdraw_capability_address</a>(cap: &<a href="Account.md#0x1_Account_WithdrawCapability">Account::WithdrawCapability</a>): &<b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_withdraw_capability_address">withdraw_capability_address</a>(cap: &<a href="Account.md#0x1_Account_WithdrawCapability">WithdrawCapability</a>): &<b>address</b> {
    &cap.account_address
}
</code></pre>



</details>

<a name="0x1_Account_key_rotation_capability_address"></a>

## Function `key_rotation_capability_address`

Return a reference to the address associated with the given key rotation capability


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_key_rotation_capability_address">key_rotation_capability_address</a>(cap: &<a href="Account.md#0x1_Account_KeyRotationCapability">Account::KeyRotationCapability</a>): &<b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_key_rotation_capability_address">key_rotation_capability_address</a>(cap: &<a href="Account.md#0x1_Account_KeyRotationCapability">KeyRotationCapability</a>): &<b>address</b> {
    &cap.account_address
}
</code></pre>



</details>

<a name="0x1_Account_exists_at"></a>

## Function `exists_at`

Checks if an account exists at <code>check_addr</code>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_exists_at">exists_at</a>(check_addr: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_exists_at">exists_at</a>(check_addr: <b>address</b>): bool {
    <b>exists</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(check_addr)
}
</code></pre>



</details>

<a name="0x1_Account_is_dummy_auth_key"></a>

## Function `is_dummy_auth_key`



<pre><code><b>fun</b> <a href="Account.md#0x1_Account_is_dummy_auth_key">is_dummy_auth_key</a>(account: &<a href="Account.md#0x1_Account_Account">Account::Account</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="Account.md#0x1_Account_is_dummy_auth_key">is_dummy_auth_key</a>(account: &<a href="Account.md#0x1_Account">Account</a>): bool {
    *&account.authentication_key == <a href="Account.md#0x1_Account_DUMMY_AUTH_KEY">DUMMY_AUTH_KEY</a>
}
</code></pre>



</details>

<a name="0x1_Account_txn_prologue"></a>

## Function `txn_prologue`

The prologue is invoked at the beginning of every transaction
It verifies:
- The account's auth key matches the transaction's public key
- That the account has enough balance to pay for all of the gas
- That the sequence number matches the transaction's sequence key


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_txn_prologue">txn_prologue</a>&lt;TokenType: store&gt;(account: &signer, txn_sender: <b>address</b>, txn_sequence_number: u64, txn_authentication_key_preimage: vector&lt;u8&gt;, txn_gas_price: u64, txn_max_gas_units: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_txn_prologue">txn_prologue</a>&lt;TokenType: store&gt;(
    account: &signer,
    txn_sender: <b>address</b>,
    txn_sequence_number: u64,
    txn_authentication_key_preimage: vector&lt;u8&gt;,
    txn_gas_price: u64,
    txn_max_gas_units: u64,
) <b>acquires</b> <a href="Account.md#0x1_Account">Account</a>, <a href="Account.md#0x1_Account_Balance">Balance</a> {
    <a href="CoreAddresses.md#0x1_CoreAddresses_assert_genesis_address">CoreAddresses::assert_genesis_address</a>(account);

    // Verify that the transaction sender's account <b>exists</b>
    <b>assert</b>!(<a href="Account.md#0x1_Account_exists_at">exists_at</a>(txn_sender), <a href="Errors.md#0x1_Errors_requires_address">Errors::requires_address</a>(<a href="Account.md#0x1_Account_EPROLOGUE_ACCOUNT_DOES_NOT_EXIST">EPROLOGUE_ACCOUNT_DOES_NOT_EXIST</a>));
    // Verify the account <b>has</b> not delegate its signer cap.
    <b>assert</b>!(!<a href="Account.md#0x1_Account_is_signer_delegated">is_signer_delegated</a>(txn_sender), <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="Account.md#0x1_Account_EPROLOGUE_SIGNER_ALREADY_DELEGATED">EPROLOGUE_SIGNER_ALREADY_DELEGATED</a>));

    // Load the transaction sender's account
    <b>let</b> sender_account = <b>borrow_global_mut</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(txn_sender);

    <b>if</b> (<a href="Account.md#0x1_Account_is_dummy_auth_key">is_dummy_auth_key</a>(sender_account)){
        // <b>if</b> sender's auth key is empty, <b>use</b> <b>address</b> <b>as</b> auth key for check transaction.
        <b>assert</b>!(
            <a href="Authenticator.md#0x1_Authenticator_derived_address">Authenticator::derived_address</a>(<a href="Hash.md#0x1_Hash_sha3_256">Hash::sha3_256</a>(txn_authentication_key_preimage)) == txn_sender,
            <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Account.md#0x1_Account_EPROLOGUE_INVALID_ACCOUNT_AUTH_KEY">EPROLOGUE_INVALID_ACCOUNT_AUTH_KEY</a>)
        );
    }<b>else</b>{
        // Check that the hash of the transaction's <b>public</b> key matches the account's auth key
        <b>assert</b>!(
            <a href="Hash.md#0x1_Hash_sha3_256">Hash::sha3_256</a>(txn_authentication_key_preimage) == *&sender_account.authentication_key,
            <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Account.md#0x1_Account_EPROLOGUE_INVALID_ACCOUNT_AUTH_KEY">EPROLOGUE_INVALID_ACCOUNT_AUTH_KEY</a>)
        );
    };

    // Check that the account <b>has</b> enough balance for all of the gas
    <b>assert</b>!(
        (txn_gas_price <b>as</b> u128) * (txn_max_gas_units <b>as</b> u128) &lt;= <a href="Account.md#0x1_Account_MAX_U64">MAX_U64</a>,
        <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Account.md#0x1_Account_EPROLOGUE_CANT_PAY_GAS_DEPOSIT">EPROLOGUE_CANT_PAY_GAS_DEPOSIT</a>),
    );
    <b>let</b> max_transaction_fee = txn_gas_price * txn_max_gas_units;
    <b>if</b> (max_transaction_fee &gt; 0) {
        <b>assert</b>!(
            <a href="STC.md#0x1_STC_is_stc">STC::is_stc</a>&lt;TokenType&gt;(),
            <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Account.md#0x1_Account_EBAD_TRANSACTION_FEE_TOKEN">EBAD_TRANSACTION_FEE_TOKEN</a>)
        );

        <b>let</b> balance_amount = <a href="Account.md#0x1_Account_balance">balance</a>&lt;TokenType&gt;(txn_sender);
        <b>assert</b>!(balance_amount &gt;= (max_transaction_fee <b>as</b> u128), <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Account.md#0x1_Account_EPROLOGUE_CANT_PAY_GAS_DEPOSIT">EPROLOGUE_CANT_PAY_GAS_DEPOSIT</a>));

        <b>assert</b>!(
            (txn_sequence_number <b>as</b> u128) &lt; <a href="Account.md#0x1_Account_MAX_U64">MAX_U64</a>,
            <a href="Errors.md#0x1_Errors_limit_exceeded">Errors::limit_exceeded</a>(<a href="Account.md#0x1_Account_EPROLOGUE_SEQUENCE_NUMBER_TOO_BIG">EPROLOGUE_SEQUENCE_NUMBER_TOO_BIG</a>)
        );
    };

    // Check that the transaction sequence number matches the sequence number of the account
    <b>assert</b>!(txn_sequence_number &gt;= sender_account.sequence_number, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Account.md#0x1_Account_EPROLOGUE_SEQUENCE_NUMBER_TOO_OLD">EPROLOGUE_SEQUENCE_NUMBER_TOO_OLD</a>));
    <b>assert</b>!(txn_sequence_number == sender_account.sequence_number, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Account.md#0x1_Account_EPROLOGUE_SEQUENCE_NUMBER_TOO_NEW">EPROLOGUE_SEQUENCE_NUMBER_TOO_NEW</a>));
}
</code></pre>



</details>

<a name="0x1_Account_txn_epilogue"></a>

## Function `txn_epilogue`

The epilogue is invoked at the end of transactions.
It collects gas and bumps the sequence number


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_txn_epilogue">txn_epilogue</a>&lt;TokenType: store&gt;(account: &signer, txn_sender: <b>address</b>, txn_sequence_number: u64, txn_gas_price: u64, txn_max_gas_units: u64, gas_units_remaining: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_txn_epilogue">txn_epilogue</a>&lt;TokenType: store&gt;(
    account: &signer,
    txn_sender: <b>address</b>,
    txn_sequence_number: u64,
    txn_gas_price: u64,
    txn_max_gas_units: u64,
    gas_units_remaining: u64,
) <b>acquires</b> <a href="Account.md#0x1_Account">Account</a>, <a href="Account.md#0x1_Account_Balance">Balance</a> {
    <a href="Account.md#0x1_Account_txn_epilogue_v2">txn_epilogue_v2</a>&lt;TokenType&gt;(account, txn_sender, txn_sequence_number, <a href="Vector.md#0x1_Vector_empty">Vector::empty</a>(), txn_gas_price, txn_max_gas_units, gas_units_remaining)
}
</code></pre>



</details>

<a name="0x1_Account_txn_epilogue_v2"></a>

## Function `txn_epilogue_v2`

The epilogue is invoked at the end of transactions.
It collects gas and bumps the sequence number


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_txn_epilogue_v2">txn_epilogue_v2</a>&lt;TokenType: store&gt;(account: &signer, txn_sender: <b>address</b>, txn_sequence_number: u64, txn_authentication_key_preimage: vector&lt;u8&gt;, txn_gas_price: u64, txn_max_gas_units: u64, gas_units_remaining: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_txn_epilogue_v2">txn_epilogue_v2</a>&lt;TokenType: store&gt;(
    account: &signer,
    txn_sender: <b>address</b>,
    txn_sequence_number: u64,
    txn_authentication_key_preimage: vector&lt;u8&gt;,
    txn_gas_price: u64,
    txn_max_gas_units: u64,
    gas_units_remaining: u64,
) <b>acquires</b> <a href="Account.md#0x1_Account">Account</a>, <a href="Account.md#0x1_Account_Balance">Balance</a> {
    <a href="CoreAddresses.md#0x1_CoreAddresses_assert_genesis_address">CoreAddresses::assert_genesis_address</a>(account);

    // Load the transaction sender's account and balance resources
    <b>let</b> sender_account = <b>borrow_global_mut</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(txn_sender);
    <b>let</b> sender_balance = <b>borrow_global_mut</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(txn_sender);

    // Charge for gas
    <b>let</b> transaction_fee_amount =(txn_gas_price * (txn_max_gas_units - gas_units_remaining) <b>as</b> u128);
    <b>assert</b>!(
        <a href="Account.md#0x1_Account_balance_for">balance_for</a>(sender_balance) &gt;= transaction_fee_amount,
        <a href="Errors.md#0x1_Errors_limit_exceeded">Errors::limit_exceeded</a>(<a href="Account.md#0x1_Account_EINSUFFICIENT_BALANCE">EINSUFFICIENT_BALANCE</a>)
    );

    // Bump the sequence number
    sender_account.sequence_number = txn_sequence_number + 1;
    // Set auth key when user send transaction first.
    <b>if</b> (<a href="Account.md#0x1_Account_is_dummy_auth_key">is_dummy_auth_key</a>(sender_account) && !<a href="Vector.md#0x1_Vector_is_empty">Vector::is_empty</a>(&txn_authentication_key_preimage)){
        sender_account.authentication_key = <a href="Hash.md#0x1_Hash_sha3_256">Hash::sha3_256</a>(txn_authentication_key_preimage);
    };
    <b>if</b> (transaction_fee_amount &gt; 0) {
        <b>let</b> transaction_fee = <a href="Account.md#0x1_Account_withdraw_from_balance">withdraw_from_balance</a>(
                sender_balance,
                transaction_fee_amount
        );
        <a href="TransactionFee.md#0x1_TransactionFee_pay_fee">TransactionFee::pay_fee</a>(transaction_fee);
    };
}
</code></pre>



</details>

<a name="@Specification_1"></a>

## Specification



<pre><code><b>pragma</b> verify = <b>false</b>;
<b>pragma</b> aborts_if_is_strict = <b>true</b>;
</code></pre>



<a name="@Specification_1_create_genesis_account"></a>

### Function `create_genesis_account`


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_create_genesis_account">create_genesis_account</a>(new_account_address: <b>address</b>): signer
</code></pre>




<pre><code><b>aborts_if</b> !<a href="Timestamp.md#0x1_Timestamp_is_genesis">Timestamp::is_genesis</a>();
<b>aborts_if</b> len(<a href="Account.md#0x1_Account_DUMMY_AUTH_KEY">DUMMY_AUTH_KEY</a>) != 32;
<b>aborts_if</b> <b>exists</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(new_account_address);
</code></pre>



<a name="@Specification_1_release_genesis_signer"></a>

### Function `release_genesis_signer`


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_release_genesis_signer">release_genesis_signer</a>(_genesis_account: signer)
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="@Specification_1_create_account"></a>

### Function `create_account`


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_create_account">create_account</a>&lt;TokenType: store&gt;(_authentication_key: vector&lt;u8&gt;): <b>address</b>
</code></pre>




<pre><code><b>aborts_if</b> <b>true</b>;
</code></pre>



<a name="@Specification_1_create_account_with_address"></a>

### Function `create_account_with_address`


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_create_account_with_address">create_account_with_address</a>&lt;TokenType: store&gt;(fresh_address: <b>address</b>)
</code></pre>




<pre><code><b>aborts_if</b> <b>exists</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(fresh_address);
<b>aborts_if</b> <a href="Token.md#0x1_Token_spec_token_code">Token::spec_token_code</a>&lt;TokenType&gt;() != <a href="Token.md#0x1_Token_spec_token_code">Token::spec_token_code</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;() && <b>exists</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;&gt;(fresh_address);
<b>aborts_if</b> <b>exists</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(fresh_address);
<b>ensures</b> <a href="Account.md#0x1_Account_exists_at">exists_at</a>(fresh_address);
<b>ensures</b> <b>exists</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(fresh_address);
</code></pre>



<a name="@Specification_1_make_account"></a>

### Function `make_account`


<pre><code><b>fun</b> <a href="Account.md#0x1_Account_make_account">make_account</a>(new_account: &signer, authentication_key: vector&lt;u8&gt;)
</code></pre>




<pre><code><b>aborts_if</b> len(authentication_key) != 32;
<b>aborts_if</b> <b>exists</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(new_account));
<b>aborts_if</b> <b>exists</b>&lt;<a href="Account.md#0x1_Account_AutoAcceptToken">AutoAcceptToken</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(new_account));
<b>ensures</b> <a href="Account.md#0x1_Account_exists_at">exists_at</a>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(new_account));
</code></pre>



<a name="@Specification_1_create_account_with_initial_amount"></a>

### Function `create_account_with_initial_amount`


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="Account.md#0x1_Account_create_account_with_initial_amount">create_account_with_initial_amount</a>&lt;TokenType: store&gt;(account: signer, fresh_address: <b>address</b>, _auth_key: vector&lt;u8&gt;, initial_amount: u128)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>



<a name="@Specification_1_create_account_with_initial_amount_v2"></a>

### Function `create_account_with_initial_amount_v2`


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="Account.md#0x1_Account_create_account_with_initial_amount_v2">create_account_with_initial_amount_v2</a>&lt;TokenType: store&gt;(account: signer, fresh_address: <b>address</b>, initial_amount: u128)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>



<a name="@Specification_1_deposit_to_self"></a>

### Function `deposit_to_self`


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_deposit_to_self">deposit_to_self</a>&lt;TokenType: store&gt;(account: &signer, to_deposit: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;)
</code></pre>




<pre><code><b>aborts_if</b> to_deposit.value == 0;
<b>let</b> is_accepts_token = <b>exists</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
<b>aborts_if</b> is_accepts_token && <b>global</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)).token.value + to_deposit.value &gt; max_u128();
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
<b>ensures</b> <b>exists</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
</code></pre>



<a name="@Specification_1_deposit"></a>

### Function `deposit`


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_deposit">deposit</a>&lt;TokenType: store&gt;(receiver: <b>address</b>, to_deposit: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;)
</code></pre>




<pre><code><b>include</b> <a href="Account.md#0x1_Account_DepositWithMetadataAbortsIf">DepositWithMetadataAbortsIf</a>&lt;TokenType&gt;;
</code></pre>



<a name="@Specification_1_deposit_with_metadata"></a>

### Function `deposit_with_metadata`


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_deposit_with_metadata">deposit_with_metadata</a>&lt;TokenType: store&gt;(receiver: <b>address</b>, to_deposit: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;, metadata: vector&lt;u8&gt;)
</code></pre>




<pre><code><b>include</b> <a href="Account.md#0x1_Account_DepositWithMetadataAbortsIf">DepositWithMetadataAbortsIf</a>&lt;TokenType&gt;;
<b>ensures</b> <b>exists</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(receiver);
<b>ensures</b> <b>old</b>(<b>global</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(receiver)).token.value + to_deposit.value == <b>global</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(receiver).token.value;
</code></pre>




<a name="0x1_Account_DepositWithMetadataAbortsIf"></a>


<pre><code><b>schema</b> <a href="Account.md#0x1_Account_DepositWithMetadataAbortsIf">DepositWithMetadataAbortsIf</a>&lt;TokenType&gt; {
    receiver: <b>address</b>;
    to_deposit: <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt;;
    <b>aborts_if</b> to_deposit.value == 0;
    <b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(receiver);
    <b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(receiver);
    <b>aborts_if</b> <b>global</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(receiver).token.value + to_deposit.value &gt; max_u128();
}
</code></pre>



<a name="@Specification_1_deposit_to_balance"></a>

### Function `deposit_to_balance`


<pre><code><b>fun</b> <a href="Account.md#0x1_Account_deposit_to_balance">deposit_to_balance</a>&lt;TokenType: store&gt;(balance: &<b>mut</b> <a href="Account.md#0x1_Account_Balance">Account::Balance</a>&lt;TokenType&gt;, token: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;)
</code></pre>




<pre><code><b>aborts_if</b> balance.token.value + token.value &gt; MAX_U128;
</code></pre>



<a name="@Specification_1_withdraw_from_balance"></a>

### Function `withdraw_from_balance`


<pre><code><b>fun</b> <a href="Account.md#0x1_Account_withdraw_from_balance">withdraw_from_balance</a>&lt;TokenType: store&gt;(balance: &<b>mut</b> <a href="Account.md#0x1_Account_Balance">Account::Balance</a>&lt;TokenType&gt;, amount: u128): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;
</code></pre>




<pre><code><b>aborts_if</b> balance.token.value &lt; amount;
</code></pre>



<a name="@Specification_1_withdraw"></a>

### Function `withdraw`


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_withdraw">withdraw</a>&lt;TokenType: store&gt;(account: &signer, amount: u128): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
<b>aborts_if</b> <b>global</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)).token.value &lt; amount;
<b>aborts_if</b> <a href="Option.md#0x1_Option_is_none">Option::is_none</a>(<b>global</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)).withdrawal_capability);
</code></pre>



<a name="@Specification_1_withdraw_with_metadata"></a>

### Function `withdraw_with_metadata`


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_withdraw_with_metadata">withdraw_with_metadata</a>&lt;TokenType: store&gt;(account: &signer, amount: u128, metadata: vector&lt;u8&gt;): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
<b>aborts_if</b> <b>global</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)).token.value &lt; amount;
<b>aborts_if</b> <a href="Option.md#0x1_Option_is_none">Option::is_none</a>(<b>global</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)).withdrawal_capability);
</code></pre>




<a name="0x1_Account_spec_withdraw"></a>


<pre><code><b>fun</b> <a href="Account.md#0x1_Account_spec_withdraw">spec_withdraw</a>&lt;TokenType&gt;(account: signer, amount: u128): <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt; {
   <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt; { value: amount }
}
</code></pre>



<a name="@Specification_1_withdraw_with_capability"></a>

### Function `withdraw_with_capability`


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_withdraw_with_capability">withdraw_with_capability</a>&lt;TokenType: store&gt;(cap: &<a href="Account.md#0x1_Account_WithdrawCapability">Account::WithdrawCapability</a>, amount: u128): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(cap.account_address);
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(cap.account_address);
<b>aborts_if</b> <b>global</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(cap.account_address).token.value &lt; amount;
</code></pre>



<a name="@Specification_1_withdraw_with_capability_and_metadata"></a>

### Function `withdraw_with_capability_and_metadata`


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_withdraw_with_capability_and_metadata">withdraw_with_capability_and_metadata</a>&lt;TokenType: store&gt;(cap: &<a href="Account.md#0x1_Account_WithdrawCapability">Account::WithdrawCapability</a>, amount: u128, metadata: vector&lt;u8&gt;): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(cap.account_address);
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(cap.account_address);
<b>aborts_if</b> <b>global</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(cap.account_address).token.value &lt; amount;
</code></pre>



<a name="@Specification_1_extract_withdraw_capability"></a>

### Function `extract_withdraw_capability`


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_extract_withdraw_capability">extract_withdraw_capability</a>(sender: &signer): <a href="Account.md#0x1_Account_WithdrawCapability">Account::WithdrawCapability</a>
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(sender));
<b>aborts_if</b> <a href="Option.md#0x1_Option_is_none">Option::is_none</a>(<b>global</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;( <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(sender)).withdrawal_capability);
</code></pre>



<a name="@Specification_1_restore_withdraw_capability"></a>

### Function `restore_withdraw_capability`


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_restore_withdraw_capability">restore_withdraw_capability</a>(cap: <a href="Account.md#0x1_Account_WithdrawCapability">Account::WithdrawCapability</a>)
</code></pre>




<pre><code><b>aborts_if</b> <a href="Option.md#0x1_Option_is_some">Option::is_some</a>(<b>global</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(cap.account_address).withdrawal_capability);
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(cap.account_address);
</code></pre>



<a name="@Specification_1_emit_account_withdraw_event"></a>

### Function `emit_account_withdraw_event`


<pre><code><b>fun</b> <a href="Account.md#0x1_Account_emit_account_withdraw_event">emit_account_withdraw_event</a>&lt;TokenType: store&gt;(account: <b>address</b>, amount: u128, metadata: vector&lt;u8&gt;)
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(account);
</code></pre>



<a name="@Specification_1_emit_account_deposit_event"></a>

### Function `emit_account_deposit_event`


<pre><code><b>fun</b> <a href="Account.md#0x1_Account_emit_account_deposit_event">emit_account_deposit_event</a>&lt;TokenType: store&gt;(account: <b>address</b>, amount: u128, metadata: vector&lt;u8&gt;)
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(account);
</code></pre>



<a name="@Specification_1_pay_from_capability"></a>

### Function `pay_from_capability`


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_pay_from_capability">pay_from_capability</a>&lt;TokenType: store&gt;(cap: &<a href="Account.md#0x1_Account_WithdrawCapability">Account::WithdrawCapability</a>, payee: <b>address</b>, amount: u128, metadata: vector&lt;u8&gt;)
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(cap.account_address);
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(cap.account_address);
<b>aborts_if</b> <b>global</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(cap.account_address).token.value &lt; amount;
<b>aborts_if</b> amount == 0;
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(payee);
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(payee);
<b>aborts_if</b> cap.account_address != payee && <b>global</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(payee).token.value + amount &gt; MAX_U128;
</code></pre>



<a name="@Specification_1_pay_from_with_metadata"></a>

### Function `pay_from_with_metadata`


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_pay_from_with_metadata">pay_from_with_metadata</a>&lt;TokenType: store&gt;(account: &signer, payee: <b>address</b>, amount: u128, metadata: vector&lt;u8&gt;)
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
<b>aborts_if</b> <b>global</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)).token.value &lt; amount;
<b>aborts_if</b> <a href="Option.md#0x1_Option_is_none">Option::is_none</a>(<b>global</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)).withdrawal_capability);
<b>aborts_if</b> amount == 0;
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(payee);
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(payee);
<b>aborts_if</b> <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) != payee && <b>global</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(payee).token.value + amount &gt; max_u128();
</code></pre>




<a name="0x1_Account_DepositWithPayerAndMetadataAbortsIf"></a>


<pre><code><b>schema</b> <a href="Account.md#0x1_Account_DepositWithPayerAndMetadataAbortsIf">DepositWithPayerAndMetadataAbortsIf</a>&lt;TokenType&gt; {
    payer: <b>address</b>;
    payee: <b>address</b>;
    to_deposit: <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt;;
    <b>aborts_if</b> to_deposit.value == 0;
    <b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(payer);
    <b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(payee);
    <b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(payee);
    <b>aborts_if</b> <b>global</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(payee).token.value + to_deposit.value &gt; max_u128();
}
</code></pre>



<a name="@Specification_1_pay_from"></a>

### Function `pay_from`


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_pay_from">pay_from</a>&lt;TokenType: store&gt;(account: &signer, payee: <b>address</b>, amount: u128)
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
<b>aborts_if</b> <b>global</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)).token.value &lt; amount;
<b>aborts_if</b> <a href="Option.md#0x1_Option_is_none">Option::is_none</a>(<b>global</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)).withdrawal_capability);
<b>aborts_if</b> amount == 0;
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(payee);
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(payee);
<b>aborts_if</b> <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) != payee && <b>global</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(payee).token.value + amount &gt; max_u128();
</code></pre>



<a name="@Specification_1_rotate_authentication_key_with_capability"></a>

### Function `rotate_authentication_key_with_capability`


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_rotate_authentication_key_with_capability">rotate_authentication_key_with_capability</a>(cap: &<a href="Account.md#0x1_Account_KeyRotationCapability">Account::KeyRotationCapability</a>, new_authentication_key: vector&lt;u8&gt;)
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(cap.account_address);
<b>aborts_if</b> len(new_authentication_key) != 32;
<b>ensures</b> <b>global</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(cap.account_address).authentication_key == new_authentication_key;
</code></pre>




<a name="0x1_Account_spec_rotate_authentication_key_with_capability"></a>


<pre><code><b>fun</b> <a href="Account.md#0x1_Account_spec_rotate_authentication_key_with_capability">spec_rotate_authentication_key_with_capability</a>(addr: <b>address</b>, new_authentication_key: vector&lt;u8&gt;): bool {
   <b>global</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(addr).authentication_key == new_authentication_key
}
</code></pre>



<a name="@Specification_1_extract_key_rotation_capability"></a>

### Function `extract_key_rotation_capability`


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_extract_key_rotation_capability">extract_key_rotation_capability</a>(account: &signer): <a href="Account.md#0x1_Account_KeyRotationCapability">Account::KeyRotationCapability</a>
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
<b>aborts_if</b> <a href="Option.md#0x1_Option_is_none">Option::is_none</a>(<b>global</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)).key_rotation_capability);
</code></pre>



<a name="@Specification_1_restore_key_rotation_capability"></a>

### Function `restore_key_rotation_capability`


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_restore_key_rotation_capability">restore_key_rotation_capability</a>(cap: <a href="Account.md#0x1_Account_KeyRotationCapability">Account::KeyRotationCapability</a>)
</code></pre>




<pre><code><b>aborts_if</b> <a href="Option.md#0x1_Option_is_some">Option::is_some</a>(<b>global</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(cap.account_address).key_rotation_capability);
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(cap.account_address);
</code></pre>



<a name="@Specification_1_rotate_authentication_key"></a>

### Function `rotate_authentication_key`


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="Account.md#0x1_Account_rotate_authentication_key">rotate_authentication_key</a>(account: signer, new_key: vector&lt;u8&gt;)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>



<a name="@Specification_1_balance_for"></a>

### Function `balance_for`


<pre><code><b>fun</b> <a href="Account.md#0x1_Account_balance_for">balance_for</a>&lt;TokenType: store&gt;(balance: &<a href="Account.md#0x1_Account_Balance">Account::Balance</a>&lt;TokenType&gt;): u128
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="@Specification_1_do_accept_token"></a>

### Function `do_accept_token`


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_do_accept_token">do_accept_token</a>&lt;TokenType: store&gt;(account: &signer)
</code></pre>




<pre><code><b>aborts_if</b> <b>exists</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
</code></pre>



<a name="@Specification_1_accept_token"></a>

### Function `accept_token`


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="Account.md#0x1_Account_accept_token">accept_token</a>&lt;TokenType: store&gt;(account: signer)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>



<a name="@Specification_1_is_accepts_token"></a>

### Function `is_accepts_token`


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_is_accepts_token">is_accepts_token</a>&lt;TokenType: store&gt;(addr: <b>address</b>): bool
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="@Specification_1_is_accept_token"></a>

### Function `is_accept_token`


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_is_accept_token">is_accept_token</a>&lt;TokenType: store&gt;(addr: <b>address</b>): bool
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="@Specification_1_set_auto_accept_token"></a>

### Function `set_auto_accept_token`


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_set_auto_accept_token">set_auto_accept_token</a>(account: &signer, enable: bool)
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="@Specification_1_try_accept_token"></a>

### Function `try_accept_token`


<pre><code><b>fun</b> <a href="Account.md#0x1_Account_try_accept_token">try_accept_token</a>&lt;TokenType: store&gt;(addr: <b>address</b>)
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="@Specification_1_sequence_number"></a>

### Function `sequence_number`


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_sequence_number">sequence_number</a>(addr: <b>address</b>): u64
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(addr);
</code></pre>



<a name="@Specification_1_authentication_key"></a>

### Function `authentication_key`


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_authentication_key">authentication_key</a>(addr: <b>address</b>): vector&lt;u8&gt;
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(addr);
</code></pre>



<a name="@Specification_1_delegated_key_rotation_capability"></a>

### Function `delegated_key_rotation_capability`


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_delegated_key_rotation_capability">delegated_key_rotation_capability</a>(addr: <b>address</b>): bool
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(addr);
</code></pre>



<a name="@Specification_1_delegated_withdraw_capability"></a>

### Function `delegated_withdraw_capability`


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_delegated_withdraw_capability">delegated_withdraw_capability</a>(addr: <b>address</b>): bool
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(addr);
</code></pre>



<a name="@Specification_1_withdraw_capability_address"></a>

### Function `withdraw_capability_address`


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_withdraw_capability_address">withdraw_capability_address</a>(cap: &<a href="Account.md#0x1_Account_WithdrawCapability">Account::WithdrawCapability</a>): &<b>address</b>
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="@Specification_1_key_rotation_capability_address"></a>

### Function `key_rotation_capability_address`


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_key_rotation_capability_address">key_rotation_capability_address</a>(cap: &<a href="Account.md#0x1_Account_KeyRotationCapability">Account::KeyRotationCapability</a>): &<b>address</b>
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="@Specification_1_exists_at"></a>

### Function `exists_at`


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_exists_at">exists_at</a>(check_addr: <b>address</b>): bool
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="@Specification_1_txn_prologue"></a>

### Function `txn_prologue`


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_txn_prologue">txn_prologue</a>&lt;TokenType: store&gt;(account: &signer, txn_sender: <b>address</b>, txn_sequence_number: u64, txn_authentication_key_preimage: vector&lt;u8&gt;, txn_gas_price: u64, txn_max_gas_units: u64)
</code></pre>




<pre><code><b>aborts_if</b> <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) != <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>();
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(txn_sender);
<b>aborts_if</b> <b>global</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(txn_sender).authentication_key == <a href="Account.md#0x1_Account_DUMMY_AUTH_KEY">DUMMY_AUTH_KEY</a> && <a href="Authenticator.md#0x1_Authenticator_spec_derived_address">Authenticator::spec_derived_address</a>(<a href="Hash.md#0x1_Hash_sha3_256">Hash::sha3_256</a>(txn_authentication_key_preimage)) != txn_sender;
<b>aborts_if</b> <b>global</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(txn_sender).authentication_key != <a href="Account.md#0x1_Account_DUMMY_AUTH_KEY">DUMMY_AUTH_KEY</a> && <a href="Hash.md#0x1_Hash_sha3_256">Hash::sha3_256</a>(txn_authentication_key_preimage) != <b>global</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(txn_sender).authentication_key;
<b>aborts_if</b> txn_gas_price * txn_max_gas_units &gt; max_u64();
<b>aborts_if</b> txn_gas_price * txn_max_gas_units &gt; 0 && !<b>exists</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(txn_sender);
<b>aborts_if</b> txn_gas_price * txn_max_gas_units &gt; 0 && <a href="Token.md#0x1_Token_spec_token_code">Token::spec_token_code</a>&lt;TokenType&gt;() != <a href="Token.md#0x1_Token_spec_token_code">Token::spec_token_code</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;();
<b>aborts_if</b> txn_gas_price * txn_max_gas_units &gt; 0 && <b>global</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(txn_sender).token.value &lt; txn_gas_price * txn_max_gas_units;
<b>aborts_if</b> txn_gas_price * txn_max_gas_units &gt; 0 && txn_sequence_number &gt;= max_u64();
<b>aborts_if</b> txn_sequence_number &lt; <b>global</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(txn_sender).sequence_number;
<b>aborts_if</b> txn_sequence_number != <b>global</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(txn_sender).sequence_number;
</code></pre>



<a name="@Specification_1_txn_epilogue"></a>

### Function `txn_epilogue`


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_txn_epilogue">txn_epilogue</a>&lt;TokenType: store&gt;(account: &signer, txn_sender: <b>address</b>, txn_sequence_number: u64, txn_gas_price: u64, txn_max_gas_units: u64, gas_units_remaining: u64)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>



<a name="@Specification_1_txn_epilogue_v2"></a>

### Function `txn_epilogue_v2`


<pre><code><b>public</b> <b>fun</b> <a href="Account.md#0x1_Account_txn_epilogue_v2">txn_epilogue_v2</a>&lt;TokenType: store&gt;(account: &signer, txn_sender: <b>address</b>, txn_sequence_number: u64, txn_authentication_key_preimage: vector&lt;u8&gt;, txn_gas_price: u64, txn_max_gas_units: u64, gas_units_remaining: u64)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
<b>aborts_if</b> <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) != <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>();
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account">Account</a>&gt;(txn_sender);
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(txn_sender);
<b>aborts_if</b> txn_max_gas_units &lt; gas_units_remaining;
<b>let</b> transaction_fee_amount = txn_gas_price * (txn_max_gas_units - gas_units_remaining);
<b>aborts_if</b> transaction_fee_amount &gt; max_u128();
<b>aborts_if</b> <b>global</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(txn_sender).token.value &lt; transaction_fee_amount;
<b>aborts_if</b> txn_sequence_number + 1 &gt; max_u64();
<b>aborts_if</b> txn_gas_price * (txn_max_gas_units - gas_units_remaining) &gt; 0 &&
        <b>global</b>&lt;<a href="Account.md#0x1_Account_Balance">Balance</a>&lt;TokenType&gt;&gt;(txn_sender).token.value  &lt; txn_gas_price * (txn_max_gas_units - gas_units_remaining);
<b>aborts_if</b> txn_gas_price * (txn_max_gas_units - gas_units_remaining) &gt; 0 &&
        !<b>exists</b>&lt;<a href="TransactionFee.md#0x1_TransactionFee_TransactionFee">TransactionFee::TransactionFee</a>&lt;TokenType&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
<b>aborts_if</b> txn_gas_price * (txn_max_gas_units - gas_units_remaining) &gt; 0 &&
        <b>global</b>&lt;<a href="TransactionFee.md#0x1_TransactionFee_TransactionFee">TransactionFee::TransactionFee</a>&lt;TokenType&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>()).fee.value + txn_gas_price * (txn_max_gas_units - gas_units_remaining) &gt; max_u128();
</code></pre>
