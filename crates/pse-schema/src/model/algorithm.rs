// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Typed signatures for finite native domain algorithms. Arguments are bound to
//! actual native plan children by the caller. There is no registry producer graph,
//! stage scheduler, stored stage identity, or reconstruction from an algorithm number.

use core::fmt;

use pse_ids::SemanticId;

/// How reproducible an algorithm is (blueprint §14.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Determinism {
    /// The same inputs give the same outputs, byte for byte.
    Deterministic,
    /// Deterministic as the least fixed point of its rules (P4, P6).
    DeterministicFixedPoint,
    /// Deterministic once a backend is chosen; different backends differ (P16).
    DeterministicPerBackend,
}

impl Determinism {
    /// Every kind, in blueprint §14.1 order.
    pub const ALL: [Self; 3] = [
        Self::Deterministic,
        Self::DeterministicFixedPoint,
        Self::DeterministicPerBackend,
    ];

    /// The wire spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Deterministic => "deterministic",
            Self::DeterministicFixedPoint => "deterministic_fixed_point",
            Self::DeterministicPerBackend => "deterministic_per_backend",
        }
    }
}

impl fmt::Display for Determinism {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One declared input of an algorithm (blueprint §6.11).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ArgumentSpec {
    /// The port name inside the algorithm.
    pub port: String,
    /// The qualified relation bound to the port.
    pub relation: String,
    /// Whether this named argument must be present. Absence is a bound input value.
    pub required: bool,
    /// Enforced argument access. Unknown algorithms consume the whole input.
    pub consumption: InputConsumption,
}

/// Declared values visible to an algorithm. Projection retains row multiplicity,
/// membership and absence; key columns are required even if not returned.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InputConsumption {
    /// Conservative input dependency, including every field.
    Whole,
    /// Top-level fields; consuming a nested field conservatively includes its root.
    Columns(std::collections::BTreeSet<String>),
}

/// One declared output of an algorithm (blueprint §6.11).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResultSpec {
    /// The port name inside the algorithm.
    pub port: String,
    /// The qualified relation the port writes.
    pub relation: String,
}

/// A declared algorithm (blueprint §6.11 `reference.algorithm_specs`, §14.1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AlgorithmSpec {
    /// `named_id(REGISTRY_PACKAGE_ID, "algorithm:<P>@<v>")` (ADR-0050).
    pub id: SemanticId,
    /// The algorithm name, for example `P2`.
    pub name: &'static str,
    /// Version of these domain semantics, independent of any execution.
    pub version: &'static str,
    /// The inputs. Reading a relation without one of these is invalid.
    pub inputs: Vec<ArgumentSpec>,
    /// The outputs. An output bundle contains every one of them, empties explicit.
    pub outputs: Vec<ResultSpec>,
    /// Invariants that must hold before the algorithm runs, as `<relation>:<name>`.
    pub preconditions: Vec<String>,
    /// Invariants the algorithm establishes, as `<relation>:<name>`.
    pub postconditions: Vec<String>,
    /// How reproducible the algorithm is.
    pub determinism: Determinism,
    /// The `FailureClass` members the algorithm can report.
    pub diagnostics: Vec<&'static str>,
    /// Effects of the native operation, composed with the invocation's scoped policy.
    pub effects: std::collections::BTreeSet<super::provider::OperationEffect>,
}

impl AlgorithmSpec {
    /// The algorithm's registry name, `<name>@<version>`.
    pub fn qualified_name(&self) -> String {
        format!("{}@{}", self.name, self.version)
    }

    /// The output port of that name, if the algorithm has one.
    pub fn output(&self, port: &str) -> Option<&ResultSpec> {
        self.outputs.iter().find(|output| output.port == port)
    }
}

/// An [`AlgorithmSpec`] before assembly, when it has no identity yet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AlgorithmDecl {
    /// See [`AlgorithmSpec::name`].
    pub name: &'static str,
    /// See [`AlgorithmSpec::version`].
    pub version: &'static str,
    /// See [`AlgorithmSpec::inputs`].
    pub inputs: Vec<ArgumentSpec>,
    /// See [`AlgorithmSpec::outputs`].
    pub outputs: Vec<ResultSpec>,
    /// See [`AlgorithmSpec::preconditions`].
    pub preconditions: Vec<String>,
    /// See [`AlgorithmSpec::postconditions`].
    pub postconditions: Vec<String>,
    /// See [`AlgorithmSpec::determinism`].
    pub determinism: Determinism,
    /// See [`AlgorithmSpec::diagnostics`].
    pub diagnostics: Vec<&'static str>,
    /// See [`AlgorithmSpec::effects`].
    pub effects: std::collections::BTreeSet<super::provider::OperationEffect>,
}

impl AlgorithmDecl {
    /// An algorithm declaration with no ports yet.
    pub fn new(name: &'static str, version: &'static str, determinism: Determinism) -> Self {
        Self {
            name,
            version,
            inputs: Vec::new(),
            outputs: Vec::new(),
            preconditions: Vec::new(),
            postconditions: Vec::new(),
            determinism,
            diagnostics: Vec::new(),
            effects: [super::provider::OperationEffect::Read]
                .into_iter()
                .collect(),
        }
    }

    /// The same declaration with `inputs` set.
    #[must_use]
    pub fn inputs(mut self, inputs: Vec<ArgumentSpec>) -> Self {
        self.inputs = inputs;
        self
    }

    /// The same declaration with `outputs` set.
    #[must_use]
    pub fn outputs(mut self, outputs: Vec<ResultSpec>) -> Self {
        self.outputs = outputs;
        self
    }

    /// Declare the actual operation effects without restricting engine feature families.
    #[must_use]
    pub fn effects(
        mut self,
        effects: impl IntoIterator<Item = super::provider::OperationEffect>,
    ) -> Self {
        self.effects = effects.into_iter().collect();
        self
    }

    /// The same declaration with its pre- and postconditions set.
    #[must_use]
    pub fn conditions(mut self, preconditions: Vec<String>, postconditions: Vec<String>) -> Self {
        self.preconditions = preconditions;
        self.postconditions = postconditions;
        self
    }

    /// The same declaration with `diagnostics` set.
    #[must_use]
    pub fn diagnostics(mut self, diagnostics: Vec<&'static str>) -> Self {
        self.diagnostics = diagnostics;
        self
    }
}
