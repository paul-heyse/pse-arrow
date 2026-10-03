// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bounded observations of production spans, including cache misses and Delta writes.
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
    time::Instant,
};
use tracing::{
    Subscriber,
    field::{Field, Visit},
    span::{Attributes, Id, Record},
};
use tracing_subscriber::{Layer, layer::Context, registry::LookupSpan};

#[derive(Clone, Default)]
pub(super) struct Phases(Arc<Mutex<Totals>>);
#[derive(Default)]
struct Totals {
    phases: BTreeMap<&'static str, (u64, f64)>,
    constructions: BTreeMap<String, Construction>,
}
#[derive(Default)]
struct Construction {
    attempts: u64,
    successes: u64,
    reuses: u64,
    derivative_operations: u64,
}
struct Started {
    at: Instant,
    fields: Fields,
}
#[derive(Default)]
struct Fields(BTreeMap<String, serde_json::Value>);
impl Visit for Fields {
    fn record_str(&mut self, field: &Field, value: &str) {
        self.0.insert(field.name().into(), value.into());
    }
    fn record_bool(&mut self, field: &Field, value: bool) {
        self.0.insert(field.name().into(), value.into());
    }
    fn record_u64(&mut self, field: &Field, value: u64) {
        self.0.insert(field.name().into(), value.into());
    }
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        self.0
            .insert(field.name().into(), format!("{value:?}").into());
    }
}
impl Phases {
    pub(super) fn reset(&self) {
        *self.0.lock().unwrap() = Totals::default();
    }
    pub(super) fn report(&self, attempts: u64) -> serde_json::Value {
        serde_json::Value::Object(self.0.lock().unwrap().phases.iter().map(|(name, (calls, seconds))| {
            ((*name).into(), serde_json::json!({"calls":calls, "total_seconds":seconds, "seconds_per_attempt":seconds/attempts as f64}))
        }).collect())
    }
    /// Actual construction boundaries, rather than structural preparation request counts.
    pub(super) fn constructions(&self) -> serde_json::Value {
        serde_json::Value::Object(self.0.lock().unwrap().constructions.iter().map(|(key, count)| {
            (key.clone(), serde_json::json!({"attempts":count.attempts,"successes":count.successes,"reuses":count.reuses,"derivative_operations":count.derivative_operations}))
        }).collect())
    }
}
impl<S: Subscriber + for<'a> LookupSpan<'a>> Layer<S> for Phases {
    fn on_new_span(&self, attributes: &Attributes<'_>, id: &Id, context: Context<'_, S>) {
        if attributes.metadata().name().starts_with("pse.case.")
            || attributes.metadata().name() == "pse.delta.write_attempt"
        {
            let mut fields = Fields::default();
            attributes.record(&mut fields);
            context.span(id).unwrap().extensions_mut().insert(Started {
                at: Instant::now(),
                fields,
            });
        }
    }
    fn on_record(&self, id: &Id, values: &Record<'_>, context: Context<'_, S>) {
        if let Some(span) = context.span(id)
            && let Some(started) = span.extensions_mut().get_mut::<Started>()
        {
            values.record(&mut started.fields);
        }
    }
    fn on_close(&self, id: Id, context: Context<'_, S>) {
        let span = context.span(&id).unwrap();
        if let Some(started) = span.extensions().get::<Started>() {
            let mut totals = self.0.lock().unwrap();
            let total = totals.phases.entry(span.name()).or_default();
            total.0 += 1;
            total.1 += started.at.elapsed().as_secs_f64();
            let fields = &started.fields.0;
            if let (Some(product), Some(order)) = (
                fields.get("product").and_then(serde_json::Value::as_str),
                fields
                    .get("derivative_order")
                    .and_then(serde_json::Value::as_str),
            ) {
                let count = totals
                    .constructions
                    .entry(format!("{product}.{order}"))
                    .or_default();
                if fields.get("reused").and_then(serde_json::Value::as_bool) == Some(true) {
                    count.reuses += 1;
                } else {
                    count.attempts += 1;
                    count.successes += u64::from(
                        fields.get("success").and_then(serde_json::Value::as_bool) == Some(true),
                    );
                    count.derivative_operations += fields
                        .get("derivative_operations")
                        .and_then(serde_json::Value::as_u64)
                        .unwrap_or(0);
                }
            }
        }
    }
}
