# `tracing_subscriber::fmt::format::pretty::Pretty`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.fmt.format.pretty.Pretty.json).

<a id="op-9ae7afec101d61be119e29f4"></a>
## Pretty

`struct` · `tracing_subscriber::fmt::format::pretty::Pretty` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
struct Pretty
```

Source: `src/fmt/format/pretty.rs:99`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

An excessively pretty, human-readable event formatter.

Unlike the [`Full`](../operations/tracing_subscriber.fmt.format.Full.md#op-b6feb895c3a5317d8f8397fd), [`Compact`](../operations/tracing_subscriber.fmt.format.Compact.md#op-b150ae7b416c1fdd163391ea), and [`Json`](../operations/tracing_subscriber.fmt.format.json.Json.md#op-8e1265e43c2726c9a4be086c) formatters, this is a
multi-line output format. Each individual event may output multiple lines of
text.

# Example Output

<pre><font color="#4E9A06"><b>:;</b></font> <font color="#4E9A06">cargo</font> run --example fmt-pretty
<font color="#4E9A06"><b>    Finished</b></font> dev [unoptimized + debuginfo] target(s) in 0.08s
<font color="#4E9A06"><b>     Running</b></font> `target/debug/examples/fmt-pretty`
  2022-02-15T18:44:24.535324Z <font color="#4E9A06"> INFO</font> <font color="#4E9A06"><b>fmt_pretty</b></font><font color="#4E9A06">: preparing to shave yaks, </font><font color="#4E9A06"><b>number_of_yaks</b></font><font color="#4E9A06">: 3</font>
    <font color="#AAAAAA"><i>at</i></font> examples/examples/fmt-pretty.rs:16 <font color="#AAAAAA"><i>on</i></font> main

  2022-02-15T18:44:24.535403Z <font color="#4E9A06"> INFO</font> <font color="#4E9A06"><b>fmt_pretty::yak_shave</b></font><font color="#4E9A06">: shaving yaks</font>
    <font color="#AAAAAA"><i>at</i></font> examples/examples/fmt/yak_shave.rs:41 <font color="#AAAAAA"><i>on</i></font> main
    <font color="#AAAAAA"><i>in</i></font> fmt_pretty::yak_shave::<b>shaving_yaks</b> <font color="#AAAAAA"><i>with</i></font> <b>yaks</b>: 3

  2022-02-15T18:44:24.535442Z <font color="#75507B">TRACE</font> <font color="#75507B"><b>fmt_pretty::yak_shave</b></font><font color="#75507B">: hello! I&apos;m gonna shave a yak, </font><font color="#75507B"><b>excitement</b></font><font color="#75507B">: &quot;yay!&quot;</font>
    <font color="#AAAAAA"><i>at</i></font> examples/examples/fmt/yak_shave.rs:16 <font color="#AAAAAA"><i>on</i></font> main
    <font color="#AAAAAA"><i>in</i></font> fmt_pretty::yak_shave::<b>shave</b> <font color="#AAAAAA"><i>with</i></font> <b>yak</b>: 1
    <font color="#AAAAAA"><i>in</i></font> fmt_pretty::yak_shave::<b>shaving_yaks</b> <font color="#AAAAAA"><i>with</i></font> <b>yaks</b>: 3

  2022-02-15T18:44:24.535469Z <font color="#75507B">TRACE</font> <font color="#75507B"><b>fmt_pretty::yak_shave</b></font><font color="#75507B">: yak shaved successfully</font>
    <font color="#AAAAAA"><i>at</i></font> examples/examples/fmt/yak_shave.rs:25 <font color="#AAAAAA"><i>on</i></font> main
    <font color="#AAAAAA"><i>in</i></font> fmt_pretty::yak_shave::<b>shave</b> <font color="#AAAAAA"><i>with</i></font> <b>yak</b>: 1
    <font color="#AAAAAA"><i>in</i></font> fmt_pretty::yak_shave::<b>shaving_yaks</b> <font color="#AAAAAA"><i>with</i></font> <b>yaks</b>: 3

  2022-02-15T18:44:24.535502Z <font color="#3465A4">DEBUG</font> <font color="#3465A4"><b>yak_events</b></font><font color="#3465A4">: </font><font color="#3465A4"><b>yak</b></font><font color="#3465A4">: 1, </font><font color="#3465A4"><b>shaved</b></font><font color="#3465A4">: true</font>
    <font color="#AAAAAA"><i>at</i></font> examples/examples/fmt/yak_shave.rs:46 <font color="#AAAAAA"><i>on</i></font> main
    <font color="#AAAAAA"><i>in</i></font> fmt_pretty::yak_shave::<b>shaving_yaks</b> <font color="#AAAAAA"><i>with</i></font> <b>yaks</b>: 3

  2022-02-15T18:44:24.535524Z <font color="#75507B">TRACE</font> <font color="#75507B"><b>fmt_pretty::yak_shave</b></font><font color="#75507B">: </font><font color="#75507B"><b>yaks_shaved</b></font><font color="#75507B">: 1</font>
    <font color="#AAAAAA"><i>at</i></font> examples/examples/fmt/yak_shave.rs:55 <font color="#AAAAAA"><i>on</i></font> main
    <font color="#AAAAAA"><i>in</i></font> fmt_pretty::yak_shave::<b>shaving_yaks</b> <font color="#AAAAAA"><i>with</i></font> <b>yaks</b>: 3

  2022-02-15T18:44:24.535551Z <font color="#75507B">TRACE</font> <font color="#75507B"><b>fmt_pretty::yak_shave</b></font><font color="#75507B">: hello! I&apos;m gonna shave a yak, </font><font color="#75507B"><b>excitement</b></font><font color="#75507B">: &quot;yay!&quot;</font>
    <font color="#AAAAAA"><i>at</i></font> examples/examples/fmt/yak_shave.rs:16 <font color="#AAAAAA"><i>on</i></font> main
    <font color="#AAAAAA"><i>in</i></font> fmt_pretty::yak_shave::<b>shave</b> <font color="#AAAAAA"><i>with</i></font> <b>yak</b>: 2
    <font color="#AAAAAA"><i>in</i></font> fmt_pretty::yak_shave::<b>shaving_yaks</b> <font color="#AAAAAA"><i>with</i></font> <b>yaks</b>: 3

  2022-02-15T18:44:24.535573Z <font color="#75507B">TRACE</font> <font color="#75507B"><b>fmt_pretty::yak_shave</b></font><font color="#75507B">: yak shaved successfully</font>
    <font color="#AAAAAA"><i>at</i></font> examples/examples/fmt/yak_shave.rs:25 <font color="#AAAAAA"><i>on</i></font> main
    <font color="#AAAAAA"><i>in</i></font> fmt_pretty::yak_shave::<b>shave</b> <font color="#AAAAAA"><i>with</i></font> <b>yak</b>: 2
    <font color="#AAAAAA"><i>in</i></font> fmt_pretty::yak_shave::<b>shaving_yaks</b> <font color="#AAAAAA"><i>with</i></font> <b>yaks</b>: 3

  2022-02-15T18:44:24.535600Z <font color="#3465A4">DEBUG</font> <font color="#3465A4"><b>yak_events</b></font><font color="#3465A4">: </font><font color="#3465A4"><b>yak</b></font><font color="#3465A4">: 2, </font><font color="#3465A4"><b>shaved</b></font><font color="#3465A4">: true</font>
    <font color="#AAAAAA"><i>at</i></font> examples/examples/fmt/yak_shave.rs:46 <font color="#AAAAAA"><i>on</i></font> main
    <font color="#AAAAAA"><i>in</i></font> fmt_pretty::yak_shave::<b>shaving_yaks</b> <font color="#AAAAAA"><i>with</i></font> <b>yaks</b>: 3

  2022-02-15T18:44:24.535618Z <font color="#75507B">TRACE</font> <font color="#75507B"><b>fmt_pretty::yak_shave</b></font><font color="#75507B">: </font><font color="#75507B"><b>yaks_shaved</b></font><font color="#75507B">: 2</font>
    <font color="#AAAAAA"><i>at</i></font> examples/examples/fmt/yak_shave.rs:55 <font color="#AAAAAA"><i>on</i></font> main
    <font color="#AAAAAA"><i>in</i></font> fmt_pretty::yak_shave::<b>shaving_yaks</b> <font color="#AAAAAA"><i>with</i></font> <b>yaks</b>: 3

  2022-02-15T18:44:24.535644Z <font color="#75507B">TRACE</font> <font color="#75507B"><b>fmt_pretty::yak_shave</b></font><font color="#75507B">: hello! I&apos;m gonna shave a yak, </font><font color="#75507B"><b>excitement</b></font><font color="#75507B">: &quot;yay!&quot;</font>
    <font color="#AAAAAA"><i>at</i></font> examples/examples/fmt/yak_shave.rs:16 <font color="#AAAAAA"><i>on</i></font> main
    <font color="#AAAAAA"><i>in</i></font> fmt_pretty::yak_shave::<b>shave</b> <font color="#AAAAAA"><i>with</i></font> <b>yak</b>: 3
    <font color="#AAAAAA"><i>in</i></font> fmt_pretty::yak_shave::<b>shaving_yaks</b> <font color="#AAAAAA"><i>with</i></font> <b>yaks</b>: 3

  2022-02-15T18:44:24.535670Z <font color="#C4A000"> WARN</font> <font color="#C4A000"><b>fmt_pretty::yak_shave</b></font><font color="#C4A000">: could not locate yak</font>
    <font color="#AAAAAA"><i>at</i></font> examples/examples/fmt/yak_shave.rs:18 <font color="#AAAAAA"><i>on</i></font> main
    <font color="#AAAAAA"><i>in</i></font> fmt_pretty::yak_shave::<b>shave</b> <font color="#AAAAAA"><i>with</i></font> <b>yak</b>: 3
    <font color="#AAAAAA"><i>in</i></font> fmt_pretty::yak_shave::<b>shaving_yaks</b> <font color="#AAAAAA"><i>with</i></font> <b>yaks</b>: 3

  2022-02-15T18:44:24.535698Z <font color="#3465A4">DEBUG</font> <font color="#3465A4"><b>yak_events</b></font><font color="#3465A4">: </font><font color="#3465A4"><b>yak</b></font><font color="#3465A4">: 3, </font><font color="#3465A4"><b>shaved</b></font><font color="#3465A4">: false</font>
    <font color="#AAAAAA"><i>at</i></font> examples/examples/fmt/yak_shave.rs:46 <font color="#AAAAAA"><i>on</i></font> main
    <font color="#AAAAAA"><i>in</i></font> fmt_pretty::yak_shave::<b>shaving_yaks</b> <font color="#AAAAAA"><i>with</i></font> <b>yaks</b>: 3

  2022-02-15T18:44:24.535720Z <font color="#CC0000">ERROR</font> <font color="#CC0000"><b>fmt_pretty::yak_shave</b></font><font color="#CC0000">: failed to shave yak, </font><font color="#CC0000"><b>yak</b></font><font color="#CC0000">: 3, </font><font color="#CC0000"><b>error</b></font><font color="#CC0000">: missing yak, </font><font color="#CC0000"><b>error.sources</b></font><font color="#CC0000">: [out of space, out of cash]</font>
    <font color="#AAAAAA"><i>at</i></font> examples/examples/fmt/yak_shave.rs:51 <font color="#AAAAAA"><i>on</i></font> main
    <font color="#AAAAAA"><i>in</i></font> fmt_pretty::yak_shave::<b>shaving_yaks</b> <font color="#AAAAAA"><i>with</i></font> <b>yaks</b>: 3

  2022-02-15T18:44:24.535742Z <font color="#75507B">TRACE</font> <font color="#75507B"><b>fmt_pretty::yak_shave</b></font><font color="#75507B">: </font><font color="#75507B"><b>yaks_shaved</b></font><font color="#75507B">: 2</font>
    <font color="#AAAAAA"><i>at</i></font> examples/examples/fmt/yak_shave.rs:55 <font color="#AAAAAA"><i>on</i></font> main
    <font color="#AAAAAA"><i>in</i></font> fmt_pretty::yak_shave::<b>shaving_yaks</b> <font color="#AAAAAA"><i>with</i></font> <b>yaks</b>: 3

  2022-02-15T18:44:24.535765Z <font color="#4E9A06"> INFO</font> <font color="#4E9A06"><b>fmt_pretty</b></font><font color="#4E9A06">: yak shaving completed, </font><font color="#4E9A06"><b>all_yaks_shaved</b></font><font color="#4E9A06">: false</font>
    <font color="#AAAAAA"><i>at</i></font> examples/examples/fmt-pretty.rs:19 <font color="#AAAAAA"><i>on</i></font> main
</pre>

<a id="op-ba34c232d31dafb639502850"></a>
## add_fields

`function` · `tracing_subscriber::fmt::format::pretty::Pretty::add_fields` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn add_fields(&self, current: &'writer mut FormattedFields<Self>, fields: &span::Record<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::pretty::Pretty", "path": "Pretty"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [337, 1], "end": [355, 2], "filename": "src/fmt/format/pretty.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatFields", "path": "FormatFields"}, "trait_path": "tracing_subscriber::fmt::format::FormatFields"}`

Source: `src/fmt/format/pretty.rs:344`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-e926ec44619a1fddeb0f7294"></a>
## clone

`function` · `tracing_subscriber::fmt::format::pretty::Pretty::clone` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> Pretty
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::pretty::Pretty", "path": "Pretty"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [98, 17], "end": [98, 22], "filename": "src/fmt/format/pretty.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/fmt/format/pretty.rs:98`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-09bf5526d43e3547b087a2f0"></a>
## default

`function` · `tracing_subscriber::fmt::format::pretty::Pretty::default` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::pretty::Pretty", "path": "Pretty"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [135, 1], "end": [141, 2], "filename": "src/fmt/format/pretty.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/fmt/format/pretty.rs:136`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-de08132849fc650a0d5619f1"></a>
## eq

`function` · `tracing_subscriber::fmt::format::pretty::Pretty::eq` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn eq(&self, other: &Pretty) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::pretty::Pretty", "path": "Pretty"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [98, 28], "end": [98, 37], "filename": "src/fmt/format/pretty.rs"}, "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "PartialEq"}, "trait_path": "core::cmp::PartialEq"}`

Source: `src/fmt/format/pretty.rs:98`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-511025ceba6b75e9264da4dd"></a>
## fmt

`function` · `tracing_subscriber::fmt::format::pretty::Pretty::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::pretty::Pretty", "path": "Pretty"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [98, 10], "end": [98, 15], "filename": "src/fmt/format/pretty.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/fmt/format/pretty.rs:98`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-f2c11560284fcc89a996036e"></a>
## format_fields

`function` · `tracing_subscriber::fmt::format::pretty::Pretty::format_fields` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn format_fields<R: RecordFields>(&self, writer: Writer<'writer>, fields: R) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::pretty::Pretty", "path": "Pretty"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [337, 1], "end": [355, 2], "filename": "src/fmt/format/pretty.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatFields", "path": "FormatFields"}, "trait_path": "tracing_subscriber::fmt::format::FormatFields"}`

Source: `src/fmt/format/pretty.rs:338`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-a7ec1c58bc260ec59c28427f"></a>
## with_source_location

`function` · `tracing_subscriber::fmt::format::pretty::Pretty::with_source_location` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn with_source_location(self, display_location: bool) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::pretty::Pretty", "path": "Pretty"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [143, 1], "end": [167, 2], "filename": "src/fmt/format/pretty.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/format/pretty.rs:161`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Sets whether the event's source code location is displayed.

This defaults to `true`.
