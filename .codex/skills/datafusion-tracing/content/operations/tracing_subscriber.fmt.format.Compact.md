# `tracing_subscriber::fmt::format::Compact`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.fmt.format.Compact.json).

<a id="op-b150ae7b416c1fdd163391ea"></a>
## Compact

`struct` · `tracing_subscriber::fmt::format::Compact` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
struct Compact
```

Source: `src/fmt/format/mod.rs:362`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Marker for [`Format`](../operations/tracing_subscriber.fmt.format.Format.md#op-758341dc152c0b1f07c0c605) that indicates that the compact log format should be used.

The compact format includes fields from all currently entered spans, after
the event's fields. Span fields are ordered (but not grouped) by
span, and span names are not shown. A more compact representation of the
event's [`Level`](../operations/tracing_core.metadata.Level.md#op-f8804717be954252aadd54ff) is used, and additional information—such as the event's
target—is disabled by default.

# Example Output

<pre><font color="#4E9A06"><b>:;</b></font> <font color="#4E9A06">cargo</font> run --example fmt-compact
<font color="#4E9A06"><b>    Finished</b></font> dev [unoptimized + debuginfo] target(s) in 0.08s
<font color="#4E9A06"><b>     Running</b></font> `target/debug/examples/fmt-compact`
<font color="#AAAAAA">2022-02-17T19:51:05.809287Z </font><font color="#4E9A06"> INFO</font> <b>fmt_compact</b><font color="#AAAAAA">: preparing to shave yaks </font><i>number_of_yaks</i><font color="#AAAAAA">=3</font>
<font color="#AAAAAA">2022-02-17T19:51:05.809367Z </font><font color="#4E9A06"> INFO</font> <b>shaving_yaks</b>: <b>fmt_compact::yak_shave</b><font color="#AAAAAA">: shaving yaks </font><font color="#AAAAAA"><i>yaks</i></font><font color="#AAAAAA">=3</font>
<font color="#AAAAAA">2022-02-17T19:51:05.809414Z </font><font color="#75507B">TRACE</font> <b>shaving_yaks</b>:<b>shave</b>: <b>fmt_compact::yak_shave</b><font color="#AAAAAA">: hello! I&apos;m gonna shave a yak </font><i>excitement</i><font color="#AAAAAA">=&quot;yay!&quot; </font><font color="#AAAAAA"><i>yaks</i></font><font color="#AAAAAA">=3 </font><font color="#AAAAAA"><i>yak</i></font><font color="#AAAAAA">=1</font>
<font color="#AAAAAA">2022-02-17T19:51:05.809443Z </font><font color="#75507B">TRACE</font> <b>shaving_yaks</b>:<b>shave</b>: <b>fmt_compact::yak_shave</b><font color="#AAAAAA">: yak shaved successfully </font><font color="#AAAAAA"><i>yaks</i></font><font color="#AAAAAA">=3 </font><font color="#AAAAAA"><i>yak</i></font><font color="#AAAAAA">=1</font>
<font color="#AAAAAA">2022-02-17T19:51:05.809477Z </font><font color="#3465A4">DEBUG</font> <b>shaving_yaks</b>: <b>yak_events</b><font color="#AAAAAA">: </font><i>yak</i><font color="#AAAAAA">=1 </font><i>shaved</i><font color="#AAAAAA">=true </font><font color="#AAAAAA"><i>yaks</i></font><font color="#AAAAAA">=3</font>
<font color="#AAAAAA">2022-02-17T19:51:05.809500Z </font><font color="#75507B">TRACE</font> <b>shaving_yaks</b>: <b>fmt_compact::yak_shave</b><font color="#AAAAAA">: </font><i>yaks_shaved</i><font color="#AAAAAA">=1 </font><font color="#AAAAAA"><i>yaks</i></font><font color="#AAAAAA">=3</font>
<font color="#AAAAAA">2022-02-17T19:51:05.809531Z </font><font color="#75507B">TRACE</font> <b>shaving_yaks</b>:<b>shave</b>: <b>fmt_compact::yak_shave</b><font color="#AAAAAA">: hello! I&apos;m gonna shave a yak </font><i>excitement</i><font color="#AAAAAA">=&quot;yay!&quot; </font><font color="#AAAAAA"><i>yaks</i></font><font color="#AAAAAA">=3 </font><font color="#AAAAAA"><i>yak</i></font><font color="#AAAAAA">=2</font>
<font color="#AAAAAA">2022-02-17T19:51:05.809554Z </font><font color="#75507B">TRACE</font> <b>shaving_yaks</b>:<b>shave</b>: <b>fmt_compact::yak_shave</b><font color="#AAAAAA">: yak shaved successfully </font><font color="#AAAAAA"><i>yaks</i></font><font color="#AAAAAA">=3 </font><font color="#AAAAAA"><i>yak</i></font><font color="#AAAAAA">=2</font>
<font color="#AAAAAA">2022-02-17T19:51:05.809581Z </font><font color="#3465A4">DEBUG</font> <b>shaving_yaks</b>: <b>yak_events</b><font color="#AAAAAA">: </font><i>yak</i><font color="#AAAAAA">=2 </font><i>shaved</i><font color="#AAAAAA">=true </font><font color="#AAAAAA"><i>yaks</i></font><font color="#AAAAAA">=3</font>
<font color="#AAAAAA">2022-02-17T19:51:05.809606Z </font><font color="#75507B">TRACE</font> <b>shaving_yaks</b>: <b>fmt_compact::yak_shave</b><font color="#AAAAAA">: </font><i>yaks_shaved</i><font color="#AAAAAA">=2 </font><font color="#AAAAAA"><i>yaks</i></font><font color="#AAAAAA">=3</font>
<font color="#AAAAAA">2022-02-17T19:51:05.809635Z </font><font color="#75507B">TRACE</font> <b>shaving_yaks</b>:<b>shave</b>: <b>fmt_compact::yak_shave</b><font color="#AAAAAA">: hello! I&apos;m gonna shave a yak </font><i>excitement</i><font color="#AAAAAA">=&quot;yay!&quot; </font><font color="#AAAAAA"><i>yaks</i></font><font color="#AAAAAA">=3 </font><font color="#AAAAAA"><i>yak</i></font><font color="#AAAAAA">=3</font>
<font color="#AAAAAA">2022-02-17T19:51:05.809664Z </font><font color="#C4A000"> WARN</font> <b>shaving_yaks</b>:<b>shave</b>: <b>fmt_compact::yak_shave</b><font color="#AAAAAA">: could not locate yak </font><font color="#AAAAAA"><i>yaks</i></font><font color="#AAAAAA">=3 </font><font color="#AAAAAA"><i>yak</i></font><font color="#AAAAAA">=3</font>
<font color="#AAAAAA">2022-02-17T19:51:05.809693Z </font><font color="#3465A4">DEBUG</font> <b>shaving_yaks</b>: <b>yak_events</b><font color="#AAAAAA">: </font><i>yak</i><font color="#AAAAAA">=3 </font><i>shaved</i><font color="#AAAAAA">=false </font><font color="#AAAAAA"><i>yaks</i></font><font color="#AAAAAA">=3</font>
<font color="#AAAAAA">2022-02-17T19:51:05.809717Z </font><font color="#CC0000">ERROR</font> <b>shaving_yaks</b>: <b>fmt_compact::yak_shave</b><font color="#AAAAAA">: failed to shave yak </font><i>yak</i><font color="#AAAAAA">=3 </font><i>error</i><font color="#AAAAAA">=missing yak </font><i>error.sources</i><font color="#AAAAAA">=[out of space, out of cash] </font><font color="#AAAAAA"><i>yaks</i></font><font color="#AAAAAA">=3</font>
<font color="#AAAAAA">2022-02-17T19:51:05.809743Z </font><font color="#75507B">TRACE</font> <b>shaving_yaks</b>: <b>fmt_compact::yak_shave</b><font color="#AAAAAA">: </font><i>yaks_shaved</i><font color="#AAAAAA">=2 </font><font color="#AAAAAA"><i>yaks</i></font><font color="#AAAAAA">=3</font>
<font color="#AAAAAA">2022-02-17T19:51:05.809768Z </font><font color="#4E9A06"> INFO</font> <b>fmt_compact</b><font color="#AAAAAA">: yak shaving completed </font><i>all_yaks_shaved</i><font color="#AAAAAA">=false</font>

</pre>

<a id="op-bddcdd0b68bbcd9e9a98295d"></a>
## clone

`function` · `tracing_subscriber::fmt::format::Compact::clone` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> Compact
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::Compact", "path": "Compact"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [361, 32], "end": [361, 37], "filename": "src/fmt/format/mod.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/fmt/format/mod.rs:361`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-7dc08f4c641f89ff79b889b6"></a>
## default

`function` · `tracing_subscriber::fmt::format::Compact::default` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> Compact
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::Compact", "path": "Compact"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [361, 10], "end": [361, 17], "filename": "src/fmt/format/mod.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/fmt/format/mod.rs:361`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-c932a1a1611d53643fbb958f"></a>
## eq

`function` · `tracing_subscriber::fmt::format::Compact::eq` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn eq(&self, other: &Compact) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::Compact", "path": "Compact"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [361, 43], "end": [361, 52], "filename": "src/fmt/format/mod.rs"}, "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "PartialEq"}, "trait_path": "core::cmp::PartialEq"}`

Source: `src/fmt/format/mod.rs:361`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-6a9d1652ff79015cbf9478e7"></a>
## fmt

`function` · `tracing_subscriber::fmt::format::Compact::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::Compact", "path": "Compact"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [361, 19], "end": [361, 24], "filename": "src/fmt/format/mod.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/fmt/format/mod.rs:361`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.
