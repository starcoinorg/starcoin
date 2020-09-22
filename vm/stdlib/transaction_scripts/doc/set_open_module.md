
<a name="SCRIPT"></a>

# Script `set_open_module.move`

### Table of Contents

-  [Function `main`](#SCRIPT_main)



<a name="SCRIPT_main"></a>

## Function `main`



<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_main">main</a>(account: &signer, open_module: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_main">main</a>(account: &signer, open_module: bool) {
    <a href="../../modules/doc/TransactionPublishOption.md#0x1_TransactionPublishOption_set_open_module">TransactionPublishOption::set_open_module</a>(account, open_module)
}
</code></pre>



</details>
