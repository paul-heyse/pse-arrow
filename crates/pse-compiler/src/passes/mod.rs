// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Typed domain algorithms behind native plan operators. Facts are owned Arrow
//! values with exact source selections, never restored stage snapshots.

pub mod bundle;
pub(crate) mod native_construction;
mod native_graph;
pub(crate) mod native_outputs;
pub(crate) mod native_rows;
pub(crate) mod native_sources;
#[cfg(test)]
pub(crate) mod native_test;
pub mod p10;
pub mod p3;
pub mod p4;
pub mod p5;
pub mod p6;
pub mod p7;
pub mod p8;
pub mod p9;
pub(crate) mod parameter_indices;

use std::collections::BTreeMap;

use pse_ids::{CancellationToken, MemoryReserver};
use pse_schema::Registry;
use pse_schema::model::AlgorithmSpec;
use std::sync::Arc;

use crate::error::CompilerError;

/// One finite domain algorithm with declared typed arguments and results.
pub trait Algorithm: std::fmt::Debug + Send + Sync {
    /// The declaration this pass implements.
    fn spec(&self) -> &AlgorithmSpec;

    /// Whether this finite algorithm consumes the declared physical quantity inventory.
    fn requires_physical(&self) -> bool {
        true
    }

    /// Run over actual bound inputs and return the catalog's producer result.
    /// The catalog executor checks complete output obligations before creating a
    /// publication completion. Native plans and Delta publications own execution and persistence.
    ///
    /// # Errors
    ///
    /// [`CompilerError`] — including the authoring, rule and registry errors it forwards
    /// transparently.
    fn run<'a>(
        &'a self,
        ctx: &'a AlgorithmContext<'a>,
        inputs: &'a AlgorithmInputs,
    ) -> pse_catalog::provider::BoxFut<'a, Result<AlgorithmOutput, CompilerError>>;
}

/// Everything a pass may read that is not one of its input ports (blueprint §14.3).
///
/// Deliberately small. A pass that reached for ambient state — the clock, an environment
/// variable, a global session — would not be a function of its declared inputs, and its
/// stage key would be a lie.
#[derive(Debug)]
pub struct AlgorithmContext<'a> {
    /// Shared physical algorithm inventory for this exact bound input set.
    /// Absent only in control-only contexts that do not run a physical pass.
    pub physical: Option<&'a Arc<crate::quantity_relations::PhysicalInventory>>,
    /// The registry every contract is read from.
    pub registry: &'a Arc<Registry>,
    /// Original documents reopened from the exact bound snapshot source artifacts.
    pub documents: &'a pse_authoring::document::OwnedDocumentSet,
    /// Cooperative cancellation at every bounded work boundary.
    pub cancel: &'a CancellationToken,
    /// The reservation provider used by store and session.
    pub reserver: &'a dyn MemoryReserver,
    /// Bound native environment required by every operation.
    pub session: &'a pse_catalog::session::SnapshotSession,
}
impl AlgorithmContext<'_> {
    /// Actual shared physical inventory established from this invocation's Arrow inputs.
    /// # Errors
    /// A physical pass was invoked without preparing its actual input inventory.
    pub fn physical(&self) -> Result<&crate::quantity_relations::PhysicalInventory, CompilerError> {
        self.physical
            .map(AsRef::as_ref)
            .ok_or_else(|| invalid("physical inventory absent from prepared pass context"))
    }
}

mod argument;
pub use argument::BoundInput;

/// Every declared input port of a pass, bound or explicitly absent.
#[derive(Clone, Debug, Default)]
pub struct AlgorithmInputs {
    /// Argument name to its binding. `None` is an explicit optional absence.
    pub ports: BTreeMap<String, Option<BoundInput>>,
}

impl AlgorithmInputs {
    /// An empty bundle.
    pub fn new() -> Self {
        Self::default()
    }

    /// The binding of `port`, distinguishing "absent" from "not declared".
    ///
    /// `Some(None)` is a declared port with no rows; `None` is a port this pass does not
    /// declare, which is a caller error rather than an empty input.
    pub fn port(&self, port: &str) -> Option<&Option<BoundInput>> {
        self.ports.get(port)
    }
}

/// A finite algorithm's typed outputs and co-located evidence. This is a transient
/// Arrow result consumed by its native operator, not a publication authority.
#[derive(Debug)]
pub struct AlgorithmOutput {
    /// Complete named declared relations, including explicit empty values.
    pub outputs: BTreeMap<String, pse_relations::columnar::FieldCheckedBatch>,
    /// Existing structured diagnostic relations.
    pub findings: Vec<pse_relations::RecordBatch>,
    /// Source-support evidence produced by actual native computations.
    pub derivations: Vec<pse_relations::RecordBatch>,
    /// Actual native plan observations.
    pub plans: Vec<pse_catalog::session::PlanObservation>,
}

pub(crate) fn invalid(reason: impl Into<String>) -> CompilerError {
    CompilerError::Internal {
        what: reason.into(),
    }
}

mod physical_subject;
