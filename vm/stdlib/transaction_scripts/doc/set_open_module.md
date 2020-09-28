
<a name="set_open_module"></a>

# Script `set_open_module`



-  [Specification](#@Specification_0)
    -  [Function <code><a href="set_open_module.md#set_open_module">set_open_module</a></code>](#@Specification_0_set_open_module)



<pre><code><b>public</b> <b>fun</b> <a href="set_open_module.md#set_open_module">set_open_module</a>(account: &signer, open_module: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="set_open_module.md#set_open_module">set_open_module</a>(account: &signer, open_module: bool) {
    <a href="../../modules/doc/TransactionPublishOption.md#0x1_TransactionPublishOption_set_open_module">TransactionPublishOption::set_open_module</a>(account, open_module)
}
</code></pre>



</details>

<a name="@Specification_0"></a>

## Specification


<a name="@Specification_0_set_open_module"></a>

### Function `set_open_module`


<pre><code><b>public</b> <b>fun</b> <a href="set_open_module.md#set_open_module">set_open_module</a>(account: &signer, open_module: bool)
</code></pre>




<pre><code>pragma verify = <b>false</b>;
</code></pre>
