
<a name="0x1_Debug"></a>

# Module `0x1::Debug`

The module provide debug print for Move.


-  [Function `print`](#0x1_Debug_print)
-  [Function `print_stack_trace`](#0x1_Debug_print_stack_trace)
-  [Specification](#@Specification_0)


<pre><code></code></pre>



<a name="0x1_Debug_print"></a>

## Function `print`

Print data of Type <code>T</code>.


<pre><code><b>public</b> <b>fun</b> <a href="Debug.md#0x1_Debug_print">print</a>&lt;T: store&gt;(x: &T)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>public</b> <b>fun</b> <a href="Debug.md#0x1_Debug_print">print</a>&lt;T: store&gt;(x: &T);
</code></pre>



</details>

<a name="0x1_Debug_print_stack_trace"></a>

## Function `print_stack_trace`

Print current stack.


<pre><code><b>public</b> <b>fun</b> <a href="Debug.md#0x1_Debug_print_stack_trace">print_stack_trace</a>()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>public</b> <b>fun</b> <a href="Debug.md#0x1_Debug_print_stack_trace">print_stack_trace</a>();
</code></pre>



</details>

<a name="@Specification_0"></a>

## Specification



<pre><code><b>pragma</b> verify;
<b>pragma</b> aborts_if_is_strict;
</code></pre>
