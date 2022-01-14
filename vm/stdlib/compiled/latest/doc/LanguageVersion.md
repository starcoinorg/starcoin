
<a name="0x1_LanguageVersion"></a>

# Module `0x1::LanguageVersion`



-  [Struct `LanguageVersion`](#0x1_LanguageVersion_LanguageVersion)
-  [Function `new`](#0x1_LanguageVersion_new)
-  [Function `version`](#0x1_LanguageVersion_version)
-  [Specification](#@Specification_0)
    -  [Function `new`](#@Specification_0_new)
    -  [Function `version`](#@Specification_0_version)


<pre><code></code></pre>



<a name="0x1_LanguageVersion_LanguageVersion"></a>

## Struct `LanguageVersion`



<pre><code><b>struct</b> <a href="LanguageVersion.md#0x1_LanguageVersion">LanguageVersion</a> <b>has</b> <b>copy</b>, drop, store
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

<a name="0x1_LanguageVersion_new"></a>

## Function `new`



<pre><code><b>public</b> <b>fun</b> <a href="LanguageVersion.md#0x1_LanguageVersion_new">new</a>(version: u64): <a href="LanguageVersion.md#0x1_LanguageVersion_LanguageVersion">LanguageVersion::LanguageVersion</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="LanguageVersion.md#0x1_LanguageVersion_new">new</a>(version: u64): <a href="LanguageVersion.md#0x1_LanguageVersion">LanguageVersion</a> {
    <a href="LanguageVersion.md#0x1_LanguageVersion">LanguageVersion</a> {major: version}
}
</code></pre>



</details>

<a name="0x1_LanguageVersion_version"></a>

## Function `version`



<pre><code><b>public</b> <b>fun</b> <a href="LanguageVersion.md#0x1_LanguageVersion_version">version</a>(version: &<a href="LanguageVersion.md#0x1_LanguageVersion_LanguageVersion">LanguageVersion::LanguageVersion</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="LanguageVersion.md#0x1_LanguageVersion_version">version</a>(version: &<a href="LanguageVersion.md#0x1_LanguageVersion">LanguageVersion</a>): u64 {
    version.major
}
</code></pre>



</details>

<a name="@Specification_0"></a>

## Specification


<a name="@Specification_0_new"></a>

### Function `new`


<pre><code><b>public</b> <b>fun</b> <a href="LanguageVersion.md#0x1_LanguageVersion_new">new</a>(version: u64): <a href="LanguageVersion.md#0x1_LanguageVersion_LanguageVersion">LanguageVersion::LanguageVersion</a>
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>



<a name="@Specification_0_version"></a>

### Function `version`


<pre><code><b>public</b> <b>fun</b> <a href="LanguageVersion.md#0x1_LanguageVersion_version">version</a>(version: &<a href="LanguageVersion.md#0x1_LanguageVersion_LanguageVersion">LanguageVersion::LanguageVersion</a>): u64
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>
