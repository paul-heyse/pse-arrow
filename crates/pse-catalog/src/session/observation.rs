// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Actual analyzer/optimizer observations and the plan that reached physical planning.
use crate::CatalogError;
use datafusion::logical_expr::LogicalPlan;
use pse_ids::{MemoryReserver, Reservation, ReservationLease};
use std::{fmt::Write, sync::Arc};

const MAX_RULES: usize = u16::MAX as usize;
/// Retained diagnostic evidence; never a semantic validation or reuse certificate.
#[derive(Clone, Debug)]
pub struct PlanObservation {
    logical: Arc<ObservedPlan>,
    physical: Option<Arc<ObservedPhysical>>,
}
#[derive(Debug)]
struct ObservedPlan {
    explain: String,
    rules: Vec<String>,
    _lease: Arc<ReservationLease>,
}
#[derive(Debug)]
struct ObservedPhysical {
    explain: String,
    _lease: Arc<ReservationLease>,
}
impl PlanObservation {
    /// PostgreSQL JSON rendering of the actual optimized plan sent to physical planning.
    pub fn explain_pgjson(&self) -> &str {
        &self.logical.explain
    }
    /// Analyzer and logical optimizer callbacks in their actual execution order.
    pub fn rules_fired(&self) -> &[String] {
        &self.logical.rules
    }
    /// The actual native physical tree after its optimizer pipeline, when executed.
    /// A preparation alone has no physical observation and executes no data operators.
    pub fn physical_plan(&self) -> Option<&str> {
        self.physical.as_ref().map(|value| value.explain.as_str())
    }
    pub(super) fn with_physical(
        &self,
        plan: &dyn datafusion::physical_plan::ExecutionPlan,
        reserver: &dyn MemoryReserver,
    ) -> Result<Self, CatalogError> {
        let mut reservation = reserver.open("session:physical-plan-observation");
        let mut writer = BoundedText {
            value: String::new(),
            reservation: &mut reservation,
            error: None,
        };
        let rendered = write!(
            &mut writer,
            "{}",
            datafusion::physical_plan::displayable(plan).indent(true)
        );
        if let Some(error) = writer.error {
            return Err(error);
        }
        rendered.map_err(|_| invalid("physical plan rendering failed"))?;
        let explain = writer.value;
        Ok(Self {
            logical: Arc::clone(&self.logical),
            physical: Some(Arc::new(ObservedPhysical {
                explain,
                _lease: ReservationLease::new(reservation),
            })),
        })
    }
}
pub(super) struct Recorder {
    reservation: Box<dyn Reservation>,
    rules: Vec<String>,
    error: Option<CatalogError>,
}
impl Recorder {
    pub(super) fn new(reserver: &dyn MemoryReserver) -> Self {
        Self {
            reservation: reserver.open("session:plan-observation"),
            rules: Vec::new(),
            error: None,
        }
    }
    pub(super) fn rule(&mut self, name: &str) {
        if self.error.is_some() {
            return;
        }
        if self.rules.len() >= MAX_RULES {
            self.error = Some(invalid("observed rule count exceeds u16 evidence contract"));
            return;
        }
        let grow = name.len().saturating_add(2 * size_of::<String>());
        if let Err(error) = self.reservation.try_grow(grow) {
            self.error = Some(error.into());
            return;
        }
        self.rules.reserve_exact(1);
        self.rules.push(name.to_owned());
    }
    pub(super) fn finish(mut self, plan: &LogicalPlan) -> Result<PlanObservation, CatalogError> {
        if let Some(error) = self.error {
            return Err(error);
        }
        let mut writer = BoundedText {
            value: String::new(),
            reservation: &mut self.reservation,
            error: None,
        };
        let rendered = write!(&mut writer, "{}", plan.display_pg_json());
        if let Some(error) = writer.error {
            return Err(error);
        }
        rendered.map_err(|_| invalid("plan rendering failed"))?;
        let explain = writer.value;
        Ok(PlanObservation {
            logical: Arc::new(ObservedPlan {
                explain,
                rules: self.rules,
                _lease: ReservationLease::new(self.reservation),
            }),
            physical: None,
        })
    }
}
struct BoundedText<'a> {
    value: String,
    reservation: &'a mut Box<dyn Reservation>,
    error: Option<CatalogError>,
}
impl Write for BoundedText<'_> {
    fn write_str(&mut self, text: &str) -> std::fmt::Result {
        if let Err(error) = self.reservation.try_grow(text.len()) {
            self.error = Some(error.into());
            return Err(std::fmt::Error);
        }
        self.value.reserve_exact(text.len());
        self.value.push_str(text);
        Ok(())
    }
}
fn invalid(reason: &str) -> CatalogError {
    CatalogError::ConfigInvalid {
        key: "session.plan_evidence".to_owned(),
        reason: reason.to_owned(),
    }
}
