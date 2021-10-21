
<a name="0x1_CoreAddresses"></a>

# Module `0x1::CoreAddresses`

The module provide addresses used in stdlib.


-  [Constants](#@Constants_0)
-  [Function `GENESIS_ADDRESS`](#0x1_CoreAddresses_GENESIS_ADDRESS)
-  [Function `assert_genesis_address`](#0x1_CoreAddresses_assert_genesis_address)
-  [Function `ASSOCIATION_ROOT_ADDRESS`](#0x1_CoreAddresses_ASSOCIATION_ROOT_ADDRESS)
-  [Function `VM_RESERVED_ADDRESS`](#0x1_CoreAddresses_VM_RESERVED_ADDRESS)
-  [Specification](#@Specification_1)
    -  [Function `assert_genesis_address`](#@Specification_1_assert_genesis_address)


<pre><code><b>use</b> <a href="Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="Signer.md#0x1_Signer">0x1::Signer</a>;
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x1_CoreAddresses_ENOT_GENESIS_ACCOUNT"></a>



<pre><code><b>const</b> <a href="CoreAddresses.md#0x1_CoreAddresses_ENOT_GENESIS_ACCOUNT">ENOT_GENESIS_ACCOUNT</a>: u64 = 11;
</code></pre>



<a name="0x1_CoreAddresses_GENESIS_ADDRESS"></a>

## Function `GENESIS_ADDRESS`

The address of the genesis


<pre><code><b>public</b> <b>fun</b> <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">GENESIS_ADDRESS</a>(): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">GENESIS_ADDRESS</a>(): <b>address</b> {
    @0x1
}
</code></pre>



</details>

<a name="0x1_CoreAddresses_assert_genesis_address"></a>

## Function `assert_genesis_address`

Assert signer is genesis.


<pre><code><b>public</b> <b>fun</b> <a href="CoreAddresses.md#0x1_CoreAddresses_assert_genesis_address">assert_genesis_address</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="CoreAddresses.md#0x1_CoreAddresses_assert_genesis_address">assert_genesis_address</a>(account: &signer) {
    <b>assert</b>!(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">GENESIS_ADDRESS</a>(), <a href="Errors.md#0x1_Errors_requires_address">Errors::requires_address</a>(<a href="CoreAddresses.md#0x1_CoreAddresses_ENOT_GENESIS_ACCOUNT">ENOT_GENESIS_ACCOUNT</a>))
}
</code></pre>



</details>

<a name="0x1_CoreAddresses_ASSOCIATION_ROOT_ADDRESS"></a>

## Function `ASSOCIATION_ROOT_ADDRESS`

The address of the root association account. This account is
created in genesis, and cannot be changed. This address has
ultimate authority over the permissions granted (or removed) from
accounts on-chain.


<pre><code><b>public</b> <b>fun</b> <a href="CoreAddresses.md#0x1_CoreAddresses_ASSOCIATION_ROOT_ADDRESS">ASSOCIATION_ROOT_ADDRESS</a>(): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="CoreAddresses.md#0x1_CoreAddresses_ASSOCIATION_ROOT_ADDRESS">ASSOCIATION_ROOT_ADDRESS</a>(): <b>address</b> {
    @0xA550C18
}
</code></pre>



</details>

<a name="0x1_CoreAddresses_VM_RESERVED_ADDRESS"></a>

## Function `VM_RESERVED_ADDRESS`

The reserved address for transactions inserted by the VM into blocks (e.g.
block metadata transactions). Because the transaction is sent from
the VM, an account _cannot_ exist at the <code>0x0</code> address since there
is no signer for the transaction.


<pre><code><b>public</b> <b>fun</b> <a href="CoreAddresses.md#0x1_CoreAddresses_VM_RESERVED_ADDRESS">VM_RESERVED_ADDRESS</a>(): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="CoreAddresses.md#0x1_CoreAddresses_VM_RESERVED_ADDRESS">VM_RESERVED_ADDRESS</a>(): <b>address</b> {
    @0x0
}
</code></pre>



</details>

<a name="@Specification_1"></a>

## Specification



<pre><code><b>pragma</b> verify;
<b>pragma</b> aborts_if_is_strict;
</code></pre>


Specification version of <code>Self::GENESIS_ACCOUNT</code>.


<a name="0x1_CoreAddresses_SPEC_GENESIS_ADDRESS"></a>


<pre><code><b>fun</b> <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">SPEC_GENESIS_ADDRESS</a>(): <b>address</b> {
   @0x1
}
</code></pre>


Specification version of <code><a href="CoreAddresses.md#0x1_CoreAddresses_ASSOCIATION_ROOT_ADDRESS">Self::ASSOCIATION_ROOT_ADDRESS</a></code>.


<a name="0x1_CoreAddresses_SPEC_ASSOCIATION_ROOT_ADDRESS"></a>


<pre><code><b>fun</b> <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_ASSOCIATION_ROOT_ADDRESS">SPEC_ASSOCIATION_ROOT_ADDRESS</a>(): <b>address</b> {
   @0xA550C18
}
</code></pre>


Specification version of <code><a href="CoreAddresses.md#0x1_CoreAddresses_VM_RESERVED_ADDRESS">Self::VM_RESERVED_ADDRESS</a></code>.


<a name="0x1_CoreAddresses_SPEC_VM_RESERVED_ADDRESS"></a>


<pre><code><b>fun</b> <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_VM_RESERVED_ADDRESS">SPEC_VM_RESERVED_ADDRESS</a>(): <b>address</b> {
   @0x0
}
</code></pre>



<a name="@Specification_1_assert_genesis_address"></a>

### Function `assert_genesis_address`


<pre><code><b>public</b> <b>fun</b> <a href="CoreAddresses.md#0x1_CoreAddresses_assert_genesis_address">assert_genesis_address</a>(account: &signer)
</code></pre>




<pre><code><b>pragma</b> opaque;
<b>include</b> <a href="CoreAddresses.md#0x1_CoreAddresses_AbortsIfNotGenesisAddress">AbortsIfNotGenesisAddress</a>;
</code></pre>


Specifies that a function aborts if the account does not have the Diem root address.


<a name="0x1_CoreAddresses_AbortsIfNotGenesisAddress"></a>


<pre><code><b>schema</b> <a href="CoreAddresses.md#0x1_CoreAddresses_AbortsIfNotGenesisAddress">AbortsIfNotGenesisAddress</a> {
    account: signer;
    <b>aborts_if</b> <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) != <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">SPEC_GENESIS_ADDRESS</a>();
}
</code></pre>
