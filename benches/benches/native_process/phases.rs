// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bounded observations of synchronous production spans, including cache misses only.
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
    time::Instant,
};
use tracing::{
    Subscriber,
    span::{Attributes, Id},
};
use tracing_subscriber::{Layer, layer::Context, registry::LookupSpan};

#[derive(Clone, Default)]
pub(super) struct Phases(Arc<Mutex<Totals>>);
type Totals = BTreeMap<&'static str, (u64, f64)>;
struct Started(Instant);
impl Phases {
    pub(super) fn reset(&self) {
        self.0.lock().unwrap().clear();
    }
    pub(super) fn report(&self, attempts: u64) -> serde_json::Value {
        serde_json::Value::Object(self.0.lock().unwrap().iter().map(|(name, (calls, seconds))| {
            ((*name).into(), serde_json::json!({"calls":calls, "total_seconds":seconds, "seconds_per_attempt":seconds/attempts as f64}))
        }).collect())
    }
}
impl<S: Subscriber + for<'a> LookupSpan<'a>> Layer<S> for Phases {
    fn on_new_span(&self, attributes: &Attributes<'_>, id: &Id, context: Context<'_, S>) {
        if attributes.metadata().name().starts_with("pse.case.") {
            context
                .span(id)
                .unwrap()
                .extensions_mut()
                .insert(Started(Instant::now()));
        }
    }
    fn on_close(&self, id: Id, context: Context<'_, S>) {
        let span = context.span(&id).unwrap();
        if let Some(started) = span.extensions().get::<Started>() {
            let mut totals = self.0.lock().unwrap();
            let total = totals.entry(span.name()).or_default();
            total.0 += 1;
            total.1 += started.0.elapsed().as_secs_f64();
        }
    }
}
