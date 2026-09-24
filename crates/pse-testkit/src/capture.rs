// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Bounded local capture with explicit inconclusive results for missing evidence.
use std::{
    collections::BTreeMap,
    fmt,
    sync::{Arc, Mutex},
};
use tracing::{
    Subscriber,
    field::{Field, Visit},
    span::{Attributes, Id, Record},
};
use tracing_subscriber::{Layer, layer::Context, prelude::*, registry::LookupSpan};

/// Observed native span fields; these are diagnostics, not input/output authority.
#[derive(Clone, Debug)]
pub struct SpanEvidence {
    /// Capture-local span identity.
    pub id: u64,
    /// Actual captured parent, including propagated task context.
    pub parent: Option<u64>,
    /// Static native span name.
    pub name: &'static str,
    /// Declared fields recorded by the native instrumentation.
    pub fields: BTreeMap<String, String>,
    /// Whether all references to this span have closed.
    pub closed: bool,
}
#[derive(Debug, Default)]
struct State {
    spans: BTreeMap<u64, SpanEvidence>,
    bytes: usize,
    truncated: bool,
}
/// A bounded capture layer. Each fixture installs its own local Dispatch.
#[derive(Clone, Debug)]
pub struct Capture {
    state: Arc<Mutex<State>>,
    max_spans: usize,
    max_bytes: usize,
    coverage: Coverage,
}

/// Completeness of the requested observation, independent of value correctness.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Coverage {
    /// Observation was intentionally disabled.
    Disabled,
    /// Every requested event is eligible for capture.
    Complete,
    /// The caller selected only a sample of events.
    Sampled,
    /// A storage/formatting limit refused evidence.
    Truncated,
    /// An operation or capture did not reach completion.
    Interrupted,
}
impl Capture {
    /// Set hard entry and retained-text bounds; overflow makes assurance inconclusive.
    pub fn new(max_spans: usize, max_bytes: usize) -> Self {
        Self {
            state: Arc::default(),
            max_spans,
            max_bytes,
            coverage: Coverage::Complete,
        }
    }
    /// Declare filtered/sampled capture; it cannot establish universal absence.
    #[must_use]
    pub fn with_coverage(mut self, coverage: Coverage) -> Self {
        self.coverage = coverage;
        self
    }
    /// Actual coverage including capture refusal and unfinished span lifetimes.
    pub fn coverage(&self) -> Coverage {
        if self.coverage != Coverage::Complete {
            return self.coverage;
        }
        let Ok(state) = self.state.lock() else {
            return Coverage::Interrupted;
        };
        if state.truncated {
            Coverage::Truncated
        } else if state.spans.values().any(|span| !span.closed) {
            Coverage::Interrupted
        } else {
            Coverage::Complete
        }
    }
    /// Unfiltered local subscriber. Use `WithSubscriber` for asynchronous work.
    pub fn dispatch(&self) -> tracing::Dispatch {
        tracing::Dispatch::new(tracing_subscriber::registry().with(self.clone()))
    }
    /// Bounded evidence snapshot and its completeness flag.
    pub fn snapshot(&self) -> (Vec<SpanEvidence>, bool) {
        match self.state.lock() {
            Ok(state) => (state.spans.values().cloned().collect(), state.truncated),
            Err(_) => (vec![], true),
        }
    }
    /// Assess one explicitly identified operation against observed native node names.
    /// This establishes execution structure only, never domain result correctness.
    pub fn assess(&self, operation_id: u64, expected_nodes: &[&str]) -> Assurance {
        if self.coverage != Coverage::Complete {
            return Assurance::Inconclusive("capture disabled, sampled or interrupted");
        }
        let (spans, truncated) = self.snapshot();
        if truncated {
            return Assurance::Inconclusive("capture truncated or unavailable");
        }
        let Some(root) = spans.iter().find(|s| {
            s.name == "pse.operation"
                && s.fields
                    .get("operation_id")
                    .is_some_and(|id| id == &operation_id.to_string())
        }) else {
            return Assurance::Inconclusive("operation root missing");
        };
        if root
            .fields
            .get("task_propagation")
            .is_none_or(|value| value != "Complete")
        {
            return Assurance::Inconclusive("task propagation unproved");
        }
        if !root.closed {
            return Assurance::Inconclusive("operation remains open");
        }
        match root.fields.get("terminal").map(String::as_str) {
            Some("Completed") => (),
            Some("Failed" | "Cancelled" | "Abandoned") => {
                return Assurance::Violated("operation did not complete");
            }
            _ => return Assurance::Inconclusive("terminal outcome missing"),
        }
        let descendant = |span: &SpanEvidence| {
            let mut parent = span.parent;
            for _ in 0..spans.len() {
                if parent == Some(root.id) {
                    return true;
                }
                parent = parent
                    .and_then(|id| spans.iter().find(|s| s.id == id))
                    .and_then(|s| s.parent);
                if parent.is_none() {
                    break;
                }
            }
            false
        };
        let children: Vec<_> = spans.iter().filter(|s| descendant(s)).collect();
        if children.iter().any(|s| !s.closed) {
            return Assurance::Inconclusive("native children remain open");
        }
        for expected in expected_nodes {
            if !children.iter().any(|s| {
                s.fields
                    .get("otel.name")
                    .is_some_and(|name| name == expected)
            }) {
                return Assurance::Inconclusive("required native operator absent");
            }
        }
        if !children.iter().any(|s| s.name == "InstrumentedExec") {
            return Assurance::Inconclusive("execution spans missing");
        }
        Assurance::Established
    }
}
/// Evidence qualification, separate from functional correctness.
#[derive(Debug, PartialEq, Eq)]
pub enum Assurance {
    /// The requested execution facts were observed in a complete local capture.
    Established,
    /// Complete captured evidence contradicts an execution requirement.
    Violated(&'static str),
    /// Missing, filtered, unfinished or truncated evidence cannot establish the claim.
    Inconclusive(&'static str),
}
struct Fields<'a> {
    state: &'a mut State,
    id: u64,
    limit: usize,
}
impl Visit for Fields<'_> {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        // Format directly into a bounded writer; even one giant Debug field must
        // not allocate beyond the capture allowance before truncation is noticed.
        use fmt::Write;
        let remaining = self.limit.saturating_sub(self.state.bytes);
        let mut text = BoundedText {
            text: String::new(),
            limit: remaining,
            truncated: false,
        };
        let _ = write!(&mut text, "{value:?}");
        if text.truncated {
            self.state.truncated = true;
            return;
        }
        self.store(field, text.text);
    }
    fn record_str(&mut self, field: &Field, value: &str) {
        if value.len() > self.limit.saturating_sub(self.state.bytes) {
            self.state.truncated = true;
            return;
        }
        self.store(field, value.to_owned());
    }
}
impl Fields<'_> {
    fn store(&mut self, field: &Field, value: String) {
        let cost = field
            .name()
            .len()
            .saturating_add(value.len())
            .saturating_add(96);
        if cost > self.limit.saturating_sub(self.state.bytes) {
            self.state.truncated = true;
            return;
        }
        self.state.bytes += cost; // conservative: replacement does not refund capacity
        if let Some(span) = self.state.spans.get_mut(&self.id) {
            span.fields.insert(field.name().to_owned(), value);
        }
    }
}
struct BoundedText {
    text: String,
    limit: usize,
    truncated: bool,
}
impl fmt::Write for BoundedText {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        if text.len() > self.limit.saturating_sub(self.text.len()) {
            self.truncated = true;
            return Err(fmt::Error);
        }
        self.text.push_str(text);
        Ok(())
    }
}
impl<S: Subscriber + for<'a> LookupSpan<'a>> Layer<S> for Capture {
    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, context: Context<'_, S>) {
        if self.coverage == Coverage::Disabled {
            return;
        }
        let Ok(mut state) = self.state.lock() else {
            return;
        };
        if state.spans.len() >= self.max_spans || self.max_bytes.saturating_sub(state.bytes) < 256 {
            state.truncated = true;
            return;
        }
        let parent = attrs.parent().map(Id::into_u64).or_else(|| {
            if attrs.is_contextual() {
                context.current_span().id().map(Id::into_u64)
            } else {
                None
            }
        });
        state.bytes += 256;
        state.spans.insert(
            id.into_u64(),
            SpanEvidence {
                id: id.into_u64(),
                parent,
                name: attrs.metadata().name(),
                fields: BTreeMap::new(),
                closed: false,
            },
        );
        attrs.record(&mut Fields {
            state: &mut state,
            id: id.into_u64(),
            limit: self.max_bytes,
        });
    }
    fn on_record(&self, id: &Id, values: &Record<'_>, _context: Context<'_, S>) {
        if let Ok(mut state) = self.state.lock() {
            values.record(&mut Fields {
                state: &mut state,
                id: id.into_u64(),
                limit: self.max_bytes,
            });
        }
    }
    fn on_close(&self, id: Id, _context: Context<'_, S>) {
        if let Ok(mut state) = self.state.lock()
            && let Some(span) = state.spans.get_mut(&id.into_u64())
        {
            span.closed = true;
        }
    }
}
