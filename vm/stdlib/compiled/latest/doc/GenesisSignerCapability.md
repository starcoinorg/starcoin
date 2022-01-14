
<a name="0x1_GenesisSignerCapability"></a>

# Module `0x1::GenesisSignerCapability`



-  [Resource `GenesisSignerCapability`](#0x1_GenesisSignerCapability_GenesisSignerCapability)
-  [Constants](#@Constants_0)
-  [Function `initialize`](#0x1_GenesisSignerCapability_initialize)
-  [Function `get_genesis_signer`](#0x1_GenesisSignerCapability_get_genesis_signer)


<pre><code><b>use</b> <a href="Account.md#0x1_Account">0x1::Account</a>;
<b>use</b> <a href="CoreAddresses.md#0x1_CoreAddresses">0x1::CoreAddresses</a>;
<b>use</b> <a href="Errors.md#0x1_Errors">0x1::Errors</a>;
</code></pre>



<a name="0x1_GenesisSignerCapability_GenesisSignerCapability"></a>

## Resource `GenesisSignerCapability`



<pre><code><b>struct</b> <a href="GenesisSignerCapability.md#0x1_GenesisSignerCapability">GenesisSignerCapability</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>cap: <a href="Account.md#0x1_Account_SignerCapability">Account::SignerCapability</a></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_GenesisSignerCapability_ENOT_GENESIS_ACCOUNT"></a>



<pre><code><b>const</b> <a href="GenesisSignerCapability.md#0x1_GenesisSignerCapability_ENOT_GENESIS_ACCOUNT">ENOT_GENESIS_ACCOUNT</a>: u64 = 11;
</code></pre>



<a name="0x1_GenesisSignerCapability_initialize"></a>

## Function `initialize`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="GenesisSignerCapability.md#0x1_GenesisSignerCapability_initialize">initialize</a>(signer: &signer, cap: <a href="Account.md#0x1_Account_SignerCapability">Account::SignerCapability</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="GenesisSignerCapability.md#0x1_GenesisSignerCapability_initialize">initialize</a>(signer:&signer, cap: <a href="Account.md#0x1_Account_SignerCapability">Account::SignerCapability</a>) {
    <a href="CoreAddresses.md#0x1_CoreAddresses_assert_genesis_address">CoreAddresses::assert_genesis_address</a>(signer);
    <b>assert</b>!(<a href="Account.md#0x1_Account_signer_address">Account::signer_address</a>(&cap) == <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>(), <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="GenesisSignerCapability.md#0x1_GenesisSignerCapability_ENOT_GENESIS_ACCOUNT">ENOT_GENESIS_ACCOUNT</a>));
    <b>move_to</b>(signer, <a href="GenesisSignerCapability.md#0x1_GenesisSignerCapability">GenesisSignerCapability</a>{cap});
}
</code></pre>



</details>

<a name="0x1_GenesisSignerCapability_get_genesis_signer"></a>

## Function `get_genesis_signer`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="GenesisSignerCapability.md#0x1_GenesisSignerCapability_get_genesis_signer">get_genesis_signer</a>(): signer
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="GenesisSignerCapability.md#0x1_GenesisSignerCapability_get_genesis_signer">get_genesis_signer</a>(): signer <b>acquires</b> <a href="GenesisSignerCapability.md#0x1_GenesisSignerCapability">GenesisSignerCapability</a> {
    <b>let</b> cap = <b>borrow_global</b>&lt;<a href="GenesisSignerCapability.md#0x1_GenesisSignerCapability">GenesisSignerCapability</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
    <a href="Account.md#0x1_Account_create_signer_with_cap">Account::create_signer_with_cap</a>(&cap.cap)
}
</code></pre>



</details>
