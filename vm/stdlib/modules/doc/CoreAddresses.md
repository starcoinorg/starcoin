
<a name="0x1_CoreAddresses"></a>

# Module `0x1::CoreAddresses`



-  [Function <code>GENESIS_ADDRESS</code>](#0x1_CoreAddresses_GENESIS_ADDRESS)
-  [Function <code>ASSOCIATION_ROOT_ADDRESS</code>](#0x1_CoreAddresses_ASSOCIATION_ROOT_ADDRESS)
-  [Function <code>VM_RESERVED_ADDRESS</code>](#0x1_CoreAddresses_VM_RESERVED_ADDRESS)
-  [Specification](#@Specification_0)


<a name="0x1_CoreAddresses_GENESIS_ADDRESS"></a>

## Function `GENESIS_ADDRESS`

The address of the genesis


<pre><code><b>public</b> <b>fun</b> <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">GENESIS_ADDRESS</a>(): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">GENESIS_ADDRESS</a>(): address {
    0x1
}
</code></pre>



</details>

<a name="0x1_CoreAddresses_ASSOCIATION_ROOT_ADDRESS"></a>

## Function `ASSOCIATION_ROOT_ADDRESS`

The address of the root association account. This account is
created in genesis, and cannot be changed. This address has
ultimate authority over the permissions granted (or removed) from
accounts on-chain.


<pre><code><b>public</b> <b>fun</b> <a href="CoreAddresses.md#0x1_CoreAddresses_ASSOCIATION_ROOT_ADDRESS">ASSOCIATION_ROOT_ADDRESS</a>(): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="CoreAddresses.md#0x1_CoreAddresses_ASSOCIATION_ROOT_ADDRESS">ASSOCIATION_ROOT_ADDRESS</a>(): address {
    0xA550C18
}
</code></pre>



</details>

<a name="0x1_CoreAddresses_VM_RESERVED_ADDRESS"></a>

## Function `VM_RESERVED_ADDRESS`

The reserved address for transactions inserted by the VM into blocks (e.g.
block metadata transactions). Because the transaction is sent from
the VM, an account _cannot_ exist at the <code>0x0</code> address since there
is no signer for the transaction.


<pre><code><b>public</b> <b>fun</b> <a href="CoreAddresses.md#0x1_CoreAddresses_VM_RESERVED_ADDRESS">VM_RESERVED_ADDRESS</a>(): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="CoreAddresses.md#0x1_CoreAddresses_VM_RESERVED_ADDRESS">VM_RESERVED_ADDRESS</a>(): address {
    0x0
}
</code></pre>



</details>

<a name="@Specification_0"></a>

## Specification



<pre><code>pragma verify;
pragma aborts_if_is_strict;
</code></pre>


Specification version of <code>Self::GENESIS_ACCOUNT</code>.


<a name="0x1_CoreAddresses_SPEC_GENESIS_ADDRESS"></a>


<pre><code><b>define</b> <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">SPEC_GENESIS_ADDRESS</a>(): address {
    0x1
}
</code></pre>


Specification version of <code><a href="CoreAddresses.md#0x1_CoreAddresses_ASSOCIATION_ROOT_ADDRESS">Self::ASSOCIATION_ROOT_ADDRESS</a></code>.


<a name="0x1_CoreAddresses_SPEC_ASSOCIATION_ROOT_ADDRESS"></a>


<pre><code><b>define</b> <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_ASSOCIATION_ROOT_ADDRESS">SPEC_ASSOCIATION_ROOT_ADDRESS</a>(): address {
    0xA550C18
}
</code></pre>


Specification version of <code><a href="CoreAddresses.md#0x1_CoreAddresses_VM_RESERVED_ADDRESS">Self::VM_RESERVED_ADDRESS</a></code>.


<a name="0x1_CoreAddresses_SPEC_VM_RESERVED_ADDRESS"></a>


<pre><code><b>define</b> <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_VM_RESERVED_ADDRESS">SPEC_VM_RESERVED_ADDRESS</a>(): address {
    0x0
}
</code></pre>
