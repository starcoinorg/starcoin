
<a name="0x1_Collection2"></a>

# Module `0x1::Collection2`

Provide a account based vector for save resource item.
The resource in CollectionStore can borrowed by anyone, anyone can get immutable ref of item.
and the owner of Collection can allow others to add item to Collection or get mut ref from Collection.git


-  [Struct `Collection`](#0x1_Collection2_Collection)
-  [Resource `CollectionStore`](#0x1_Collection2_CollectionStore)
-  [Constants](#@Constants_0)
-  [Function `length`](#0x1_Collection2_length)
-  [Function `borrow`](#0x1_Collection2_borrow)
-  [Function `push_back`](#0x1_Collection2_push_back)
-  [Function `borrow_mut`](#0x1_Collection2_borrow_mut)
-  [Function `pop_back`](#0x1_Collection2_pop_back)
-  [Function `remove`](#0x1_Collection2_remove)
-  [Function `append`](#0x1_Collection2_append)
-  [Function `append_all`](#0x1_Collection2_append_all)
-  [Function `exists_at`](#0x1_Collection2_exists_at)
-  [Function `is_accept`](#0x1_Collection2_is_accept)
-  [Function `accept`](#0x1_Collection2_accept)
-  [Function `put`](#0x1_Collection2_put)
-  [Function `put_all`](#0x1_Collection2_put_all)
-  [Function `take`](#0x1_Collection2_take)
-  [Function `create_collection`](#0x1_Collection2_create_collection)
-  [Function `length_of`](#0x1_Collection2_length_of)
-  [Function `borrow_collection`](#0x1_Collection2_borrow_collection)
-  [Function `return_collection`](#0x1_Collection2_return_collection)
-  [Function `destroy_collection`](#0x1_Collection2_destroy_collection)
-  [Function `destroy_empty`](#0x1_Collection2_destroy_empty)
-  [Specification](#@Specification_1)
    -  [Function `length`](#@Specification_1_length)
    -  [Function `exists_at`](#@Specification_1_exists_at)
    -  [Function `put`](#@Specification_1_put)
    -  [Function `put_all`](#@Specification_1_put_all)
    -  [Function `take`](#@Specification_1_take)
    -  [Function `borrow_collection`](#@Specification_1_borrow_collection)
    -  [Function `return_collection`](#@Specification_1_return_collection)
    -  [Function `destroy_collection`](#@Specification_1_destroy_collection)
    -  [Function `destroy_empty`](#@Specification_1_destroy_empty)


<pre><code><b>use</b> <a href="Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="Option.md#0x1_Option">0x1::Option</a>;
<b>use</b> <a href="Signer.md#0x1_Signer">0x1::Signer</a>;
<b>use</b> <a href="Vector.md#0x1_Vector">0x1::Vector</a>;
</code></pre>



<a name="0x1_Collection2_Collection"></a>

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
<code>owner: <b>address</b></code>
</dt>
<dd>

</dd>
<dt>
<code>can_put: bool</code>
</dt>
<dd>

</dd>
<dt>
<code>can_mut: bool</code>
</dt>
<dd>

</dd>
<dt>
<code>can_take: bool</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_Collection2_CollectionStore"></a>

## Resource `CollectionStore`

Collection in global store.


<pre><code><b>struct</b> <a href="Collection2.md#0x1_Collection2_CollectionStore">CollectionStore</a>&lt;T: store&gt; <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>items: <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;vector&lt;T&gt;&gt;</code>
</dt>
<dd>
 items in the CollectionStore.
 use Option at here is for temporary take away from store to construct Collection.
</dd>
<dt>
<code>anyone_can_put: bool</code>
</dt>
<dd>

</dd>
<dt>
<code>anyone_can_mut: bool</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_Collection2_ERR_COLLECTION_CAN_NOT_ADD"></a>

The operator require the collection owner or collection set anyone_can_put to true.


<pre><code><b>const</b> <a href="Collection2.md#0x1_Collection2_ERR_COLLECTION_CAN_NOT_ADD">ERR_COLLECTION_CAN_NOT_ADD</a>: u64 = 102;
</code></pre>



<a name="0x1_Collection2_ERR_COLLECTION_CAN_NOT_MUT"></a>

The operator require the collection owner or collection set anyone_can_mut to true.


<pre><code><b>const</b> <a href="Collection2.md#0x1_Collection2_ERR_COLLECTION_CAN_NOT_MUT">ERR_COLLECTION_CAN_NOT_MUT</a>: u64 = 103;
</code></pre>



<a name="0x1_Collection2_ERR_COLLECTION_CAN_NOT_TAKE"></a>

The operator require the collection owner


<pre><code><b>const</b> <a href="Collection2.md#0x1_Collection2_ERR_COLLECTION_CAN_NOT_TAKE">ERR_COLLECTION_CAN_NOT_TAKE</a>: u64 = 104;
</code></pre>



<a name="0x1_Collection2_ERR_COLLECTION_INVALID_BORROW_STATE"></a>



<pre><code><b>const</b> <a href="Collection2.md#0x1_Collection2_ERR_COLLECTION_INVALID_BORROW_STATE">ERR_COLLECTION_INVALID_BORROW_STATE</a>: u64 = 105;
</code></pre>



<a name="0x1_Collection2_ERR_COLLECTION_IS_NOT_EMPTY"></a>



<pre><code><b>const</b> <a href="Collection2.md#0x1_Collection2_ERR_COLLECTION_IS_NOT_EMPTY">ERR_COLLECTION_IS_NOT_EMPTY</a>: u64 = 106;
</code></pre>



<a name="0x1_Collection2_ERR_COLLECTION_NOT_EXIST"></a>



<pre><code><b>const</b> <a href="Collection2.md#0x1_Collection2_ERR_COLLECTION_NOT_EXIST">ERR_COLLECTION_NOT_EXIST</a>: u64 = 101;
</code></pre>



<a name="0x1_Collection2_length"></a>

## Function `length`

Return the length of the collection.


<pre><code><b>public</b> <b>fun</b> <a href="Collection2.md#0x1_Collection2_length">length</a>&lt;T&gt;(c: &<a href="Collection2.md#0x1_Collection2_Collection">Collection2::Collection</a>&lt;T&gt;): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Collection2.md#0x1_Collection2_length">length</a>&lt;T&gt;(c: &<a href="Collection.md#0x1_Collection">Collection</a>&lt;T&gt;): u64{
    <a href="Vector.md#0x1_Vector_length">Vector::length</a>(&c.items)
}
</code></pre>



</details>

<a name="0x1_Collection2_borrow"></a>

## Function `borrow`

Acquire an immutable reference to the <code>i</code>th element of the collection <code>c</code>.
Aborts if <code>i</code> is out of bounds.


<pre><code><b>public</b> <b>fun</b> <a href="Collection2.md#0x1_Collection2_borrow">borrow</a>&lt;T&gt;(c: &<a href="Collection2.md#0x1_Collection2_Collection">Collection2::Collection</a>&lt;T&gt;, i: u64): &T
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Collection2.md#0x1_Collection2_borrow">borrow</a>&lt;T&gt;(c: &<a href="Collection.md#0x1_Collection">Collection</a>&lt;T&gt;, i: u64): &T{
    <a href="Vector.md#0x1_Vector_borrow">Vector::borrow</a>(&c.items, i)
}
</code></pre>



</details>

<a name="0x1_Collection2_push_back"></a>

## Function `push_back`

Add item <code>v</code> to the end of the collection <code>c</code>.
require owner of Collection.


<pre><code><b>public</b> <b>fun</b> <a href="Collection2.md#0x1_Collection2_push_back">push_back</a>&lt;T&gt;(c: &<b>mut</b> <a href="Collection2.md#0x1_Collection2_Collection">Collection2::Collection</a>&lt;T&gt;, t: T)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Collection2.md#0x1_Collection2_push_back">push_back</a>&lt;T&gt;(c: &<b>mut</b> <a href="Collection.md#0x1_Collection">Collection</a>&lt;T&gt;, t: T){
    <b>assert</b>!(c.can_put, <a href="Errors.md#0x1_Errors_requires_address">Errors::requires_address</a>(<a href="Collection2.md#0x1_Collection2_ERR_COLLECTION_CAN_NOT_ADD">ERR_COLLECTION_CAN_NOT_ADD</a>));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>&lt;T&gt;(&<b>mut</b> c.items, t);
}
</code></pre>



</details>

<a name="0x1_Collection2_borrow_mut"></a>

## Function `borrow_mut`

Return a mutable reference to the <code>i</code>th item in the Collection <code>c</code>.
Aborts if <code>i</code> is out of bounds.


<pre><code><b>public</b> <b>fun</b> <a href="Collection2.md#0x1_Collection2_borrow_mut">borrow_mut</a>&lt;T&gt;(c: &<b>mut</b> <a href="Collection2.md#0x1_Collection2_Collection">Collection2::Collection</a>&lt;T&gt;, i: u64): &<b>mut</b> T
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Collection2.md#0x1_Collection2_borrow_mut">borrow_mut</a>&lt;T&gt;(c: &<b>mut</b> <a href="Collection.md#0x1_Collection">Collection</a>&lt;T&gt;, i: u64): &<b>mut</b> T{
    <b>assert</b>!(c.can_mut, <a href="Errors.md#0x1_Errors_requires_address">Errors::requires_address</a>(<a href="Collection2.md#0x1_Collection2_ERR_COLLECTION_CAN_NOT_MUT">ERR_COLLECTION_CAN_NOT_MUT</a>));
    <a href="Vector.md#0x1_Vector_borrow_mut">Vector::borrow_mut</a>&lt;T&gt;(&<b>mut</b> c.items, i)
}
</code></pre>



</details>

<a name="0x1_Collection2_pop_back"></a>

## Function `pop_back`

Pop an element from the end of vector <code>v</code>.
Aborts if <code>v</code> is empty.


<pre><code><b>public</b> <b>fun</b> <a href="Collection2.md#0x1_Collection2_pop_back">pop_back</a>&lt;T&gt;(c: &<b>mut</b> <a href="Collection2.md#0x1_Collection2_Collection">Collection2::Collection</a>&lt;T&gt;): T
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Collection2.md#0x1_Collection2_pop_back">pop_back</a>&lt;T&gt;(c: &<b>mut</b> <a href="Collection.md#0x1_Collection">Collection</a>&lt;T&gt;): T {
    <b>assert</b>!(c.can_take, <a href="Errors.md#0x1_Errors_requires_address">Errors::requires_address</a>(<a href="Collection2.md#0x1_Collection2_ERR_COLLECTION_CAN_NOT_TAKE">ERR_COLLECTION_CAN_NOT_TAKE</a>));
    <a href="Vector.md#0x1_Vector_pop_back">Vector::pop_back</a>&lt;T&gt;(&<b>mut</b> c.items)
}
</code></pre>



</details>

<a name="0x1_Collection2_remove"></a>

## Function `remove`



<pre><code><b>public</b> <b>fun</b> <a href="Collection2.md#0x1_Collection2_remove">remove</a>&lt;T&gt;(c: &<b>mut</b> <a href="Collection2.md#0x1_Collection2_Collection">Collection2::Collection</a>&lt;T&gt;, i: u64): T
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Collection2.md#0x1_Collection2_remove">remove</a>&lt;T&gt;(c: &<b>mut</b> <a href="Collection.md#0x1_Collection">Collection</a>&lt;T&gt;, i: u64): T{
    <b>assert</b>!(c.can_take, <a href="Errors.md#0x1_Errors_requires_address">Errors::requires_address</a>(<a href="Collection2.md#0x1_Collection2_ERR_COLLECTION_CAN_NOT_TAKE">ERR_COLLECTION_CAN_NOT_TAKE</a>));
    <a href="Vector.md#0x1_Vector_remove">Vector::remove</a>&lt;T&gt;(&<b>mut</b> c.items, i)
}
</code></pre>



</details>

<a name="0x1_Collection2_append"></a>

## Function `append`



<pre><code><b>public</b> <b>fun</b> <a href="Collection2.md#0x1_Collection2_append">append</a>&lt;T&gt;(c: &<b>mut</b> <a href="Collection2.md#0x1_Collection2_Collection">Collection2::Collection</a>&lt;T&gt;, other: T)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Collection2.md#0x1_Collection2_append">append</a>&lt;T&gt;(c: &<b>mut</b> <a href="Collection.md#0x1_Collection">Collection</a>&lt;T&gt;, other: T) {
    <b>assert</b>!(c.can_put, <a href="Errors.md#0x1_Errors_requires_address">Errors::requires_address</a>(<a href="Collection2.md#0x1_Collection2_ERR_COLLECTION_CAN_NOT_ADD">ERR_COLLECTION_CAN_NOT_ADD</a>));
    <a href="Vector.md#0x1_Vector_append">Vector::append</a>&lt;T&gt;(&<b>mut</b> c.items, <a href="Vector.md#0x1_Vector_singleton">Vector::singleton</a>(other))
}
</code></pre>



</details>

<a name="0x1_Collection2_append_all"></a>

## Function `append_all`



<pre><code><b>public</b> <b>fun</b> <a href="Collection2.md#0x1_Collection2_append_all">append_all</a>&lt;T&gt;(c: &<b>mut</b> <a href="Collection2.md#0x1_Collection2_Collection">Collection2::Collection</a>&lt;T&gt;, other: vector&lt;T&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Collection2.md#0x1_Collection2_append_all">append_all</a>&lt;T&gt;(c: &<b>mut</b> <a href="Collection.md#0x1_Collection">Collection</a>&lt;T&gt;, other: vector&lt;T&gt;) {
    <b>assert</b>!(c.can_put, <a href="Errors.md#0x1_Errors_requires_address">Errors::requires_address</a>(<a href="Collection2.md#0x1_Collection2_ERR_COLLECTION_CAN_NOT_ADD">ERR_COLLECTION_CAN_NOT_ADD</a>));
    <a href="Vector.md#0x1_Vector_append">Vector::append</a>&lt;T&gt;(&<b>mut</b> c.items, other)
}
</code></pre>



</details>

<a name="0x1_Collection2_exists_at"></a>

## Function `exists_at`

check the Collection exists in <code>addr</code>


<pre><code><b>public</b> <b>fun</b> <a href="Collection2.md#0x1_Collection2_exists_at">exists_at</a>&lt;T: store&gt;(addr: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Collection2.md#0x1_Collection2_exists_at">exists_at</a>&lt;T: store&gt;(addr: <b>address</b>): bool{
    <b>exists</b>&lt;<a href="Collection2.md#0x1_Collection2_CollectionStore">CollectionStore</a>&lt;T&gt;&gt;(addr)
}
</code></pre>



</details>

<a name="0x1_Collection2_is_accept"></a>

## Function `is_accept`

check <code>addr</code> is accept T and other can send T to <code>addr</code>,
it means exists a Collection of T at <code>addr</code> and anyone_can_put of the Collection is true


<pre><code><b>public</b> <b>fun</b> <a href="Collection2.md#0x1_Collection2_is_accept">is_accept</a>&lt;T: store&gt;(addr: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Collection2.md#0x1_Collection2_is_accept">is_accept</a>&lt;T: store&gt;(addr: <b>address</b>): bool <b>acquires</b> <a href="Collection2.md#0x1_Collection2_CollectionStore">CollectionStore</a> {
    <b>if</b> (!<b>exists</b>&lt;<a href="Collection2.md#0x1_Collection2_CollectionStore">CollectionStore</a>&lt;T&gt;&gt;(addr)){
        <b>return</b> <b>false</b>
    };
    <b>let</b> cs = <b>borrow_global</b>&lt;<a href="Collection2.md#0x1_Collection2_CollectionStore">CollectionStore</a>&lt;T&gt;&gt;(addr);
    cs.anyone_can_put
}
</code></pre>



</details>

<a name="0x1_Collection2_accept"></a>

## Function `accept`

signer allow other send T to self
create a Collection of T and set anyone_can_put to true
if the Collection exists, just update anyone_can_put to true


<pre><code><b>public</b> <b>fun</b> <a href="Collection2.md#0x1_Collection2_accept">accept</a>&lt;T: store&gt;(signer: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Collection2.md#0x1_Collection2_accept">accept</a>&lt;T: store&gt;(signer: &signer) <b>acquires</b> <a href="Collection2.md#0x1_Collection2_CollectionStore">CollectionStore</a> {
     <b>let</b> addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer);
    <b>if</b> (!<b>exists</b>&lt;<a href="Collection2.md#0x1_Collection2_CollectionStore">CollectionStore</a>&lt;T&gt;&gt;(addr)){
        <a href="Collection2.md#0x1_Collection2_create_collection">Self::create_collection</a>&lt;T&gt;(signer, <b>true</b>, <b>false</b>);
    };
    <b>let</b> cs = <b>borrow_global_mut</b>&lt;<a href="Collection2.md#0x1_Collection2_CollectionStore">CollectionStore</a>&lt;T&gt;&gt;(addr);
    <b>if</b> (!cs.anyone_can_put) {
        cs.anyone_can_put = <b>true</b>;
    }
}
</code></pre>



</details>

<a name="0x1_Collection2_put"></a>

## Function `put`

Put items to <code>to_addr</code>'s Collection of T
put = borrow_collection<T> + append + return_collection.


<pre><code><b>public</b> <b>fun</b> <a href="Collection2.md#0x1_Collection2_put">put</a>&lt;T: store&gt;(signer: &signer, owner: <b>address</b>, item: T)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Collection2.md#0x1_Collection2_put">put</a>&lt;T: store&gt;(signer: &signer, owner: <b>address</b>, item: T) <b>acquires</b> <a href="Collection2.md#0x1_Collection2_CollectionStore">CollectionStore</a>{
    <b>let</b> c = <a href="Collection2.md#0x1_Collection2_borrow_collection">Self::borrow_collection</a>(signer, owner);
    <a href="Collection2.md#0x1_Collection2_append">Self::append</a>(&<b>mut</b> c, item);
    <a href="Collection2.md#0x1_Collection2_return_collection">Self::return_collection</a>(c);
}
</code></pre>



</details>

<a name="0x1_Collection2_put_all"></a>

## Function `put_all`

Put all items to owner's collection of T.


<pre><code><b>public</b> <b>fun</b> <a href="Collection2.md#0x1_Collection2_put_all">put_all</a>&lt;T: store&gt;(signer: &signer, owner: <b>address</b>, items: vector&lt;T&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Collection2.md#0x1_Collection2_put_all">put_all</a>&lt;T: store&gt;(signer: &signer, owner: <b>address</b>, items: vector&lt;T&gt;) <b>acquires</b> <a href="Collection2.md#0x1_Collection2_CollectionStore">CollectionStore</a>{
    <b>let</b> c = <a href="Collection2.md#0x1_Collection2_borrow_collection">Self::borrow_collection</a>(signer, owner);
    <a href="Collection2.md#0x1_Collection2_append_all">Self::append_all</a>(&<b>mut</b> c, items);
    <a href="Collection2.md#0x1_Collection2_return_collection">Self::return_collection</a>(c);
}
</code></pre>



</details>

<a name="0x1_Collection2_take"></a>

## Function `take`

Take last item from signer's Collection of T.


<pre><code><b>public</b> <b>fun</b> <a href="Collection2.md#0x1_Collection2_take">take</a>&lt;T: store&gt;(signer: &signer): T
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Collection2.md#0x1_Collection2_take">take</a>&lt;T: store&gt;(signer: &signer): T <b>acquires</b> <a href="Collection2.md#0x1_Collection2_CollectionStore">CollectionStore</a>{
    <b>let</b> addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer);
    <b>let</b> c = <a href="Collection2.md#0x1_Collection2_borrow_collection">borrow_collection</a>&lt;T&gt;(signer, addr);
    <b>let</b> item = <a href="Collection2.md#0x1_Collection2_pop_back">pop_back</a>(&<b>mut</b> c);
    <a href="Collection2.md#0x1_Collection2_return_collection">return_collection</a>(c);
    item
}
</code></pre>



</details>

<a name="0x1_Collection2_create_collection"></a>

## Function `create_collection`



<pre><code><b>public</b> <b>fun</b> <a href="Collection2.md#0x1_Collection2_create_collection">create_collection</a>&lt;T: store&gt;(signer: &signer, anyone_can_put: bool, anyone_can_mut: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Collection2.md#0x1_Collection2_create_collection">create_collection</a>&lt;T:store&gt;(signer: &signer, anyone_can_put: bool, anyone_can_mut: bool) {
    <b>move_to</b>(signer, <a href="Collection2.md#0x1_Collection2_CollectionStore">CollectionStore</a>&lt;T&gt;{items: <a href="Option.md#0x1_Option_some">Option::some</a>(<a href="Vector.md#0x1_Vector_empty">Vector::empty</a>&lt;T&gt;()), anyone_can_put, anyone_can_mut})
}
</code></pre>



</details>

<a name="0x1_Collection2_length_of"></a>

## Function `length_of`

Return the length of Collection<T> from <code>owner</code>, if collection do not exist, return 0.


<pre><code><b>public</b> <b>fun</b> <a href="Collection2.md#0x1_Collection2_length_of">length_of</a>&lt;T: store&gt;(owner: <b>address</b>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Collection2.md#0x1_Collection2_length_of">length_of</a>&lt;T: store&gt;(owner: <b>address</b>) : u64 <b>acquires</b> <a href="Collection2.md#0x1_Collection2_CollectionStore">CollectionStore</a>{
    <b>if</b> (<a href="Collection2.md#0x1_Collection2_exists_at">exists_at</a>&lt;T&gt;(owner)){
        <b>let</b> cs = <b>borrow_global_mut</b>&lt;<a href="Collection2.md#0x1_Collection2_CollectionStore">CollectionStore</a>&lt;T&gt;&gt;(owner);
        //<b>if</b> items is None, indicate it is borrowed
        <b>assert</b>!(!<a href="Option.md#0x1_Option_is_none">Option::is_none</a>(&cs.items), <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="Collection2.md#0x1_Collection2_ERR_COLLECTION_INVALID_BORROW_STATE">ERR_COLLECTION_INVALID_BORROW_STATE</a>));
        <b>let</b> items = <a href="Option.md#0x1_Option_borrow">Option::borrow</a>(&cs.items);
        <a href="Vector.md#0x1_Vector_length">Vector::length</a>(items)
    }<b>else</b>{
        0
    }
}
</code></pre>



</details>

<a name="0x1_Collection2_borrow_collection"></a>

## Function `borrow_collection`

Borrow collection of T from <code>owner</code>, auto detected the collection's can_put|can_mut|can_take by the <code>sender</code> and Collection config.


<pre><code><b>public</b> <b>fun</b> <a href="Collection2.md#0x1_Collection2_borrow_collection">borrow_collection</a>&lt;T: store&gt;(sender: &signer, owner: <b>address</b>): <a href="Collection2.md#0x1_Collection2_Collection">Collection2::Collection</a>&lt;T&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Collection2.md#0x1_Collection2_borrow_collection">borrow_collection</a>&lt;T: store&gt;(sender: &signer, owner: <b>address</b>): <a href="Collection.md#0x1_Collection">Collection</a>&lt;T&gt; <b>acquires</b> <a href="Collection2.md#0x1_Collection2_CollectionStore">CollectionStore</a>{
    <b>assert</b>!(<a href="Collection2.md#0x1_Collection2_exists_at">exists_at</a>&lt;T&gt;(owner), <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="Collection2.md#0x1_Collection2_ERR_COLLECTION_NOT_EXIST">ERR_COLLECTION_NOT_EXIST</a>));
    <b>let</b> cs = <b>borrow_global_mut</b>&lt;<a href="Collection2.md#0x1_Collection2_CollectionStore">CollectionStore</a>&lt;T&gt;&gt;(owner);
    //<b>if</b> items is None, indicate it is borrowed
    <b>assert</b>!(!<a href="Option.md#0x1_Option_is_none">Option::is_none</a>(&cs.items), <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="Collection2.md#0x1_Collection2_ERR_COLLECTION_INVALID_BORROW_STATE">ERR_COLLECTION_INVALID_BORROW_STATE</a>));
    <b>let</b> items = <a href="Option.md#0x1_Option_extract">Option::extract</a>(&<b>mut</b> cs.items);
    <b>let</b> is_owner = owner == <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(sender);
    <b>let</b> can_put = cs.anyone_can_put || is_owner;
    <b>let</b> can_mut = cs.anyone_can_mut || is_owner;
    <b>let</b> can_take = is_owner;
    <a href="Collection.md#0x1_Collection">Collection</a>{
        items,
        owner,
        can_put,
        can_mut,
        can_take,
    }
}
</code></pre>



</details>

<a name="0x1_Collection2_return_collection"></a>

## Function `return_collection`

Return the Collection of T


<pre><code><b>public</b> <b>fun</b> <a href="Collection2.md#0x1_Collection2_return_collection">return_collection</a>&lt;T: store&gt;(c: <a href="Collection2.md#0x1_Collection2_Collection">Collection2::Collection</a>&lt;T&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Collection2.md#0x1_Collection2_return_collection">return_collection</a>&lt;T: store&gt;(c: <a href="Collection.md#0x1_Collection">Collection</a>&lt;T&gt;) <b>acquires</b> <a href="Collection2.md#0x1_Collection2_CollectionStore">CollectionStore</a>{
    <b>let</b> <a href="Collection.md#0x1_Collection">Collection</a>{ items, owner, can_put:_, can_mut:_, can_take:_ } = c;
    <b>let</b> cs = <b>borrow_global_mut</b>&lt;<a href="Collection2.md#0x1_Collection2_CollectionStore">CollectionStore</a>&lt;T&gt;&gt;(owner);
    <b>assert</b>!(<a href="Option.md#0x1_Option_is_none">Option::is_none</a>(&cs.items), <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="Collection2.md#0x1_Collection2_ERR_COLLECTION_INVALID_BORROW_STATE">ERR_COLLECTION_INVALID_BORROW_STATE</a>));
    <a href="Option.md#0x1_Option_fill">Option::fill</a>(&<b>mut</b> cs.items, items);
}
</code></pre>



</details>

<a name="0x1_Collection2_destroy_collection"></a>

## Function `destroy_collection`



<pre><code><b>public</b> <b>fun</b> <a href="Collection2.md#0x1_Collection2_destroy_collection">destroy_collection</a>&lt;T: store&gt;(signer: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Collection2.md#0x1_Collection2_destroy_collection">destroy_collection</a>&lt;T: store&gt;(signer: &signer) <b>acquires</b> <a href="Collection2.md#0x1_Collection2_CollectionStore">CollectionStore</a>{
    <b>let</b> c = <b>move_from</b>&lt;<a href="Collection2.md#0x1_Collection2_CollectionStore">CollectionStore</a>&lt;T&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer));
    <a href="Collection2.md#0x1_Collection2_destroy_empty">destroy_empty</a>(c);
}
</code></pre>



</details>

<a name="0x1_Collection2_destroy_empty"></a>

## Function `destroy_empty`



<pre><code><b>fun</b> <a href="Collection2.md#0x1_Collection2_destroy_empty">destroy_empty</a>&lt;T: store&gt;(c: <a href="Collection2.md#0x1_Collection2_CollectionStore">Collection2::CollectionStore</a>&lt;T&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="Collection2.md#0x1_Collection2_destroy_empty">destroy_empty</a>&lt;T: store&gt;(c: <a href="Collection2.md#0x1_Collection2_CollectionStore">CollectionStore</a>&lt;T&gt;){
    <b>let</b> <a href="Collection2.md#0x1_Collection2_CollectionStore">CollectionStore</a>{ items, anyone_can_put:_, anyone_can_mut:_,} = c;
    <b>if</b> (<a href="Option.md#0x1_Option_is_some">Option::is_some</a>(&items)) {
        <b>let</b> item_vec = <a href="Option.md#0x1_Option_extract">Option::extract</a>(&<b>mut</b> items);
        <b>assert</b>!(<a href="Vector.md#0x1_Vector_is_empty">Vector::is_empty</a>(&item_vec), <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="Collection2.md#0x1_Collection2_ERR_COLLECTION_IS_NOT_EMPTY">ERR_COLLECTION_IS_NOT_EMPTY</a>));
        <a href="Vector.md#0x1_Vector_destroy_empty">Vector::destroy_empty</a>(item_vec);
        <a href="Option.md#0x1_Option_destroy_none">Option::destroy_none</a>(items);
    }<b>else</b>{
        <a href="Option.md#0x1_Option_destroy_none">Option::destroy_none</a>(items);
    }
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


<pre><code><b>public</b> <b>fun</b> <a href="Collection2.md#0x1_Collection2_length">length</a>&lt;T&gt;(c: &<a href="Collection2.md#0x1_Collection2_Collection">Collection2::Collection</a>&lt;T&gt;): u64
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="@Specification_1_exists_at"></a>

### Function `exists_at`


<pre><code><b>public</b> <b>fun</b> <a href="Collection2.md#0x1_Collection2_exists_at">exists_at</a>&lt;T: store&gt;(addr: <b>address</b>): bool
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="@Specification_1_put"></a>

### Function `put`


<pre><code><b>public</b> <b>fun</b> <a href="Collection2.md#0x1_Collection2_put">put</a>&lt;T: store&gt;(signer: &signer, owner: <b>address</b>, item: T)
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="@Specification_1_put_all"></a>

### Function `put_all`


<pre><code><b>public</b> <b>fun</b> <a href="Collection2.md#0x1_Collection2_put_all">put_all</a>&lt;T: store&gt;(signer: &signer, owner: <b>address</b>, items: vector&lt;T&gt;)
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="@Specification_1_take"></a>

### Function `take`


<pre><code><b>public</b> <b>fun</b> <a href="Collection2.md#0x1_Collection2_take">take</a>&lt;T: store&gt;(signer: &signer): T
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="@Specification_1_borrow_collection"></a>

### Function `borrow_collection`


<pre><code><b>public</b> <b>fun</b> <a href="Collection2.md#0x1_Collection2_borrow_collection">borrow_collection</a>&lt;T: store&gt;(sender: &signer, owner: <b>address</b>): <a href="Collection2.md#0x1_Collection2_Collection">Collection2::Collection</a>&lt;T&gt;
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="@Specification_1_return_collection"></a>

### Function `return_collection`


<pre><code><b>public</b> <b>fun</b> <a href="Collection2.md#0x1_Collection2_return_collection">return_collection</a>&lt;T: store&gt;(c: <a href="Collection2.md#0x1_Collection2_Collection">Collection2::Collection</a>&lt;T&gt;)
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="@Specification_1_destroy_collection"></a>

### Function `destroy_collection`


<pre><code><b>public</b> <b>fun</b> <a href="Collection2.md#0x1_Collection2_destroy_collection">destroy_collection</a>&lt;T: store&gt;(signer: &signer)
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="@Specification_1_destroy_empty"></a>

### Function `destroy_empty`


<pre><code><b>fun</b> <a href="Collection2.md#0x1_Collection2_destroy_empty">destroy_empty</a>&lt;T: store&gt;(c: <a href="Collection2.md#0x1_Collection2_CollectionStore">Collection2::CollectionStore</a>&lt;T&gt;)
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>
