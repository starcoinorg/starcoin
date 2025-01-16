
<a id="0x1_dao_features_proposal"></a>

# Module `0x1::dao_features_proposal`



-  [Struct `FeaturesUpdate`](#0x1_dao_features_proposal_FeaturesUpdate)
-  [Constants](#@Constants_0)
-  [Function `propose`](#0x1_dao_features_proposal_propose)
-  [Function `execute`](#0x1_dao_features_proposal_execute)
-  [Function `execute_urgent`](#0x1_dao_features_proposal_execute_urgent)
-  [Specification](#@Specification_1)


<pre><code><b>use</b> <a href="create_signer.md#0x1_create_signer">0x1::create_signer</a>;
<b>use</b> <a href="dao.md#0x1_dao">0x1::dao</a>;
<b>use</b> <a href="../../move-stdlib/doc/error.md#0x1_error">0x1::error</a>;
<b>use</b> <a href="../../move-stdlib/doc/features.md#0x1_features">0x1::features</a>;
<b>use</b> <a href="../../move-stdlib/doc/signer.md#0x1_signer">0x1::signer</a>;
<b>use</b> <a href="starcoin_coin.md#0x1_starcoin_coin">0x1::starcoin_coin</a>;
<b>use</b> <a href="system_addresses.md#0x1_system_addresses">0x1::system_addresses</a>;
<b>use</b> <a href="../../move-stdlib/doc/vector.md#0x1_vector">0x1::vector</a>;
</code></pre>



<a id="0x1_dao_features_proposal_FeaturesUpdate"></a>

## Struct `FeaturesUpdate`



<pre><code><b>struct</b> <a href="dao_features_proposal.md#0x1_dao_features_proposal_FeaturesUpdate">FeaturesUpdate</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>enable: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u64&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>disable: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u64&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="@Constants_0"></a>

## Constants


<a id="0x1_dao_features_proposal_E_NOT_ANY_FLAGS"></a>



<pre><code><b>const</b> <a href="dao_features_proposal.md#0x1_dao_features_proposal_E_NOT_ANY_FLAGS">E_NOT_ANY_FLAGS</a>: u64 = 2;
</code></pre>



<a id="0x1_dao_features_proposal_E_NOT_AUTHORIZED"></a>



<pre><code><b>const</b> <a href="dao_features_proposal.md#0x1_dao_features_proposal_E_NOT_AUTHORIZED">E_NOT_AUTHORIZED</a>: u64 = 1;
</code></pre>



<a id="0x1_dao_features_proposal_propose"></a>

## Function `propose`

Entrypoint for the proposal.


<pre><code><b>public</b> entry <b>fun</b> <a href="dao_features_proposal.md#0x1_dao_features_proposal_propose">propose</a>(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, enable: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u64&gt;, disable: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u64&gt;, exec_delay: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="dao_features_proposal.md#0x1_dao_features_proposal_propose">propose</a>(
    <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    enable: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u64&gt;,
    disable: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u64&gt;,
    exec_delay: u64,
) {
    <b>assert</b>!(
        !<a href="../../move-stdlib/doc/vector.md#0x1_vector_is_empty">vector::is_empty</a>(&enable) || !<a href="../../move-stdlib/doc/vector.md#0x1_vector_is_empty">vector::is_empty</a>(&disable),
        <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="dao_features_proposal.md#0x1_dao_features_proposal_E_NOT_ANY_FLAGS">E_NOT_ANY_FLAGS</a>)
    );
    <b>let</b> action = <a href="dao_features_proposal.md#0x1_dao_features_proposal_FeaturesUpdate">FeaturesUpdate</a> {
        enable,
        disable,
    };
    <a href="dao.md#0x1_dao_propose">dao::propose</a>&lt;STC, <a href="dao_features_proposal.md#0x1_dao_features_proposal_FeaturesUpdate">FeaturesUpdate</a>&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, action, exec_delay);
}
</code></pre>



</details>

<a id="0x1_dao_features_proposal_execute"></a>

## Function `execute`

Once the proposal is agreed, anyone can call the method to make the proposal happen.


<pre><code><b>public</b> entry <b>fun</b> <a href="dao_features_proposal.md#0x1_dao_features_proposal_execute">execute</a>(proposal_adderss: <b>address</b>, proposal_id: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="dao_features_proposal.md#0x1_dao_features_proposal_execute">execute</a>(proposal_adderss: <b>address</b>, proposal_id: u64) {
    <b>let</b> <a href="dao_features_proposal.md#0x1_dao_features_proposal_FeaturesUpdate">FeaturesUpdate</a> {
        enable,
        disable,
    } = <a href="dao.md#0x1_dao_extract_proposal_action">dao::extract_proposal_action</a>&lt;STC, <a href="dao_features_proposal.md#0x1_dao_features_proposal_FeaturesUpdate">FeaturesUpdate</a>&gt;(
        proposal_adderss,
        proposal_id
    );
    <b>let</b> starcoin_framework = &<a href="create_signer.md#0x1_create_signer">create_signer</a>(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>());
    <a href="../../move-stdlib/doc/features.md#0x1_features_change_feature_flags_for_next_epoch">features::change_feature_flags_for_next_epoch</a>(starcoin_framework, enable, disable);
    <a href="../../move-stdlib/doc/features.md#0x1_features_on_new_epoch">features::on_new_epoch</a>(starcoin_framework);
}
</code></pre>



</details>

<a id="0x1_dao_features_proposal_execute_urgent"></a>

## Function `execute_urgent`



<pre><code><b>public</b> entry <b>fun</b> <a href="dao_features_proposal.md#0x1_dao_features_proposal_execute_urgent">execute_urgent</a>(core_resource: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, enable: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u64&gt;, disable: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u64&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="dao_features_proposal.md#0x1_dao_features_proposal_execute_urgent">execute_urgent</a>(core_resource: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, enable: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u64&gt;, disable: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u64&gt;) {
    <b>assert</b>!(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(core_resource) == @core_resources, <a href="../../move-stdlib/doc/error.md#0x1_error_unauthenticated">error::unauthenticated</a>(<a href="dao_features_proposal.md#0x1_dao_features_proposal_E_NOT_AUTHORIZED">E_NOT_AUTHORIZED</a>));
    <b>let</b> framework = &<a href="create_signer.md#0x1_create_signer">create_signer</a>(@starcoin_framework);
    <a href="../../move-stdlib/doc/features.md#0x1_features_change_feature_flags_for_next_epoch">features::change_feature_flags_for_next_epoch</a>(framework, enable, disable);
    <a href="../../move-stdlib/doc/features.md#0x1_features_on_new_epoch">features::on_new_epoch</a>(framework);
}
</code></pre>



</details>

<a id="@Specification_1"></a>

## Specification



<pre><code><b>pragma</b> verify = <b>false</b>;
<b>pragma</b> aborts_if_is_strict;
<b>pragma</b> aborts_if_is_partial;
</code></pre>


[move-book]: https://starcoin.dev/move/book/SUMMARY
