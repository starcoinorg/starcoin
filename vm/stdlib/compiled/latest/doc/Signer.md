
<a name="0x1_Signer"></a>

# Module `0x1::Signer`

Provide access methods for Signer.


-  [Function `borrow_address`](#0x1_Signer_borrow_address)
-  [Function `address_of`](#0x1_Signer_address_of)
-  [Specification](#@Specification_0)
    -  [Function `address_of`](#@Specification_0_address_of)


<pre><code></code></pre>



<a name="0x1_Signer_borrow_address"></a>

## Function `borrow_address`

Borrows the address of the signer
Conceptually, you can think of the <code>signer</code> as being a resource struct wrapper around an
address
```
resource struct Signer has key, store { addr: address }
```
<code>borrow_address</code> borrows this inner field


<pre><code><b>public</b> <b>fun</b> <a href="Signer.md#0x1_Signer_borrow_address">borrow_address</a>(s: &signer): &<b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>public</b> <b>fun</b> <a href="Signer.md#0x1_Signer_borrow_address">borrow_address</a>(s: &signer): &<b>address</b>;
</code></pre>



</details>

<a name="0x1_Signer_address_of"></a>

## Function `address_of`

Copies the address of the signer


<pre><code><b>public</b> <b>fun</b> <a href="Signer.md#0x1_Signer_address_of">address_of</a>(s: &signer): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Signer.md#0x1_Signer_address_of">address_of</a>(s: &signer): <b>address</b> {
    *<a href="Signer.md#0x1_Signer_borrow_address">borrow_address</a>(s)
}
</code></pre>



</details>

<a name="@Specification_0"></a>

## Specification



<pre><code><b>pragma</b> verify;
<b>pragma</b> aborts_if_is_strict;
</code></pre>



<a name="@Specification_0_address_of"></a>

### Function `address_of`


<pre><code><b>public</b> <b>fun</b> <a href="Signer.md#0x1_Signer_address_of">address_of</a>(s: &signer): <b>address</b>
</code></pre>




<pre><code><b>pragma</b> opaque = <b>true</b>;
<b>aborts_if</b> <b>false</b>;
<b>ensures</b> result == <a href="Signer.md#0x1_Signer_address_of">address_of</a>(s);
</code></pre>
