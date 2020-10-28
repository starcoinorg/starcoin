
<a name="0x1_Box"></a>

# Module `0x1::Box`



-  [Resource `Box`](#0x1_Box_Box)
-  [Constants](#@Constants_0)
-  [Function `exists_at`](#0x1_Box_exists_at)
-  [Function `length`](#0x1_Box_length)
-  [Function `put`](#0x1_Box_put)
-  [Function `put_all`](#0x1_Box_put_all)
-  [Function `take`](#0x1_Box_take)
-  [Function `take_all`](#0x1_Box_take_all)
-  [Function `destroy_empty`](#0x1_Box_destroy_empty)
-  [Specification](#@Specification_1)
    -  [Function `exists_at`](#@Specification_1_exists_at)
    -  [Function `length`](#@Specification_1_length)
    -  [Function `put`](#@Specification_1_put)
    -  [Function `put_all`](#@Specification_1_put_all)
    -  [Function `take`](#@Specification_1_take)
    -  [Function `take_all`](#@Specification_1_take_all)
    -  [Function `destroy_empty`](#@Specification_1_destroy_empty)


<pre><code><b>use</b> <a href="Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="Signer.md#0x1_Signer">0x1::Signer</a>;
<b>use</b> <a href="Vector.md#0x1_Vector">0x1::Vector</a>;
</code></pre>



<a name="0x1_Box_Box"></a>

## Resource `Box`



<pre><code><b>resource</b> <b>struct</b> <a href="Box.md#0x1_Box">Box</a>&lt;T&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>thing: vector&lt;T&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_Box_EBOX_NOT_EXIST"></a>



<pre><code><b>const</b> <a href="Box.md#0x1_Box_EBOX_NOT_EXIST">EBOX_NOT_EXIST</a>: u64 = 101;
</code></pre>



<a name="0x1_Box_exists_at"></a>

## Function `exists_at`



<pre><code><b>public</b> <b>fun</b> <a href="Box.md#0x1_Box_exists_at">exists_at</a>&lt;T&gt;(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Box.md#0x1_Box_exists_at">exists_at</a>&lt;T&gt;(addr: address): bool{
    <b>exists</b>&lt;<a href="Box.md#0x1_Box">Box</a>&lt;T&gt;&gt;(addr)
}
</code></pre>



</details>

<a name="0x1_Box_length"></a>

## Function `length`



<pre><code><b>public</b> <b>fun</b> <a href="Box.md#0x1_Box_length">length</a>&lt;T&gt;(addr: address): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Box.md#0x1_Box_length">length</a>&lt;T&gt;(addr: address): u64 <b>acquires</b> <a href="Box.md#0x1_Box">Box</a>{
    <b>if</b> (<a href="Box.md#0x1_Box_exists_at">exists_at</a>&lt;T&gt;(addr)) {
        <b>let</b> box = borrow_global&lt;<a href="Box.md#0x1_Box">Box</a>&lt;T&gt;&gt;(addr);
        <a href="Vector.md#0x1_Vector_length">Vector::length</a>(&box.thing)
    }<b>else</b>{
       0
    }
}
</code></pre>



</details>

<a name="0x1_Box_put"></a>

## Function `put`



<pre><code><b>public</b> <b>fun</b> <a href="Box.md#0x1_Box_put">put</a>&lt;T&gt;(account: &signer, thing: T)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Box.md#0x1_Box_put">put</a>&lt;T&gt;(account: &signer, thing: T) <b>acquires</b> <a href="Box.md#0x1_Box">Box</a>{
    <b>let</b> addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>if</b> (<a href="Box.md#0x1_Box_exists_at">exists_at</a>&lt;T&gt;(addr)) {
        <b>let</b> box = borrow_global_mut&lt;<a href="Box.md#0x1_Box">Box</a>&lt;T&gt;&gt;(addr);
        <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> box.thing, thing);
    }<b>else</b>{
        move_to(account, <a href="Box.md#0x1_Box">Box</a>&lt;T&gt;{thing: <a href="Vector.md#0x1_Vector_singleton">Vector::singleton</a>(thing)})
    }
}
</code></pre>



</details>

<a name="0x1_Box_put_all"></a>

## Function `put_all`



<pre><code><b>public</b> <b>fun</b> <a href="Box.md#0x1_Box_put_all">put_all</a>&lt;T&gt;(account: &signer, thing: vector&lt;T&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Box.md#0x1_Box_put_all">put_all</a>&lt;T&gt;(account: &signer, thing: vector&lt;T&gt;) <b>acquires</b> <a href="Box.md#0x1_Box">Box</a>{
    <b>let</b> addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>if</b> (<a href="Box.md#0x1_Box_exists_at">exists_at</a>&lt;T&gt;(addr)) {
        <b>let</b> box = borrow_global_mut&lt;<a href="Box.md#0x1_Box">Box</a>&lt;T&gt;&gt;(addr);
        <a href="Vector.md#0x1_Vector_append">Vector::append</a>(&<b>mut</b> box.thing, thing);
    }<b>else</b>{
        move_to(account, <a href="Box.md#0x1_Box">Box</a>&lt;T&gt;{thing})
    }
}
</code></pre>



</details>

<a name="0x1_Box_take"></a>

## Function `take`



<pre><code><b>public</b> <b>fun</b> <a href="Box.md#0x1_Box_take">take</a>&lt;T&gt;(account: &signer): T
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Box.md#0x1_Box_take">take</a>&lt;T&gt;(account: &signer): T <b>acquires</b> <a href="Box.md#0x1_Box">Box</a>{
    <b>let</b> addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>assert</b>(<a href="Box.md#0x1_Box_exists_at">exists_at</a>&lt;T&gt;(addr), <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="Box.md#0x1_Box_EBOX_NOT_EXIST">EBOX_NOT_EXIST</a>));
    <b>let</b> box = borrow_global_mut&lt;<a href="Box.md#0x1_Box">Box</a>&lt;T&gt;&gt;(addr);
    <b>let</b> thing = <a href="Vector.md#0x1_Vector_pop_back">Vector::pop_back</a>(&<b>mut</b> box.thing);
    <b>if</b> (<a href="Vector.md#0x1_Vector_is_empty">Vector::is_empty</a>(&box.thing)){
        <a href="Box.md#0x1_Box_destroy_empty">destroy_empty</a>&lt;T&gt;(addr);
    };
    thing
}
</code></pre>



</details>

<a name="0x1_Box_take_all"></a>

## Function `take_all`



<pre><code><b>public</b> <b>fun</b> <a href="Box.md#0x1_Box_take_all">take_all</a>&lt;T&gt;(account: &signer): vector&lt;T&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Box.md#0x1_Box_take_all">take_all</a>&lt;T&gt;(account: &signer): vector&lt;T&gt; <b>acquires</b> <a href="Box.md#0x1_Box">Box</a>{
    <b>let</b> addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>assert</b>(<a href="Box.md#0x1_Box_exists_at">exists_at</a>&lt;T&gt;(addr), <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="Box.md#0x1_Box_EBOX_NOT_EXIST">EBOX_NOT_EXIST</a>));
    <b>let</b> <a href="Box.md#0x1_Box">Box</a>{ thing } = move_from&lt;<a href="Box.md#0x1_Box">Box</a>&lt;T&gt;&gt;(addr);
    thing
}
</code></pre>



</details>

<a name="0x1_Box_destroy_empty"></a>

## Function `destroy_empty`



<pre><code><b>fun</b> <a href="Box.md#0x1_Box_destroy_empty">destroy_empty</a>&lt;T&gt;(addr: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="Box.md#0x1_Box_destroy_empty">destroy_empty</a>&lt;T&gt;(addr: address) <b>acquires</b> <a href="Box.md#0x1_Box">Box</a>{
    <b>let</b> <a href="Box.md#0x1_Box">Box</a>{ thing } = move_from&lt;<a href="Box.md#0x1_Box">Box</a>&lt;T&gt;&gt;(addr);
    <a href="Vector.md#0x1_Vector_destroy_empty">Vector::destroy_empty</a>(thing);
}
</code></pre>



</details>

<a name="@Specification_1"></a>

## Specification



<pre><code><b>pragma</b> verify = <b>true</b>;
<b>pragma</b> aborts_if_is_strict = <b>true</b>;
</code></pre>



<a name="@Specification_1_exists_at"></a>

### Function `exists_at`


<pre><code><b>public</b> <b>fun</b> <a href="Box.md#0x1_Box_exists_at">exists_at</a>&lt;T&gt;(addr: address): bool
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="@Specification_1_length"></a>

### Function `length`


<pre><code><b>public</b> <b>fun</b> <a href="Box.md#0x1_Box_length">length</a>&lt;T&gt;(addr: address): u64
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="@Specification_1_put"></a>

### Function `put`


<pre><code><b>public</b> <b>fun</b> <a href="Box.md#0x1_Box_put">put</a>&lt;T&gt;(account: &signer, thing: T)
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="@Specification_1_put_all"></a>

### Function `put_all`


<pre><code><b>public</b> <b>fun</b> <a href="Box.md#0x1_Box_put_all">put_all</a>&lt;T&gt;(account: &signer, thing: vector&lt;T&gt;)
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="@Specification_1_take"></a>

### Function `take`


<pre><code><b>public</b> <b>fun</b> <a href="Box.md#0x1_Box_take">take</a>&lt;T&gt;(account: &signer): T
</code></pre>




<pre><code><b>aborts_if</b> !<a href="Box.md#0x1_Box_exists_at">exists_at</a>&lt;T&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Box.md#0x1_Box">Box</a>&lt;T&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
<b>aborts_if</b> len(<b>global</b>&lt;<a href="Box.md#0x1_Box">Box</a>&lt;T&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)).thing) == 0;
</code></pre>



<a name="@Specification_1_take_all"></a>

### Function `take_all`


<pre><code><b>public</b> <b>fun</b> <a href="Box.md#0x1_Box_take_all">take_all</a>&lt;T&gt;(account: &signer): vector&lt;T&gt;
</code></pre>




<pre><code><b>aborts_if</b> !<a href="Box.md#0x1_Box_exists_at">exists_at</a>&lt;T&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
</code></pre>



<a name="@Specification_1_destroy_empty"></a>

### Function `destroy_empty`


<pre><code><b>fun</b> <a href="Box.md#0x1_Box_destroy_empty">destroy_empty</a>&lt;T&gt;(addr: address)
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Box.md#0x1_Box">Box</a>&lt;T&gt;&gt;(addr);
<b>aborts_if</b> len(<b>global</b>&lt;<a href="Box.md#0x1_Box">Box</a>&lt;T&gt;&gt;(addr).thing) &gt; 0;
</code></pre>
