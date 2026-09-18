// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Actual analyzer/optimizer observations and the plan that reached physical planning.
use crate::CatalogError;
use datafusion::logical_expr::LogicalPlan;
use pse_ids::{MemoryReserver, Reservation, ReservationLease};
use std::{fmt::Write, sync::Arc};

mod graph;

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
    /// Native PostgreSQL JSON for each actual optimized node, in a flat diagnostic
    /// graph. `Root` and ordered `Edges` reference `Nodes[].Id`; each node's `Native`
    /// value is DataFusion's own JSON. Shared nodes have one definition. No live
    /// provider, buffer or execution owner is retained by this evidence.
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
    pub(super) fn finish(
        mut self,
        plan: &LogicalPlan,
        cancel: &pse_ids::CancellationToken,
    ) -> Result<PlanObservation, CatalogError> {
        if let Some(error) = self.error {
            return Err(error);
        }
        let mut writer = BoundedText {
            value: String::new(),
            reservation: &mut self.reservation,
            error: None,
        };
        let rendered = graph::write(plan, &mut writer, cancel);
        if let Some(error) = writer.error {
            return Err(error);
        }
        rendered?;
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
        let Some(required) = self.value.len().checked_add(text.len()) else {
            self.error = Some(invalid("plan rendering size overflow"));
            return Err(std::fmt::Error);
        };
        let current = self.value.capacity();
        if required > current {
            // Native display emits many short writes. Reserving exactly each
            // fragment repeatedly copies the entire accumulated plan. Charge
            // amortized capacity before allocation, with an exact-size fallback
            // when the remaining budget cannot admit speculative spare space.
            let preferred = required.checked_next_power_of_two().unwrap_or(required);
            let capacity = if self.reservation.try_grow(preferred - current).is_ok() {
                preferred
            } else {
                if let Err(error) = self.reservation.try_grow(required - current) {
                    self.error = Some(error.into());
                    return Err(std::fmt::Error);
                }
                required
            };
            if self
                .value
                .try_reserve_exact(capacity - self.value.len())
                .is_err()
            {
                self.error = Some(invalid("plan rendering allocation failed"));
                return Err(std::fmt::Error);
            }
            if self.value.capacity() > capacity
                && let Err(error) = self.reservation.try_grow(self.value.capacity() - capacity)
            {
                self.error = Some(error.into());
                return Err(std::fmt::Error);
            }
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_json_reports_each_extension_once_without_recursive_debug_payloads() {
        use datafusion::execution::session_state::{CacheFactory, SessionStateBuilder};
        let state = SessionStateBuilder::new().with_default_features().build();
        let mut plan = datafusion::logical_expr::LogicalPlanBuilder::empty(true)
            .build()
            .unwrap();
        for _ in 0..8 {
            plan = crate::session::cache::NativeCacheFactory
                .create(plan, &state)
                .unwrap();
        }
        let budget = pse_ids::FixedBudget::new(64 << 10);
        let observation = Recorder::new(budget.as_ref())
            .finish(&plan, &pse_ids::CancellationToken::new())
            .unwrap();
        let json: serde_json::Value = serde_json::from_str(observation.explain_pgjson()).unwrap();
        assert_eq!(json["Nodes"].as_array().unwrap().len(), 17);
        assert_eq!(json["Edges"].as_array().unwrap().len(), 16);
        for node in &json["Nodes"].as_array().unwrap()[..16] {
            let native = &node["Native"][0]["Plan"];
            assert!(native["Detail"].as_str().unwrap().len() < 160);
            assert!(native["Plans"].as_array().unwrap().is_empty());
        }
        assert_eq!(
            json["Nodes"][16]["Native"][0]["Plan"]["Node Type"],
            "EmptyRelation"
        );
        drop(observation);
        assert_eq!(budget.reserved(), 0);
    }

    #[test]
    fn shared_producer_json_has_complete_definitions_and_explicit_references() {
        use datafusion::{
            execution::session_state::{CacheFactory, SessionStateBuilder},
            logical_expr::{LogicalPlanBuilder, Union},
        };
        let state = SessionStateBuilder::new().with_default_features().build();
        let mut plan = LogicalPlanBuilder::empty(true).build().unwrap();
        for _ in 0..12 {
            let cached = Arc::new(
                super::super::cache::NativeCacheFactory
                    .create(plan, &state)
                    .unwrap(),
            );
            plan = LogicalPlan::Union(Union {
                inputs: vec![Arc::clone(&cached), Arc::clone(&cached)],
                schema: Arc::clone(cached.schema()),
            });
        }
        let budget = pse_ids::FixedBudget::new(128 << 10);
        let observation = Recorder::new(budget.as_ref())
            .finish(&plan, &pse_ids::CancellationToken::new())
            .unwrap();
        let json: serde_json::Value = serde_json::from_str(observation.explain_pgjson()).unwrap();
        let nodes = json["Nodes"].as_array().unwrap();
        let edges = json["Edges"].as_array().unwrap();
        assert_eq!(nodes.len(), 37);
        assert_eq!(edges.len(), 48);
        assert_eq!(
            nodes
                .iter()
                .filter(|node| node["Native"][0]["Plan"]["Node Type"] == "NativeCache")
                .count(),
            12
        );
        for edge in edges {
            assert!(edge[0].as_u64().unwrap() < nodes.len() as u64);
            assert!(edge[2].as_u64().unwrap() < nodes.len() as u64);
        }
        assert!(
            restore_graph(&json, 0)
                == serde_json::from_str::<serde_json::Value>(&plan.display_pg_json().to_string())
                    .unwrap()[0]["Plan"],
            "flat evidence must reconstruct the native tree including its output fields"
        );
        assert!(observation.explain_pgjson().len() < 32 << 10);
        drop(observation);
        assert_eq!(budget.reserved(), 0);
    }

    fn restore_graph(graph: &serde_json::Value, id: usize) -> serde_json::Value {
        let mut node = graph["Nodes"][id]["Native"][0]["Plan"].clone();
        let mut edges: Vec<_> = graph["Edges"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|edge| usize::try_from(edge[0].as_u64().unwrap()).unwrap() == id)
            .collect();
        edges.sort_by_key(|edge| edge[1].as_u64().unwrap());
        node["Plans"] = edges
            .iter()
            .map(|edge| restore_graph(graph, usize::try_from(edge[2].as_u64().unwrap()).unwrap()))
            .collect::<Vec<_>>()
            .into();
        node
    }

    #[test]
    fn deep_observation_is_linear_and_releases_every_live_plan_owner() {
        use datafusion::{
            execution::session_state::{CacheFactory, SessionStateBuilder},
            logical_expr::{LogicalPlanBuilder, Projection},
        };
        let state = SessionStateBuilder::new().with_default_features().build();
        let mut plan = super::super::cache::NativeCacheFactory
            .create(LogicalPlanBuilder::empty(true).build().unwrap(), &state)
            .unwrap();
        let LogicalPlan::Extension(extension) = &plan else {
            panic!("expected native extension")
        };
        let owner = Arc::downgrade(&extension.node);
        for _ in 0..512 {
            plan = LogicalPlan::Projection(Projection::try_new(vec![], Arc::new(plan)).unwrap());
        }
        let budget = pse_ids::FixedBudget::new(512 << 10);
        let observation = Recorder::new(budget.as_ref())
            .finish(&plan, &pse_ids::CancellationToken::new())
            .unwrap();
        let graph: serde_json::Value = serde_json::from_str(observation.explain_pgjson()).unwrap();
        assert_eq!(graph["Nodes"].as_array().unwrap().len(), 515);
        assert_eq!(graph["Edges"].as_array().unwrap().len(), 514);
        assert!(observation.explain_pgjson().len() < 128 << 10);
        drop(plan);
        assert!(
            owner.upgrade().is_none(),
            "diagnostics must not pin live native owners"
        );
        drop(observation);
        assert_eq!(budget.reserved(), 0);
    }

    #[test]
    fn native_subquery_edges_and_resource_refusals_remain_observable() {
        use datafusion::logical_expr::{LogicalPlanBuilder, expr_fn::scalar_subquery, lit};
        let subquery = Arc::new(
            LogicalPlanBuilder::empty(true)
                .project(vec![lit(7_i64)])
                .unwrap()
                .build()
                .unwrap(),
        );
        let plan = LogicalPlanBuilder::empty(true)
            .project(vec![
                scalar_subquery(Arc::clone(&subquery)).alias("a"),
                scalar_subquery(subquery).alias("b"),
            ])
            .unwrap()
            .build()
            .unwrap();
        let cancel = pse_ids::CancellationToken::new();
        let budget = pse_ids::FixedBudget::new(64 << 10);
        let observation = Recorder::new(budget.as_ref())
            .finish(&plan, &cancel)
            .unwrap();
        let graph: serde_json::Value = serde_json::from_str(observation.explain_pgjson()).unwrap();
        let expected: serde_json::Value =
            serde_json::from_str(&plan.display_pg_json().to_string()).unwrap();
        assert_eq!(restore_graph(&graph, 0), expected[0]["Plan"]);
        drop(observation);
        assert_eq!(budget.reserved(), 0);
        cancel.cancel();
        assert!(
            Recorder::new(budget.as_ref())
                .finish(&plan, &cancel)
                .is_err()
        );
        assert_eq!(budget.reserved(), 0);
        let tiny = pse_ids::FixedBudget::new(100);
        assert!(
            Recorder::new(tiny.as_ref())
                .finish(&plan, &pse_ids::CancellationToken::new())
                .is_err()
        );
        assert_eq!(tiny.reserved(), 0);
    }

    #[test]
    fn diagnostic_fragments_charge_capacity_and_use_the_exact_budget_tail() {
        let budget = pse_ids::FixedBudget::new(100_003);
        let mut reservation = budget.open("test:observation");
        {
            let mut writer = BoundedText {
                value: String::new(),
                reservation: &mut reservation,
                error: None,
            };
            for _ in 0..10_000 {
                writer.write_str("1234567890").unwrap();
            }
            assert_eq!(writer.value.len(), 100_000);
            assert_eq!(budget.reserved(), writer.value.capacity());
            writer.write_str("end").unwrap();
            assert_eq!(writer.value.len(), 100_003);
            assert!(writer.value.ends_with("end"));
            assert!(writer.write_str("!").is_err());
            assert_eq!(writer.value.len(), 100_003);
        }
        drop(reservation);
        assert_eq!(budget.reserved(), 0);
    }
}
