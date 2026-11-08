# indicatif — Rust progress reporting and terminal rendering

Technical capability reference | Version 0.18.6 | Researched October 29, 2026

Package: `indicatif` (the published crate corresponding to “indicateif”)

Repository: [https://github.com/console-rs/indicatif](https://github.com/console-rs/indicatif?utm_source=chatgpt.com)\
API reference: [https://docs.rs/indicatif/0.18.6/indicatif/](https://docs.rs/indicatif/0.18.6/indicatif/?utm_source=chatgpt.com)\
Crates.io: [https://crates.io/crates/indicatif](https://crates.io/crates/indicatif?utm_source=chatgpt.com)

## 1. Architectural orientation

`indicatif` is a Rust library for interactive command-line progress reporting. It provides progress bars, activity spinners, concurrent progress displays, configurable terminal rendering, progress-aware I/O and iterator adapters, throughput and completion-time estimation, and extensible formatting.

Version 0.18.6, published July 1, 2026, is the latest published stable release identified in the crate registry documentation.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs

+1



Its architectural significance extends beyond drawing progress bars: it separates progress state, work execution, presentation, and terminal output coordination. These capabilities can be composed without implementing a custom terminal-rendering loop.

### Core capability map

| Capability                               | Primary mechanisms                                                   |
| ---------------------------------------- | -------------------------------------------------------------------- |
| Track work and runtime status            | `ProgressBar`, `ProgressState`, `WeakProgressBar`                    |
| Coordinate concurrent jobs               | `MultiProgress`, shared progress handles, dynamic bar insertion      |
| Render status and statistics             | `ProgressStyle`, templates, custom `ProgressTracker`                 |
| Control terminal behavior                | `ProgressDrawTarget`, `TermLike`, refresh limits                     |
| Instrument existing operations           | `ProgressIterator`, `ProgressBarIter`, Rayon, I/O adapters           |
| Manage presentation lifecycle            | `ProgressFinish`, explicit finish/clear, temporary output suspension |
| Produce consistent human-readable values | Byte, duration, count, and throughput formatters                     |
| Integrate diagnostics and logging        | `indicatif-log-bridge`, `tracing-indicatif` (separate crates)        |

### Core model

Work execution

Threads · Iterators · Rayon · Streams · I/O

ProgressBar

Shared progress state · counters · timing · lifecycle

ProgressStyle

Templates · metrics · formatting

MultiProgress

Multiple bars · ordering · coordination

ProgressDrawTarget / TermLike

Refresh limiting · terminal detection · rendering destination

This is a conceptual decomposition rather than a strict execution pipeline: a standalone bar can render without `MultiProgress`, and styles are attached directly to bars.

The important ownership boundary is that indicatif reports progress; it does not schedule, execute, or cancel the underlying work. The application or an adapter updates progress state, while indicatif manages formatting and display.

## 2. Progress state, ownership, and timing

### 2.1 Shared, mutable progress state

`ProgressBar` is an `Arc`-backed handle to shared internal state. Cloning it produces another handle to the same bar, not an independent progress indicator. It implements `Send` and `Sync`, enabling multiple threads to update one logical progress display.

Three creation modes have distinct semantics:

| Constructor                  | Meaning                             | Typical use                                      |
| ---------------------------- | ----------------------------------- | ------------------------------------------------ |
| `ProgressBar::new(len)`      | Known total work                    | Files, records, bytes, iterations                |
| `ProgressBar::no_length()`   | Position can advance; total unknown | Discovery, streaming ingestion                   |
| `ProgressBar::new_spinner()` | Indeterminate activity              | Initialization, waiting, long-running operations |
| `ProgressBar::hidden()`      | State exists without rendering      | Noninteractive or selectively disabled reporting |

All progress amounts are represented as `u64`. Their units are application-defined: an increment can represent one item, one byte, one batch, or another quantity. Indicatif does not infer the work’s semantic meaning.

The important mutation APIs include `inc`, `dec`, `set_position`, `position`, `set_length`, `inc_length`, `dec_length`, `unset_length`, `set_message`, and `set_prefix`. Mutable totals mean a progress bar can represent dynamically discovered work rather than only a fixed iteration count.

`ProgressBar::update` takes a closure over mutable `ProgressState`. This provides a direct state-update mechanism when multiple related changes should be handled together instead of expressed as separate progress-bar calls.

`ProgressState` exposes `pos`, `len`, `fraction`, `elapsed`, `per_sec`, `eta`, `duration`, and `is_finished`, plus setters for position and length. Its type is `#[non_exhaustive]`, so downstream extensions should use its public methods rather than assume an exhaustive structure.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

+1



Sources:

[https://docs.rs/indicatif/0.18.6/indicatif/struct.ProgressBar.html](https://docs.rs/indicatif/0.18.6/indicatif/struct.ProgressBar.html?utm_source=chatgpt.com)

[https://docs.rs/indicatif/0.18.6/indicatif/struct.ProgressState.html](https://docs.rs/indicatif/0.18.6/indicatif/struct.ProgressState.html?utm_source=chatgpt.com)

### 2.2 Progress handles and lifetime separation

The separate `WeakProgressBar` type supports non-owning references to progress state. `ProgressBar::downgrade()` creates a weak handle, and `upgrade()` returns a live progress bar if one still exists.

This matters for supervisory components, callbacks, or background observers that should be able to update a bar without independently extending its lifetime.

It also clarifies the intended separation between a job’s actual lifetime and the lifetime of a presentation handle. The two are related by the application, not intrinsically coupled by indicatif.

[https://docs.rs/indicatif/0.18.6/indicatif/struct.WeakProgressBar.html](https://docs.rs/indicatif/0.18.6/indicatif/struct.WeakProgressBar.html?utm_source=chatgpt.com)

### 2.3 Time, throughput, and ETA

Indicatif maintains timing information as part of progress state:

| API               | Interpretation                                |
| ----------------- | --------------------------------------------- |
| `elapsed()`       | Time since the current progress timing origin |
| `per_sec()`       | Estimated progress steps per second           |
| `eta()`           | Estimated remaining duration                  |
| `duration()`      | Estimated total duration                      |
| `reset_eta()`     | Discard the current ETA estimation history    |
| `reset_elapsed()` | Restart elapsed time and ETA estimation       |
| `set_elapsed()`   | Explicitly establish elapsed time             |
| `reset()`         | Reset the progress bar state                  |

The estimator uses smoothed progress-rate information rather than simply dividing total progress by total elapsed time. Upstream source documents a time-weighted, double-smoothed exponential estimator designed to respond to changing rates while damping fluctuations.

Consequently, ETA is a presentation estimate, not a scheduling guarantee. Its meaning depends on whether steps have reasonably comparable costs and whether the remaining work is represented accurately. Large jumps, pauses, and changes in workload composition can justify `reset_eta()`.

A subtle architectural distinction: `per_sec()` means progress units per second. Displaying it as bytes/s is correct only if the application increments the bar in bytes.

Source: [https://docs.rs/indicatif/0.18.6/src/indicatif/state.rs.html](https://docs.rs/indicatif/0.18.6/src/indicatif/state.rs.html?utm_source=chatgpt.com)

### 2.4 Rendering cadence versus state updates

`tick()` requests a manual animation/update tick. `enable_steady_tick(Duration)` creates a background thread to tick regularly, particularly useful for spinners or jobs that update their counters infrequently. `disable_steady_tick()` stops this mechanism.

While steady ticking is enabled, explicit `tick()` calls have no effect. This is not a Tokio task or an event-loop scheduler; the implementation uses its own background thread.

State updates are deliberately decoupled from full terminal redraws. `force_draw()` can synchronize the visible display, while ordinary updates are subject to the draw target’s rate limiting. Changing style with `set_style()` also does not itself redraw the bar; a tick or subsequent redraw is needed.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

+1



Performance implication: high-frequency progress updates need not cause equally frequent terminal I/O. However, contention, message allocation, custom formatter work, and drawing coordination are not eliminated merely because rendering is throttled.

## 3. Completion, failure presentation, and lifecycle

Finishing is more than setting a numeric counter.

| Operation                   | Result                                                |
| --------------------------- | ----------------------------------------------------- |
| `finish()`                  | Marks finished and leaves a completed display         |
| `finish_with_message(msg)`  | Completes with a replacement message                  |
| `finish_and_clear()`        | Completes and removes the display                     |
| `abandon()`                 | Stops while preserving the current partial progress   |
| `abandon_with_message(msg)` | Preserves partial progress with a replacement message |
| `finish_using_style()`      | Applies the configured `ProgressFinish` policy        |

`ProgressFinish` supports `AndLeave`, `WithMessage`, `AndClear`, `Abandon`, and `AbandonWithMessage`. `ProgressBar::with_finish` sets the completion behavior applied when the bar or its adapter completes and is not already finished.

These distinctions matter for pipelines with success, failure, interruption, or cancellation outcomes. A progress adapter reaching its end does not independently prove that every downstream side effect succeeded. An application may need to finish explicitly after its result is known.

Likewise, reaching a numerical total and declaring a job successfully complete are separate concepts. The correct success/failure status belongs to the work-execution layer.

The display can also preserve final status messages when the progress is retained in a larger `MultiProgress` layout; lifetime and removal behavior are important there.

Sources:

[https://docs.rs/indicatif/0.18.6/indicatif/enum.ProgressFinish.html](https://docs.rs/indicatif/0.18.6/indicatif/enum.ProgressFinish.html?utm_source=chatgpt.com)

[https://docs.rs/indicatif/0.18.6/indicatif/struct.ProgressBar.html](https://docs.rs/indicatif/0.18.6/indicatif/struct.ProgressBar.html?utm_source=chatgpt.com)

## 4. Concurrent progress management with `MultiProgress`

`MultiProgress` coordinates terminal presentation of multiple progress bars, potentially updated by separate threads. It is the core mechanism for reporting several simultaneous jobs without having independent progress bars overwrite each other.

### 4.1 Dynamic layout

`MultiProgress` supports `add`, `insert`, `insert_from_back`, `insert_before`, `insert_after`, `remove`, and `clear`.

These operations are more capable than a fixed collection of bars: a program can introduce new jobs as discovered, retain a total bar while child jobs appear and disappear, and control how progress indicators are ordered.

`MultiProgressAlignment` supports top and bottom alignment. `set_move_cursor(true)` can reduce flickering by using cursor movement rather than line clearing, but the documentation explicitly warns against enabling this when the number of bars will change.

Default group rendering goes to stderr, throttled to at most 15 redraws per second, compared with 20 per second for standalone default progress bars. This is a presentation ceiling, not a task-update frequency.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

+1



### 4.2 Coordinated draw ownership

Once `MultiProgress::add` incorporates a bar, its independent draw target is replaced with an internal, group-coordinated target.

This has two important consequences.

First, add a bar before its first draw. The upstream documentation explicitly warns that drawing a bar before registration can corrupt the coordinated terminal display because `MultiProgress` cannot undo earlier independently rendered output.

Second, changing a member bar’s draw target with `ProgressBar::set_draw_target` unlinks it from `MultiProgress`. To relocate the whole display, use `MultiProgress::set_draw_target`.

These rules make drawing ownership a meaningful part of the API rather than a cosmetic setting.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

+1



### 4.3 The easily overlooked “zombie” behavior

`MultiProgress` does not simply retain every completed bar’s complete renderable state forever.

The documentation describes a zombie algorithm for bars whose handles have been dropped. Previously drawn lines may remain briefly, but operations such as `println` or `suspend` can clear them. Since the original progress bar no longer exists, its finished display cannot necessarily be reconstructed.

Therefore, if completed child jobs should remain visibly represented in a multi-bar display, their handles must remain alive long enough. Retaining them in a collection is an example of preserving those presentation lifetimes.

Conversely, deliberately removing or dropping completed job bars can support a compact display that primarily shows active work.

This is a significant distinction between job completion, object lifetime, and terminal persistence.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 4.4 Output interleaving and terminal coordination

Both `ProgressBar` and `MultiProgress` provide `println`, which positions a message above active progress indicators.

`println` is intentionally a presentation operation: when the draw target is hidden, it does not print anything. This differs from ordinary logging, which may need to remain available in noninteractive environments.

The `suspend` method temporarily hides the progress display, executes a closure, then redraws it. It works for arbitrary terminal output, including when the progress display itself is hidden.

Concurrency limitation: `suspend` holds the relevant internal lock while executing its closure. Extended work inside the closure can block other threads attempting progress-related drawing. It is intended for short output operations, not long-running critical sections.

Source: [https://docs.rs/indicatif/0.18.6/indicatif/struct.MultiProgress.html](https://docs.rs/indicatif/0.18.6/indicatif/struct.MultiProgress.html?utm_source=chatgpt.com)

## 5. Template rendering and extensible progress formatting

`ProgressStyle` is the central presentation customization facility. It has considerably more capability than changing bar colors.

### 5.1 Rich templates

`ProgressStyle::with_template` parses a format string and returns `Result<ProgressStyle, TemplateError>`.

Common placeholders include:

| Family     | Keys                                                                               |
| ---------- | ---------------------------------------------------------------------------------- |
| State      | `pos`, `len`, `percent`, `percent_precise`, `human_pos`, `human_len`               |
| Activity   | `spinner`, `bar`, `wide_bar`, `msg`, `wide_msg`, `prefix`                          |
| Timing     | `elapsed`, `elapsed_precise`, `eta`, `eta_precise`, `duration`, `duration_precise` |
| Throughput | `per_sec`, `bytes_per_sec`, and decimal/binary variants                            |
| Bytes      | `bytes`, `total_bytes`, and decimal/binary variants                                |
| Extension  | User-defined keys through `with_key`                                               |

A template can combine a progress visualization and state information, for example:

`{prefix:<12} [{bar:30.cyan/blue}] {pos:>8}/{len} {eta} {msg}`

The template language supports alignment, width, truncation, color/style specifications, and multiline output.

The `wide_bar` and `wide_msg` forms consume available terminal width. Using both in the same template is not supported as a meaningful combined flexible-width layout; select which component should expand.

An important behavioral detail: a bar can have a message or prefix in state without displaying it. The relevant placeholder must exist in its style.

Source: [https://docs.rs/indicatif/0.18.6/indicatif/style/struct.ProgressStyle.html](https://docs.rs/indicatif/0.18.6/indicatif/style/struct.ProgressStyle.html?utm_source=chatgpt.com)

### 5.2 Custom bar and spinner glyphs

`progress_chars` specifies filled, current, and unfinished cells. It can contain more than three glyphs to provide finer visual gradations of a partially occupied progress cell; the grapheme clusters must have compatible display widths.

`tick_chars` and `tick_strings` define animation sequences for spinners. In both cases, the final entry is reserved for the completed state.

This distinction allows a progress bar or spinner to represent an explicit final visual condition without requiring the caller to manually replace the animation at completion.

Rendering uses terminal width and Unicode-display logic, which matters when using multicolumn characters, combining marks, emoji, or non-ASCII symbols.

### 5.3 Stateful and stateless custom template keys

One of indicatif’s most useful extension points is `ProgressStyle::with_key`.

It registers an application-defined key backed by a `ProgressTracker`. A tracker has four essential operations:

| Method                 | Role                                         |
| ---------------------- | -------------------------------------------- |
| `tick(state, now)`     | Update tracker-local state as progress ticks |
| `reset(state, now)`    | Reinitialize tracker state                   |
| `write(state, writer)` | Render its current representation            |
| `clone_box()`          | Preserve cloneability of styles and trackers |

The trait requires `Send + Sync` and gives access to the current `ProgressState`.

For a stateless formatter, indicatif also supports appropriately bounded cloneable closures implementing the required formatting function, so a dedicated struct and full tracker implementation are unnecessary.

A stateful `ProgressTracker` can maintain display-oriented history—such as a moving activity indicator, custom rate calculation, or status derived across ticks—while keeping that logic out of the execution pipeline.

This is an extension mechanism for formatting progress state, not a general application-state or metrics framework. Application-specific information not present in `ProgressState` must be supplied by the chosen extension or external shared state.

Sources:

[https://docs.rs/indicatif/0.18.6/indicatif/style/trait.ProgressTracker.html](https://docs.rs/indicatif/0.18.6/indicatif/style/trait.ProgressTracker.html?utm_source=chatgpt.com)

[https://docs.rs/indicatif/0.18.6/indicatif/style/struct.ProgressStyle.html](https://docs.rs/indicatif/0.18.6/indicatif/style/struct.ProgressStyle.html?utm_source=chatgpt.com)

## 6. Instrumenting existing computation and data movement

Indicatif includes adapters that attach progress reporting to existing Rust abstraction boundaries, reducing the need to scatter explicit counter updates throughout work loops.

### 6.1 Sequential iterators

The `ProgressIterator` trait extends ordinary `Iterator`s with `progress`, `progress_count`, `try_progress`, `progress_with`, and `progress_with_style`.

These methods make meaningful distinctions about total length:

| Adapter                      | Constraint / behavior                                             |
| ---------------------------- | ----------------------------------------------------------------- |
| `progress()`                 | Requires `ExactSizeIterator`                                      |
| `progress_with_style(style)` | Requires `ExactSizeIterator`                                      |
| `progress_count(len)`        | Uses an explicitly supplied total                                 |
| `try_progress()`             | Uses the upper bound from `size_hint`, returning `None` if absent |
| `progress_with(bar)`         | Uses an externally managed bar                                    |

The `try_progress` upper bound is not necessarily the final number of items. An iterator may produce fewer than its upper bound, so a progress display constructed this way can overstate the expected total.

`ProgressBarIter<T>` preserves relevant iterator capabilities of its underlying type and exposes a progress bar handle for customization and external coordination.

Pipeline semantics matter: the adapter observes item traversal, not an arbitrary notion of completion. Its increment occurs as an item is yielded. Therefore, where the adapter is placed relative to expensive transformations determines whether the displayed position reflects input consumption or completed downstream work.

An iterator that is abandoned early may not reach its normal exhaustion-based finish behavior. Resource cleanup and progress presentation should not be conflated.

Sources:

[https://docs.rs/indicatif/0.18.6/indicatif/trait.ProgressIterator.html](https://docs.rs/indicatif/0.18.6/indicatif/trait.ProgressIterator.html?utm_source=chatgpt.com)

[https://docs.rs/indicatif/0.18.6/indicatif/struct.ProgressBarIter.html](https://docs.rs/indicatif/0.18.6/indicatif/struct.ProgressBarIter.html?utm_source=chatgpt.com)

### 6.2 Parallel iteration with Rayon

With feature `rayon`, `ParallelProgressIterator` brings corresponding instrumentation to Rayon parallel iterators.

`progress()` and `progress_with_style()` require `IndexedParallelIterator`, while `progress_count()` accepts an explicit total for other parallel iterators.

The progress state can be updated safely across Rayon workers through the shared progress handle, without the application implementing its own terminal aggregation layer.

This does not mean indicatif schedules or synchronizes the parallel computation. Rayon owns work splitting, thread scheduling, and completion; indicatif observes progress at the adapter boundary.

Consequently, the location of progress instrumentation in a Rayon chain should be interpreted carefully, especially when work has a substantial gap between item production and processing completion.

Source: [https://docs.rs/indicatif/0.18.6/indicatif/trait.ParallelProgressIterator.html](https://docs.rs/indicatif/0.18.6/indicatif/trait.ParallelProgressIterator.html?utm_source=chatgpt.com)

### 6.3 Synchronous I/O

`ProgressBar::wrap_read` and `wrap_write` produce progress-aware adapters for `std::io::Read` and `Write`.

They update progress based on successfully transferred byte counts. This makes operations such as copying, compression pipelines, downloads, and serialization streams measurable without manually changing counters inside each read/write invocation.

The wrapper also supports relevant I/O trait behavior including buffered operations and seeking, subject to the wrapped type’s capabilities.

A less obvious source-level behavior is that seek progress can use recent maximum positions to smooth the visible position when navigating backward and forward. A seeking adapter’s progress display therefore should not automatically be interpreted as the exact instantaneous file cursor or as a perfect count of unique bytes processed.

This distinction matters for random-access processing, repeated reads, and verification workloads.

Sources:

[https://docs.rs/indicatif/0.18.6/indicatif/struct.ProgressBarIter.html](https://docs.rs/indicatif/0.18.6/indicatif/struct.ProgressBarIter.html?utm_source=chatgpt.com)

[https://docs.rs/indicatif/0.18.6/src/indicatif/iter.rs.html](https://docs.rs/indicatif/0.18.6/src/indicatif/iter.rs.html?utm_source=chatgpt.com)

### 6.4 Tokio asynchronous I/O

The optional `tokio` feature enables `wrap_async_read` and `wrap_async_write` for Tokio’s `AsyncRead` and `AsyncWrite`, with the documented `Unpin` bounds.

These adapters record successful asynchronous I/O progress while leaving polling, readiness, task scheduling, and cancellation to Tokio and the caller.

This is deliberately narrower than automatic monitoring of a Tokio runtime or all spawned tasks. Indicatif’s progress API can be used from async tasks, but only the documented adapters and explicit progress updates provide instrumentation.

Also, `enable_steady_tick` still uses its own background thread; enabling the Tokio feature does not transform it into a runtime-managed interval.

Source: [https://docs.rs/indicatif/0.18.6/indicatif/struct.ProgressBar.html](https://docs.rs/indicatif/0.18.6/indicatif/struct.ProgressBar.html?utm_source=chatgpt.com)

### 6.5 Futures streams

The optional `futures` feature adds `ProgressBar::wrap_stream` for `futures_core::Stream` implementations satisfying the required bounds.

It increments progress as the stream yields items, not when the stream is pending. The adapter can apply its finish behavior when stream exhaustion is observed.

This provides convenient visibility into received messages or emitted results, but does not by itself track processing completion after each emission. Nor does it add backpressure control, retry handling, or a stream scheduler.

Collectively, these adapters make indicatif useful at existing language and library boundaries, while keeping progress state observational rather than authoritative.

## 7. Terminal rendering, custom destinations, and testability

### 7.1 Draw targets and refresh control

`ProgressDrawTarget` encapsulates the destination and rendering-rate policy.

Available constructors include `stdout`, `stderr`, `stdout_with_hz`, `stderr_with_hz`, `term`, `term_like`, `term_like_with_hz`, and `hidden`.

`term` accepts a `console::Term` and a refresh rate. It suppresses interactive rendering when the destination is not a suitable attended terminal or when `TERM` is unset or `dumb`. A refresh rate of zero is invalid and causes a panic.

`term_like` accepts an arbitrary `Box<dyn TermLike>` but does not establish a default refresh limit. For custom terminal destinations where drawing could be expensive, `term_like_with_hz` is an important alternative.

This separation enables a rendering policy that is independent of the rate at which work reports progress.

Source: [https://docs.rs/indicatif/0.18.6/indicatif/struct.ProgressDrawTarget.html](https://docs.rs/indicatif/0.18.6/indicatif/struct.ProgressDrawTarget.html?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 7.2 `TermLike`: terminal abstraction

`TermLike` is an object-safe interface requiring `Debug + Send + Sync`. It describes a minimal terminal-like target with terminal dimensions, cursor movements, line/string writes, line clearing, and flushing.

Custom implementors can support special output surfaces, controlled tests, or integration layers that translate indicatif’s rendering calls into another terminal-like environment.

This is particularly useful because it does not require replacing the progress-state, style, or multi-bar mechanisms.

However, `TermLike` is not a complete screen-management framework. If another component independently controls the terminal cursor, the system must still coordinate ownership of the output region. Indicatif does not inherently resolve conflicts with a separate full-screen terminal UI.

[https://docs.rs/indicatif/0.18.6/indicatif/trait.TermLike.html](https://docs.rs/indicatif/0.18.6/indicatif/trait.TermLike.html?utm_source=chatgpt.com)

### 7.3 In-memory terminal emulation

The optional `in_memory` feature provides `InMemoryTerm`, built over `vt100::Parser`.

It offers `new(rows, cols)`, `contents`, `contents_formatted`, `moves_since_last_check`, and `reset`. It implements `TermLike` and is cloneable through shared internal state.

This is a particularly valuable, easily overlooked capability for testing:

- Assert final rendered contents rather than only internal progress counters.
- Exercise dynamic multi-bar insertion/removal and finished-state persistence.
- Observe cursor-movement behavior.
- Verify layout behavior under controlled terminal dimensions.
- Test renderers without relying on an actual interactive terminal.

In-memory rendering does not automatically make timer-driven tests deterministic. Tests involving steady ticking or refresh limits still need to account for timing separately.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



Source: [https://docs.rs/indicatif/0.18.6/indicatif/struct.InMemoryTerm.html](https://docs.rs/indicatif/0.18.6/indicatif/struct.InMemoryTerm.html?utm_source=chatgpt.com)

### 7.4 Terminal width and Unicode

The default `unicode-width` feature supports display-width calculations. The additional `improved_unicode` feature brings Unicode grapheme segmentation, improving behavior with multicodepoint displayed characters.

This matters when terminal columns differ from character count, especially with combining sequences, emoji, and styled output.

Nevertheless, precise appearance remains dependent on the terminal’s own character-width and rendering behavior. Unicode-aware width calculation reduces common layout errors; it does not guarantee identical rendering across every terminal emulator.

## 8. Formatting utilities outside progress bars

Indicatif exposes reusable display-formatting wrappers that can be used independently of a rendered progress bar.

| Utility             | Purpose                                         |
| ------------------- | ----------------------------------------------- |
| `HumanBytes`        | Human-readable byte values using binary scaling |
| `DecimalBytes`      | Decimal/SI-style byte scaling                   |
| `BinaryBytes`       | Explicit binary/IEC byte scaling                |
| `HumanDuration`     | Readable duration representation                |
| `FormattedDuration` | Duration-oriented formatting                    |
| `HumanCount`        | Compact human-readable counts                   |
| `HumanFloatCount`   | Human-readable floating-point counts            |

These help maintain consistent terminal reporting between progress indicators and ordinary status messages.

The distinction between decimal and binary byte units is consequential: 1 kB and 1 KiB represent different magnitudes. For storage, network transfer, and memory reporting, the formatter should reflect the intended unit convention rather than merely produce an attractive number.

The formatting utilities rely on the crate’s formatting functionality; they do not require that a visible progress bar be active.

Reference: [https://docs.rs/indicatif/0.18.6/indicatif/](https://docs.rs/indicatif/0.18.6/indicatif/?utm_source=chatgpt.com)

## 9. Cargo features and runtime requirements

Version 0.18.6 declares Rust 1.85 as its minimum supported Rust version and uses Rust edition 2021. Its crate license is MIT.

| Cargo feature      | Default? | Capability enabled                                        |
| ------------------ | -------- | --------------------------------------------------------- |
| `unicode-width`    | Yes      | Unicode display-width support                             |
| `wasmbind`         | Yes      | Time-related WebAssembly build support through `web-time` |
| `improved_unicode` | No       | Additional grapheme segmentation                          |
| `rayon`            | No       | `ParallelProgressIterator`                                |
| `tokio`            | No       | Tokio async I/O adapters                                  |
| `futures`          | No       | Futures stream adapter                                    |
| `in_memory`        | No       | `InMemoryTerm` backed by `vt100`                          |

Two details deserve attention.

Documentation feature exposure: docs.rs builds indicatif with all features enabled. An API appearing in the documentation therefore does not mean it exists under the crate’s default Cargo features.

WebAssembly: the `wasmbind` feature provides compilation-related support for WASM timing, not a guarantee of terminal presentation in a browser. The available rendering target and actual output environment remain separate concerns.

The crate uses `console` for terminal functionality, while optional integrations depend on the respective Rayon, Tokio, futures-core, Unicode, or terminal-emulation libraries.

Primary source: [https://github.com/console-rs/indicatif/blob/0.18.6/Cargo.toml](https://github.com/console-rs/indicatif/blob/0.18.6/Cargo.toml?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



Feature reference: [https://docs.rs/crate/indicatif/0.18.6/features](https://docs.rs/crate/indicatif/0.18.6/features?utm_source=chatgpt.com)

## 10. Companion libraries and integration boundaries

Several important integrations are provided by companion crates rather than indicatif itself.

| Library                | Relationship                                         | Important distinction                                                                 |
| ---------------------- | ---------------------------------------------------- | ------------------------------------------------------------------------------------- |
| `console`              | Underlying terminal formatting and control           | Supplies terminal/color facilities; indicatif supplies progress semantics             |
| `indicatif-log-bridge` | Connects Rust `log` output with active progress bars | Helps avoid logger output corrupting an indicatif display                             |
| `tracing-indicatif`    | Connects tracing spans and progress indicators       | Can manage progress indicators from span lifecycles rather than explicit job counters |
| `rayon`                | Parallel work execution                              | Indicatif instruments the iterator but does not run the thread pool                   |
| `tokio`                | Asynchronous runtime and I/O                         | Indicatif provides progress-aware I/O adapters, not runtime scheduling                |
| `dialoguer`            | Interactive prompts and selection                    | Complementary terminal interaction, not an indicatif feature                          |

### Logging with `indicatif-log-bridge`

`indicatif-log-bridge` provides a coordinated logger integration for applications using the `log` facade.

The point is not merely styling log records. It addresses the underlying problem of multiple components writing to the same interactive terminal while progress bars must remain redrawable.

Relevant source:

[https://docs.rs/indicatif-log-bridge/latest/indicatif_log_bridge/](https://docs.rs/indicatif-log-bridge/latest/indicatif_log_bridge/?utm_source=chatgpt.com)

### Tracing span visualization with `tracing-indicatif`

`tracing-indicatif` supplies a `tracing_subscriber` layer that can create and manage progress indicators for active tracing spans.

This enables an alternative instrumentation model: rather than constructing bars for every operation, activity can be represented through existing structured span lifecycles, with appropriate filtering and display configuration.

That does not make tracing spans equivalent to numeric progress. Span activity describes an execution interval or operation scope; explicit progress counters remain necessary when completion fraction, throughput, or ETA should be meaningful.

The library also provides coordination for formatting events without disrupting progress displays.

Relevant source:

[https://docs.rs/tracing-indicatif/latest/tracing_indicatif/](https://docs.rs/tracing-indicatif/latest/tracing_indicatif/?utm_source=chatgpt.com)

These are genuine complementary mechanisms. `indicatif` can present explicitly tracked work; `tracing-indicatif` can present span-derived activity; and conventional logs can communicate durable diagnostic events.

## 11. Consequential design distinctions and limitations

| Distinction                                  | Why it matters                                                                                                        |
| -------------------------------------------- | --------------------------------------------------------------------------------------------------------------------- |
| Progress versus work completion              | A displayed counter does not establish that processing or side effects succeeded                                      |
| Position versus known total                  | `no_length` and mutable totals support open-ended work; percentage and ETA require meaningful denominators            |
| Shared state versus independent bars         | Cloning a `ProgressBar` shares state; construct a separate bar for a separate logical job                             |
| Update versus redraw                         | Progress state can change more frequently than terminal rendering                                                     |
| MultiProgress ownership                      | Bars must be registered before drawing, and finished handles may need retention to preserve visible lines             |
| Normal finish versus abandon                 | A failed or interrupted job should not automatically be represented as a successful completion                        |
| Interactive display versus logging           | Hidden progress and `println` are not substitutes for machine-consumable or durable diagnostics                       |
| Adapter position versus completed processing | Iterator/stream instrumentation observes its own boundary, not necessarily downstream completion                      |
| Custom drawing versus terminal ownership     | `TermLike` permits alternative targets but does not solve conflicts with other components controlling the same screen |
| Refresh limit versus total overhead          | Drawing throttling reduces terminal I/O; it does not eliminate updates, locks, allocation, or custom formatting work  |

### Where indicatif can replace bespoke infrastructure

Indicatif is particularly capable when the required abstraction is observable progress over work whose execution is managed elsewhere.

Its native components can replace much custom machinery for synchronized progress counters, dynamic child-job displays, human-readable rate and ETA estimation, style customization, progress-aware byte/iterator adapters, and testing of terminal output.

The combination of `MultiProgress`, customizable `ProgressTracker`s, adapter-based instrumentation, and `TermLike` makes it substantially more flexible than a simple progress-bar widget.

### Where its responsibility stops

It is not a task orchestrator, cancellation framework, tracing backend, durable telemetry system, metrics registry, or full-screen terminal application framework.

It also does not determine what constitutes one completed unit of work. That remains the responsibility of the instrumentation boundary.

The strongest architectural use is to let execution, measurement, and terminal presentation remain separate while minimizing the coordination and rendering code that must be implemented manually.

## 12. Primary-source reference map

The following documentation pages are all applicable to the verified 0.18.6 release.

| Topic                     | Direct source URL                                                                                                                                                                       |
| ------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Crate API                 | [https://docs.rs/indicatif/0.18.6/indicatif/](https://docs.rs/indicatif/0.18.6/indicatif/?utm_source=chatgpt.com)                                                                       |
| `ProgressBar`             | [https://docs.rs/indicatif/0.18.6/indicatif/struct.ProgressBar.html](https://docs.rs/indicatif/0.18.6/indicatif/struct.ProgressBar.html?utm_source=chatgpt.com)                         |
| `ProgressState`           | [https://docs.rs/indicatif/0.18.6/indicatif/struct.ProgressState.html](https://docs.rs/indicatif/0.18.6/indicatif/struct.ProgressState.html?utm_source=chatgpt.com)                     |
| `MultiProgress`           | [https://docs.rs/indicatif/0.18.6/indicatif/struct.MultiProgress.html](https://docs.rs/indicatif/0.18.6/indicatif/struct.MultiProgress.html?utm_source=chatgpt.com)                     |
| `ProgressStyle`           | [https://docs.rs/indicatif/0.18.6/indicatif/style/struct.ProgressStyle.html](https://docs.rs/indicatif/0.18.6/indicatif/style/struct.ProgressStyle.html?utm_source=chatgpt.com)         |
| `ProgressTracker`         | [https://docs.rs/indicatif/0.18.6/indicatif/style/trait.ProgressTracker.html](https://docs.rs/indicatif/0.18.6/indicatif/style/trait.ProgressTracker.html?utm_source=chatgpt.com)       |
| Iterator adapter          | [https://docs.rs/indicatif/0.18.6/indicatif/trait.ProgressIterator.html](https://docs.rs/indicatif/0.18.6/indicatif/trait.ProgressIterator.html?utm_source=chatgpt.com)                 |
| Rayon adapter             | [https://docs.rs/indicatif/0.18.6/indicatif/trait.ParallelProgressIterator.html](https://docs.rs/indicatif/0.18.6/indicatif/trait.ParallelProgressIterator.html?utm_source=chatgpt.com) |
| `ProgressBarIter`         | [https://docs.rs/indicatif/0.18.6/indicatif/struct.ProgressBarIter.html](https://docs.rs/indicatif/0.18.6/indicatif/struct.ProgressBarIter.html?utm_source=chatgpt.com)                 |
| Draw targets              | [https://docs.rs/indicatif/0.18.6/indicatif/struct.ProgressDrawTarget.html](https://docs.rs/indicatif/0.18.6/indicatif/struct.ProgressDrawTarget.html?utm_source=chatgpt.com)           |
| Terminal abstraction      | [https://docs.rs/indicatif/0.18.6/indicatif/trait.TermLike.html](https://docs.rs/indicatif/0.18.6/indicatif/trait.TermLike.html?utm_source=chatgpt.com)                                 |
| In-memory terminal        | [https://docs.rs/indicatif/0.18.6/indicatif/struct.InMemoryTerm.html](https://docs.rs/indicatif/0.18.6/indicatif/struct.InMemoryTerm.html?utm_source=chatgpt.com)                       |
| Cargo feature definitions | [https://github.com/console-rs/indicatif/blob/0.18.6/Cargo.toml](https://github.com/console-rs/indicatif/blob/0.18.6/Cargo.toml?utm_source=chatgpt.com)                                 |
| Tagged release source     | [https://github.com/console-rs/indicatif/tree/0.18.6](https://github.com/console-rs/indicatif/tree/0.18.6?utm_source=chatgpt.com)                                                       |
| Release notes             | [https://github.com/console-rs/indicatif/releases/tag/0.18.6](https://github.com/console-rs/indicatif/releases/tag/0.18.6?utm_source=chatgpt.com)                                       |

Overall: Indicatif 0.18.6 provides a mature separation between shared progress state, presentation logic, concurrent terminal coordination, and instrumentation adapters. Its most consequential advanced capabilities are the customizable tracker system, dynamic multi-progress lifecycle management, iterator/parallel/I/O instrumentation, draw-target abstraction, and in-memory terminal testing. Together, these make it a capable general-purpose foundation for interactive Rust CLI progress reporting without coupling the UI to the underlying execution model.

# Indicatif 0.18.6 — Advanced Runtime and API Semantics Addendum

Supplement to `indicateif.md` | Research date: November 7, 2026 | Target release: 0.18.6

This addendum supplements the existing Indicatif capability reference with implementation mechanisms, lifecycle semantics, adapter behavior, and lower-level APIs not fully developed in the original document. It is intended to be read alongside that reference rather than replace its architectural and functional coverage.

The analysis targets the published Indicatif 0.18.6 release, which remains the latest stable release identified in the registry as of the research date. Claims about internal mechanisms describe the tagged implementation, not necessarily cross-version public API guarantees.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+1



## 1. Atomic progress updates and hierarchical rendering throttling

### 1.1 A distinct atomic position-update path

Indicatif separates progress-position mutations from the more expensive work of updating derived state, invoking custom trackers, formatting output, and drawing to a terminal.

A `ProgressBar` contains three shared components:

- `Arc<AtomicPosition>` for its position counter and update-admission state.
- `Arc<Mutex<BarState>>` for progress metadata, estimates, formatting, rendering, and lifecycle state.
- `Arc<Mutex<Option<Ticker>>>` for controlling periodic ticking.

This decomposition is significant because ordinary position increments do not necessarily acquire the main `BarState` mutex.

The source implementation of `ProgressBar::inc`, `dec`, and `set_position` follows a common pattern:

1. Update the position through `AtomicPosition`.
2. Obtain the current time.
3. Ask `AtomicPosition::allow(now)` whether a tick should proceed.
4. Only when admitted, call `tick_inner`, which can acquire the state lock and perform estimate, tracker, and rendering work.

The `AtomicPosition` counter uses `portable_atomic::AtomicU64`. Increment and decrement operations use atomic read-modify-write instructions, while `set_position` uses an atomic store.

The observable progress position can therefore change without a corresponding immediate style update, custom tracker invocation, or terminal redraw.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

source+2



### 1.2 Token-bucket-like admission control

In version 0.18.6, `AtomicPosition::allow` uses an internal budget with two constants:

| Internal constant | Value                 | Role                                         |
| ----------------- | --------------------- | -------------------------------------------- |
| `INTERVAL`        | 1,000,000 nanoseconds | Approximately 1 ms per replenished admission |
| `MAX_BURST`       | 10                    | Maximum accumulated admission capacity       |

The initial capacity allows a limited number of rapid ticks. Once depleted, additional ticks are admitted as sufficient time elapses.

The mechanism resembles a token bucket, although its implementation uses atomic loads and stores rather than a separately synchronized queue or strict, globally serialized admission counter.

These are internal implementation constants, not public configuration settings. They should not be interpreted as an exact contractual maximum of 1,000 ticks per second under arbitrary contention.

The purpose is to prevent a stream of inexpensive atomic position changes from causing an equally large number of mutex acquisitions and expensive presentation updates.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

source+1



### 1.3 The two throttling layers are independent

An admitted position tick is not equivalent to an actual terminal redraw.

Indicatif's update path

`inc` / `dec` / `set_position`

Atomic position mutation

`AtomicPosition::allow`

Admission gate, roughly 1 ms with burst capacity

`BarState::tick`

Mutex-protected estimator and tracker updates

`ProgressDrawTarget::drawable`

Independent redraw limit, terminal visibility, and destination policy

Formatting and terminal I/O

Performed only when a draw is admitted or forced

The draw target applies its own rate limiter after the bar's state-update gate. In the default standalone configuration, the target is configured for approximately 20 redraws per second, with support for occasional bursts.

The separation permits high-frequency progress accounting without equivalent terminal I/O or formatting overhead.

It also means that a custom `ProgressTracker` is not necessarily notified of every individual position change. Its `tick` method is invoked through admitted state-update operations, not directly for every atomic increment.

For trackers that derive cumulative progress, the current `ProgressState::pos()` remains the relevant source. Trackers that assume exactly one tick for each unit of progress can behave incorrectly under throttling.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

source+2



### 1.4 Scope and performance implications

This optimization applies particularly to the ordinary position-update path. Operations such as changing messages, prefixes, or lengths enter `BarState` through separate methods and do not necessarily follow the same atomic admission route.

A `force_draw()` can also bypass ordinary redraw throttling, while steady ticking provides another source of periodic state updates.

Consequently, the mechanism does not imply that every `ProgressBar` method is lock-free, that every mutation incurs constant cost independent of concurrency, or that custom formatters cannot become a performance bottleneck.

The architectural benefit is narrower and more useful: Indicatif can preserve frequent, shared progress-count updates while performing substantially less derived-state and rendering work. This reduces the need for application-specific progress aggregation, timer management, and draw-rate control.

Primary sources:

`https://docs.rs/indicatif/0.18.6/src/indicatif/progress_bar.rs.html`

`https://docs.rs/indicatif/0.18.6/src/indicatif/state.rs.html`

`https://docs.rs/indicatif/0.18.6/src/indicatif/draw_target.rs.html`

## 2. Automatic completion, final-handle destruction, and presentation lifetime

### 2.1 Completion is also a destruction-time operation

The original guide describes explicit finishing and the `ProgressFinish` variants. An additional lifecycle mechanism is important: dropping the final owning reference to an unfinished bar can automatically apply its configured finish behavior.

This behavior is implemented by `Drop for BarState`.

When the shared `BarState` is destroyed:

1. If the progress state is already finished, it does not apply a second finish policy.
2. Otherwise, it invokes `finish_using_style` using its stored `on_finish` policy.
3. It notifies the draw target that the progress bar has become a zombie, where applicable to `MultiProgress`.

Because `BarState` is held through an `Arc`, this destruction behavior is associated with the last owning reference, not with dropping an arbitrary cloned `ProgressBar` handle.

A `WeakProgressBar` does not keep the shared state alive, although successfully upgrading one temporarily creates another strong handle.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

source+2



### 2.2 The default finish policy is `AndClear`

`ProgressFinish::default()` is `ProgressFinish::AndClear`.

Therefore, when an unfinished bar reaches its final-handle destruction, the default behavior marks it finished and clears its visible progress display.

This differs materially from the explicit `ProgressBar::finish()` method, which corresponds to `ProgressFinish::AndLeave`.

The distinction has implications for progress displays expected to preserve their final output after the owning scope exits. Merely reaching the numerical total and allowing a handle to go out of scope is not equivalent to explicitly finishing with a retained final display.

`ProgressBar::with_finish` allows the automatic policy to be changed. For example, `ProgressFinish::WithMessage` can establish a final message, while `ProgressFinish::Abandon` preserves the current partial progress instead of marking the bar numerically complete.

In the latter case, the display is nevertheless considered finished from Indicatif's lifecycle perspective. `Abandon` is not a mechanism for keeping the progress bar active indefinitely; it is a different terminal presentation state.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



### 2.3 Automatic completion depends on the owning abstraction

There are two distinct automatic completion events:

| Event                                                   | Behavior in 0.18.6                                             |
| ------------------------------------------------------- | -------------------------------------------------------------- |
| Iterator or stream adapter observes exhaustion          | Applies `finish_using_style()` to the associated bar           |
| Last owning reference to an unfinished bar is destroyed | Applies the stored `on_finish` policy through `BarState::drop` |

These events can occur at different times.

An iterator can reach exhaustion and mark a bar finished while other strong handles remain alive. Conversely, an iterator can be abandoned before exhaustion while another strong handle keeps the bar alive and unfinished.

For I/O wrappers, receiving an EOF or completing a write does not follow the same exhaustion-triggered finish path. Their completion behavior is discussed further in Section 4.

An explicitly finished bar avoids reapplication of its automatic finish policy when its final owning handle is subsequently destroyed.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+2



### 2.4 Interaction with `MultiProgress` zombie handling

Automatic completion and zombie handling should be understood as separate mechanisms.

A bar can be finished while its handle remains alive, allowing `MultiProgress` to reconstruct its final appearance during subsequent redraws.

When its last strong handle is dropped, `MultiProgress` loses access to the underlying bar state. It may temporarily preserve previously drawn terminal lines, but it cannot reliably reconstruct that bar after a later clear or redraw operation.

Thus, the finish policy determines the final state of a bar, while ownership lifetime determines whether a multi-bar renderer can continue reconstructing it.

A configured `WithMessage` or `AndLeave` policy alone is insufficient to guarantee persistent display of a completed child bar across future `MultiProgress` redraws if all owning handles have been dropped.

Primary sources:

`https://docs.rs/indicatif/0.18.6/indicatif/enum.ProgressFinish.html`

`https://docs.rs/indicatif/0.18.6/src/indicatif/state.rs.html`

`https://docs.rs/indicatif/0.18.6/indicatif/struct.MultiProgress.html`

## 3. `MultiProgress` removal, clearing, and ownership constraints

### 3.1 Removing a progress bar changes its draw target

`MultiProgress::remove(&bar)` does more than remove an entry from a visual ordering.

In version 0.18.6, the implementation:

1. Retrieves the bar's internal remote draw target.
2. Confirms that the bar belongs to the invoking `MultiProgress`.
3. Replaces the member bar's draw target with `ProgressDrawTarget::hidden()`.
4. Removes the corresponding member index from the group's internal presentation state.

Importantly, removal does not destroy the `ProgressBar` or reset its position, total length, message, or other progress state.

The application may retain and continue updating the removed handle. However, the bar no longer participates in the original coordinated display and is left with a hidden draw target.

It can subsequently be attached to an appropriate draw target or registered with a progress group again.

This creates a useful lifecycle distinction between removing an indicator from a visual group and terminating the underlying progress state.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+1



### 3.2 Ownership checks are stricter than a general no-op rule

The public documentation says that attempting to remove a progress bar not previously registered with the group has no effect.

The tagged implementation has an additional distinction:

- A bar without a remote `MultiProgress` draw target is ignored.
- A bar associated with a different `MultiProgress` causes an assertion failure because its group identity does not match.

The assertion uses `Arc::ptr_eq` to compare the shared group-state identities.

Therefore, `remove` should not be interpreted as an idempotent operation that safely accepts a bar belonging to an arbitrary group.

The exact assertion is a source-observed 0.18.6 behavior rather than a separately documented compatibility guarantee.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+1



### 3.3 Clearing is not equivalent to removal

`MultiProgress::clear()` clears the current terminal presentation without unregistering its active member bars.

The underlying group retains its membership records and ordering. Its bars can be rendered again on subsequent progress updates or forced redraws.

Internally, the clear operation also accounts for zombie lines previously retained by the renderer. These are cleared as part of the terminal operation, even though the original bar states may no longer exist.

The distinction is:

| Operation         | Member registration       | Progress state                | Terminal effect                               |
| ----------------- | ------------------------- | ----------------------------- | --------------------------------------------- |
| `remove(&bar)`    | Removes one member        | Retained by surviving handles | Removes the bar from coordinated presentation |
| `clear()`         | Retains active members    | Unchanged                     | Clears the current rendered display           |
| Final-handle drop | Ends ownership of the bar | Destroyed                     | Applies finish policy and zombie handling     |

`clear()` is therefore a presentation reset rather than a lifecycle reset. It also returns `io::Result<()>`, allowing its caller to observe at least some terminal-output failures.

These distinctions matter for dynamic interfaces in which indicators can be temporarily cleared, removed, reintroduced, or retained after completion without changing the execution lifecycle of the represented work.

Primary source:

`https://docs.rs/indicatif/0.18.6/src/indicatif/multi.rs.html`

## 4. Trait-preserving adapters and precise accounting boundaries

### 4.1 `ProgressBarIter<T>` preserves multiple abstraction interfaces

Although its name emphasizes iteration, `ProgressBarIter<T>` is a general-purpose progress-aware wrapper over several Rust abstraction families.

Its trait implementations are conditional on the wrapped type's own capabilities.

| Wrapped interface                            | Implemented interface      | Important condition                                                            |
| -------------------------------------------- | -------------------------- | ------------------------------------------------------------------------------ |
| `Iterator`                                   | `Iterator`                 | Automatically observes item yields and exhaustion                              |
| `ExactSizeIterator`                          | `ExactSizeIterator`        | Retains the inner iterator's remaining-length behavior                         |
| `DoubleEndedIterator`                        | `DoubleEndedIterator`      | Also instruments traversal through `next_back()`                               |
| `FusedIterator`                              | `FusedIterator`            | Retained when the inner iterator implements it                                 |
| `std::io::Read`                              | `Read`                     | Instruments successful reads                                                   |
| `std::io::BufRead`                           | `BufRead`                  | Instruments consumption of buffered bytes                                      |
| `std::io::Seek`                              | `Seek`                     | Retains seeking and position-query methods                                     |
| `std::io::Write`                             | `Write`                    | Instruments accepted writes                                                    |
| Tokio `AsyncRead`, `AsyncWrite`, `AsyncSeek` | Corresponding Tokio traits | Requires `tokio` feature and documented `Unpin` bounds                         |
| Tokio `AsyncBufRead`                         | `AsyncBufRead`             | Requires `tokio`; inner type must implement `AsyncBufRead + AsyncRead + Unpin` |
| Futures `Stream`                             | `Stream`                   | Requires `futures` feature and `Unpin`                                         |

These implementations make the wrapper useful beyond a single interface. A buffered, seekable reader can retain those capabilities after progress instrumentation is introduced.

This can avoid creating separate instrumentation wrappers for buffering, seeking, and ordinary reading.

The available interfaces depend on the wrapped type and enabled Cargo features; constructing a `ProgressBarIter<T>` does not itself confer traits that `T` lacks.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+2



### 4.2 Buffered reads are counted when consumed

The buffered I/O implementations make an important distinction between making bytes available and consuming them.

For synchronous `BufRead`:

- `fill_buf()` delegates to the wrapped reader without advancing progress.
- `consume(amt)` delegates consumption and then advances the progress position by the consumed amount.

The equivalent Tokio `AsyncBufRead` implementation follows the same principle:

- `poll_fill_buf()` does not advance progress merely because data becomes available.
- `consume(amt)` updates the position.

This prevents repeated buffer inspection from being counted as repeated byte consumption.

It also means that progress is tied to the `BufRead` consumption boundary, not necessarily to underlying system-level reads. A buffer may have been populated earlier without being fully consumed.

For general buffered processing, this is an appropriate distinction. However, the counter should not be interpreted as a measure of physical disk reads, network transfers, or unique source bytes independently of the adapter's position in the I/O stack.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+1



### 4.3 Write progress measures accepted bytes, not durable output

The synchronous `Write` implementation updates progress using the number of bytes returned by successful `write()` and `write_vectored()` calls.

`flush()` delegates to the inner writer without adding progress.

The Tokio implementation similarly updates the counter from completed `poll_write()` operations, while `poll_flush()` and `poll_shutdown()` delegate without introducing additional progress accounting.

These behaviors have an important consequence for buffered writers.

A successful write may mean that bytes have been accepted into an intermediate buffer rather than transmitted to a remote destination or persisted to storage.

Thus, the adapter measures accepted write bytes at the instrumented interface, not application-level durability.

For operations involving buffered storage, compression, remote transfer, or transactional output, successful finalization is separate from the progress counter. The adapter neither guarantees persistence nor replaces the relevant completion acknowledgment.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+1



### 4.4 Partial failures and asynchronous readiness require care

The adapter also has some consequential implementation-level error behavior.

In the synchronous `Read` implementation, `read_exact()` delegates to the underlying reader and increments the counter by the full buffer length only after the operation returns successfully.

If `read_exact()` fails after the underlying reader has consumed some data, the wrapper does not account for that partial consumption through this overridden method.

This means the progress position can underrepresent actual consumed bytes following certain partial failures.

For Tokio `AsyncRead`, the wrapper calculates the change in `ReadBuf::filled().len()` when the underlying read returns `Poll::Ready`. A `Poll::Pending` result does not advance progress.

These details reinforce that progress accounting is derived from the specific trait method's observable behavior. It is not an independent byte-level audit of all effects performed by the wrapped object.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+1



### 4.5 Seek tracking is intentionally presentation-oriented

The I/O wrapper retains `Seek` and Tokio `AsyncSeek`, but its displayed progress is not simply a direct copy of the current underlying cursor.

Internally, the `SeekMax` mechanism maintains recent positions to reduce visible instability from backward-and-forward seeks.

The default implementation uses a recent-position history of ten entries and can discard that history after five consecutive sequential operations.

Several details distinguish this from ordinary cursor tracking:

- A seek can alter displayed progress through the recent-position estimator.
- `SeekFrom::Current(0)` is treated as a position query rather than a meaningful seek update.
- `stream_position()` delegates to the underlying object without introducing a progress-position update.
- After a period of sequential operations, the smoothing history can be discarded.

The resulting progress value is a useful presentation estimate for workloads that seek within a file, but is not necessarily a precise representation of the current cursor, unique bytes processed, or total bytes transferred.

The parameters and implementation are internal details of 0.18.6.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+1



### 4.6 Exhaustion-driven completion does not extend to ordinary I/O

The adapter's automatic finish semantics differ across abstraction families.

For `Iterator`, the implementation checks the result of `next()` or `next_back()`. When exhaustion is observed, it applies `finish_using_style()` if the bar is not already finished.

For a Futures `Stream`, the analogous trigger is `Poll::Ready(None)`.

In contrast, the `Read`, `Write`, `AsyncRead`, and `AsyncWrite` implementations do not introduce equivalent automatic finish transitions when an EOF, successful flush, or shutdown is encountered.

Their methods instrument progress and propagate the underlying I/O results, while presentation completion remains dependent on explicit finish operations or the separate destruction-time behavior described in Section 2.

This distinction is useful when the end of byte transfer and the end of the broader operation do not coincide—for example, when validation, commit, checksum verification, or remote acknowledgment follows transfer.

Primary sources:

`https://docs.rs/indicatif/0.18.6/src/indicatif/iter.rs.html`

`https://docs.rs/indicatif/0.18.6/indicatif/struct.ProgressBarIter.html`

## 5. Terminal output errors and rendering reliability

### 5.1 Fallible terminal operations do not imply uniformly fallible progress APIs

The `TermLike` abstraction exposes fallible methods for terminal operations, including cursor movement, writing strings and lines, clearing lines, and flushing.

These methods return `io::Result<()>`.

Internally, Indicatif's drawing machinery can propagate such errors through its lower-level rendering operations. However, higher-level public APIs do not consistently expose those errors to their callers.

The important distinction is between the fallibility of the underlying terminal operation and the error contract of the API through which rendering is requested.

| Public operation               | Return type        | Relevant 0.18.6 behavior                                 |
| ------------------------------ | ------------------ | -------------------------------------------------------- |
| `ProgressBar::println()`       | `()`               | Internal drawing errors are discarded                    |
| `ProgressBar::force_draw()`    | `()`               | Discards the result of the forced draw                   |
| Ordinary progress updates      | `()`               | State-update rendering failures can be discarded         |
| `MultiProgress::println()`     | `io::Result<()>`   | Propagates group rendering errors                        |
| `MultiProgress::clear()`       | `io::Result<()>`   | Propagates clear errors                                  |
| `ProgressBar::suspend()`       | Closure result `R` | Standalone rendering errors are suppressed               |
| `MultiProgress::suspend()`     | Closure result `R` | Internal clear/redraw errors are unwrapped and can panic |
| `ProgressBar::set_tab_width()` | `()`               | Forces a redraw and unwraps its result                   |

The last two behaviors are source-level details worth recognizing when custom `TermLike` implementations can return actual I/O errors.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

source+2



### 5.2 Rendering should be treated as best-effort presentation

The immediate consequence is that a progress-state update does not prove successful delivery of its visual representation.

There are several distinct reasons:

- Rendering may be suppressed because the draw target is hidden.
- The redraw limiter may skip an attempted refresh.
- A write may fail without the public progress operation returning an error.
- Some APIs may instead propagate or panic on terminal errors.

Even a successful `MultiProgress::println()` can be a no-op on a hidden draw target.

Therefore, a custom rendering destination should not treat Indicatif operations as reliable event acknowledgments.

For integrations where output failures must be observable, the application must consider the error contract of the selected presentation API and the implementation of its terminal destination.

This is a boundary of Indicatif's progress-rendering model rather than a defect in its underlying progress counters.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



### 5.3 Additional `TermLike` geometry behavior

`TermLike` provides a default implementation of `height()` returning 20 rows, while `width()` must be supplied by the implementor.

This is a useful detail for custom terminal-like destinations: relying on the default height may provide an inaccurate model of the actual output surface.

The source identifies this default as a compatibility accommodation that may change in a future major release. It should not be assumed to reflect the dimensions of a real terminal or virtual display.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs



Primary sources:

`https://docs.rs/indicatif/0.18.6/indicatif/trait.TermLike.html`

`https://docs.rs/indicatif/0.18.6/src/indicatif/state.rs.html`

`https://docs.rs/indicatif/0.18.6/src/indicatif/multi.rs.html`

## 6. Additional formatting and presentation APIs

### 6.1 `style::write_ansi_range`: ANSI-preserving text extraction

Indicatif exposes a public function that is not discussed in the original guide:

```
indicatif::style::write_ansi_range
```

Its signature is conceptually:

```
pub fn write_ansi_range(
    formatter: &mut std::fmt::Formatter,
    text: &str,
    start: usize,
    end: usize,
) -> std::fmt::Result
```

The function writes the requested visible range of text while preserving ANSI escape sequences.

This differs from ordinary string slicing, where byte offsets do not correspond directly to terminal columns and embedded ANSI control sequences can distort naive length calculations.

Its implementation uses `console::AnsiCodeIterator` to distinguish ANSI sequences from visible text. ANSI sequences are written unchanged, while printable characters are selected according to their accumulated display width.

When the `unicode-width` feature is enabled, the function uses Unicode character-width information. Otherwise, it treats each non-ANSI character as width one.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



The function is useful for implementing display-oriented truncation or partial rendering in custom `Display` implementations. Indicatif itself uses it when formatting certain padded and truncated template values.

Its scope has several important limitations.

First, the function accepts a `std::fmt::Formatter`, not an arbitrary `std::io::Write` or `std::fmt::Write`. It is therefore most naturally used inside Rust formatting implementations.

Second, the implementation preserves ANSI sequences even when they occur outside the selected visible-character range. It is not an ANSI sanitizer or a general-purpose mechanism for balancing terminal styles.

Third, its iteration is character-oriented, with width measured per Unicode scalar value. This does not establish perfect grapheme-cluster-aware substring behavior for every complex Unicode sequence or terminal emulator.

It is best understood as a specialized ANSI-aware visible-range formatter, not as a full terminal-text layout engine.

Source:

`https://docs.rs/indicatif/0.18.6/indicatif/style/fn.write_ansi_range.html`

### 6.2 Tab expansion is configurable but not tab-stop alignment

`ProgressBar` provides:

- `with_tab_width(usize)`
- `set_tab_width(usize)`
- `tab_width()`

The default tab width is eight spaces.

The underlying `TabExpandedString` implementation stores the original text and expands tab characters lazily.

The expansion uses a direct replacement operation equivalent to replacing each `\t` with exactly the configured number of spaces.

This is not conventional tab-stop expansion, where the number of inserted spaces depends on the current column.

For example, a tab width of four replaces a tab with four spaces whether the preceding text occupies one column or three columns.

The configured width applies to the bar's stored message and prefix, and the style's formatting path is updated as well.

`set_tab_width()` additionally forces a redraw in the current implementation; `with_tab_width()` provides a builder-like configuration path.

The difference matters when using tabs to align labels of different lengths. Indicatif's setting provides consistent replacement width, not automatic alignment to shared terminal columns.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

source+2



Primary sources:

`https://docs.rs/indicatif/0.18.6/indicatif/struct.ProgressBar.html`

`https://docs.rs/indicatif/0.18.6/src/indicatif/state.rs.html`

### 6.3 Style introspection and spinner-state access

Two additional public facilities make existing presentation configuration reusable.

`ProgressBar::style()` returns a clone of the bar's current `ProgressStyle`. This can be useful when coordinating presentation across separately constructed bars without independently reconstructing their style configuration.

`ProgressStyle` also provides:

| Method                 | Behavior                                                              |
| ---------------------- | --------------------------------------------------------------------- |
| `get_tick_str(idx)`    | Returns the spinner string selected for the supplied tick index       |
| `get_final_tick_str()` | Returns the configured spinner string reserved for the finished state |

The non-final tick lookup cycles through all entries except the last, while the final-state lookup returns the last entry.

The internal rendering logic selects the finished-state tick string according to `ProgressState::is_finished()`.

This means the completed spinner glyph follows Indicatif's presentation lifecycle rather than independently evaluating whether the bar's numeric position equals its total. An abandoned bar is considered finished for this purpose.

These accessors provide a small but useful bridge between Indicatif's configured animation style and other presentation logic.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+1



Source:

`https://docs.rs/indicatif/0.18.6/indicatif/style/struct.ProgressStyle.html`

## 7. Additional adapter configuration and hidden-state construction

### 7.1 `ProgressBar::wrap_iter` attaches an existing progress handle

In addition to the extension methods on `ProgressIterator`, Indicatif provides:

```
ProgressBar::wrap_iter(&self, iterator)
```

The implementation delegates to the iterator's `progress_with` method, supplying a clone of the existing progress handle.

This makes it possible to attach progress instrumentation after constructing a bar with its desired length, style, message, finish policy, or group membership.

The cloned handle refers to the same shared bar state, not a new independently configured progress indicator.

This is particularly useful when progress presentation must be configured separately from the iterator's construction.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs



### 7.2 Adapter-level configuration preserves shared identity

`ProgressBarIter<T>` also provides builder-like methods corresponding to several `ProgressBar` configuration operations:

`with_style`, `with_prefix`, `with_message`, `with_position`, `with_elapsed`, and `with_finish`.

These methods configure the underlying shared progress bar.

They should not be interpreted as creating independent copies of its progress state. If another strong handle refers to the same bar, the configuration changes can be observed through that handle as well.

The distinction can be useful for building reusable instrumentation layers without separately maintaining a configuration object and synchronized progress metadata.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+1



Primary sources:

`https://docs.rs/indicatif/0.18.6/indicatif/struct.ProgressBar.html`

`https://docs.rs/indicatif/0.18.6/indicatif/struct.ProgressBarIter.html`

### 7.3 Hidden progress can retain a known total

`ProgressBar::hidden()` creates a progress indicator without a known length and with rendering disabled.

However, hidden presentation and unknown total length are separate properties.

A hidden bar with a known total can be constructed directly:

```
use indicatif::{ProgressBar, ProgressDrawTarget};

let bar = ProgressBar::with_draw_target(
    Some(1_000),
    ProgressDrawTarget::hidden(),
);
```

This preserves a meaningful total and allows progress updates and queries while suppressing rendering.

Alternatively, a visible bar can have its draw target changed to `ProgressDrawTarget::hidden()` without discarding its progress state.

Indicatif also provides `ProgressBar::is_hidden()` and `MultiProgress::is_hidden()`. These are useful for determining whether the configured presentation destination is currently hidden, including when visibility is inherited from a multi-bar group's draw target.

A hidden bar is still a progress-state object; invisibility does not imply that its counters or timing information cease to exist.

This separation can support optional terminal presentation without requiring an application to implement a parallel progress-state model.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



Primary source:

`https://docs.rs/indicatif/0.18.6/indicatif/struct.ProgressBar.html`

## 8. Completion fractions, ETA sentinel values, and timing semantics

### 8.1 Numerical completion has defined edge-case behavior

`ProgressState::fraction()` returns a floating-point completion fraction clamped to the range `[0.0, 1.0]`.

The tagged implementation handles several cases explicitly:

| Progress state                       | Fraction returned     |
| ------------------------------------ | --------------------- |
| Unknown total length                 | `0.0`                 |
| Known zero length                    | `1.0`                 |
| Position zero with a positive length | `0.0`                 |
| Position between zero and total      | `position / length`   |
| Position greater than total          | `1.0`, after clamping |

This means `fraction()` is a normalized presentation quantity, not an uncapped ratio or a reliable indicator of lifecycle completion.

For example, a bar with zero length can report `fraction() == 1.0` while `is_finished()` is still false.

Similarly, a position beyond the declared total is not represented as a fraction greater than one.

Applications requiring the actual position-to-length relationship should use the underlying position and length methods rather than infer it from the clamped fraction.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

source



### 8.2 A zero ETA can mean that no useful estimate exists

`ProgressState::eta()` returns `Duration::ZERO` in several cases:

- The bar is finished.
- Its total length is unknown.
- Its estimated progress rate is zero.
- The calculated remaining work is zero.

These outcomes have different meanings.

In particular, a zero ETA for an unfinished bar with no length or no established rate does not indicate that completion is imminent.

The implementation uses zero as a fallback value for certain unavailable or uninformative estimates rather than returning `Option<Duration>`.

Likewise, `ProgressState::duration()` returns zero while an unbounded bar is still in progress. This is distinct from the elapsed runtime reported by `elapsed()`.

This is especially relevant when custom formatting or external consumers attempt to interpret Indicatif's timing values as structured estimates rather than human-readable presentation metrics.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

source+1



### 8.3 Elapsed time and completed duration are not identical

The source also distinguishes elapsed wall-clock time from the duration captured at completion.

For an unfinished bar with a known total, `duration()` estimates total runtime using elapsed time plus ETA.

For a finished bar, `duration()` returns the duration recorded when the bar transitioned to its finished state.

In contrast, `elapsed()` computes elapsed time relative to the original start instant, including when the bar has already finished.

An additional implementation detail concerns `per_sec()`: for an active bar it uses the smoothed rate estimator, while for a finished bar it divides the current position by elapsed time since the start instant.

Consequently, the value returned by `per_sec()` after completion is not necessarily an immutable snapshot of the rate at the moment of finishing. It can change if queried later.

These are source-observed semantics of the 0.18.6 implementation. For external reporting that requires immutable completion metrics, captured completion values are conceptually different from querying an object that continues to reference the current clock.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

source+1



Primary sources:

`https://docs.rs/indicatif/0.18.6/indicatif/struct.ProgressState.html`

`https://docs.rs/indicatif/0.18.6/src/indicatif/state.rs.html`

## 9. Platform compatibility clarification

The Indicatif crate documentation explicitly identifies Linux, macOS, and Windows as supported operating systems.

For Windows, the documentation notes that terminal color support requires Windows 10.

This is separate from the Cargo feature requirements for Tokio, Futures, Unicode handling, and WebAssembly. Platform support does not imply that every output destination behaves identically, since terminal capabilities and automatic interactive-terminal detection remain environment-dependent.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



Primary source:

`https://docs.rs/indicatif/0.18.6/indicatif/`

## Addendum synthesis

The additions clarify several consequential aspects of Indicatif's architecture without changing the broader capability model established in the original reference.

The most significant newly documented mechanism is its hierarchy of update and rendering controls. Atomic counters allow frequent state mutations, while separate admission and redraw controls reduce the amount of work required to maintain an interactive display.

The lifecycle analysis establishes that explicit completion, iterator exhaustion, final-handle destruction, and multi-bar persistence are different events with different implications. These distinctions are important whenever progress presentation must survive changes in ownership or accurately represent incomplete work.

The adapter analysis clarifies that Indicatif can preserve a broader collection of I/O traits than its high-level wrapper methods initially suggest, while its counters remain tied to precise observation boundaries. Buffered reads, accepted writes, seeking, and EOF detection do not all represent the same notion of completed work.

Finally, the formatting, terminal-error, and timing details expose capabilities and limitations that matter when treating Indicatif as an extensible rendering component rather than merely a convenient progress-bar widget.

These additions are specific to Indicatif 0.18.6, with internal implementation observations distinguished from stable public API behavior. They complement the existing guide's coverage without requiring changes to its overall organization.