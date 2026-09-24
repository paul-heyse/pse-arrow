# `tracing::field::AsField`

Full upstream contracts; raw type trees and source locators in [structured records](tracing.field.AsField.json).

<a id="op-797bbacde9c2232f8405761a"></a>
## AsField

`trait` · `tracing::field::AsField` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
trait AsField: sealed::Sealed
```

Source: `src/field.rs:129`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

Trait implemented to allow a type to be used as a field key.

<pre class="ignore" style="white-space:normal;font:inherit;">
<strong>Note</strong>: Although this is implemented for both the
<a href="./struct.Field.html"><code>Field</code></a> type <em>and</em> any
type that can be borrowed as an <code>&str</code>, only <code>Field</code>
allows <em>O</em>(1) access.
Indexing a field with a string results in an iterative search that performs
string comparisons. Thus, if possible, once the key for a field is known, it
should be used whenever possible.
</pre>

<a id="op-2894fc97293d77611914476d"></a>
## as_field

`function` · `tracing::field::AsField::as_field` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
fn as_field(&self, metadata: &Metadata<'_>) -> Option<Field>
```

Source: `src/field.rs:134`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

Attempts to convert `&self` into a `Field` with the specified `metadata`.

If `metadata` defines this field, then the field is returned. Otherwise,
this returns `None`.
