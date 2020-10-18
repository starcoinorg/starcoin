
<a name="0x1_Errors"></a>

# Module `0x1::Errors`

Module defining error codes used in Move aborts throughout the framework.

A <code>u64</code> error code is constructed from two values:

1. The *error category* which is encoded in the lower 8 bits of the code. Error categories are
declared in this module and are globally unique across the Libra framework. There is a limited
fixed set of predefined categories, and the framework is guaranteed to use those consistently.

2. The *error reason* which is encoded in the remaining 56 bits of the code. The reason is a unique
number relative to the module which raised the error and can be used to obtain more information about
the error at hand. It is mostly used for diagnosis purposes. Error reasons may change over time as the
framework evolves.


-  [Constants](#@Constants_0)
-  [Function `PROLOGUE_ACCOUNT_DOES_NOT_EXIST`](#0x1_Errors_PROLOGUE_ACCOUNT_DOES_NOT_EXIST)
-  [Function `PROLOGUE_INVALID_ACCOUNT_AUTH_KEY`](#0x1_Errors_PROLOGUE_INVALID_ACCOUNT_AUTH_KEY)
-  [Function `PROLOGUE_SEQUENCE_NUMBER_TOO_OLD`](#0x1_Errors_PROLOGUE_SEQUENCE_NUMBER_TOO_OLD)
-  [Function `PROLOGUE_SEQUENCE_NUMBER_TOO_NEW`](#0x1_Errors_PROLOGUE_SEQUENCE_NUMBER_TOO_NEW)
-  [Function `PROLOGUE_CANT_PAY_GAS_DEPOSIT`](#0x1_Errors_PROLOGUE_CANT_PAY_GAS_DEPOSIT)
-  [Function `PROLOGUE_TRANSACTION_EXPIRED`](#0x1_Errors_PROLOGUE_TRANSACTION_EXPIRED)
-  [Function `PROLOGUE_BAD_CHAIN_ID`](#0x1_Errors_PROLOGUE_BAD_CHAIN_ID)
-  [Function `PROLOGUE_MODULE_NOT_ALLOWED`](#0x1_Errors_PROLOGUE_MODULE_NOT_ALLOWED)
-  [Function `PROLOGUE_SCRIPT_NOT_ALLOWED`](#0x1_Errors_PROLOGUE_SCRIPT_NOT_ALLOWED)
-  [Function `EINSUFFICIENT_BALANCE`](#0x1_Errors_EINSUFFICIENT_BALANCE)
-  [Function `ENOT_GENESIS_ACCOUNT`](#0x1_Errors_ENOT_GENESIS_ACCOUNT)
-  [Function `ENOT_GENESIS`](#0x1_Errors_ENOT_GENESIS)
-  [Function `ECONFIG_VALUE_DOES_NOT_EXIST`](#0x1_Errors_ECONFIG_VALUE_DOES_NOT_EXIST)
-  [Function `EINVALID_TIMESTAMP`](#0x1_Errors_EINVALID_TIMESTAMP)
-  [Function `ECOIN_DEPOSIT_IS_ZERO`](#0x1_Errors_ECOIN_DEPOSIT_IS_ZERO)
-  [Function `EDESTORY_TOKEN_NON_ZERO`](#0x1_Errors_EDESTORY_TOKEN_NON_ZERO)
-  [Function `EBLOCK_NUMBER_MISMATCH`](#0x1_Errors_EBLOCK_NUMBER_MISMATCH)
-  [Function `EINVALID_ARGUMENT`](#0x1_Errors_EINVALID_ARGUMENT)
-  [Function `EUNREACHABLE`](#0x1_Errors_EUNREACHABLE)
-  [Function `make`](#0x1_Errors_make)
-  [Function `invalid_state`](#0x1_Errors_invalid_state)
-  [Function `requires_address`](#0x1_Errors_requires_address)
-  [Function `requires_role`](#0x1_Errors_requires_role)
-  [Function `requires_capability`](#0x1_Errors_requires_capability)
-  [Function `not_published`](#0x1_Errors_not_published)
-  [Function `already_published`](#0x1_Errors_already_published)
-  [Function `invalid_argument`](#0x1_Errors_invalid_argument)
-  [Function `limit_exceeded`](#0x1_Errors_limit_exceeded)
-  [Function `internal`](#0x1_Errors_internal)
-  [Function `custom`](#0x1_Errors_custom)
-  [Specification](#@Specification_1)
    -  [Function `make`](#@Specification_1_make)
    -  [Function `invalid_state`](#@Specification_1_invalid_state)
    -  [Function `requires_address`](#@Specification_1_requires_address)
    -  [Function `requires_role`](#@Specification_1_requires_role)
    -  [Function `requires_capability`](#@Specification_1_requires_capability)
    -  [Function `not_published`](#@Specification_1_not_published)
    -  [Function `already_published`](#@Specification_1_already_published)
    -  [Function `invalid_argument`](#@Specification_1_invalid_argument)
    -  [Function `limit_exceeded`](#@Specification_1_limit_exceeded)
    -  [Function `internal`](#@Specification_1_internal)
    -  [Function `custom`](#@Specification_1_custom)


<pre><code></code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x1_Errors_ALREADY_PUBLISHED"></a>

Attempting to publish a resource that is already published. Example: calling an initialization function
twice.


<pre><code><b>const</b> <a href="Errors.md#0x1_Errors_ALREADY_PUBLISHED">ALREADY_PUBLISHED</a>: u8 = 6;
</code></pre>



<a name="0x1_Errors_CUSTOM"></a>

A custom error category for extension points.


<pre><code><b>const</b> <a href="Errors.md#0x1_Errors_CUSTOM">CUSTOM</a>: u8 = 255;
</code></pre>



<a name="0x1_Errors_INTERNAL"></a>

An internal error (bug) has occurred.


<pre><code><b>const</b> <a href="Errors.md#0x1_Errors_INTERNAL">INTERNAL</a>: u8 = 10;
</code></pre>



<a name="0x1_Errors_INVALID_ARGUMENT"></a>

An argument provided to an operation is invalid. Example: a signing key has the wrong format.


<pre><code><b>const</b> <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">INVALID_ARGUMENT</a>: u8 = 7;
</code></pre>



<a name="0x1_Errors_INVALID_STATE"></a>

The system is in a state where the performed operation is not allowed. Example: call to a function only allowed
in genesis.


<pre><code><b>const</b> <a href="Errors.md#0x1_Errors_INVALID_STATE">INVALID_STATE</a>: u8 = 1;
</code></pre>



<a name="0x1_Errors_LIMIT_EXCEEDED"></a>

A limit on an amount, e.g. a currency, is exceeded. Example: withdrawal of money after account limits window
is exhausted.


<pre><code><b>const</b> <a href="Errors.md#0x1_Errors_LIMIT_EXCEEDED">LIMIT_EXCEEDED</a>: u8 = 8;
</code></pre>



<a name="0x1_Errors_NOT_PUBLISHED"></a>

A resource is required but not published. Example: access to non-existing AccountLimits resource.


<pre><code><b>const</b> <a href="Errors.md#0x1_Errors_NOT_PUBLISHED">NOT_PUBLISHED</a>: u8 = 5;
</code></pre>



<a name="0x1_Errors_REQUIRES_ADDRESS"></a>

The signer of a transaction does not have the expected address for this operation. Example: a call to a function
which publishes a resource under a particular address.


<pre><code><b>const</b> <a href="Errors.md#0x1_Errors_REQUIRES_ADDRESS">REQUIRES_ADDRESS</a>: u8 = 2;
</code></pre>



<a name="0x1_Errors_REQUIRES_CAPABILITY"></a>

The signer of a transaction does not have a required capability.


<pre><code><b>const</b> <a href="Errors.md#0x1_Errors_REQUIRES_CAPABILITY">REQUIRES_CAPABILITY</a>: u8 = 4;
</code></pre>



<a name="0x1_Errors_REQUIRES_ROLE"></a>

The signer of a transaction does not have the expected  role for this operation. Example: a call to a function
which requires the signer to have the role of treasury compliance.


<pre><code><b>const</b> <a href="Errors.md#0x1_Errors_REQUIRES_ROLE">REQUIRES_ROLE</a>: u8 = 3;
</code></pre>



<a name="0x1_Errors_PROLOGUE_ACCOUNT_DOES_NOT_EXIST"></a>

## Function `PROLOGUE_ACCOUNT_DOES_NOT_EXIST`



<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_PROLOGUE_ACCOUNT_DOES_NOT_EXIST">PROLOGUE_ACCOUNT_DOES_NOT_EXIST</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_PROLOGUE_ACCOUNT_DOES_NOT_EXIST">PROLOGUE_ACCOUNT_DOES_NOT_EXIST</a>(): u64 {0}
</code></pre>



</details>

<a name="0x1_Errors_PROLOGUE_INVALID_ACCOUNT_AUTH_KEY"></a>

## Function `PROLOGUE_INVALID_ACCOUNT_AUTH_KEY`



<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_PROLOGUE_INVALID_ACCOUNT_AUTH_KEY">PROLOGUE_INVALID_ACCOUNT_AUTH_KEY</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_PROLOGUE_INVALID_ACCOUNT_AUTH_KEY">PROLOGUE_INVALID_ACCOUNT_AUTH_KEY</a>(): u64 {1}
</code></pre>



</details>

<a name="0x1_Errors_PROLOGUE_SEQUENCE_NUMBER_TOO_OLD"></a>

## Function `PROLOGUE_SEQUENCE_NUMBER_TOO_OLD`



<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_PROLOGUE_SEQUENCE_NUMBER_TOO_OLD">PROLOGUE_SEQUENCE_NUMBER_TOO_OLD</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_PROLOGUE_SEQUENCE_NUMBER_TOO_OLD">PROLOGUE_SEQUENCE_NUMBER_TOO_OLD</a>(): u64 {2}
</code></pre>



</details>

<a name="0x1_Errors_PROLOGUE_SEQUENCE_NUMBER_TOO_NEW"></a>

## Function `PROLOGUE_SEQUENCE_NUMBER_TOO_NEW`



<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_PROLOGUE_SEQUENCE_NUMBER_TOO_NEW">PROLOGUE_SEQUENCE_NUMBER_TOO_NEW</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_PROLOGUE_SEQUENCE_NUMBER_TOO_NEW">PROLOGUE_SEQUENCE_NUMBER_TOO_NEW</a>(): u64 {3}
</code></pre>



</details>

<a name="0x1_Errors_PROLOGUE_CANT_PAY_GAS_DEPOSIT"></a>

## Function `PROLOGUE_CANT_PAY_GAS_DEPOSIT`



<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_PROLOGUE_CANT_PAY_GAS_DEPOSIT">PROLOGUE_CANT_PAY_GAS_DEPOSIT</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_PROLOGUE_CANT_PAY_GAS_DEPOSIT">PROLOGUE_CANT_PAY_GAS_DEPOSIT</a>(): u64 {4}
</code></pre>



</details>

<a name="0x1_Errors_PROLOGUE_TRANSACTION_EXPIRED"></a>

## Function `PROLOGUE_TRANSACTION_EXPIRED`



<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_PROLOGUE_TRANSACTION_EXPIRED">PROLOGUE_TRANSACTION_EXPIRED</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_PROLOGUE_TRANSACTION_EXPIRED">PROLOGUE_TRANSACTION_EXPIRED</a>(): u64 {5}
</code></pre>



</details>

<a name="0x1_Errors_PROLOGUE_BAD_CHAIN_ID"></a>

## Function `PROLOGUE_BAD_CHAIN_ID`



<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_PROLOGUE_BAD_CHAIN_ID">PROLOGUE_BAD_CHAIN_ID</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_PROLOGUE_BAD_CHAIN_ID">PROLOGUE_BAD_CHAIN_ID</a>(): u64 {6}
</code></pre>



</details>

<a name="0x1_Errors_PROLOGUE_MODULE_NOT_ALLOWED"></a>

## Function `PROLOGUE_MODULE_NOT_ALLOWED`



<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_PROLOGUE_MODULE_NOT_ALLOWED">PROLOGUE_MODULE_NOT_ALLOWED</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_PROLOGUE_MODULE_NOT_ALLOWED">PROLOGUE_MODULE_NOT_ALLOWED</a>(): u64 {7}
</code></pre>



</details>

<a name="0x1_Errors_PROLOGUE_SCRIPT_NOT_ALLOWED"></a>

## Function `PROLOGUE_SCRIPT_NOT_ALLOWED`



<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_PROLOGUE_SCRIPT_NOT_ALLOWED">PROLOGUE_SCRIPT_NOT_ALLOWED</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_PROLOGUE_SCRIPT_NOT_ALLOWED">PROLOGUE_SCRIPT_NOT_ALLOWED</a>(): u64 {8}
</code></pre>



</details>

<a name="0x1_Errors_EINSUFFICIENT_BALANCE"></a>

## Function `EINSUFFICIENT_BALANCE`



<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_EINSUFFICIENT_BALANCE">EINSUFFICIENT_BALANCE</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_EINSUFFICIENT_BALANCE">EINSUFFICIENT_BALANCE</a>(): u64 {10}
</code></pre>



</details>

<a name="0x1_Errors_ENOT_GENESIS_ACCOUNT"></a>

## Function `ENOT_GENESIS_ACCOUNT`



<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_ENOT_GENESIS_ACCOUNT">ENOT_GENESIS_ACCOUNT</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_ENOT_GENESIS_ACCOUNT">ENOT_GENESIS_ACCOUNT</a>(): u64 {11}
</code></pre>



</details>

<a name="0x1_Errors_ENOT_GENESIS"></a>

## Function `ENOT_GENESIS`



<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_ENOT_GENESIS">ENOT_GENESIS</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_ENOT_GENESIS">ENOT_GENESIS</a>(): u64 {12}
</code></pre>



</details>

<a name="0x1_Errors_ECONFIG_VALUE_DOES_NOT_EXIST"></a>

## Function `ECONFIG_VALUE_DOES_NOT_EXIST`



<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_ECONFIG_VALUE_DOES_NOT_EXIST">ECONFIG_VALUE_DOES_NOT_EXIST</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_ECONFIG_VALUE_DOES_NOT_EXIST">ECONFIG_VALUE_DOES_NOT_EXIST</a>(): u64 {13}
</code></pre>



</details>

<a name="0x1_Errors_EINVALID_TIMESTAMP"></a>

## Function `EINVALID_TIMESTAMP`



<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_EINVALID_TIMESTAMP">EINVALID_TIMESTAMP</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_EINVALID_TIMESTAMP">EINVALID_TIMESTAMP</a>(): u64 {14}
</code></pre>



</details>

<a name="0x1_Errors_ECOIN_DEPOSIT_IS_ZERO"></a>

## Function `ECOIN_DEPOSIT_IS_ZERO`



<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_ECOIN_DEPOSIT_IS_ZERO">ECOIN_DEPOSIT_IS_ZERO</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_ECOIN_DEPOSIT_IS_ZERO">ECOIN_DEPOSIT_IS_ZERO</a>(): u64 {15}
</code></pre>



</details>

<a name="0x1_Errors_EDESTORY_TOKEN_NON_ZERO"></a>

## Function `EDESTORY_TOKEN_NON_ZERO`



<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_EDESTORY_TOKEN_NON_ZERO">EDESTORY_TOKEN_NON_ZERO</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_EDESTORY_TOKEN_NON_ZERO">EDESTORY_TOKEN_NON_ZERO</a>(): u64 {16}
</code></pre>



</details>

<a name="0x1_Errors_EBLOCK_NUMBER_MISMATCH"></a>

## Function `EBLOCK_NUMBER_MISMATCH`



<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_EBLOCK_NUMBER_MISMATCH">EBLOCK_NUMBER_MISMATCH</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_EBLOCK_NUMBER_MISMATCH">EBLOCK_NUMBER_MISMATCH</a>(): u64 {17}
</code></pre>



</details>

<a name="0x1_Errors_EINVALID_ARGUMENT"></a>

## Function `EINVALID_ARGUMENT`

Invalid argument.


<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_EINVALID_ARGUMENT">EINVALID_ARGUMENT</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_EINVALID_ARGUMENT">EINVALID_ARGUMENT</a>(): u64 {18}
</code></pre>



</details>

<a name="0x1_Errors_EUNREACHABLE"></a>

## Function `EUNREACHABLE`

There code should unreacheable


<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_EUNREACHABLE">EUNREACHABLE</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_EUNREACHABLE">EUNREACHABLE</a>(): u64 {19}
</code></pre>



</details>

<a name="0x1_Errors_make"></a>

## Function `make`

A function to create an error from from a category and a reason.


<pre><code><b>fun</b> <a href="Errors.md#0x1_Errors_make">make</a>(category: u8, reason: u64): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="Errors.md#0x1_Errors_make">make</a>(category: u8, reason: u64): u64 {
    (category <b>as</b> u64) + (reason &lt;&lt; 8)
}
</code></pre>



</details>

<a name="0x1_Errors_invalid_state"></a>

## Function `invalid_state`



<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_invalid_state">invalid_state</a>(reason: u64): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_invalid_state">invalid_state</a>(reason: u64): u64 { <a href="Errors.md#0x1_Errors_make">make</a>(<a href="Errors.md#0x1_Errors_INVALID_STATE">INVALID_STATE</a>, reason) }
</code></pre>



</details>

<a name="0x1_Errors_requires_address"></a>

## Function `requires_address`



<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_requires_address">requires_address</a>(reason: u64): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_requires_address">requires_address</a>(reason: u64): u64 { <a href="Errors.md#0x1_Errors_make">make</a>(<a href="Errors.md#0x1_Errors_REQUIRES_ADDRESS">REQUIRES_ADDRESS</a>, reason) }
</code></pre>



</details>

<a name="0x1_Errors_requires_role"></a>

## Function `requires_role`



<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_requires_role">requires_role</a>(reason: u64): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_requires_role">requires_role</a>(reason: u64): u64 { <a href="Errors.md#0x1_Errors_make">make</a>(<a href="Errors.md#0x1_Errors_REQUIRES_ROLE">REQUIRES_ROLE</a>, reason) }
</code></pre>



</details>

<a name="0x1_Errors_requires_capability"></a>

## Function `requires_capability`



<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_requires_capability">requires_capability</a>(reason: u64): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_requires_capability">requires_capability</a>(reason: u64): u64 { <a href="Errors.md#0x1_Errors_make">make</a>(<a href="Errors.md#0x1_Errors_REQUIRES_CAPABILITY">REQUIRES_CAPABILITY</a>, reason) }
</code></pre>



</details>

<a name="0x1_Errors_not_published"></a>

## Function `not_published`



<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_not_published">not_published</a>(reason: u64): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_not_published">not_published</a>(reason: u64): u64 { <a href="Errors.md#0x1_Errors_make">make</a>(<a href="Errors.md#0x1_Errors_NOT_PUBLISHED">NOT_PUBLISHED</a>, reason) }
</code></pre>



</details>

<a name="0x1_Errors_already_published"></a>

## Function `already_published`



<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_already_published">already_published</a>(reason: u64): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_already_published">already_published</a>(reason: u64): u64 { <a href="Errors.md#0x1_Errors_make">make</a>(<a href="Errors.md#0x1_Errors_ALREADY_PUBLISHED">ALREADY_PUBLISHED</a>, reason) }
</code></pre>



</details>

<a name="0x1_Errors_invalid_argument"></a>

## Function `invalid_argument`



<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_invalid_argument">invalid_argument</a>(reason: u64): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_invalid_argument">invalid_argument</a>(reason: u64): u64 { <a href="Errors.md#0x1_Errors_make">make</a>(<a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">INVALID_ARGUMENT</a>, reason) }
</code></pre>



</details>

<a name="0x1_Errors_limit_exceeded"></a>

## Function `limit_exceeded`



<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_limit_exceeded">limit_exceeded</a>(reason: u64): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_limit_exceeded">limit_exceeded</a>(reason: u64): u64 { <a href="Errors.md#0x1_Errors_make">make</a>(<a href="Errors.md#0x1_Errors_LIMIT_EXCEEDED">LIMIT_EXCEEDED</a>, reason) }
</code></pre>



</details>

<a name="0x1_Errors_internal"></a>

## Function `internal`



<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_internal">internal</a>(reason: u64): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_internal">internal</a>(reason: u64): u64 { <a href="Errors.md#0x1_Errors_make">make</a>(<a href="Errors.md#0x1_Errors_INTERNAL">INTERNAL</a>, reason) }
</code></pre>



</details>

<a name="0x1_Errors_custom"></a>

## Function `custom`



<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_custom">custom</a>(reason: u64): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_custom">custom</a>(reason: u64): u64 { <a href="Errors.md#0x1_Errors_make">make</a>(<a href="Errors.md#0x1_Errors_CUSTOM">CUSTOM</a>, reason) }
</code></pre>



</details>

<a name="@Specification_1"></a>

## Specification



<pre><code><b>pragma</b> verify;
<b>pragma</b> aborts_if_is_strict;
</code></pre>



<a name="@Specification_1_make"></a>

### Function `make`


<pre><code><b>fun</b> <a href="Errors.md#0x1_Errors_make">make</a>(category: u8, reason: u64): u64
</code></pre>




<pre><code><b>pragma</b> opaque = <b>true</b>;
<b>pragma</b> verify = <b>false</b>;
<b>aborts_if</b> [abstract] <b>false</b>;
<b>ensures</b> [abstract] result == category;
</code></pre>



<a name="@Specification_1_invalid_state"></a>

### Function `invalid_state`


<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_invalid_state">invalid_state</a>(reason: u64): u64
</code></pre>




<pre><code><b>pragma</b> opaque = <b>true</b>;
<b>aborts_if</b> <b>false</b>;
<b>ensures</b> result == <a href="Errors.md#0x1_Errors_INVALID_STATE">INVALID_STATE</a>;
</code></pre>



<a name="@Specification_1_requires_address"></a>

### Function `requires_address`


<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_requires_address">requires_address</a>(reason: u64): u64
</code></pre>




<pre><code><b>pragma</b> opaque = <b>true</b>;
<b>aborts_if</b> <b>false</b>;
<b>ensures</b> result == <a href="Errors.md#0x1_Errors_REQUIRES_ADDRESS">REQUIRES_ADDRESS</a>;
</code></pre>



<a name="@Specification_1_requires_role"></a>

### Function `requires_role`


<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_requires_role">requires_role</a>(reason: u64): u64
</code></pre>




<pre><code><b>pragma</b> opaque = <b>true</b>;
<b>aborts_if</b> <b>false</b>;
<b>ensures</b> result == <a href="Errors.md#0x1_Errors_REQUIRES_ROLE">REQUIRES_ROLE</a>;
</code></pre>



<a name="@Specification_1_requires_capability"></a>

### Function `requires_capability`


<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_requires_capability">requires_capability</a>(reason: u64): u64
</code></pre>




<pre><code><b>pragma</b> opaque = <b>true</b>;
<b>aborts_if</b> <b>false</b>;
<b>ensures</b> result == <a href="Errors.md#0x1_Errors_REQUIRES_CAPABILITY">REQUIRES_CAPABILITY</a>;
</code></pre>



<a name="@Specification_1_not_published"></a>

### Function `not_published`


<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_not_published">not_published</a>(reason: u64): u64
</code></pre>




<pre><code><b>pragma</b> opaque = <b>true</b>;
<b>aborts_if</b> <b>false</b>;
<b>ensures</b> result == <a href="Errors.md#0x1_Errors_NOT_PUBLISHED">NOT_PUBLISHED</a>;
</code></pre>



<a name="@Specification_1_already_published"></a>

### Function `already_published`


<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_already_published">already_published</a>(reason: u64): u64
</code></pre>




<pre><code><b>pragma</b> opaque = <b>true</b>;
<b>aborts_if</b> <b>false</b>;
<b>ensures</b> result == <a href="Errors.md#0x1_Errors_ALREADY_PUBLISHED">ALREADY_PUBLISHED</a>;
</code></pre>



<a name="@Specification_1_invalid_argument"></a>

### Function `invalid_argument`


<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_invalid_argument">invalid_argument</a>(reason: u64): u64
</code></pre>




<pre><code><b>pragma</b> opaque = <b>true</b>;
<b>aborts_if</b> <b>false</b>;
<b>ensures</b> result == <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">INVALID_ARGUMENT</a>;
</code></pre>



<a name="@Specification_1_limit_exceeded"></a>

### Function `limit_exceeded`


<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_limit_exceeded">limit_exceeded</a>(reason: u64): u64
</code></pre>




<pre><code><b>pragma</b> opaque = <b>true</b>;
<b>aborts_if</b> <b>false</b>;
<b>ensures</b> result == <a href="Errors.md#0x1_Errors_LIMIT_EXCEEDED">LIMIT_EXCEEDED</a>;
</code></pre>



<a name="@Specification_1_internal"></a>

### Function `internal`


<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_internal">internal</a>(reason: u64): u64
</code></pre>




<pre><code><b>pragma</b> opaque = <b>true</b>;
<b>aborts_if</b> <b>false</b>;
<b>ensures</b> result == <a href="Errors.md#0x1_Errors_INTERNAL">INTERNAL</a>;
</code></pre>



<a name="@Specification_1_custom"></a>

### Function `custom`


<pre><code><b>public</b> <b>fun</b> <a href="Errors.md#0x1_Errors_custom">custom</a>(reason: u64): u64
</code></pre>




<pre><code><b>pragma</b> opaque = <b>true</b>;
<b>aborts_if</b> <b>false</b>;
<b>ensures</b> result == <a href="Errors.md#0x1_Errors_CUSTOM">CUSTOM</a>;
</code></pre>
