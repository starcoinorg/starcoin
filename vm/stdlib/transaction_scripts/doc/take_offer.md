
<a name="take_offer"></a>

# Script `take_offer`



-  [Specification](#@Specification_0)
    -  [Function `take_offer`](#@Specification_0_take_offer)


<pre><code><b>use</b> <a href="../../modules/doc/Box.md#0x1_Box">0x1::Box</a>;
<b>use</b> <a href="../../modules/doc/Offer.md#0x1_Offer">0x1::Offer</a>;
</code></pre>




<pre><code><b>public</b> <b>fun</b> <a href="take_offer.md#take_offer">take_offer</a>&lt;Offered: <b>copyable</b>&gt;(signer: &signer, offer_address: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="take_offer.md#take_offer">take_offer</a>&lt;Offered: <b>copyable</b>&gt;(
    signer: &signer,
    offer_address: address,
) {
    <b>let</b> offered = <a href="../../modules/doc/Offer.md#0x1_Offer_redeem">Offer::redeem</a>&lt;Offered&gt;(signer, offer_address);
    <a href="../../modules/doc/Box.md#0x1_Box_put">Box::put</a>(signer, offered);
}
</code></pre>



</details>

<a name="@Specification_0"></a>

## Specification


<a name="@Specification_0_take_offer"></a>

### Function `take_offer`


<pre><code><b>public</b> <b>fun</b> <a href="take_offer.md#take_offer">take_offer</a>&lt;Offered: <b>copyable</b>&gt;(signer: &signer, offer_address: address)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>
