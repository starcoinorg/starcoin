
<a id="0x1_reserved_accounts_signer"></a>

# Module `0x1::reserved_accounts_signer`



-  [Resource `SignerResponsbility`](#0x1_reserved_accounts_signer_SignerResponsbility)
-  [Function `store_signer_cap`](#0x1_reserved_accounts_signer_store_signer_cap)
-  [Function `get_stored_signer`](#0x1_reserved_accounts_signer_get_stored_signer)


<pre><code><b>use</b> <a href="account.md#0x1_account">0x1::account</a>;
<b>use</b> <a href="../../starcoin-stdlib/doc/simple_map.md#0x1_simple_map">0x1::simple_map</a>;
<b>use</b> <a href="system_addresses.md#0x1_system_addresses">0x1::system_addresses</a>;
</code></pre>



<a id="0x1_reserved_accounts_signer_SignerResponsbility"></a>

## Resource `SignerResponsbility`



<pre><code><b>struct</b> <a href="reserved_accounts_signer.md#0x1_reserved_accounts_signer_SignerResponsbility">SignerResponsbility</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>signer_caps: <a href="../../starcoin-stdlib/doc/simple_map.md#0x1_simple_map_SimpleMap">simple_map::SimpleMap</a>&lt;<b>address</b>, <a href="account.md#0x1_account_SignerCapability">account::SignerCapability</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="0x1_reserved_accounts_signer_store_signer_cap"></a>

## Function `store_signer_cap`

Can be called during genesis or by the governance itself.
Stores the signer capability for a given address.


<pre><code><b>public</b> <b>fun</b> <a href="reserved_accounts_signer.md#0x1_reserved_accounts_signer_store_signer_cap">store_signer_cap</a>(starcoin_framework: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, signer_address: <b>address</b>, signer_cap: <a href="account.md#0x1_account_SignerCapability">account::SignerCapability</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="reserved_accounts_signer.md#0x1_reserved_accounts_signer_store_signer_cap">store_signer_cap</a>(
    starcoin_framework: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    signer_address: <b>address</b>,
    signer_cap: <a href="account.md#0x1_account_SignerCapability">account::SignerCapability</a>,
) <b>acquires</b> <a href="reserved_accounts_signer.md#0x1_reserved_accounts_signer_SignerResponsbility">SignerResponsbility</a> {
    <a href="system_addresses.md#0x1_system_addresses_assert_starcoin_framework">system_addresses::assert_starcoin_framework</a>(starcoin_framework);
    <a href="system_addresses.md#0x1_system_addresses_assert_framework_reserved">system_addresses::assert_framework_reserved</a>(signer_address);

    <b>if</b> (!<b>exists</b>&lt;<a href="reserved_accounts_signer.md#0x1_reserved_accounts_signer_SignerResponsbility">SignerResponsbility</a>&gt;(@starcoin_framework)) {
        <b>move_to</b>(
            starcoin_framework,
            <a href="reserved_accounts_signer.md#0x1_reserved_accounts_signer_SignerResponsbility">SignerResponsbility</a> { signer_caps: <a href="../../starcoin-stdlib/doc/simple_map.md#0x1_simple_map_create">simple_map::create</a>&lt;<b>address</b>, <a href="account.md#0x1_account_SignerCapability">account::SignerCapability</a>&gt;() }
        );
    };

    <b>let</b> signer_caps =
        &<b>mut</b> <b>borrow_global_mut</b>&lt;<a href="reserved_accounts_signer.md#0x1_reserved_accounts_signer_SignerResponsbility">SignerResponsbility</a>&gt;(@starcoin_framework).signer_caps;
    <a href="../../starcoin-stdlib/doc/simple_map.md#0x1_simple_map_add">simple_map::add</a>(signer_caps, signer_address, signer_cap);
}
</code></pre>



</details>

<a id="0x1_reserved_accounts_signer_get_stored_signer"></a>

## Function `get_stored_signer`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="reserved_accounts_signer.md#0x1_reserved_accounts_signer_get_stored_signer">get_stored_signer</a>(addr: <b>address</b>): <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="reserved_accounts_signer.md#0x1_reserved_accounts_signer_get_stored_signer">get_stored_signer</a>(addr: <b>address</b>): <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a> <b>acquires</b> <a href="reserved_accounts_signer.md#0x1_reserved_accounts_signer_SignerResponsbility">SignerResponsbility</a> {
    <b>let</b> cap = <b>borrow_global</b>&lt;<a href="reserved_accounts_signer.md#0x1_reserved_accounts_signer_SignerResponsbility">SignerResponsbility</a>&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>());
    <a href="account.md#0x1_account_create_signer_with_capability">account::create_signer_with_capability</a>(<a href="../../starcoin-stdlib/doc/simple_map.md#0x1_simple_map_borrow">simple_map::borrow</a>(&cap.signer_caps, &addr))
}
</code></pre>



</details>


[move-book]: https://starcoin.dev/move/book/SUMMARY
