# `tracing_subscriber::fmt::format::json::Json`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.fmt.format.json.Json.json).

<a id="op-8e1265e43c2726c9a4be086c"></a>
## Json

`struct` · `tracing_subscriber::fmt::format::json::Json` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
struct Json
```

Source: `src/fmt/format/json.rs:92`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Marker for [`Format`](../operations/tracing_subscriber.fmt.format.Format.md#op-758341dc152c0b1f07c0c605) that indicates that the newline-delimited JSON log
format should be used.

This formatter is intended for production use with systems where structured
logs are consumed as JSON by analysis and viewing tools. The JSON output is
not optimized for human readability; instead, it should be pretty-printed
using external JSON tools such as `jq`, or using a JSON log viewer.

# Example Output

<pre><font color="#4E9A06"><b>:;</b></font> <font color="#4E9A06">cargo</font> run --example fmt-json
<font color="#4E9A06"><b>    Finished</b></font> dev [unoptimized + debuginfo] target(s) in 0.08s
<font color="#4E9A06"><b>     Running</b></font> `target/debug/examples/fmt-json`
{&quot;timestamp&quot;:&quot;2022-02-15T18:47:10.821315Z&quot;,&quot;level&quot;:&quot;INFO&quot;,&quot;fields&quot;:{&quot;message&quot;:&quot;preparing to shave yaks&quot;,&quot;number_of_yaks&quot;:3},&quot;target&quot;:&quot;fmt_json&quot;}
{&quot;timestamp&quot;:&quot;2022-02-15T18:47:10.821422Z&quot;,&quot;level&quot;:&quot;INFO&quot;,&quot;fields&quot;:{&quot;message&quot;:&quot;shaving yaks&quot;},&quot;target&quot;:&quot;fmt_json::yak_shave&quot;,&quot;spans&quot;:[{&quot;yaks&quot;:3,&quot;name&quot;:&quot;shaving_yaks&quot;}]}
{&quot;timestamp&quot;:&quot;2022-02-15T18:47:10.821495Z&quot;,&quot;level&quot;:&quot;TRACE&quot;,&quot;fields&quot;:{&quot;message&quot;:&quot;hello! I&apos;m gonna shave a yak&quot;,&quot;excitement&quot;:&quot;yay!&quot;},&quot;target&quot;:&quot;fmt_json::yak_shave&quot;,&quot;spans&quot;:[{&quot;yaks&quot;:3,&quot;name&quot;:&quot;shaving_yaks&quot;},{&quot;yak&quot;:1,&quot;name&quot;:&quot;shave&quot;}]}
{&quot;timestamp&quot;:&quot;2022-02-15T18:47:10.821546Z&quot;,&quot;level&quot;:&quot;TRACE&quot;,&quot;fields&quot;:{&quot;message&quot;:&quot;yak shaved successfully&quot;},&quot;target&quot;:&quot;fmt_json::yak_shave&quot;,&quot;spans&quot;:[{&quot;yaks&quot;:3,&quot;name&quot;:&quot;shaving_yaks&quot;},{&quot;yak&quot;:1,&quot;name&quot;:&quot;shave&quot;}]}
{&quot;timestamp&quot;:&quot;2022-02-15T18:47:10.821598Z&quot;,&quot;level&quot;:&quot;DEBUG&quot;,&quot;fields&quot;:{&quot;yak&quot;:1,&quot;shaved&quot;:true},&quot;target&quot;:&quot;yak_events&quot;,&quot;spans&quot;:[{&quot;yaks&quot;:3,&quot;name&quot;:&quot;shaving_yaks&quot;}]}
{&quot;timestamp&quot;:&quot;2022-02-15T18:47:10.821637Z&quot;,&quot;level&quot;:&quot;TRACE&quot;,&quot;fields&quot;:{&quot;yaks_shaved&quot;:1},&quot;target&quot;:&quot;fmt_json::yak_shave&quot;,&quot;spans&quot;:[{&quot;yaks&quot;:3,&quot;name&quot;:&quot;shaving_yaks&quot;}]}
{&quot;timestamp&quot;:&quot;2022-02-15T18:47:10.821684Z&quot;,&quot;level&quot;:&quot;TRACE&quot;,&quot;fields&quot;:{&quot;message&quot;:&quot;hello! I&apos;m gonna shave a yak&quot;,&quot;excitement&quot;:&quot;yay!&quot;},&quot;target&quot;:&quot;fmt_json::yak_shave&quot;,&quot;spans&quot;:[{&quot;yaks&quot;:3,&quot;name&quot;:&quot;shaving_yaks&quot;},{&quot;yak&quot;:2,&quot;name&quot;:&quot;shave&quot;}]}
{&quot;timestamp&quot;:&quot;2022-02-15T18:47:10.821727Z&quot;,&quot;level&quot;:&quot;TRACE&quot;,&quot;fields&quot;:{&quot;message&quot;:&quot;yak shaved successfully&quot;},&quot;target&quot;:&quot;fmt_json::yak_shave&quot;,&quot;spans&quot;:[{&quot;yaks&quot;:3,&quot;name&quot;:&quot;shaving_yaks&quot;},{&quot;yak&quot;:2,&quot;name&quot;:&quot;shave&quot;}]}
{&quot;timestamp&quot;:&quot;2022-02-15T18:47:10.821773Z&quot;,&quot;level&quot;:&quot;DEBUG&quot;,&quot;fields&quot;:{&quot;yak&quot;:2,&quot;shaved&quot;:true},&quot;target&quot;:&quot;yak_events&quot;,&quot;spans&quot;:[{&quot;yaks&quot;:3,&quot;name&quot;:&quot;shaving_yaks&quot;}]}
{&quot;timestamp&quot;:&quot;2022-02-15T18:47:10.821806Z&quot;,&quot;level&quot;:&quot;TRACE&quot;,&quot;fields&quot;:{&quot;yaks_shaved&quot;:2},&quot;target&quot;:&quot;fmt_json::yak_shave&quot;,&quot;spans&quot;:[{&quot;yaks&quot;:3,&quot;name&quot;:&quot;shaving_yaks&quot;}]}
{&quot;timestamp&quot;:&quot;2022-02-15T18:47:10.821909Z&quot;,&quot;level&quot;:&quot;TRACE&quot;,&quot;fields&quot;:{&quot;message&quot;:&quot;hello! I&apos;m gonna shave a yak&quot;,&quot;excitement&quot;:&quot;yay!&quot;},&quot;target&quot;:&quot;fmt_json::yak_shave&quot;,&quot;spans&quot;:[{&quot;yaks&quot;:3,&quot;name&quot;:&quot;shaving_yaks&quot;},{&quot;yak&quot;:3,&quot;name&quot;:&quot;shave&quot;}]}
{&quot;timestamp&quot;:&quot;2022-02-15T18:47:10.821956Z&quot;,&quot;level&quot;:&quot;WARN&quot;,&quot;fields&quot;:{&quot;message&quot;:&quot;could not locate yak&quot;},&quot;target&quot;:&quot;fmt_json::yak_shave&quot;,&quot;spans&quot;:[{&quot;yaks&quot;:3,&quot;name&quot;:&quot;shaving_yaks&quot;},{&quot;yak&quot;:3,&quot;name&quot;:&quot;shave&quot;}]}
{&quot;timestamp&quot;:&quot;2022-02-15T18:47:10.822006Z&quot;,&quot;level&quot;:&quot;DEBUG&quot;,&quot;fields&quot;:{&quot;yak&quot;:3,&quot;shaved&quot;:false},&quot;target&quot;:&quot;yak_events&quot;,&quot;spans&quot;:[{&quot;yaks&quot;:3,&quot;name&quot;:&quot;shaving_yaks&quot;}]}
{&quot;timestamp&quot;:&quot;2022-02-15T18:47:10.822041Z&quot;,&quot;level&quot;:&quot;ERROR&quot;,&quot;fields&quot;:{&quot;message&quot;:&quot;failed to shave yak&quot;,&quot;yak&quot;:3,&quot;error&quot;:&quot;missing yak&quot;},&quot;target&quot;:&quot;fmt_json::yak_shave&quot;,&quot;spans&quot;:[{&quot;yaks&quot;:3,&quot;name&quot;:&quot;shaving_yaks&quot;}]}
{&quot;timestamp&quot;:&quot;2022-02-15T18:47:10.822079Z&quot;,&quot;level&quot;:&quot;TRACE&quot;,&quot;fields&quot;:{&quot;yaks_shaved&quot;:2},&quot;target&quot;:&quot;fmt_json::yak_shave&quot;,&quot;spans&quot;:[{&quot;yaks&quot;:3,&quot;name&quot;:&quot;shaving_yaks&quot;}]}
{&quot;timestamp&quot;:&quot;2022-02-15T18:47:10.822117Z&quot;,&quot;level&quot;:&quot;INFO&quot;,&quot;fields&quot;:{&quot;message&quot;:&quot;yak shaving completed&quot;,&quot;all_yaks_shaved&quot;:false},&quot;target&quot;:&quot;fmt_json&quot;}
</pre>

# Options

This formatter exposes additional options to configure the structure of the
output JSON objects:

- [`Json::flatten_event`] can be used to enable flattening event fields into
  the root
- [`Json::with_current_span`] can be used to control logging of the current
  span
- [`Json::with_span_list`] can be used to control logging of the span list
  object.

By default, event fields are not flattened, and both current span and span
list are logged.

# Valuable Support

Experimental support is available for using the [`valuable`] crate to record
user-defined values as structured JSON. When the ["valuable" unstable
feature][unstable] is enabled, types implementing [`valuable::Valuable`] will
be recorded as structured JSON, rather than
using their [`std::fmt::Debug`] implementations.

**Note**: This is an experimental feature. [Unstable features][unstable]
must be enabled in order to use `valuable` support.

[`Json::flatten_event`]: Json::flatten_event()
[`Json::with_current_span`]: Json::with_current_span()
[`Json::with_span_list`]: Json::with_span_list()
[`valuable`]: https://crates.io/crates/valuable
[unstable]: crate#unstable-features
[`valuable::Valuable`]: https://docs.rs/valuable/latest/valuable/trait.Valuable.html

Unresolved upstream links (retained, not inferred): ``std::fmt::Debug``.

<a id="op-95a2f47eab7f2d1e03834574"></a>
## clone

`function` · `tracing_subscriber::fmt::format::json::Json::clone` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> Json
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::json::Json", "path": "Json"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [91, 23], "end": [91, 28], "filename": "src/fmt/format/json.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/fmt/format/json.rs:91`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-2cda448341e111a5281e60d1"></a>
## default

`function` · `tracing_subscriber::fmt::format::json::Json::default` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> Json
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::json::Json", "path": "Json"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [334, 1], "end": [342, 2], "filename": "src/fmt/format/json.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/fmt/format/json.rs:335`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-b12cdd58eda6ee8609b03b8b"></a>
## eq

`function` · `tracing_subscriber::fmt::format::json::Json::eq` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn eq(&self, other: &Json) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::json::Json", "path": "Json"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [91, 34], "end": [91, 43], "filename": "src/fmt/format/json.rs"}, "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "PartialEq"}, "trait_path": "core::cmp::PartialEq"}`

Source: `src/fmt/format/json.rs:91`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-e99e00e29fc4168b2986916c"></a>
## flatten_event

`function` · `tracing_subscriber::fmt::format::json::Json::flatten_event` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn flatten_event(&mut self, flatten_event: bool)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::json::Json", "path": "Json"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [98, 1], "end": [114, 2], "filename": "src/fmt/format/json.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/format/json.rs:100`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

If set to `true` event metadata will be flattened into the root object.

<a id="op-ac193c0b64a5c38c44fdf7ad"></a>
## fmt

`function` · `tracing_subscriber::fmt::format::json::Json::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::json::Json", "path": "Json"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [91, 10], "end": [91, 15], "filename": "src/fmt/format/json.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/fmt/format/json.rs:91`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-5cf2b414a189228c876463f5"></a>
## with_current_span

`function` · `tracing_subscriber::fmt::format::json::Json::with_current_span` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn with_current_span(&mut self, display_current_span: bool)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::json::Json", "path": "Json"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [98, 1], "end": [114, 2], "filename": "src/fmt/format/json.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/format/json.rs:105`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

If set to `false`, formatted events won't contain a field for the current span.

<a id="op-fdb52d1b2b845cbe10385888"></a>
## with_span_list

`function` · `tracing_subscriber::fmt::format::json::Json::with_span_list` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn with_span_list(&mut self, display_span_list: bool)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::json::Json", "path": "Json"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [98, 1], "end": [114, 2], "filename": "src/fmt/format/json.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/format/json.rs:111`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

If set to `false`, formatted events won't contain a list of all currently
entered spans. Spans are logged in a list from root to leaf.
