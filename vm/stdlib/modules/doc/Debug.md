
<a name="0x1_Debug"></a>

# Module `0x1::Debug`



-  [Function <code>print</code>](#0x1_Debug_print)
-  [Function <code>print_stack_trace</code>](#0x1_Debug_print_stack_trace)
-  [Specification](#@Specification_0)


<a name="0x1_Debug_print"></a>

## Function `print`



<pre><code><b>public</b> <b>fun</b> <a href="Debug.md#0x1_Debug_print">print</a>&lt;T&gt;(x: &T)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>public</b> <b>fun</b> <a href="Debug.md#0x1_Debug_print">print</a>&lt;T&gt;(x: &T);
</code></pre>



</details>

<a name="0x1_Debug_print_stack_trace"></a>

## Function `print_stack_trace`



<pre><code><b>public</b> <b>fun</b> <a href="Debug.md#0x1_Debug_print_stack_trace">print_stack_trace</a>()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>public</b> <b>fun</b> <a href="Debug.md#0x1_Debug_print_stack_trace">print_stack_trace</a>();
</code></pre>



</details>

<a name="@Specification_0"></a>

## Specification



<pre><code>pragma verify;
pragma aborts_if_is_strict;
</code></pre>
