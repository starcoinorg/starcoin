
<a name="0x1_SortedLinkedList"></a>

# Module `0x1::SortedLinkedList`

### Table of Contents

-  [Struct `EntryHandle`](#0x1_SortedLinkedList_EntryHandle)
-  [Resource `Node`](#0x1_SortedLinkedList_Node)
-  [Resource `NodeVector`](#0x1_SortedLinkedList_NodeVector)
-  [Function `entry_handle`](#0x1_SortedLinkedList_entry_handle)
-  [Function `get_addr`](#0x1_SortedLinkedList_get_addr)
-  [Function `get_index`](#0x1_SortedLinkedList_get_index)
-  [Function `node_exists`](#0x1_SortedLinkedList_node_exists)
-  [Function `get_data`](#0x1_SortedLinkedList_get_data)
-  [Function `get_prev_node_addr`](#0x1_SortedLinkedList_get_prev_node_addr)
-  [Function `is_head_node`](#0x1_SortedLinkedList_is_head_node)
-  [Function `create_new_list`](#0x1_SortedLinkedList_create_new_list)
-  [Function `insert_node`](#0x1_SortedLinkedList_insert_node)
-  [Function `remove_node`](#0x1_SortedLinkedList_remove_node)
-  [Function `remove_node_by_list_owner`](#0x1_SortedLinkedList_remove_node_by_list_owner)
-  [Function `remove_node_by_node_owner`](#0x1_SortedLinkedList_remove_node_by_node_owner)
-  [Function `remove_list`](#0x1_SortedLinkedList_remove_list)
-  [Function `find_position_and_insert`](#0x1_SortedLinkedList_find_position_and_insert)



<a name="0x1_SortedLinkedList_EntryHandle"></a>

## Struct `EntryHandle`



<pre><code><b>struct</b> <a href="#0x1_SortedLinkedList_EntryHandle">EntryHandle</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>addr: address</code>
</dt>
<dd>

</dd>
<dt>

<code>index: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_SortedLinkedList_Node"></a>

## Resource `Node`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_SortedLinkedList_Node">Node</a>&lt;T&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>prev: <a href="#0x1_SortedLinkedList_EntryHandle">SortedLinkedList::EntryHandle</a></code>
</dt>
<dd>

</dd>
<dt>

<code>next: <a href="#0x1_SortedLinkedList_EntryHandle">SortedLinkedList::EntryHandle</a></code>
</dt>
<dd>

</dd>
<dt>

<code>head: <a href="#0x1_SortedLinkedList_EntryHandle">SortedLinkedList::EntryHandle</a></code>
</dt>
<dd>

</dd>
<dt>

<code>data: T</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_SortedLinkedList_NodeVector"></a>

## Resource `NodeVector`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_SortedLinkedList_NodeVector">NodeVector</a>&lt;T&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>nodes: vector&lt;<a href="#0x1_SortedLinkedList_Node">SortedLinkedList::Node</a>&lt;T&gt;&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_SortedLinkedList_entry_handle"></a>

## Function `entry_handle`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_SortedLinkedList_entry_handle">entry_handle</a>(addr: address, index: u64): <a href="#0x1_SortedLinkedList_EntryHandle">SortedLinkedList::EntryHandle</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_SortedLinkedList_entry_handle">entry_handle</a>(addr: address, index: u64): <a href="#0x1_SortedLinkedList_EntryHandle">EntryHandle</a> {
    <a href="#0x1_SortedLinkedList_EntryHandle">EntryHandle</a> { addr, index }
}
</code></pre>



</details>

<a name="0x1_SortedLinkedList_get_addr"></a>

## Function `get_addr`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_SortedLinkedList_get_addr">get_addr</a>(entry: <a href="#0x1_SortedLinkedList_EntryHandle">SortedLinkedList::EntryHandle</a>): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_SortedLinkedList_get_addr">get_addr</a>(entry: <a href="#0x1_SortedLinkedList_EntryHandle">EntryHandle</a>): address {
    entry.addr
}
</code></pre>



</details>

<a name="0x1_SortedLinkedList_get_index"></a>

## Function `get_index`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_SortedLinkedList_get_index">get_index</a>(entry: <a href="#0x1_SortedLinkedList_EntryHandle">SortedLinkedList::EntryHandle</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_SortedLinkedList_get_index">get_index</a>(entry: <a href="#0x1_SortedLinkedList_EntryHandle">EntryHandle</a>): u64 {
    entry.index
}
</code></pre>



</details>

<a name="0x1_SortedLinkedList_node_exists"></a>

## Function `node_exists`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_SortedLinkedList_node_exists">node_exists</a>&lt;T: <b>copyable</b>&gt;(entry: <a href="#0x1_SortedLinkedList_EntryHandle">SortedLinkedList::EntryHandle</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_SortedLinkedList_node_exists">node_exists</a>&lt;T: <b>copyable</b>&gt;(entry: <a href="#0x1_SortedLinkedList_EntryHandle">EntryHandle</a>): bool <b>acquires</b> <a href="#0x1_SortedLinkedList_NodeVector">NodeVector</a> {
    <b>if</b> (!exists&lt;<a href="#0x1_SortedLinkedList_NodeVector">NodeVector</a>&lt;T&gt;&gt;(entry.addr)) <b>return</b> <b>false</b>;
    <b>let</b> node_vector = &borrow_global&lt;<a href="#0x1_SortedLinkedList_NodeVector">NodeVector</a>&lt;T&gt;&gt;(entry.addr).nodes;
    <b>if</b> (entry.index &gt;= <a href="Vector.md#0x1_Vector_length">Vector::length</a>&lt;<a href="#0x1_SortedLinkedList_Node">Node</a>&lt;T&gt;&gt;(node_vector)) <b>return</b> <b>false</b>;
    <b>true</b>
}
</code></pre>



</details>

<a name="0x1_SortedLinkedList_get_data"></a>

## Function `get_data`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_SortedLinkedList_get_data">get_data</a>&lt;T: <b>copyable</b>&gt;(entry: <a href="#0x1_SortedLinkedList_EntryHandle">SortedLinkedList::EntryHandle</a>): T
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_SortedLinkedList_get_data">get_data</a>&lt;T: <b>copyable</b>&gt;(entry: <a href="#0x1_SortedLinkedList_EntryHandle">EntryHandle</a>): T <b>acquires</b> <a href="#0x1_SortedLinkedList_NodeVector">NodeVector</a> {
    //make sure a node exists in entry
    <b>assert</b>(<a href="#0x1_SortedLinkedList_node_exists">node_exists</a>&lt;T&gt;(<b>copy</b> entry), 1);
    <b>let</b> nodes = &borrow_global&lt;<a href="#0x1_SortedLinkedList_NodeVector">NodeVector</a>&lt;T&gt;&gt;(entry.addr).nodes;
    <b>let</b> node = <a href="Vector.md#0x1_Vector_borrow">Vector::borrow</a>&lt;<a href="#0x1_SortedLinkedList_Node">Node</a>&lt;T&gt;&gt;(nodes, entry.index);
    *&node.data
}
</code></pre>



</details>

<a name="0x1_SortedLinkedList_get_prev_node_addr"></a>

## Function `get_prev_node_addr`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_SortedLinkedList_get_prev_node_addr">get_prev_node_addr</a>&lt;T: <b>copyable</b>&gt;(entry: <a href="#0x1_SortedLinkedList_EntryHandle">SortedLinkedList::EntryHandle</a>): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_SortedLinkedList_get_prev_node_addr">get_prev_node_addr</a>&lt;T: <b>copyable</b>&gt;(entry: <a href="#0x1_SortedLinkedList_EntryHandle">EntryHandle</a>): address <b>acquires</b> <a href="#0x1_SortedLinkedList_NodeVector">NodeVector</a> {
    //make sure a node exists in entry
    <b>assert</b>(<a href="#0x1_SortedLinkedList_node_exists">node_exists</a>&lt;T&gt;(<b>copy</b> entry), 2);
    <b>let</b> nodes = &borrow_global&lt;<a href="#0x1_SortedLinkedList_NodeVector">NodeVector</a>&lt;T&gt;&gt;(entry.addr).nodes;
    <b>let</b> node = <a href="Vector.md#0x1_Vector_borrow">Vector::borrow</a>&lt;<a href="#0x1_SortedLinkedList_Node">Node</a>&lt;T&gt;&gt;(nodes, entry.index);
    *&node.prev.addr
}
</code></pre>



</details>

<a name="0x1_SortedLinkedList_is_head_node"></a>

## Function `is_head_node`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_SortedLinkedList_is_head_node">is_head_node</a>&lt;T: <b>copyable</b>&gt;(entry: &<a href="#0x1_SortedLinkedList_EntryHandle">SortedLinkedList::EntryHandle</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_SortedLinkedList_is_head_node">is_head_node</a>&lt;T: <b>copyable</b>&gt;(entry: &<a href="#0x1_SortedLinkedList_EntryHandle">EntryHandle</a>): bool <b>acquires</b> <a href="#0x1_SortedLinkedList_NodeVector">NodeVector</a> {
		//check that a node exists
    <b>assert</b>(<a href="#0x1_SortedLinkedList_node_exists">node_exists</a>&lt;T&gt;(*entry), 3);
    <b>let</b> nodes = &borrow_global&lt;<a href="#0x1_SortedLinkedList_NodeVector">NodeVector</a>&lt;T&gt;&gt;(entry.addr).nodes;
    //find the head node
    <b>let</b> node = <a href="Vector.md#0x1_Vector_borrow">Vector::borrow</a>&lt;<a href="#0x1_SortedLinkedList_Node">Node</a>&lt;T&gt;&gt;(nodes, entry.index);

    //check <b>if</b> this is the head node
    node.head.addr == entry.addr && node.head.index == entry.index
}
</code></pre>



</details>

<a name="0x1_SortedLinkedList_create_new_list"></a>

## Function `create_new_list`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_SortedLinkedList_create_new_list">create_new_list</a>&lt;T: <b>copyable</b>&gt;(account: &signer, data: T)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_SortedLinkedList_create_new_list">create_new_list</a>&lt;T: <b>copyable</b>&gt;(account: &signer, data: T) {
    <b>let</b> sender = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);

    //make sure no node/list is already stored in this account
    <b>assert</b>(!exists&lt;<a href="#0x1_SortedLinkedList_NodeVector">NodeVector</a>&lt;T&gt;&gt;(sender), 3);
    <b>let</b> head_handle = <a href="#0x1_SortedLinkedList_entry_handle">entry_handle</a>(sender, 0);
    <b>let</b> head = <a href="#0x1_SortedLinkedList_Node">Self::Node</a>&lt;T&gt; {
        prev: <b>copy</b> head_handle,
        next: <b>copy</b> head_handle,
        head: head_handle,
        data: data
    };

    <b>let</b> node_vector = <a href="Vector.md#0x1_Vector_singleton">Vector::singleton</a>(head);
    move_to&lt;<a href="#0x1_SortedLinkedList_NodeVector">NodeVector</a>&lt;T&gt;&gt;(account, <a href="#0x1_SortedLinkedList_NodeVector">NodeVector</a>&lt;T&gt; { nodes: node_vector });
}
</code></pre>



</details>

<a name="0x1_SortedLinkedList_insert_node"></a>

## Function `insert_node`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_SortedLinkedList_insert_node">insert_node</a>&lt;T: <b>copyable</b>&gt;(account: &signer, data: T, prev_entry: <a href="#0x1_SortedLinkedList_EntryHandle">SortedLinkedList::EntryHandle</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_SortedLinkedList_insert_node">insert_node</a>&lt;T: <b>copyable</b>&gt;(account: &signer, data: T, prev_entry: <a href="#0x1_SortedLinkedList_EntryHandle">EntryHandle</a>) <b>acquires</b> <a href="#0x1_SortedLinkedList_NodeVector">NodeVector</a> {
    <b>let</b> sender_address = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);

    //make sure a node exists in prev_entry
    <b>assert</b>(<a href="#0x1_SortedLinkedList_node_exists">node_exists</a>&lt;T&gt;(<b>copy</b> prev_entry), 1);
    <b>let</b> prev_nodes = &borrow_global&lt;<a href="#0x1_SortedLinkedList_NodeVector">NodeVector</a>&lt;T&gt;&gt;(prev_entry.addr).nodes;

    //get a reference <b>to</b> prev_node and find the address and reference <b>to</b> next_node, head
    <b>let</b> prev_node = <a href="Vector.md#0x1_Vector_borrow">Vector::borrow</a>(prev_nodes, prev_entry.index);
    <b>let</b> next_entry = *&prev_node.next;
    <b>let</b> next_node_vector = &borrow_global&lt;<a href="#0x1_SortedLinkedList_NodeVector">NodeVector</a>&lt;T&gt;&gt;(next_entry.addr).nodes;
    <b>let</b> next_node = <a href="Vector.md#0x1_Vector_borrow">Vector::borrow</a>(next_node_vector, next_entry.index);
    <b>let</b> head_entry = *&next_node.head;

    //see <b>if</b> either prev or next are the head and get their datas
    <b>let</b> prev_data = *&prev_node.data;
    <b>let</b> next_data = *&next_node.data;
    <b>let</b> data_lcs_bytes = <a href="LCS.md#0x1_LCS_to_bytes">LCS::to_bytes</a>(&data);
    <b>let</b> cmp_with_prev = <a href="Compare.md#0x1_Compare_cmp_lcs_bytes">Compare::cmp_lcs_bytes</a>(&data_lcs_bytes, &<a href="LCS.md#0x1_LCS_to_bytes">LCS::to_bytes</a>(&prev_data));
    <b>let</b> cmp_with_next = <a href="Compare.md#0x1_Compare_cmp_lcs_bytes">Compare::cmp_lcs_bytes</a>(&data_lcs_bytes, &<a href="LCS.md#0x1_LCS_to_bytes">LCS::to_bytes</a>(&next_data));

    <b>let</b> prev_is_head = <a href="#0x1_SortedLinkedList_is_head_node">Self::is_head_node</a>&lt;T&gt;(&prev_entry);
    <b>let</b> next_is_head = <a href="#0x1_SortedLinkedList_is_head_node">Self::is_head_node</a>&lt;T&gt;(&next_entry);

    //check the order -- the list must be sorted
    <b>assert</b>(prev_is_head || cmp_with_prev == 2u8, 6); // prev_is_head || data &gt; prev_data
    <b>assert</b>(next_is_head || cmp_with_next == 1u8, 7); // next_is_head || data &lt; next_data

    //create the new node
    <b>let</b> node = <a href="#0x1_SortedLinkedList_Node">Self::Node</a>&lt;T&gt; {
        prev: <b>copy</b> prev_entry,
        next: <b>copy</b> next_entry,
        head: head_entry,
        data: data
    };

    <b>let</b> index = 0u64;
    <b>if</b> (!exists&lt;<a href="#0x1_SortedLinkedList_NodeVector">NodeVector</a>&lt;T&gt;&gt;(sender_address)) {
        move_to&lt;<a href="#0x1_SortedLinkedList_NodeVector">NodeVector</a>&lt;T&gt;&gt;(account, <a href="#0x1_SortedLinkedList_NodeVector">NodeVector</a>&lt;T&gt; { nodes: <a href="Vector.md#0x1_Vector_singleton">Vector::singleton</a>(node) });
    } <b>else</b> {
        <b>let</b> node_vector_mut = &<b>mut</b> borrow_global_mut&lt;<a href="#0x1_SortedLinkedList_NodeVector">NodeVector</a>&lt;T&gt;&gt;(sender_address).nodes;
        <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>&lt;<a href="#0x1_SortedLinkedList_Node">Node</a>&lt;T&gt;&gt;(node_vector_mut, node);
        index = <a href="Vector.md#0x1_Vector_length">Vector::length</a>&lt;<a href="#0x1_SortedLinkedList_Node">Node</a>&lt;T&gt;&gt;(node_vector_mut) - 1;
    };

    <b>let</b> prev_node_vector_mut = &<b>mut</b> borrow_global_mut&lt;<a href="#0x1_SortedLinkedList_NodeVector">NodeVector</a>&lt;T&gt;&gt;(prev_entry.addr).nodes;
    <b>let</b> prev_node_mut = <a href="Vector.md#0x1_Vector_borrow_mut">Vector::borrow_mut</a>(prev_node_vector_mut, prev_entry.index);
    //fix the pointers at prev
    prev_node_mut.next.addr = sender_address;
    prev_node_mut.next.index = index;

    <b>let</b> next_node_vector_mut = &<b>mut</b> borrow_global_mut&lt;<a href="#0x1_SortedLinkedList_NodeVector">NodeVector</a>&lt;T&gt;&gt;(next_entry.addr).nodes;
    <b>let</b> next_node_mut = <a href="Vector.md#0x1_Vector_borrow_mut">Vector::borrow_mut</a>(next_node_vector_mut, next_entry.index);
    //fix the pointers at next
    next_node_mut.prev.addr = sender_address;
    next_node_mut.prev.index = index;
}
</code></pre>



</details>

<a name="0x1_SortedLinkedList_remove_node"></a>

## Function `remove_node`



<pre><code><b>fun</b> <a href="#0x1_SortedLinkedList_remove_node">remove_node</a>&lt;T: <b>copyable</b>&gt;(entry: <a href="#0x1_SortedLinkedList_EntryHandle">SortedLinkedList::EntryHandle</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_SortedLinkedList_remove_node">remove_node</a>&lt;T: <b>copyable</b>&gt;(entry: <a href="#0x1_SortedLinkedList_EntryHandle">EntryHandle</a>) <b>acquires</b> <a href="#0x1_SortedLinkedList_NodeVector">NodeVector</a> {
    //check that a node exists
    <b>assert</b>(<a href="#0x1_SortedLinkedList_node_exists">node_exists</a>&lt;T&gt;(<b>copy</b> entry), 1);
    <b>let</b> nodes = &borrow_global&lt;<a href="#0x1_SortedLinkedList_NodeVector">NodeVector</a>&lt;T&gt;&gt;(entry.addr).nodes;

    //find prev and next
    <b>let</b> current_node = <a href="Vector.md#0x1_Vector_borrow">Vector::borrow</a>(nodes, entry.index);
    <b>let</b> prev_entry = *&current_node.prev;
    <b>let</b> next_entry = *&current_node.next;

    <b>let</b> prev_node_vector_mut = &<b>mut</b> borrow_global_mut&lt;<a href="#0x1_SortedLinkedList_NodeVector">NodeVector</a>&lt;T&gt;&gt;(prev_entry.addr).nodes;
    <b>let</b> prev_node_mut = <a href="Vector.md#0x1_Vector_borrow_mut">Vector::borrow_mut</a>(prev_node_vector_mut, prev_entry.index);
    //fix the pointers at prev
    prev_node_mut.next.addr = next_entry.addr;
    prev_node_mut.next.index = next_entry.index;

    <b>let</b> next_node_vector_mut = &<b>mut</b> borrow_global_mut&lt;<a href="#0x1_SortedLinkedList_NodeVector">NodeVector</a>&lt;T&gt;&gt;(next_entry.addr).nodes;
    <b>let</b> next_node_mut = <a href="Vector.md#0x1_Vector_borrow_mut">Vector::borrow_mut</a>(next_node_vector_mut, next_entry.index);
    //fix the pointers at next
    next_node_mut.prev.addr = prev_entry.addr;
    next_node_mut.prev.index = prev_entry.index;

    <b>let</b> node_vector_mut = &<b>mut</b> borrow_global_mut&lt;<a href="#0x1_SortedLinkedList_NodeVector">NodeVector</a>&lt;T&gt;&gt;(entry.addr).nodes;
    //destroy the current node
    <b>let</b> <a href="#0x1_SortedLinkedList_Node">Node</a>&lt;T&gt; { prev: _, next: _, head: _, data: _ } = <a href="Vector.md#0x1_Vector_remove">Vector::remove</a>&lt;<a href="#0x1_SortedLinkedList_Node">Node</a>&lt;T&gt;&gt;(node_vector_mut, entry.index);
}
</code></pre>



</details>

<a name="0x1_SortedLinkedList_remove_node_by_list_owner"></a>

## Function `remove_node_by_list_owner`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_SortedLinkedList_remove_node_by_list_owner">remove_node_by_list_owner</a>&lt;T: <b>copyable</b>&gt;(account: &signer, entry: <a href="#0x1_SortedLinkedList_EntryHandle">SortedLinkedList::EntryHandle</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_SortedLinkedList_remove_node_by_list_owner">remove_node_by_list_owner</a>&lt;T: <b>copyable</b>&gt;(account: &signer, entry: <a href="#0x1_SortedLinkedList_EntryHandle">EntryHandle</a>) <b>acquires</b> <a href="#0x1_SortedLinkedList_NodeVector">NodeVector</a> {
    //check that a node exists
    <b>assert</b>(<a href="#0x1_SortedLinkedList_node_exists">node_exists</a>&lt;T&gt;(<b>copy</b> entry), 1);
    //make sure it is not a head node
    <b>assert</b>(!<a href="#0x1_SortedLinkedList_is_head_node">Self::is_head_node</a>&lt;T&gt;(&<b>copy</b> entry), 10);
    //make sure the caller owns the list

    <b>let</b> nodes = &borrow_global&lt;<a href="#0x1_SortedLinkedList_NodeVector">NodeVector</a>&lt;T&gt;&gt;(entry.addr).nodes;
    <b>let</b> current_node = <a href="Vector.md#0x1_Vector_borrow">Vector::borrow</a>(nodes, entry.index);
    <b>let</b> list_owner = current_node.head.addr;
    <b>assert</b>(list_owner == <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account), 11);

    //remove it
    <a href="#0x1_SortedLinkedList_remove_node">Self::remove_node</a>&lt;T&gt;(entry);
}
</code></pre>



</details>

<a name="0x1_SortedLinkedList_remove_node_by_node_owner"></a>

## Function `remove_node_by_node_owner`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_SortedLinkedList_remove_node_by_node_owner">remove_node_by_node_owner</a>&lt;T: <b>copyable</b>&gt;(account: &signer, entry: <a href="#0x1_SortedLinkedList_EntryHandle">SortedLinkedList::EntryHandle</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_SortedLinkedList_remove_node_by_node_owner">remove_node_by_node_owner</a>&lt;T: <b>copyable</b>&gt;(account: &signer, entry: <a href="#0x1_SortedLinkedList_EntryHandle">EntryHandle</a>) <b>acquires</b> <a href="#0x1_SortedLinkedList_NodeVector">NodeVector</a> {
    //check that a node exists
    <b>assert</b>(<a href="#0x1_SortedLinkedList_node_exists">node_exists</a>&lt;T&gt;(<b>copy</b> entry), 1);
    //make sure it is not a head node
    <b>assert</b>(!<a href="#0x1_SortedLinkedList_is_head_node">Self::is_head_node</a>&lt;T&gt;(&<b>copy</b> entry), 10);
    //make sure the caller owns the node
    <b>assert</b>(entry.addr == <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account), 11);

    //remove it
    <a href="#0x1_SortedLinkedList_remove_node">Self::remove_node</a>&lt;T&gt;(entry);
}
</code></pre>



</details>

<a name="0x1_SortedLinkedList_remove_list"></a>

## Function `remove_list`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_SortedLinkedList_remove_list">remove_list</a>&lt;T: <b>copyable</b>&gt;(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_SortedLinkedList_remove_list">remove_list</a>&lt;T: <b>copyable</b>&gt;(account: &signer) <b>acquires</b> <a href="#0x1_SortedLinkedList_NodeVector">NodeVector</a> {
    <b>let</b> sender_address = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);

    //fail <b>if</b> the caller does not own a list
    <b>assert</b>(<a href="#0x1_SortedLinkedList_is_head_node">Self::is_head_node</a>&lt;T&gt;(&<a href="#0x1_SortedLinkedList_entry_handle">Self::entry_handle</a>(sender_address, 0)), 14);

    <b>let</b> node_vector = &borrow_global&lt;<a href="#0x1_SortedLinkedList_NodeVector">NodeVector</a>&lt;T&gt;&gt;(sender_address).nodes;
    <b>let</b> current_node = <a href="Vector.md#0x1_Vector_borrow">Vector::borrow</a>(node_vector, 0);

    //check that the list is empty
    <b>assert</b>(current_node.next.addr == sender_address, 15);
    <b>assert</b>(current_node.next.index == 0, 16);
    <b>assert</b>(current_node.prev.addr == sender_address, 17);
    <b>assert</b>(current_node.prev.index == 0, 18);

    //destroy the <a href="#0x1_SortedLinkedList_Node">Node</a>
    <b>let</b> <a href="#0x1_SortedLinkedList_NodeVector">NodeVector</a> { nodes: nodes } = move_from&lt;<a href="#0x1_SortedLinkedList_NodeVector">NodeVector</a>&lt;T&gt;&gt;(sender_address);
    <b>let</b> <a href="#0x1_SortedLinkedList_Node">Node</a>&lt;T&gt; { prev: _, next: _, head: _, data: _ } = <a href="Vector.md#0x1_Vector_remove">Vector::remove</a>&lt;<a href="#0x1_SortedLinkedList_Node">Node</a>&lt;T&gt;&gt;(&<b>mut</b> nodes, 0);
    <a href="Vector.md#0x1_Vector_destroy_empty">Vector::destroy_empty</a>(nodes);
}
</code></pre>



</details>

<a name="0x1_SortedLinkedList_find_position_and_insert"></a>

## Function `find_position_and_insert`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_SortedLinkedList_find_position_and_insert">find_position_and_insert</a>&lt;T: <b>copyable</b>&gt;(account: &signer, data: T, head: <a href="#0x1_SortedLinkedList_EntryHandle">SortedLinkedList::EntryHandle</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_SortedLinkedList_find_position_and_insert">find_position_and_insert</a>&lt;T: <b>copyable</b>&gt;(account: &signer, data: T, head: <a href="#0x1_SortedLinkedList_EntryHandle">EntryHandle</a>): bool <b>acquires</b> <a href="#0x1_SortedLinkedList_NodeVector">NodeVector</a> {
    <b>assert</b>(<a href="#0x1_SortedLinkedList_is_head_node">Self::is_head_node</a>&lt;T&gt;(&<b>copy</b> head), 18);

    <b>let</b> data_lcs_bytes = <a href="LCS.md#0x1_LCS_to_bytes">LCS::to_bytes</a>(&data);
    <b>let</b> nodes = &borrow_global&lt;<a href="#0x1_SortedLinkedList_NodeVector">NodeVector</a>&lt;T&gt;&gt;(head.addr).nodes;
    <b>let</b> head_node = <a href="Vector.md#0x1_Vector_borrow">Vector::borrow</a>&lt;<a href="#0x1_SortedLinkedList_Node">Node</a>&lt;T&gt;&gt;(nodes, head.index);
    <b>let</b> next_entry = *&head_node.next;
    <b>let</b> last_entry = *&head_node.prev;

    <b>while</b> (!<a href="#0x1_SortedLinkedList_is_head_node">Self::is_head_node</a>&lt;T&gt;(&next_entry)) {
        <b>let</b> next_nodes = &borrow_global&lt;<a href="#0x1_SortedLinkedList_NodeVector">NodeVector</a>&lt;T&gt;&gt;(next_entry.addr).nodes;
        <b>let</b> next_node = <a href="Vector.md#0x1_Vector_borrow">Vector::borrow</a>&lt;<a href="#0x1_SortedLinkedList_Node">Node</a>&lt;T&gt;&gt;(next_nodes, next_entry.index);

        <b>let</b> next_node_data = *&next_node.data;
        <b>let</b> next_data_lcs_bytes = <a href="LCS.md#0x1_LCS_to_bytes">LCS::to_bytes</a>(&next_node_data);
        <b>let</b> cmp = <a href="Compare.md#0x1_Compare_cmp_lcs_bytes">Compare::cmp_lcs_bytes</a>(&next_data_lcs_bytes, &data_lcs_bytes);

        <b>if</b> (cmp == 0u8) { // next_data == data
            <b>return</b> <b>false</b>  // data already exist
        } <b>else</b> <b>if</b> (cmp == 1u8) { // next_data &lt; data, <b>continue</b>
            next_entry = *&next_node.next;
        } <b>else</b> { // next_data &gt; data, nothing found
            <b>let</b> prev_entry = *&next_node.prev;
            <a href="#0x1_SortedLinkedList_insert_node">insert_node</a>(account, data, prev_entry);
            <b>return</b> <b>true</b>
        }
    };
    // list is empty, insert after head
    <a href="#0x1_SortedLinkedList_insert_node">insert_node</a>(account, data, last_entry);
    <b>true</b>
}
</code></pre>



</details>
