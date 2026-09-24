# `tracing_subscriber::fmt::format::Full`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.fmt.format.Full.json).

<a id="op-b6feb895c3a5317d8f8397fd"></a>
## Full

`struct` · `tracing_subscriber::fmt::format::Full` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
struct Full
```

Source: `src/fmt/format/mod.rs:392`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Marker for [`Format`](../operations/tracing_subscriber.fmt.format.Format.md#op-758341dc152c0b1f07c0c605) that indicates that the default log format should be used.

This formatter shows the span context before printing event data. Spans are
displayed including their names and fields.

# Example Output

<pre><font color="#4E9A06"><b>:;</b></font> <font color="#4E9A06">cargo</font> run --example fmt
<font color="#4E9A06"><b>    Finished</b></font> dev [unoptimized + debuginfo] target(s) in 0.08s
<font color="#4E9A06"><b>     Running</b></font> `target/debug/examples/fmt`
<font color="#AAAAAA">2022-02-15T18:40:14.289898Z </font><font color="#4E9A06"> INFO</font> fmt: preparing to shave yaks <i>number_of_yaks</i><font color="#AAAAAA">=3</font>
<font color="#AAAAAA">2022-02-15T18:40:14.289974Z </font><font color="#4E9A06"> INFO</font> <b>shaving_yaks{</b><i>yaks</i><font color="#AAAAAA">=3</font><b>}</b><font color="#AAAAAA">: fmt::yak_shave: shaving yaks</font>
<font color="#AAAAAA">2022-02-15T18:40:14.290011Z </font><font color="#75507B">TRACE</font> <b>shaving_yaks{</b><i>yaks</i><font color="#AAAAAA">=3</font><b>}</b><font color="#AAAAAA">:</font><b>shave{</b><i>yak</i><font color="#AAAAAA">=1</font><b>}</b><font color="#AAAAAA">: fmt::yak_shave: hello! I&apos;m gonna shave a yak </font><i>excitement</i><font color="#AAAAAA">=&quot;yay!&quot;</font>
<font color="#AAAAAA">2022-02-15T18:40:14.290038Z </font><font color="#75507B">TRACE</font> <b>shaving_yaks{</b><i>yaks</i><font color="#AAAAAA">=3</font><b>}</b><font color="#AAAAAA">:</font><b>shave{</b><i>yak</i><font color="#AAAAAA">=1</font><b>}</b><font color="#AAAAAA">: fmt::yak_shave: yak shaved successfully</font>
<font color="#AAAAAA">2022-02-15T18:40:14.290070Z </font><font color="#3465A4">DEBUG</font> <b>shaving_yaks{</b><i>yaks</i><font color="#AAAAAA">=3</font><b>}</b><font color="#AAAAAA">: yak_events: </font><i>yak</i><font color="#AAAAAA">=1 </font><i>shaved</i><font color="#AAAAAA">=true</font>
<font color="#AAAAAA">2022-02-15T18:40:14.290089Z </font><font color="#75507B">TRACE</font> <b>shaving_yaks{</b><i>yaks</i><font color="#AAAAAA">=3</font><b>}</b><font color="#AAAAAA">: fmt::yak_shave: </font><i>yaks_shaved</i><font color="#AAAAAA">=1</font>
<font color="#AAAAAA">2022-02-15T18:40:14.290114Z </font><font color="#75507B">TRACE</font> <b>shaving_yaks{</b><i>yaks</i><font color="#AAAAAA">=3</font><b>}</b><font color="#AAAAAA">:</font><b>shave{</b><i>yak</i><font color="#AAAAAA">=2</font><b>}</b><font color="#AAAAAA">: fmt::yak_shave: hello! I&apos;m gonna shave a yak </font><i>excitement</i><font color="#AAAAAA">=&quot;yay!&quot;</font>
<font color="#AAAAAA">2022-02-15T18:40:14.290134Z </font><font color="#75507B">TRACE</font> <b>shaving_yaks{</b><i>yaks</i><font color="#AAAAAA">=3</font><b>}</b><font color="#AAAAAA">:</font><b>shave{</b><i>yak</i><font color="#AAAAAA">=2</font><b>}</b><font color="#AAAAAA">: fmt::yak_shave: yak shaved successfully</font>
<font color="#AAAAAA">2022-02-15T18:40:14.290157Z </font><font color="#3465A4">DEBUG</font> <b>shaving_yaks{</b><i>yaks</i><font color="#AAAAAA">=3</font><b>}</b><font color="#AAAAAA">: yak_events: </font><i>yak</i><font color="#AAAAAA">=2 </font><i>shaved</i><font color="#AAAAAA">=true</font>
<font color="#AAAAAA">2022-02-15T18:40:14.290174Z </font><font color="#75507B">TRACE</font> <b>shaving_yaks{</b><i>yaks</i><font color="#AAAAAA">=3</font><b>}</b><font color="#AAAAAA">: fmt::yak_shave: </font><i>yaks_shaved</i><font color="#AAAAAA">=2</font>
<font color="#AAAAAA">2022-02-15T18:40:14.290198Z </font><font color="#75507B">TRACE</font> <b>shaving_yaks{</b><i>yaks</i><font color="#AAAAAA">=3</font><b>}</b><font color="#AAAAAA">:</font><b>shave{</b><i>yak</i><font color="#AAAAAA">=3</font><b>}</b><font color="#AAAAAA">: fmt::yak_shave: hello! I&apos;m gonna shave a yak </font><i>excitement</i><font color="#AAAAAA">=&quot;yay!&quot;</font>
<font color="#AAAAAA">2022-02-15T18:40:14.290222Z </font><font color="#C4A000"> WARN</font> <b>shaving_yaks{</b><i>yaks</i><font color="#AAAAAA">=3</font><b>}</b><font color="#AAAAAA">:</font><b>shave{</b><i>yak</i><font color="#AAAAAA">=3</font><b>}</b><font color="#AAAAAA">: fmt::yak_shave: could not locate yak</font>
<font color="#AAAAAA">2022-02-15T18:40:14.290247Z </font><font color="#3465A4">DEBUG</font> <b>shaving_yaks{</b><i>yaks</i><font color="#AAAAAA">=3</font><b>}</b><font color="#AAAAAA">: yak_events: </font><i>yak</i><font color="#AAAAAA">=3 </font><i>shaved</i><font color="#AAAAAA">=false</font>
<font color="#AAAAAA">2022-02-15T18:40:14.290268Z </font><font color="#CC0000">ERROR</font> <b>shaving_yaks{</b><i>yaks</i><font color="#AAAAAA">=3</font><b>}</b><font color="#AAAAAA">: fmt::yak_shave: failed to shave yak </font><i>yak</i><font color="#AAAAAA">=3 </font><i>error</i><font color="#AAAAAA">=missing yak </font><i>error.sources</i><font color="#AAAAAA">=[out of space, out of cash]</font>
<font color="#AAAAAA">2022-02-15T18:40:14.290287Z </font><font color="#75507B">TRACE</font> <b>shaving_yaks{</b><i>yaks</i><font color="#AAAAAA">=3</font><b>}</b><font color="#AAAAAA">: fmt::yak_shave: </font><i>yaks_shaved</i><font color="#AAAAAA">=2</font>
<font color="#AAAAAA">2022-02-15T18:40:14.290309Z </font><font color="#4E9A06"> INFO</font> fmt: yak shaving completed. <i>all_yaks_shaved</i><font color="#AAAAAA">=false</font>
</pre>

<a id="op-b172f4ed551437bd06f4194f"></a>
## clone

`function` · `tracing_subscriber::fmt::format::Full::clone` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> Full
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::Full", "path": "Full"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [391, 32], "end": [391, 37], "filename": "src/fmt/format/mod.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/fmt/format/mod.rs:391`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-f61b3c01e513a2fc5b62abb5"></a>
## default

`function` · `tracing_subscriber::fmt::format::Full::default` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> Full
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::Full", "path": "Full"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [391, 10], "end": [391, 17], "filename": "src/fmt/format/mod.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/fmt/format/mod.rs:391`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-68c22d4fa7e27ab75d16b843"></a>
## eq

`function` · `tracing_subscriber::fmt::format::Full::eq` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn eq(&self, other: &Full) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::Full", "path": "Full"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [391, 43], "end": [391, 52], "filename": "src/fmt/format/mod.rs"}, "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "PartialEq"}, "trait_path": "core::cmp::PartialEq"}`

Source: `src/fmt/format/mod.rs:391`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-026e96d65e626ffc9270588b"></a>
## fmt

`function` · `tracing_subscriber::fmt::format::Full::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::Full", "path": "Full"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [391, 19], "end": [391, 24], "filename": "src/fmt/format/mod.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/fmt/format/mod.rs:391`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.
