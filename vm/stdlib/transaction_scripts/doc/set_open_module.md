
<a name="SCRIPT"></a>

# Script `set_open_module.move`

### Table of Contents

-  [Function `set_open_module`](#SCRIPT_set_open_module)
-  [Specification](#SCRIPT_Specification)
    -  [Function `set_open_module`](#SCRIPT_Specification_set_open_module)



<a name="SCRIPT_set_open_module"></a>

## Function `set_open_module`



<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_set_open_module">set_open_module</a>(account: &signer, open_module: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_set_open_module">set_open_module</a>(account: &signer, open_module: bool) {
    <a href="../../modules/doc/TransactionPublishOption.md#0x1_TransactionPublishOption_set_open_module">TransactionPublishOption::set_open_module</a>(account, open_module)
}
</code></pre>



</details>

<a name="SCRIPT_Specification"></a>

## Specification


<a name="SCRIPT_Specification_set_open_module"></a>

### Function `set_open_module`


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_set_open_module">set_open_module</a>(account: &signer, open_module: bool)
</code></pre>




<pre><code>pragma verify = <b>false</b>;
</code></pre>
