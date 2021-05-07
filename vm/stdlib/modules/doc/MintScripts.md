
<a name="0x1_MintScripts"></a>

# Module `0x1::MintScripts`



-  [Constants](#@Constants_0)
-  [Function `mint_and_split_by_linear_key`](#0x1_MintScripts_mint_and_split_by_linear_key)
-  [Function `mint_token_by_fixed_key`](#0x1_MintScripts_mint_token_by_fixed_key)
-  [Function `mint_token_by_linear_key`](#0x1_MintScripts_mint_token_by_linear_key)
-  [Function `split_fixed_key`](#0x1_MintScripts_split_fixed_key)
-  [Specification](#@Specification_1)
    -  [Function `mint_and_split_by_linear_key`](#@Specification_1_mint_and_split_by_linear_key)
    -  [Function `mint_token_by_fixed_key`](#@Specification_1_mint_token_by_fixed_key)
    -  [Function `mint_token_by_linear_key`](#@Specification_1_mint_token_by_linear_key)
    -  [Function `split_fixed_key`](#@Specification_1_split_fixed_key)


<pre><code><b>use</b> <a href="Errors.md#0x1_Errors">0x1::Errors</a>;
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x1_MintScripts_EDEPRECATED_FUNCTION"></a>



<pre><code><b>const</b> <a href="MintScripts.md#0x1_MintScripts_EDEPRECATED_FUNCTION">EDEPRECATED_FUNCTION</a>: u64 = 11;
</code></pre>



<a name="0x1_MintScripts_mint_and_split_by_linear_key"></a>

## Function `mint_and_split_by_linear_key`



<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="MintScripts.md#0x1_MintScripts_mint_and_split_by_linear_key">mint_and_split_by_linear_key</a>&lt;<a href="Token.md#0x1_Token">Token</a>&gt;(_signer: signer, _for_address: address, _amount: u128, _lock_period: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="MintScripts.md#0x1_MintScripts_mint_and_split_by_linear_key">mint_and_split_by_linear_key</a>&lt;<a href="Token.md#0x1_Token">Token</a>: store&gt;(
    _signer: signer,
    _for_address: address,
    _amount: u128,
    _lock_period: u64,
) {
    <b>abort</b> <a href="Errors.md#0x1_Errors_deprecated">Errors::deprecated</a>(<a href="MintScripts.md#0x1_MintScripts_EDEPRECATED_FUNCTION">EDEPRECATED_FUNCTION</a>)
}
</code></pre>



</details>

<a name="0x1_MintScripts_mint_token_by_fixed_key"></a>

## Function `mint_token_by_fixed_key`



<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="MintScripts.md#0x1_MintScripts_mint_token_by_fixed_key">mint_token_by_fixed_key</a>&lt;<a href="Token.md#0x1_Token">Token</a>&gt;(_signer: signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="MintScripts.md#0x1_MintScripts_mint_token_by_fixed_key">mint_token_by_fixed_key</a>&lt;<a href="Token.md#0x1_Token">Token</a>: store&gt;(
    _signer: signer,
) {
   <b>abort</b> <a href="Errors.md#0x1_Errors_deprecated">Errors::deprecated</a>(<a href="MintScripts.md#0x1_MintScripts_EDEPRECATED_FUNCTION">EDEPRECATED_FUNCTION</a>)
}
</code></pre>



</details>

<a name="0x1_MintScripts_mint_token_by_linear_key"></a>

## Function `mint_token_by_linear_key`



<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="MintScripts.md#0x1_MintScripts_mint_token_by_linear_key">mint_token_by_linear_key</a>&lt;<a href="Token.md#0x1_Token">Token</a>&gt;(_signer: signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="MintScripts.md#0x1_MintScripts_mint_token_by_linear_key">mint_token_by_linear_key</a>&lt;<a href="Token.md#0x1_Token">Token</a>: store&gt;(
    _signer: signer,
) {
   <b>abort</b> <a href="Errors.md#0x1_Errors_deprecated">Errors::deprecated</a>(<a href="MintScripts.md#0x1_MintScripts_EDEPRECATED_FUNCTION">EDEPRECATED_FUNCTION</a>)
}
</code></pre>



</details>

<a name="0x1_MintScripts_split_fixed_key"></a>

## Function `split_fixed_key`



<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="MintScripts.md#0x1_MintScripts_split_fixed_key">split_fixed_key</a>&lt;<a href="Token.md#0x1_Token">Token</a>&gt;(_signer: signer, _for_address: address, _amount: u128, _lock_period: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="MintScripts.md#0x1_MintScripts_split_fixed_key">split_fixed_key</a>&lt;<a href="Token.md#0x1_Token">Token</a>: store&gt;(
    _signer: signer,
    _for_address: address,
    _amount: u128,
    _lock_period: u64,
) {
   <b>abort</b> <a href="Errors.md#0x1_Errors_deprecated">Errors::deprecated</a>(<a href="MintScripts.md#0x1_MintScripts_EDEPRECATED_FUNCTION">EDEPRECATED_FUNCTION</a>)
}
</code></pre>



</details>

<a name="@Specification_1"></a>

## Specification


<a name="@Specification_1_mint_and_split_by_linear_key"></a>

### Function `mint_and_split_by_linear_key`


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="MintScripts.md#0x1_MintScripts_mint_and_split_by_linear_key">mint_and_split_by_linear_key</a>&lt;<a href="Token.md#0x1_Token">Token</a>&gt;(_signer: signer, _for_address: address, _amount: u128, _lock_period: u64)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>



<a name="@Specification_1_mint_token_by_fixed_key"></a>

### Function `mint_token_by_fixed_key`


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="MintScripts.md#0x1_MintScripts_mint_token_by_fixed_key">mint_token_by_fixed_key</a>&lt;<a href="Token.md#0x1_Token">Token</a>&gt;(_signer: signer)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>



<a name="@Specification_1_mint_token_by_linear_key"></a>

### Function `mint_token_by_linear_key`


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="MintScripts.md#0x1_MintScripts_mint_token_by_linear_key">mint_token_by_linear_key</a>&lt;<a href="Token.md#0x1_Token">Token</a>&gt;(_signer: signer)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>



<a name="@Specification_1_split_fixed_key"></a>

### Function `split_fixed_key`


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="MintScripts.md#0x1_MintScripts_split_fixed_key">split_fixed_key</a>&lt;<a href="Token.md#0x1_Token">Token</a>&gt;(_signer: signer, _for_address: address, _amount: u128, _lock_period: u64)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>
