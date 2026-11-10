// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Authored test-fixture requirements, passive readiness and explicit selected preparation.
//! The execution adapter builds demanded helper inputs; original product owners alone create
//! and settle canonical services. Requirement/readiness queries remain passive.

use crate::{
    HostError, correctness::CompilerUnit, discovery::Executable, selection, server, study_module,
};
use cargo_metadata::Metadata;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

mod execution;
pub use execution::{ExecutionInputs, PreparedExecution, prepare_module, selected_module};
#[doc(hidden)]
pub mod target_settlement;

/// Explicit preparation scope, distinct from an assertion that a live fixture is ready.
#[derive(Clone, Copy, Debug, clap::ValueEnum)]
pub enum FixtureKind {
    /// Production module receipt through its existing native owner.
    StudyModule,
}
impl FixtureKind {
    pub(crate) fn argument(self) -> &'static str {
        match self {
            Self::StudyModule => "study-module",
        }
    }
}

/// Native owner-selected preparation input; no service or scientific readiness is asserted.
#[derive(Debug, Serialize, JsonSchema)]
#[serde(untagged)]
pub enum FixtureSelection {
    /// Existing module receipt retains its public representation.
    StudyModule(study_module::StudyModuleReceipt),
}

/// Explicit ordinary Study qualification companion receipt.
pub const QUALIFICATION_MODULE_RECEIPT_ENV: &str =
    "PSE_NATIVE_FIXTURE_MODULE_QUALIFICATION_RECEIPT";
/// Explicit long-hold Study qualification companion receipt.
pub const LONG_HOLD_MODULE_RECEIPT_ENV: &str = "PSE_NATIVE_FIXTURE_MODULE_LONG_HOLD_RECEIPT";

/// Original runner and fixture facts, published by the existing canonical fixture owner.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CanonicalRunReport {
    /// Report contract version.
    pub version: u32,
    /// Original normally reaped Nextest status, independently of wrapper failure.
    pub runner_exit: Option<i32>,
    /// Original service shutdown/disposal succeeded with a positive child-exit premise.
    pub fixture_settled: bool,
}

/// Service ownership stays distinct from preparation-only source inputs.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Ownership {
    /// The target constructs and settles its own NativeCanonicalService.
    TargetOwned,
    /// NativeTestCommand source-builds before lending its own fixture selection.
    RunnerBorrowed,
    /// Inputs are consumed without creating a second service owner.
    PreparationOnly,
}

/// Existing input roles; declarations do not create an alternative validity owner.
#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "kebab-case")]
pub enum Input {
    /// Immutable released server, selected by the server owner.
    Server,
    /// Production module receipt, or original recorded module state admitted by the service owner.
    StudyModule,
    /// Separately prepared qualification companion for selected Study controls.
    QualificationStudyModule,
    /// Separately prepared long-hold companion for the Study capacity control.
    LongHoldStudyModule,
    /// Source-built pse-canonical-service helper, retained by its artifact owner.
    ServiceHelper,
    /// Source-built product worker retained in original observations.
    Worker,
    /// Source-built explicitly prepared scientific-provider process.
    ScientificWorker,
    /// Explicit original managed engineering-service readiness artifact.
    EngineeringEndpoint,
    /// Explicit source-built/captured product reader, selected by its existing transport owner.
    ProductReader,
    /// Source-built runtime publication fixture; it constructs no database or mathematical service.
    ObservationHelper,
    /// Explicit immutable scientific source capsule retained in original observations.
    ScientificInputs,
    /// Captured canonical diagnostic executor.
    CanonicalTools,
    /// Operator-selected evidence destination, owned by the evidence operation.
    DiagnosticEvidence,
    /// Explicit receiving producer qualification, admitted by the existing replay owner.
    ProducerFixtureReceipt,
    /// Actual worker producer capture, interpreted by deployment qualification.
    WorkerProducerReceipt,
    /// Independent artifact observations used by deployment qualification.
    DeploymentArtifactObservations,
    /// The separately prepared covariance scientific reference interpreter.
    CovarianceReferencePython,
    /// The separately prepared ordinary-process independent scientific interpreter.
    OrdinaryReferencePython,
    /// The separately prepared degeneracy scientific reference interpreter.
    DegeneracyReferencePython,
    /// The separately prepared sensitivity scientific reference interpreter.
    SensitivityReferencePython,
    /// The separately prepared PID scientific reference interpreter.
    PidReferencePython,
    /// Explicit Trace Processor binary for its ignored consumer qualification.
    TraceProcessor,
    /// Existing checksum-verified Collector, selected by the diagnostic-tools owner.
    Collector,
    /// External owned server-restart rendezvous, interpreted by the original lifecycle owner.
    CanonicalRestartControl,
}

/// Direct authored test identity scopes, distinct from runner filter syntax.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TestScope {
    /// Exact harness IDs; empty lists together mean the entire target.
    #[serde(default)]
    pub tests: BTreeSet<String>,
    /// Exact qualified Rust modules; membership requires a following `::` boundary.
    #[serde(default)]
    pub modules: BTreeSet<String>,
}
impl TestScope {
    /// Match a listed harness identity without interpreting nextest filters.
    pub fn contains(&self, test: &str) -> bool {
        self.is_whole_target()
            || self.tests.contains(test)
            || self.modules.iter().any(|module| in_module(test, module))
    }
    /// Before listing, a narrower scope remains conditional rather than known selected.
    pub fn is_whole_target(&self) -> bool {
        self.tests.is_empty() && self.modules.is_empty()
    }
}

fn in_module(test: &str, module: &str) -> bool {
    test.strip_prefix(module)
        .is_some_and(|suffix| suffix.starts_with("::"))
}

/// Wire declaration owned once by `package.metadata.pse.test-fixtures`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct Declaration {
    /// Package-local declaration key.
    pub id: String,
    /// Cargo target name.
    pub target: String,
    /// Cargo target kind, disambiguating bins, libs and integration tests.
    pub kind: String,
    /// Actual selected compiler features under which this requirement applies.
    #[serde(default)]
    pub required_features: BTreeSet<String>,
    /// Exact target or listed-test scope.
    #[serde(default)]
    pub scope: TestScope,
    /// Original service owner, not a host-created service surrogate.
    pub ownership: Ownership,
    /// Actual preparation inputs, not inferred from package names.
    pub inputs: BTreeSet<Input>,
}

/// Checked declaration applied to one admitted test root, before actual binary listing.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct Requirement {
    group: usize,
    unit: CompilerUnit,
    declaration: Declaration,
}
impl Requirement {
    /// Admitted compiler group, retained through later binary/test discovery.
    pub fn group(&self) -> usize {
        self.group
    }
    /// Actual selected compiler context; this is not an observed test binary.
    pub fn unit(&self) -> &CompilerUnit {
        &self.unit
    }
    /// Exact Cargo package identity.
    pub fn package_id(&self) -> &str {
        &self.unit.package_id
    }
    /// Actual selected target name.
    pub fn target_name(&self) -> &str {
        &self.unit.target_name
    }
    /// Original execution owner.
    pub fn ownership(&self) -> Ownership {
        self.declaration.ownership
    }
    /// Declared prerequisite roles.
    pub fn inputs(&self) -> &BTreeSet<Input> {
        &self.declaration.inputs
    }
    /// Authored declaration and exact conditional test scope.
    pub fn declaration(&self) -> &Declaration {
        &self.declaration
    }
    /// Narrow only after an actual harness test ID is available.
    pub fn applies_to_test(&self, exact_id: &str) -> bool {
        self.declaration.scope.contains(exact_id)
    }

    /// Check authored names against the complete native harness population, including exclusions.
    pub(crate) fn validate_test_scope<'a>(
        &self,
        cases: impl Iterator<Item = &'a str> + Clone,
    ) -> Result<(), FixtureError> {
        let scope = &self.declaration.scope;
        let missing_test = scope
            .tests
            .iter()
            .find(|name| !cases.clone().any(|case| case == name.as_str()));
        let missing_module = scope
            .modules
            .iter()
            .find(|module| !cases.clone().any(|case| in_module(case, module)));
        if let Some((kind, name)) = missing_test
            .map(|name| ("case", name))
            .or_else(|| missing_module.map(|name| ("module", name)))
        {
            return Err(invalid(format!(
                "fixture {} for {}:{} in compiler group {} names absent test {kind} {name:?}",
                self.declaration.id,
                self.package_id(),
                self.target_name(),
                self.group,
            )));
        }
        Ok(())
    }
}

fn invalid(message: impl Into<String>) -> FixtureError {
    FixtureError::Declaration(message.into())
}

fn declarations(package: &cargo_metadata::Package) -> Result<Vec<Declaration>, FixtureError> {
    let Some(raw) = package
        .metadata
        .get("pse")
        .and_then(|pse| pse.get("test-fixtures"))
    else {
        return Ok(Vec::new());
    };
    let declarations: Vec<Declaration> = serde_json::from_value(raw.clone())?;
    let mut ids = BTreeSet::new();
    for declaration in &declarations {
        if declaration.id.trim().is_empty()
            || !ids.insert(&declaration.id)
            || declaration.inputs.is_empty()
        {
            return Err(invalid(format!(
                "{} has an empty or repeated fixture declaration",
                package.name
            )));
        }
        if declaration.inputs.contains(&Input::Worker)
            || declaration.inputs.contains(&Input::ScientificInputs)
        {
            return Err(invalid(format!(
                "{} fixture {} declares a retired input role",
                package.name, declaration.id
            )));
        }
        let mut targets = package.targets.iter().filter(|target| {
            target.name == declaration.target
                && target
                    .kind
                    .iter()
                    .any(|kind| kind.to_string() == declaration.kind)
        });
        if targets.next().is_none() || targets.next().is_some() {
            return Err(invalid(format!(
                "{} fixture {} has no unique declared target",
                package.name, declaration.id
            )));
        }
        if declaration
            .required_features
            .iter()
            .any(|feature| !package.features.contains_key(feature))
        {
            return Err(invalid(format!(
                "{} fixture {} has an unknown feature",
                package.name, declaration.id
            )));
        }
        for name in declaration
            .scope
            .tests
            .iter()
            .chain(&declaration.scope.modules)
        {
            if name.split("::").any(str::is_empty)
                || name.chars().any(|character| {
                    !(character.is_alphanumeric() || character == '_' || character == ':')
                })
            {
                return Err(invalid(
                    "fixture test scope must be a direct qualified identity, not a filter",
                ));
            }
        }
        if declaration.ownership != Ownership::PreparationOnly
            && ![Input::Server, Input::StudyModule]
                .iter()
                .all(|input| declaration.inputs.contains(input))
        {
            return Err(invalid(
                "native service fixture must retain its server and production module requirements",
            ));
        }
        if declaration.ownership == Ownership::RunnerBorrowed
            && !declaration.inputs.contains(&Input::ServiceHelper)
        {
            return Err(invalid(
                "runner-borrowed fixture must retain its original service-helper requirement",
            ));
        }
    }
    Ok(declarations)
}

/// Project authored requirements onto actual admitted test roots, without build/list or setup.
/// Check operations have no runtime fixture requirements. Conditional test scopes remain visible
/// until official listing/filtering establishes whether their exact tests are selected.
pub fn requirements(
    metadata: &Metadata,
    resolved: &selection::Resolved,
) -> Result<Vec<Requirement>, FixtureError> {
    if resolved.plan().operation() != selection::Operation::Test {
        return Ok(Vec::new());
    }
    let mut result = Vec::new();
    for (group, resolution) in resolved.admitted().iter().enumerate() {
        for unit in resolution
            .units
            .iter()
            .filter(|unit| unit.root && unit.mode == "test")
        {
            let package = metadata
                .packages
                .iter()
                .find(|package| package.id.repr == unit.package_id)
                .ok_or_else(|| invalid("admitted fixture root is absent from captured metadata"))?;
            for declaration in declarations(package)? {
                if declaration.target != unit.target_name
                    || !unit.target_kinds.contains(&declaration.kind)
                    || !declaration.required_features.is_subset(&unit.features)
                {
                    continue;
                }
                let target = package
                    .targets
                    .iter()
                    .find(|target| {
                        target.name == declaration.target
                            && target
                                .kind
                                .iter()
                                .any(|kind| kind.to_string() == declaration.kind)
                    })
                    .ok_or_else(|| invalid("fixture target disappeared from captured metadata"))?;
                if target.src_path.as_str() != unit.source_path {
                    return Err(invalid(
                        "fixture target source differs from admitted source",
                    ));
                }
                result.push(Requirement {
                    group,
                    unit: unit.clone(),
                    declaration,
                });
            }
        }
    }
    Ok(result)
}

/// Observation of one input; none of these states is an execution or disposal grant.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "state", rename_all = "kebab-case")]
pub enum InputState {
    /// The original preparation owner refused this exact selection; native causes remain
    /// in the aggregate fixture error rather than a second rendered error string.
    Refused {
        /// Typed original cause, for passive presentation only.
        code: Option<String>,
        /// Existing explicit preparation route, where one is available.
        #[serde(deserialize_with = "original_preparation_arguments")]
        next: Vec<String>,
    },
    /// Original immutable owner successfully reselected its own exact artifact.
    Selected {
        /// Original immutable path.
        path: PathBuf,
        /// Original owner identity, not a source-build proof.
        identity: String,
    },
    /// No owner selection was supplied. `next` invokes an existing explicit prepare route.
    Unprepared {
        /// Typed diagnostic identity for callers constructing their envelope.
        code: String,
        /// Exact preparation argv when an existing route is known.
        #[serde(deserialize_with = "original_preparation_arguments")]
        next: Vec<String>,
    },
    /// A path alone is not sufficient to interpret a capsule, old state or evidence destination.
    OwnerAdmissionRequired {
        /// Original caller selection, without filesystem guessing.
        path: Option<PathBuf>,
    },
}

/// Passive prerequisite account, retaining the original service ownership distinction.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct Readiness {
    /// Owner that must still source-build/start/lend/settle; no live fixture is asserted.
    pub ownership: Ownership,
    /// One observation per authored input, with no implicit preparation.
    #[serde(deserialize_with = "original_input_states")]
    pub inputs: BTreeMap<Input, InputState>,
}

fn original_preparation_arguments<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<String>, D::Error> {
    struct Arguments;
    impl<'de> serde::de::Visitor<'de> for Arguments {
        type Value = Vec<String>;
        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("at most 64 passive original preparation arguments")
        }
        fn visit_seq<A: serde::de::SeqAccess<'de>>(
            self,
            mut seq: A,
        ) -> Result<Self::Value, A::Error> {
            let mut args = Vec::new();
            while let Some(argument) = seq.next_element::<String>()? {
                if args.len() == 64 {
                    return Err(serde::de::Error::custom(
                        "original preparation argument limit exceeded",
                    ));
                }
                args.push(argument);
            }
            Ok(args)
        }
    }
    deserializer.deserialize_seq(Arguments)
}

fn original_input_states<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<BTreeMap<Input, InputState>, D::Error> {
    struct Inputs;
    impl<'de> serde::de::Visitor<'de> for Inputs {
        type Value = BTreeMap<Input, InputState>;
        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("unique source-owned original fixture input states")
        }
        fn visit_map<A: serde::de::MapAccess<'de>>(
            self,
            mut map: A,
        ) -> Result<Self::Value, A::Error> {
            let mut inputs = BTreeMap::new();
            while let Some(input) = map.next_key::<Input>()? {
                if inputs.contains_key(&input) {
                    return Err(serde::de::Error::custom("duplicate original fixture input"));
                }
                inputs.insert(input, map.next_value::<InputState>()?);
            }
            Ok(inputs)
        }
    }
    deserializer.deserialize_map(Inputs)
}

fn prepare(input: Input) -> Vec<String> {
    let args: &[&str] = match input {
        Input::Server => &["scripts/pse-host", "server", "prepare"],
        Input::StudyModule => &["scripts/pse", "fixture", "prepare", "study-module"],
        Input::QualificationStudyModule => &["just", "surreal-module", "--qualification"],
        Input::LongHoldStudyModule => &["just", "surreal-module", "--qualification", "--long-hold"],
        Input::Collector => &[
            "scripts/pse-host",
            "diagnostic-tools",
            "install",
            "collector",
        ],
        _ => &[],
    };
    args.iter()
        .map(|argument| String::from(*argument))
        .collect()
}

/// Native causes survive declaration and owner-admission refusal.
#[derive(Debug, thiserror::Error)]
pub enum FixtureError {
    /// Every independently refused or missing preparation input, before helper effects.
    #[error("selected fixture readiness has {} unfulfilled input owners", .0.len())]
    Inputs(Vec<InputFailure>),
    /// This prerequisite still requires its actual source/evidence owner to admit it.
    #[error(
        "fixture input {0:?} is unprepared; select its original preparation owner before execution"
    )]
    Unprepared(Input),
    /// Authoritative metadata does not define a coherent target requirement.
    #[error("fixture declaration: {0}")]
    Declaration(String),
    /// Typed metadata input could not be decoded.
    #[error("fixture declaration JSON: {0}")]
    Json(#[from] serde_json::Error),
    /// Original artifact owner refused its own selection.
    #[error(transparent)]
    Owner(#[from] HostError),
    /// Captured executable bytes or their source-build owner refused admission.
    #[error(transparent)]
    Capture(#[from] crate::correctness::CorrectnessError),
}
pse_diagnostics::impl_diagnostic! {
    FixtureError,
    code(this) { match this { Self::Unprepared(_) | Self::Inputs(_) => Some(pse_diagnostics::DiagnosticCode::FixtureUnprepared), Self::Owner(error) => pse_diagnostics::TypedDiagnostic::diagnostic_code(error), Self::Capture(error) => pse_diagnostics::TypedDiagnostic::diagnostic_code(error), _ => Some(pse_diagnostics::DiagnosticCode::ConfigInvalid) } },
    forward(this) { match this { Self::Owner(error) => Some(error), Self::Capture(error) => Some(error), _ => None } },
    help(_this) { None }, related(this) { match this { Self::Inputs(errors) => Some(Box::new(errors.iter().map(|error| -> &dyn pse_diagnostics::TypedDiagnostic { error }))), _ => None } }, source(_this) { None }
}

/// One original owner refusal, attributed to its authored input role.
#[derive(Debug, thiserror::Error)]
#[error("fixture input {input:?}: {cause}")]
pub struct InputFailure {
    /// Original declaration role, not an inferred filename meaning.
    pub input: Input,
    /// Original typed cause, retained independently from other failed owners.
    #[source]
    pub cause: std::sync::Arc<FixtureError>,
}
pse_diagnostics::impl_diagnostic! {
    InputFailure,
    code(this) { pse_diagnostics::TypedDiagnostic::diagnostic_code(this.cause.as_ref()) },
    forward(this) { Some(this.cause.as_ref()) }, help(_this) { None }, related(_this) { None }, source(_this) { None }
}

#[cfg(test)]
mod tests;
