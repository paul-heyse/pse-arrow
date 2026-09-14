// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Pass contracts: the declared ports that are the compiler's only dependency edges
//! (blueprint §6.11 `reference.pass_*`, §14.1).
//!
//! Ports, not relation names, carry producer ownership. The same `compiled.math_*` schema
//! occurs at four successive stages, so "who wrote this relation" has no answer while
//! "who wrote this port" has exactly one (§4.1). A read without a declared input binding
//! is invalid — that is what stops a pass from silently consuming whatever happened to be
//! latest.

use core::fmt;

use pse_ids::SemanticId;

/// How reproducible a pass is (blueprint §14.1).
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

/// Where an input port's rows come from (blueprint §6.11 `pass_input_ports`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PortSource {
    /// The pinned snapshot the pipeline was started on.
    Pinned,
    /// Another pass's output port.
    Derived {
        /// The producing pass, by name, for example `P7`.
        pass: &'static str,
        /// The producing pass's output port.
        port: &'static str,
    },
}

/// One declared input of a pass (blueprint §6.11).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InputPort {
    /// The port name inside the pass.
    pub port: &'static str,
    /// The qualified relation bound to the port.
    pub relation: String,
    /// Where the rows come from.
    pub source: PortSource,
    /// Whether the pass refuses to run without it.
    ///
    /// An optional absence is a typed input value and enters the stage key (§14.3): "this
    /// port was absent" and "this port was never asked for" have to be different keys or a
    /// memo hit means nothing.
    pub required: bool,
}

/// One declared output of a pass (blueprint §6.11).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OutputPort {
    /// The port name inside the pass.
    pub port: &'static str,
    /// The qualified relation the port writes.
    pub relation: String,
}

/// A declared pass (blueprint §6.11 `reference.pass_specs`, §14.1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PassSpec {
    /// `named_id(REGISTRY_PACKAGE_ID, "pass:<P>@<v>")` (ADR-0050).
    pub id: SemanticId,
    /// The pass name, for example `P2`.
    pub name: &'static str,
    /// The pass version. A pass version is a stage-key input (§14.3).
    pub version: &'static str,
    /// The inputs. Reading a relation without one of these is invalid.
    pub inputs: Vec<InputPort>,
    /// The outputs. An output bundle contains every one of them, empties explicit.
    pub outputs: Vec<OutputPort>,
    /// Invariants that must hold before the pass runs, as `<relation>:<name>`.
    pub preconditions: Vec<String>,
    /// Invariants the pass establishes, as `<relation>:<name>`.
    pub postconditions: Vec<String>,
    /// How reproducible the pass is.
    pub determinism: Determinism,
    /// The `FailureClass` members the pass can report.
    pub diagnostics: Vec<&'static str>,
    /// Whether the pass executes DataFusion plans, and therefore whether the engine
    /// profile and the function registry enter its stage key (§14.2 rule 5).
    pub executes_plans: bool,
}

impl PassSpec {
    /// The pass's registry name, `<name>@<version>`.
    pub fn qualified_name(&self) -> String {
        format!("{}@{}", self.name, self.version)
    }

    /// The output port of that name, if the pass has one.
    pub fn output(&self, port: &str) -> Option<&OutputPort> {
        self.outputs.iter().find(|output| output.port == port)
    }
}

/// A [`PassSpec`] before assembly, when it has no identity yet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PassDecl {
    /// See [`PassSpec::name`].
    pub name: &'static str,
    /// See [`PassSpec::version`].
    pub version: &'static str,
    /// See [`PassSpec::inputs`].
    pub inputs: Vec<InputPort>,
    /// See [`PassSpec::outputs`].
    pub outputs: Vec<OutputPort>,
    /// See [`PassSpec::preconditions`].
    pub preconditions: Vec<String>,
    /// See [`PassSpec::postconditions`].
    pub postconditions: Vec<String>,
    /// See [`PassSpec::determinism`].
    pub determinism: Determinism,
    /// See [`PassSpec::diagnostics`].
    pub diagnostics: Vec<&'static str>,
    /// See [`PassSpec::executes_plans`].
    pub executes_plans: bool,
}

impl PassDecl {
    /// A pass declaration with no ports yet.
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
            executes_plans: false,
        }
    }

    /// The same declaration with `inputs` set.
    #[must_use]
    pub fn inputs(mut self, inputs: Vec<InputPort>) -> Self {
        self.inputs = inputs;
        self
    }

    /// The same declaration with `outputs` set.
    #[must_use]
    pub fn outputs(mut self, outputs: Vec<OutputPort>) -> Self {
        self.outputs = outputs;
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

    /// The same declaration, marked as executing DataFusion plans.
    #[must_use]
    pub const fn executes_plans(mut self) -> Self {
        self.executes_plans = true;
        self
    }
}
