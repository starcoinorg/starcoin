
<a name="add_to_script_allow_list"></a>

# Script `add_to_script_allow_list`



-  [Specification](#@Specification_0)
    -  [Function <code><a href="add_to_script_allow_list.md#add_to_script_allow_list">add_to_script_allow_list</a></code>](#@Specification_0_add_to_script_allow_list)

Append the <code>hash</code> to script hashes list allowed to be executed by the network.
Todo: it's dangous to run the script when publish option is VMPublishingOption::Open
because the list is empty at the moment, adding script into the empty list will lead to
that only the added script is allowed to execute.


<pre><code><b>public</b> <b>fun</b> <a href="add_to_script_allow_list.md#add_to_script_allow_list">add_to_script_allow_list</a>(account: &signer, hash: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="add_to_script_allow_list.md#add_to_script_allow_list">add_to_script_allow_list</a>(account: &signer, hash: vector&lt;u8&gt;) {
    <a href="../../modules/doc/TransactionPublishOption.md#0x1_TransactionPublishOption_add_to_script_allow_list">TransactionPublishOption::add_to_script_allow_list</a>(account, hash)
}
</code></pre>



</details>

<a name="@Specification_0"></a>

## Specification


<a name="@Specification_0_add_to_script_allow_list"></a>

### Function `add_to_script_allow_list`


<pre><code><b>public</b> <b>fun</b> <a href="add_to_script_allow_list.md#add_to_script_allow_list">add_to_script_allow_list</a>(account: &signer, hash: vector&lt;u8&gt;)
</code></pre>




<pre><code>pragma verify = <b>false</b>;
</code></pre>
