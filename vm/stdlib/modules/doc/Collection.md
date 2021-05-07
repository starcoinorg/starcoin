
<a name="0x1_Collection"></a>

# Module `0x1::Collection`

Deprecated since @v3 please use Collection2
Provide a account based vector for save resource.


-  [Struct `Collection`](#0x1_Collection_Collection)
-  [Resource `CollectionStore`](#0x1_Collection_CollectionStore)
-  [Constants](#@Constants_0)
-  [Function `length`](#0x1_Collection_length)
-  [Function `borrow`](#0x1_Collection_borrow)
-  [Function `push_back`](#0x1_Collection_push_back)
-  [Function `borrow_mut`](#0x1_Collection_borrow_mut)
-  [Function `pop_back`](#0x1_Collection_pop_back)
-  [Function `remove`](#0x1_Collection_remove)
-  [Function `append`](#0x1_Collection_append)
-  [Function `append_all`](#0x1_Collection_append_all)
-  [Function `has`](#0x1_Collection_has)
-  [Function `exists_at`](#0x1_Collection_exists_at)
-  [Function `put`](#0x1_Collection_put)
-  [Function `put_all`](#0x1_Collection_put_all)
-  [Function `take`](#0x1_Collection_take)
-  [Function `borrow_collection`](#0x1_Collection_borrow_collection)
-  [Function `return_collection`](#0x1_Collection_return_collection)
-  [Function `destroy_empty`](#0x1_Collection_destroy_empty)
-  [Specification](#@Specification_1)
    -  [Function `length`](#@Specification_1_length)
    -  [Function `exists_at`](#@Specification_1_exists_at)
    -  [Function `put`](#@Specification_1_put)
    -  [Function `put_all`](#@Specification_1_put_all)
    -  [Function `take`](#@Specification_1_take)
    -  [Function `borrow_collection`](#@Specification_1_borrow_collection)
    -  [Function `return_collection`](#@Specification_1_return_collection)
    -  [Function `destroy_empty`](#@Specification_1_destroy_empty)


<pre><code><b>use</b> <a href="Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="Option.md#0x1_Option">0x1::Option</a>;
<b>use</b> <a href="Signer.md#0x1_Signer">0x1::Signer</a>;
<b>use</b> <a href="Vector.md#0x1_Vector">0x1::Vector</a>;
</code></pre>



<a name="0x1_Collection_Collection"></a>

## Struct `Collection`

Collection in memory, can not drop & store.


<pre><code><b>struct</b> <a href="Collection.md#0x1_Collection">Collection</a>&lt;T&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>items: vector&lt;T&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>owner: address</code>
</dt>
<dd>
 the owner of Collection.
</dd>
</dl>


</details>

<a name="0x1_Collection_CollectionStore"></a>

## Resource `CollectionStore`

Collection in global store.


<pre><code><b>resource</b> <b>struct</b> <a href="Collection.md#0x1_Collection_CollectionStore">CollectionStore</a>&lt;T&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>items: <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;vector&lt;T&gt;&gt;</code>
</dt>
<dd>
 items in the CollectionStore.
 use Option at  here is for temporary take away from store to construct Collection.
</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_Collection_EDEPRECATED_FUNCTION"></a>



<pre><code><b>const</b> <a href="Collection.md#0x1_Collection_EDEPRECATED_FUNCTION">EDEPRECATED_FUNCTION</a>: u64 = 11;
</code></pre>



<a name="0x1_Collection_ECOLLECTION_NOT_EXIST"></a>



<pre><code><b>const</b> <a href="Collection.md#0x1_Collection_ECOLLECTION_NOT_EXIST">ECOLLECTION_NOT_EXIST</a>: u64 = 101;
</code></pre>



<a name="0x1_Collection_ECOLLECTION_NOT_OWNER"></a>

The operator require the collection owner.


<pre><code><b>const</b> <a href="Collection.md#0x1_Collection_ECOLLECTION_NOT_OWNER">ECOLLECTION_NOT_OWNER</a>: u64 = 102;
</code></pre>



<a name="0x1_Collection_length"></a>

## Function `length`

Return the length of the collection.


<pre><code><b>public</b> <b>fun</b> <a href="Collection.md#0x1_Collection_length">length</a>&lt;T&gt;(c: &<a href="Collection.md#0x1_Collection_Collection">Collection::Collection</a>&lt;T&gt;): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Collection.md#0x1_Collection_length">length</a>&lt;T&gt;(c: &<a href="Collection.md#0x1_Collection">Collection</a>&lt;T&gt;): u64{
    <a href="Vector.md#0x1_Vector_length">Vector::length</a>(&c.items)
}
</code></pre>



</details>

<a name="0x1_Collection_borrow"></a>

## Function `borrow`

Acquire an immutable reference to the <code>i</code>th element of the collection <code>c</code>.
Aborts if <code>i</code> is out of bounds.


<pre><code><b>public</b> <b>fun</b> <a href="Collection.md#0x1_Collection_borrow">borrow</a>&lt;T&gt;(c: &<a href="Collection.md#0x1_Collection_Collection">Collection::Collection</a>&lt;T&gt;, i: u64): &T
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Collection.md#0x1_Collection_borrow">borrow</a>&lt;T&gt;(c: &<a href="Collection.md#0x1_Collection">Collection</a>&lt;T&gt;, i: u64): &T{
    <a href="Vector.md#0x1_Vector_borrow">Vector::borrow</a>(&c.items, i)
}
</code></pre>



</details>

<a name="0x1_Collection_push_back"></a>

## Function `push_back`

Deprecated since @v3
Add item <code>v</code> to the end of the collection <code>c</code>.
require owner of Collection.


<pre><code><b>public</b> <b>fun</b> <a href="Collection.md#0x1_Collection_push_back">push_back</a>&lt;T&gt;(_account: &signer, _c: &<b>mut</b> <a href="Collection.md#0x1_Collection_Collection">Collection::Collection</a>&lt;T&gt;, _t: T)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Collection.md#0x1_Collection_push_back">push_back</a>&lt;T&gt;(_account: &signer, _c: &<b>mut</b> <a href="Collection.md#0x1_Collection">Collection</a>&lt;T&gt;, _t: T){
    <b>abort</b> <a href="Errors.md#0x1_Errors_deprecated">Errors::deprecated</a>(<a href="Collection.md#0x1_Collection_EDEPRECATED_FUNCTION">EDEPRECATED_FUNCTION</a>)
}
</code></pre>



</details>

<a name="0x1_Collection_borrow_mut"></a>

## Function `borrow_mut`

Return a mutable reference to the <code>i</code>th item in the Collection <code>c</code>.
Aborts if <code>i</code> is out of bounds.


<pre><code><b>public</b> <b>fun</b> <a href="Collection.md#0x1_Collection_borrow_mut">borrow_mut</a>&lt;T&gt;(account: &signer, c: &<b>mut</b> <a href="Collection.md#0x1_Collection_Collection">Collection::Collection</a>&lt;T&gt;, i: u64): &<b>mut</b> T
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Collection.md#0x1_Collection_borrow_mut">borrow_mut</a>&lt;T&gt;(account: &signer, c: &<b>mut</b> <a href="Collection.md#0x1_Collection">Collection</a>&lt;T&gt;, i: u64): &<b>mut</b> T{
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == c.owner, <a href="Errors.md#0x1_Errors_requires_address">Errors::requires_address</a>(<a href="Collection.md#0x1_Collection_ECOLLECTION_NOT_OWNER">ECOLLECTION_NOT_OWNER</a>));
    <a href="Vector.md#0x1_Vector_borrow_mut">Vector::borrow_mut</a>&lt;T&gt;(&<b>mut</b> c.items, i)
}
</code></pre>



</details>

<a name="0x1_Collection_pop_back"></a>

## Function `pop_back`

Pop an element from the end of vector <code>v</code>.
Aborts if <code>v</code> is empty.


<pre><code><b>public</b> <b>fun</b> <a href="Collection.md#0x1_Collection_pop_back">pop_back</a>&lt;T&gt;(account: &signer, c: &<b>mut</b> <a href="Collection.md#0x1_Collection_Collection">Collection::Collection</a>&lt;T&gt;): T
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Collection.md#0x1_Collection_pop_back">pop_back</a>&lt;T&gt;(account: &signer, c: &<b>mut</b> <a href="Collection.md#0x1_Collection">Collection</a>&lt;T&gt;): T {
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == c.owner, <a href="Errors.md#0x1_Errors_requires_address">Errors::requires_address</a>(<a href="Collection.md#0x1_Collection_ECOLLECTION_NOT_OWNER">ECOLLECTION_NOT_OWNER</a>));
    <a href="Vector.md#0x1_Vector_pop_back">Vector::pop_back</a>&lt;T&gt;(&<b>mut</b> c.items)
}
</code></pre>



</details>

<a name="0x1_Collection_remove"></a>

## Function `remove`



<pre><code><b>public</b> <b>fun</b> <a href="Collection.md#0x1_Collection_remove">remove</a>&lt;T&gt;(account: &signer, c: &<b>mut</b> <a href="Collection.md#0x1_Collection_Collection">Collection::Collection</a>&lt;T&gt;, i: u64): T
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Collection.md#0x1_Collection_remove">remove</a>&lt;T&gt;(account: &signer, c: &<b>mut</b> <a href="Collection.md#0x1_Collection">Collection</a>&lt;T&gt;, i: u64): T{
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == c.owner, <a href="Errors.md#0x1_Errors_requires_address">Errors::requires_address</a>(<a href="Collection.md#0x1_Collection_ECOLLECTION_NOT_OWNER">ECOLLECTION_NOT_OWNER</a>));
    <a href="Vector.md#0x1_Vector_remove">Vector::remove</a>&lt;T&gt;(&<b>mut</b> c.items, i)
}
</code></pre>



</details>

<a name="0x1_Collection_append"></a>

## Function `append`

Deprecated since @v3


<pre><code><b>public</b> <b>fun</b> <a href="Collection.md#0x1_Collection_append">append</a>&lt;T&gt;(_account: &signer, _c: &<b>mut</b> <a href="Collection.md#0x1_Collection_Collection">Collection::Collection</a>&lt;T&gt;, _other: T)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Collection.md#0x1_Collection_append">append</a>&lt;T&gt;(_account: &signer, _c: &<b>mut</b> <a href="Collection.md#0x1_Collection">Collection</a>&lt;T&gt;, _other: T) {
    <b>abort</b> <a href="Errors.md#0x1_Errors_deprecated">Errors::deprecated</a>(<a href="Collection.md#0x1_Collection_EDEPRECATED_FUNCTION">EDEPRECATED_FUNCTION</a>)
}
</code></pre>



</details>

<a name="0x1_Collection_append_all"></a>

## Function `append_all`

Deprecated since @v3


<pre><code><b>public</b> <b>fun</b> <a href="Collection.md#0x1_Collection_append_all">append_all</a>&lt;T&gt;(_account: &signer, _c: &<b>mut</b> <a href="Collection.md#0x1_Collection_Collection">Collection::Collection</a>&lt;T&gt;, _other: vector&lt;T&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Collection.md#0x1_Collection_append_all">append_all</a>&lt;T&gt;(_account: &signer, _c: &<b>mut</b> <a href="Collection.md#0x1_Collection">Collection</a>&lt;T&gt;, _other: vector&lt;T&gt;) {
    <b>abort</b> <a href="Errors.md#0x1_Errors_deprecated">Errors::deprecated</a>(<a href="Collection.md#0x1_Collection_EDEPRECATED_FUNCTION">EDEPRECATED_FUNCTION</a>)
}
</code></pre>



</details>

<a name="0x1_Collection_has"></a>

## Function `has`

Check if the addr has T


<pre><code><b>public</b> <b>fun</b> <a href="Collection.md#0x1_Collection_has">has</a>&lt;T&gt;(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Collection.md#0x1_Collection_has">has</a>&lt;T: store&gt;(addr: address): bool {
    // just <b>return</b> <b>exists</b> <a href="Collection.md#0x1_Collection_CollectionStore">CollectionStore</a>&lt;T&gt;, because we will ensure at least one item in <a href="Collection.md#0x1_Collection_CollectionStore">CollectionStore</a>.
    <a href="Collection.md#0x1_Collection_exists_at">exists_at</a>&lt;T&gt;(addr)
}
</code></pre>



</details>

<a name="0x1_Collection_exists_at"></a>

## Function `exists_at`

check the Collection exists in <code>addr</code>


<pre><code><b>fun</b> <a href="Collection.md#0x1_Collection_exists_at">exists_at</a>&lt;T&gt;(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="Collection.md#0x1_Collection_exists_at">exists_at</a>&lt;T: store&gt;(addr: address): bool{
    <b>exists</b>&lt;<a href="Collection.md#0x1_Collection_CollectionStore">CollectionStore</a>&lt;T&gt;&gt;(addr)
}
</code></pre>



</details>

<a name="0x1_Collection_put"></a>

## Function `put`

Deprecated since @v3
Put items to account's Collection last position.


<pre><code><b>public</b> <b>fun</b> <a href="Collection.md#0x1_Collection_put">put</a>&lt;T&gt;(_account: &signer, _item: T)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Collection.md#0x1_Collection_put">put</a>&lt;T: store&gt;(_account: &signer, _item: T) {
    <b>abort</b> <a href="Errors.md#0x1_Errors_deprecated">Errors::deprecated</a>(<a href="Collection.md#0x1_Collection_EDEPRECATED_FUNCTION">EDEPRECATED_FUNCTION</a>)
}
</code></pre>



</details>

<a name="0x1_Collection_put_all"></a>

## Function `put_all`

Deprecated since @v3
Put itemss to account's box last position.


<pre><code><b>public</b> <b>fun</b> <a href="Collection.md#0x1_Collection_put_all">put_all</a>&lt;T&gt;(_account: &signer, _items: vector&lt;T&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Collection.md#0x1_Collection_put_all">put_all</a>&lt;T: store&gt;(_account: &signer, _items: vector&lt;T&gt;) {
    <b>abort</b> <a href="Errors.md#0x1_Errors_deprecated">Errors::deprecated</a>(<a href="Collection.md#0x1_Collection_EDEPRECATED_FUNCTION">EDEPRECATED_FUNCTION</a>)
}
</code></pre>



</details>

<a name="0x1_Collection_take"></a>

## Function `take`

Take last item from account's Collection of T.


<pre><code><b>public</b> <b>fun</b> <a href="Collection.md#0x1_Collection_take">take</a>&lt;T&gt;(account: &signer): T
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Collection.md#0x1_Collection_take">take</a>&lt;T: store&gt;(account: &signer): T <b>acquires</b> <a href="Collection.md#0x1_Collection_CollectionStore">CollectionStore</a>{
    <b>let</b> addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>let</b> c = <a href="Collection.md#0x1_Collection_borrow_collection">borrow_collection</a>&lt;T&gt;(addr);
    <b>let</b> item = <a href="Collection.md#0x1_Collection_pop_back">pop_back</a>(account, &<b>mut</b> c);
    <a href="Collection.md#0x1_Collection_return_collection">return_collection</a>(c);
    item
}
</code></pre>



</details>

<a name="0x1_Collection_borrow_collection"></a>

## Function `borrow_collection`

Borrow collection of T from <code>addr</code>


<pre><code><b>public</b> <b>fun</b> <a href="Collection.md#0x1_Collection_borrow_collection">borrow_collection</a>&lt;T&gt;(addr: address): <a href="Collection.md#0x1_Collection_Collection">Collection::Collection</a>&lt;T&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Collection.md#0x1_Collection_borrow_collection">borrow_collection</a>&lt;T: store&gt;(addr: address): <a href="Collection.md#0x1_Collection">Collection</a>&lt;T&gt; <b>acquires</b> <a href="Collection.md#0x1_Collection_CollectionStore">CollectionStore</a>{
    <b>assert</b>(<a href="Collection.md#0x1_Collection_exists_at">exists_at</a>&lt;T&gt;(addr), <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="Collection.md#0x1_Collection_ECOLLECTION_NOT_EXIST">ECOLLECTION_NOT_EXIST</a>));
    <b>let</b> c = borrow_global_mut&lt;<a href="Collection.md#0x1_Collection_CollectionStore">CollectionStore</a>&lt;T&gt;&gt;(addr);
    <b>let</b> items = <a href="Option.md#0x1_Option_extract">Option::extract</a>(&<b>mut</b> c.items);
    <a href="Collection.md#0x1_Collection">Collection</a>{
        items,
        owner: addr
    }
}
</code></pre>



</details>

<a name="0x1_Collection_return_collection"></a>

## Function `return_collection`

Return the Collection of T


<pre><code><b>public</b> <b>fun</b> <a href="Collection.md#0x1_Collection_return_collection">return_collection</a>&lt;T&gt;(c: <a href="Collection.md#0x1_Collection_Collection">Collection::Collection</a>&lt;T&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Collection.md#0x1_Collection_return_collection">return_collection</a>&lt;T: store&gt;(c: <a href="Collection.md#0x1_Collection">Collection</a>&lt;T&gt;) <b>acquires</b> <a href="Collection.md#0x1_Collection_CollectionStore">CollectionStore</a>{
    <b>let</b> <a href="Collection.md#0x1_Collection">Collection</a>{ items, owner } = c;
    <b>if</b> (<a href="Vector.md#0x1_Vector_is_empty">Vector::is_empty</a>(&items)) {
        <b>let</b> c = move_from&lt;<a href="Collection.md#0x1_Collection_CollectionStore">CollectionStore</a>&lt;T&gt;&gt;(owner);
        <a href="Collection.md#0x1_Collection_destroy_empty">destroy_empty</a>(c);
        <a href="Vector.md#0x1_Vector_destroy_empty">Vector::destroy_empty</a>(items);
    }<b>else</b>{
        <b>let</b> c = borrow_global_mut&lt;<a href="Collection.md#0x1_Collection_CollectionStore">CollectionStore</a>&lt;T&gt;&gt;(owner);
        <a href="Option.md#0x1_Option_fill">Option::fill</a>(&<b>mut</b> c.items, items);
    }
}
</code></pre>



</details>

<a name="0x1_Collection_destroy_empty"></a>

## Function `destroy_empty`



<pre><code><b>fun</b> <a href="Collection.md#0x1_Collection_destroy_empty">destroy_empty</a>&lt;T&gt;(c: <a href="Collection.md#0x1_Collection_CollectionStore">Collection::CollectionStore</a>&lt;T&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="Collection.md#0x1_Collection_destroy_empty">destroy_empty</a>&lt;T: store&gt;(c: <a href="Collection.md#0x1_Collection_CollectionStore">CollectionStore</a>&lt;T&gt;){
    <b>let</b> <a href="Collection.md#0x1_Collection_CollectionStore">CollectionStore</a>{ items } = c;
    <a href="Option.md#0x1_Option_destroy_none">Option::destroy_none</a>(items);
}
</code></pre>



</details>

<a name="@Specification_1"></a>

## Specification



<pre><code><b>pragma</b> verify = <b>false</b>;
<b>pragma</b> aborts_if_is_strict = <b>false</b>;
</code></pre>



<a name="@Specification_1_length"></a>

### Function `length`


<pre><code><b>public</b> <b>fun</b> <a href="Collection.md#0x1_Collection_length">length</a>&lt;T&gt;(c: &<a href="Collection.md#0x1_Collection_Collection">Collection::Collection</a>&lt;T&gt;): u64
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="@Specification_1_exists_at"></a>

### Function `exists_at`


<pre><code><b>fun</b> <a href="Collection.md#0x1_Collection_exists_at">exists_at</a>&lt;T&gt;(addr: address): bool
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="@Specification_1_put"></a>

### Function `put`


<pre><code><b>public</b> <b>fun</b> <a href="Collection.md#0x1_Collection_put">put</a>&lt;T&gt;(_account: &signer, _item: T)
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="@Specification_1_put_all"></a>

### Function `put_all`


<pre><code><b>public</b> <b>fun</b> <a href="Collection.md#0x1_Collection_put_all">put_all</a>&lt;T&gt;(_account: &signer, _items: vector&lt;T&gt;)
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="@Specification_1_take"></a>

### Function `take`


<pre><code><b>public</b> <b>fun</b> <a href="Collection.md#0x1_Collection_take">take</a>&lt;T&gt;(account: &signer): T
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="@Specification_1_borrow_collection"></a>

### Function `borrow_collection`


<pre><code><b>public</b> <b>fun</b> <a href="Collection.md#0x1_Collection_borrow_collection">borrow_collection</a>&lt;T&gt;(addr: address): <a href="Collection.md#0x1_Collection_Collection">Collection::Collection</a>&lt;T&gt;
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="@Specification_1_return_collection"></a>

### Function `return_collection`


<pre><code><b>public</b> <b>fun</b> <a href="Collection.md#0x1_Collection_return_collection">return_collection</a>&lt;T&gt;(c: <a href="Collection.md#0x1_Collection_Collection">Collection::Collection</a>&lt;T&gt;)
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="@Specification_1_destroy_empty"></a>

### Function `destroy_empty`


<pre><code><b>fun</b> <a href="Collection.md#0x1_Collection_destroy_empty">destroy_empty</a>&lt;T&gt;(c: <a href="Collection.md#0x1_Collection_CollectionStore">Collection::CollectionStore</a>&lt;T&gt;)
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>
