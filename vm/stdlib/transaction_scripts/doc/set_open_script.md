
<a name="SCRIPT"></a>

# Script `set_open_script.move`

### Table of Contents

-  [Function `set_open_script`](#SCRIPT_set_open_script)
-  [Specification](#SCRIPT_Specification)
    -  [Function `set_open_script`](#SCRIPT_Specification_set_open_script)



<a name="SCRIPT_set_open_script"></a>

## Function `set_open_script`



<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_set_open_script">set_open_script</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_set_open_script">set_open_script</a>(account: &signer) {
    <a href="../../modules/doc/TransactionPublishOption.md#0x1_TransactionPublishOption_set_open_script">TransactionPublishOption::set_open_script</a>(account)
}
</code></pre>



</details>

<a name="SCRIPT_Specification"></a>

## Specification


<a name="SCRIPT_Specification_set_open_script"></a>

### Function `set_open_script`


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_set_open_script">set_open_script</a>(account: &signer)
</code></pre>




<pre><code>pragma verify = <b>false</b>;
</code></pre>
