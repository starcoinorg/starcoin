
<a id="0x1_stc_language_version"></a>

# Module `0x1::stc_language_version`



-  [Struct `LanguageVersion`](#0x1_stc_language_version_LanguageVersion)
-  [Function `new`](#0x1_stc_language_version_new)
-  [Function `version`](#0x1_stc_language_version_version)
-  [Specification](#@Specification_0)
    -  [Function `new`](#@Specification_0_new)
    -  [Function `version`](#@Specification_0_version)


<pre><code></code></pre>



<a id="0x1_stc_language_version_LanguageVersion"></a>

## Struct `LanguageVersion`



<pre><code><b>struct</b> <a href="stc_language_version.md#0x1_stc_language_version_LanguageVersion">LanguageVersion</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>major: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="0x1_stc_language_version_new"></a>

## Function `new`



<pre><code><b>public</b> <b>fun</b> <a href="stc_language_version.md#0x1_stc_language_version_new">new</a>(<a href="version.md#0x1_version">version</a>: u64): <a href="stc_language_version.md#0x1_stc_language_version_LanguageVersion">stc_language_version::LanguageVersion</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_language_version.md#0x1_stc_language_version_new">new</a>(<a href="version.md#0x1_version">version</a>: u64): <a href="stc_language_version.md#0x1_stc_language_version_LanguageVersion">LanguageVersion</a> {
    <a href="stc_language_version.md#0x1_stc_language_version_LanguageVersion">LanguageVersion</a> { major: <a href="version.md#0x1_version">version</a> }
}
</code></pre>



</details>

<a id="0x1_stc_language_version_version"></a>

## Function `version`



<pre><code><b>public</b> <b>fun</b> <a href="version.md#0x1_version">version</a>(<a href="version.md#0x1_version">version</a>: &<a href="stc_language_version.md#0x1_stc_language_version_LanguageVersion">stc_language_version::LanguageVersion</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="version.md#0x1_version">version</a>(<a href="version.md#0x1_version">version</a>: &<a href="stc_language_version.md#0x1_stc_language_version_LanguageVersion">LanguageVersion</a>): u64 {
    <a href="version.md#0x1_version">version</a>.major
}
</code></pre>



</details>

<a id="@Specification_0"></a>

## Specification


<a id="@Specification_0_new"></a>

### Function `new`


<pre><code><b>public</b> <b>fun</b> <a href="stc_language_version.md#0x1_stc_language_version_new">new</a>(<a href="version.md#0x1_version">version</a>: u64): <a href="stc_language_version.md#0x1_stc_language_version_LanguageVersion">stc_language_version::LanguageVersion</a>
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>



<a id="@Specification_0_version"></a>

### Function `version`


<pre><code><b>public</b> <b>fun</b> <a href="version.md#0x1_version">version</a>(<a href="version.md#0x1_version">version</a>: &<a href="stc_language_version.md#0x1_stc_language_version_LanguageVersion">stc_language_version::LanguageVersion</a>): u64
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>


[move-book]: https://starcoin.dev/move/book/SUMMARY
