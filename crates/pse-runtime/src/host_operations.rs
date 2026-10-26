// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Finite host operation declarations over the existing scientific document owners.
//!
//! Header schemas describe host transport only. Actual scientific bodies retain
//! the original typed document decoder and receiver's contextual admission.
use serde::Serialize;
use serde_json::Value;

/// The original document registration, derived from the boundary owner macro.
pub trait RegisteredDocument {
    /// Existing directional registration; no second scientific schema is authored here.
    const REFERENCE: DocumentReference;
}
/// A responsibility-local reference to an existing Rust-owned document.
#[derive(Clone, Copy, Debug, Serialize, schemars::JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct DocumentReference {
    /// Existing published schema slug.
    pub slug: &'static str,
    /// Existing generated input type name.
    pub name: &'static str,
    /// Whether its owning catalogue admits input.
    pub input: bool,
    /// Whether its owning catalogue admits output.
    pub output: bool,
}
macro_rules! register_document_references {
    ($(($name:ident, $owner:ty, $slug:literal, $direction:ident, $wire:ident)),* $(,)?) => {
        $(impl RegisteredDocument for $owner {
            const REFERENCE: DocumentReference = DocumentReference {
                slug: $slug, name: stringify!($name),
                input: register_document_references!(@input $direction),
                output: register_document_references!(@output $direction),
            };
        })*
    };
    (@input Output) => { false };
    (@input $other:ident) => { true };
    (@output Input) => { false };
    (@output $other:ident) => { true };
}
crate::for_each_boundary_document!(register_document_references);

/// One scientific field in a host header, never a replacement scientific definition.
#[derive(Clone, Copy, Debug, Serialize, schemars::JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct DocumentField {
    /// Field of the generated host envelope.
    pub field: &'static str,
    /// Original directional owner reference.
    pub document: DocumentReference,
}
impl DocumentField {
    /// Bind a field to its actual compiled scientific Rust owner.
    pub const fn of<T: RegisteredDocument>(field: &'static str) -> Self {
        Self {
            field,
            document: T::REFERENCE,
        }
    }
}

/// Closed host header shapes; their schemas below are the sole field/bound declaration.
#[derive(Clone, Copy, Debug, Serialize, schemars::JsonSchema)]
pub enum EnvelopeKind {
    /// Original source bytes plus production-profile resource allowances.
    Source,
    /// Source plus one original request document.
    Preparation,
    /// Original source pairs and comparison request.
    ModelingComparison,
    /// Existing canonical service selection.
    Deployment,
    /// Existing service plus one request.
    Metadata,
    /// Existing service, physical source and selection request.
    Select,
    /// Existing retained revision and bounded logical names.
    Revision,
    /// Existing retained revision and preparation request.
    RevisionPreparation,
    /// Supplied project bundle.
    Bundle,
    /// Supplied diagnostic checkpoint and request.
    Diagnostic,
    /// Bounded process discovery selection.
    DiagnosticProcesses,
    /// Exact checkpoint and optional identity.
    DiagnosticCheckpoint,
    /// Supplied frozen evidence and request.
    Evidence,
    /// Supplied invocation evidence.
    Invocation,
    /// Exact canonical result window.
    Result,
    /// Physical source and inspection request.
    PhysicalInspection,
    /// Canonical result inspection request.
    ResultInspection,
    /// Supplied evidence comparison; contributed only by tooling.
    EvidenceCompare,
    /// Explicit finite capture and telemetry association.
    ProfileCaptureRequest,
    /// Linked capabilities or one backend's selected settings.
    ConfiguredCapabilities,
}
/// Responsibility providing the native receiver.
#[derive(Clone, Copy, Debug, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum WorkerFamily {
    /// Actual scientific extension with native environment/admission.
    Native,
    /// Independently built supplied-only tooling executable.
    Evidence,
}
/// Actual prerequisites, independently selected from header shape and descriptions.
#[derive(Clone, Copy, Debug, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Prerequisites {
    /// Compatible prebuilt scientific extension; no service required.
    Extension,
    /// Extension and explicitly selected existing canonical service.
    ExtensionAndCanonical,
    /// Tooling executable, with no extension or canonical service.
    EvidenceExecutable,
}
/// Existing helper cancellation behavior, separate from original-document preservation.
#[derive(Clone, Copy, Debug, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum InvocationLifetime {
    /// Cancellation signals and actually drains the helper owner.
    Attached,
    /// A cancelled observer leaves the admitted exact read to settle.
    Detached,
}
/// Eligibility only; retained reuse still requires complete current context admission.
#[derive(Clone, Copy, Debug, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReadEligibility {
    /// Dedicated effect/capture lifetime.
    Dedicated,
    /// Eligible for exact immutable source/receiver/image context reuse.
    ExactContext,
    /// Tooling-supplied inputs, never a native extension read worker.
    SuppliedTooling,
}
/// Kind of an original paging field; an offset is not an opaque ownership token.
#[derive(Clone, Copy, Debug, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ContinuationKind {
    /// Native offset/seek selector, preserving the original selected basis.
    Offset,
    /// Native opaque issuer-bound cursor.
    Opaque,
}
/// An explicitly paired original request/result paging field, never a JSON scan.
#[derive(Clone, Copy, Debug, Serialize, schemars::JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct ContinuationField {
    /// Exact path within the host request arguments.
    pub input: &'static [&'static str],
    /// Exact path within the original worker response.
    pub output: &'static [&'static str],
    /// Original field's semantics.
    pub kind: ContinuationKind,
}
/// Finite observable effect scope, never inferred from presentation or helper lifetime.
#[derive(Clone, Copy, Debug, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EffectScope {
    /// Read original supplied or protected facts without a requested mutation.
    Observation,
    /// Admit original source and its actual providers into a local native owner.
    SourceAdmission,
    /// Original caller-issued canonical command or publication.
    CanonicalCommand,
    /// Explicit release of retained canonical custody.
    RetentionRelease,
    /// Create a new bounded requested output file.
    FileExport,
    /// Explicit finite runtime sampling/capture.
    Capture,
}
/// Machine-readable facts used by host annotations and operation schema metadata.
#[derive(Clone, Copy, Debug, Serialize, schemars::JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct EffectFacts {
    /// Actual operation scope.
    pub scope: EffectScope,
    /// The operation requests observation rather than mutation/admission/capture.
    pub read_only: bool,
    /// Repeating the original request has the owner's stated repeatability contract.
    pub idempotent: bool,
    /// The operation explicitly releases retained data/custody.
    pub destructive: bool,
}
impl EffectFacts {
    /// Ordinary supplied/protected observation, without granting source completeness.
    pub const OBSERVATION: Self = Self {
        scope: EffectScope::Observation,
        read_only: true,
        idempotent: true,
        destructive: false,
    };
    /// Native original-source admission; no scientific solve is implied.
    pub const SOURCE_ADMISSION: Self = Self {
        scope: EffectScope::SourceAdmission,
        read_only: false,
        idempotent: true,
        destructive: false,
    };
    /// Canonical command under its original exact-operation/intent owner.
    pub const CANONICAL_COMMAND: Self = Self {
        scope: EffectScope::CanonicalCommand,
        read_only: false,
        idempotent: true,
        destructive: false,
    };
    /// Explicit retained-custody release, independently admitted by its original owner.
    pub const RETENTION_RELEASE: Self = Self {
        scope: EffectScope::RetentionRelease,
        read_only: false,
        idempotent: false,
        destructive: true,
    };
    /// Fresh explicit output; existing destinations refuse, with no overwrite fallback.
    pub const FILE_EXPORT: Self = Self {
        scope: EffectScope::FileExport,
        read_only: false,
        idempotent: false,
        destructive: false,
    };
    /// A new finite capture cannot replay an earlier time interval.
    pub const CAPTURE: Self = Self {
        scope: EffectScope::Capture,
        read_only: false,
        idempotent: false,
        destructive: false,
    };
}
/// Actual selected extension interpretation; installed mode comes from its producing receipt.
#[derive(Clone, Copy, Debug, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExtensionMode {
    /// The actual installed dev/native correctness image, preserving its own provider requirements.
    InstalledCorrectness,
    /// An operation actually requiring linked backend execution.
    NativeCorrectness,
}
/// Extension requirements, distinct from source/runtime prerequisites and effect scope.
#[derive(Clone, Copy, Debug, Serialize, schemars::JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct ExtensionRequirement {
    /// Actual allowed image selection contract.
    pub mode: ExtensionMode,
    /// Correctness validation is required independently of selected dev/native mode.
    pub validated: bool,
}
/// One finite supported host operation, linked to actual typed receiver inputs/results.
#[derive(Clone, Copy, Debug, Serialize, schemars::JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct OperationDeclaration {
    /// Stable selected operation identity.
    pub operation: &'static str,
    /// Public host title.
    pub title: &'static str,
    /// Owned host header shape.
    pub envelope: EnvelopeKind,
    /// Original typed document fields.
    pub documents: &'static [DocumentField],
    /// Actual receiver entry point, including necessary small typed glue.
    pub receiver: &'static str,
    /// Public behavior and limits.
    pub description: &'static str,
    /// Existing capability domain.
    pub domain: &'static str,
    /// Human explanation only; structured facts below drive host semantics.
    pub effects: &'static str,
    /// Finite effect facts, independent of helper preparation or lifetime.
    pub effect_facts: EffectFacts,
    /// Actual extension interpretation, absent for supplied-only tooling.
    pub extension: Option<ExtensionRequirement>,
    /// Original result document when one exists; Arrow IPC has no fake JSON document.
    pub output_document: Option<DocumentReference>,
    /// Whether the original result is a bounded array of the declared document.
    pub output_many: bool,
    /// Actual native or supplied-only executable owner.
    pub worker: WorkerFamily,
    /// Actual prerequisite class.
    pub prerequisites: Prerequisites,
    /// Existing explicit provider preparation selection.
    pub prepare_import: bool,
    /// Existing attached/detached cancellation selection.
    pub invocation_lifetime: InvocationLifetime,
    /// Keep the entire original selected native document or refuse its extent.
    pub preserve_document: bool,
    /// Explicit read-context eligibility; never inferred from preservation.
    pub read_eligibility: ReadEligibility,
    /// Whether normal native bindings publish this operation (capture has its own composition).
    pub public_binding: bool,
    /// Complete immutable receiver-construction fields, independent of paging selectors.
    pub context_fields: &'static [&'static str],
    /// Original explicitly paired continuation fields, if any.
    pub continuations: &'static [ContinuationField],
}
/// Compiled operation metadata; tooling adds only its independent contribution at generation.
#[derive(Clone, Debug, Serialize, schemars::JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct OperationCatalogue {
    /// Original product-owned operation declarations, in declaration order.
    pub operations: Vec<OperationDeclaration>,
}
/// Read the same declaration consumed by codegen and the actual extension/facade/worker.
pub fn catalogue() -> OperationCatalogue {
    OperationCatalogue {
        operations: operations().to_vec(),
    }
}

/// Existing finite host header bound, independent of actual machine CPU placement.
pub const HOST_THREADS_MAX: usize = 16;
/// Existing host bundle transport bound.
pub const HOST_DOCUMENT_MAX: usize = 16 << 20;
/// Existing exact result transport bound.
pub const HOST_RESULT_MAX: usize = 65_536;

#[derive(Clone, Debug, Serialize, schemars::JsonSchema)]
#[schemars(transparent)]
/// Nonempty host selection text.
pub struct NonemptyText(#[schemars(length(min = 1))] String);

/// Resource defaults are references to the existing production capability declaration.
#[derive(Clone, Debug, Serialize, schemars::JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct Resources {
    /// Accounted pool allowance; default is production.capabilities.memory_limit_bytes.
    #[schemars(range(min = 1), extend("x-pse-host-default" = "production.capabilities.memory_limit_bytes"))]
    pub memory_limit_bytes: usize,
    /// Scientific CPU permits; default is production.capabilities.threads.
    #[schemars(range(min = 1, max = HOST_THREADS_MAX), extend("x-pse-host-default" = "production.capabilities.threads"))]
    pub threads: usize,
    /// Bounded spill allowance; default is production.capabilities.max_spill_bytes.
    #[schemars(range(min = 1), extend("x-pse-host-default" = "production.capabilities.max_spill_bytes"))]
    pub max_spill_bytes: usize,
}

/// Finite `Source` host transport header.
#[derive(Clone, Debug, Serialize, schemars::JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct Source {
    /// Original `physical` transport field.
    pub physical: Value,
    /// Original `documents` transport field.
    pub documents: Value,
    /// Original `resources` transport field.
    #[schemars(extend("x-pse-host-default" = "Resources"))]
    pub resources: Resources,
}

/// Finite `Preparation` host transport header.
#[derive(Clone, Debug, Serialize, schemars::JsonSchema)]
#[schemars(deny_unknown_fields)]
#[schemars(extend("x-pse-host-base" = "Source"))]
pub struct Preparation {
    /// Existing inherited header.
    #[schemars(flatten)]
    pub base: Source,
    /// Original `request` transport field.
    pub request: Value,
}

/// Finite `ModelingComparison` host transport header.
#[derive(Clone, Debug, Serialize, schemars::JsonSchema)]
#[schemars(deny_unknown_fields)]
#[schemars(extend("x-pse-host-base" = "Preparation"))]
pub struct ModelingComparison {
    /// Existing inherited header.
    #[schemars(flatten)]
    pub base: Preparation,
    /// Original `other_physical` transport field.
    pub other_physical: Value,
    /// Original `other_documents` transport field.
    pub other_documents: Value,
}

/// Finite `Deployment` host transport header.
#[derive(Clone, Debug, Serialize, schemars::JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct Deployment {
    /// Original `state` transport field.
    pub state: NonemptyText,
    /// Original `database` transport field.
    pub database: NonemptyText,
    /// Original `resources` transport field.
    #[schemars(extend("x-pse-host-default" = "Resources"))]
    pub resources: Resources,
}

/// Finite `Metadata` host transport header.
#[derive(Clone, Debug, Serialize, schemars::JsonSchema)]
#[schemars(deny_unknown_fields)]
#[schemars(extend("x-pse-host-base" = "Deployment"))]
pub struct Metadata {
    /// Existing inherited header.
    #[schemars(flatten)]
    pub base: Deployment,
    /// Original `request` transport field.
    pub request: Value,
}

/// Finite `Select` host transport header.
#[derive(Clone, Debug, Serialize, schemars::JsonSchema)]
#[schemars(deny_unknown_fields)]
#[schemars(extend("x-pse-host-base" = "Metadata"))]
pub struct Select {
    /// Existing inherited header.
    #[schemars(flatten)]
    pub base: Metadata,
    /// Original `physical` transport field.
    pub physical: Value,
}

/// Finite `Revision` host transport header.
#[derive(Clone, Debug, Serialize, schemars::JsonSchema)]
#[schemars(deny_unknown_fields)]
#[schemars(extend("x-pse-host-base" = "Deployment"))]
pub struct Revision {
    /// Existing inherited header.
    #[schemars(flatten)]
    pub base: Deployment,
    /// Original `physical` transport field.
    pub physical: Value,
    /// Original `revision` transport field.
    pub revision: NonemptyText,
    /// Original `logical` transport field.
    #[schemars(length(min = 1, max = 32))]
    pub logical: Vec<NonemptyText>,
}

/// Finite `RevisionPreparation` host transport header.
#[derive(Clone, Debug, Serialize, schemars::JsonSchema)]
#[schemars(deny_unknown_fields)]
#[schemars(extend("x-pse-host-base" = "Deployment"))]
pub struct RevisionPreparation {
    /// Existing inherited header.
    #[schemars(flatten)]
    pub base: Deployment,
    /// Original `physical` transport field.
    pub physical: Value,
    /// Original `revision` transport field.
    pub revision: NonemptyText,
    /// Original `request` transport field.
    pub request: Value,
}

/// Finite `Bundle` host transport header.
#[derive(Clone, Debug, Serialize, schemars::JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct Bundle {
    /// Original `path` transport field.
    pub path: NonemptyText,
    /// Original `maximum_document_bytes` transport field.
    #[schemars(range(min = 1, max = HOST_DOCUMENT_MAX), default = "document_default")]
    pub maximum_document_bytes: usize,
}

/// Finite `Diagnostic` host transport header.
#[derive(Clone, Debug, Serialize, schemars::JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct Diagnostic {
    /// Original `checkpoint` transport field.
    pub checkpoint: NonemptyText,
    /// Original `request` transport field.
    pub request: Value,
}

/// Finite `DiagnosticProcesses` host transport header.
#[derive(Clone, Debug, Serialize, schemars::JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct DiagnosticProcesses {
    /// Original `request` transport field.
    pub request: Value,
    /// Original `directory` transport field.
    #[schemars(default)]
    pub directory: Option<NonemptyText>,
}

/// Finite `DiagnosticCheckpoint` host transport header.
#[derive(Clone, Debug, Serialize, schemars::JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct DiagnosticCheckpoint {
    /// Original `checkpoint` transport field.
    pub checkpoint: NonemptyText,
    /// Original `expected` transport field.
    #[schemars(default)]
    pub expected: Option<Value>,
}

/// Finite `Evidence` host transport header.
#[derive(Clone, Debug, Serialize, schemars::JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct Evidence {
    /// Original `path` transport field.
    pub path: NonemptyText,
    /// Original `request` transport field.
    pub request: Value,
}

/// Finite `Invocation` host transport header.
#[derive(Clone, Debug, Serialize, schemars::JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct Invocation {
    /// Original `path` transport field.
    pub path: NonemptyText,
}

/// Finite `Result` host transport header.
#[derive(Clone, Debug, Serialize, schemars::JsonSchema)]
#[schemars(deny_unknown_fields)]
#[schemars(extend("x-pse-host-base" = "Deployment"))]
pub struct Result {
    /// Existing inherited header.
    #[schemars(flatten)]
    pub base: Deployment,
    /// Original `run` transport field.
    pub run: NonemptyText,
    /// Original `attempt` transport field.
    pub attempt: NonemptyText,
    /// Original `relation` transport field.
    pub relation: NonemptyText,
    /// Original `start` transport field.
    #[schemars(range(max = u64::MAX))]
    pub start: u64,
    /// Original `end` transport field.
    #[schemars(range(max = u64::MAX))]
    pub end: u64,
    /// Original `maximum_bytes` transport field.
    #[schemars(range(min = 4096, max = HOST_RESULT_MAX), default = "result_default")]
    pub maximum_bytes: usize,
}

/// Finite `PhysicalInspection` host transport header.
#[derive(Clone, Debug, Serialize, schemars::JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct PhysicalInspection {
    /// Original `physical` transport field.
    pub physical: Value,
    /// Original `request` transport field.
    pub request: Value,
    /// Original `resources` transport field.
    #[schemars(extend("x-pse-host-default" = "Resources"))]
    pub resources: Resources,
}

/// Finite `ResultInspection` host transport header.
#[derive(Clone, Debug, Serialize, schemars::JsonSchema)]
#[schemars(deny_unknown_fields)]
#[schemars(extend("x-pse-host-base" = "Deployment"))]
pub struct ResultInspection {
    /// Existing inherited header.
    #[schemars(flatten)]
    pub base: Deployment,
    /// Original `request` transport field.
    pub request: Value,
}

/// Finite `EvidenceCompare` host transport header.
#[derive(Clone, Debug, Serialize, schemars::JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct EvidenceCompare {
    /// Original `request` transport field.
    pub request: Value,
    /// Original `snapshots` transport field.
    #[schemars(length(min = 1, max = 64))]
    pub snapshots: Vec<Value>,
    /// Original `observations` transport field.
    #[schemars(length(max = 64), default)]
    pub observations: Vec<Value>,
}

/// Finite `ProfileCaptureRequest` host transport header.
#[derive(Clone, Debug, Serialize, schemars::JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct ProfileCaptureRequest {
    /// Original `request` transport field.
    pub request: Value,
    /// Original `expected` transport field.
    pub expected: Value,
    /// Original `suspect` transport field.
    pub suspect: Value,
}

/// Finite `ConfiguredCapabilities` host transport header.
#[derive(Clone, Debug, Serialize, schemars::JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct ConfiguredCapabilities {
    /// Original `backend` transport field.
    #[schemars(default)]
    pub backend: Option<pse_model::generated::enums::NativeBackend>,
    /// Original `settings` transport field.
    #[schemars(default)]
    pub settings: Option<Value>,
    /// Original `resources` transport field.
    #[schemars(extend("x-pse-host-default" = "Resources"))]
    pub resources: Resources,
}

fn document_default() -> usize {
    HOST_DOCUMENT_MAX
}
fn result_default() -> usize {
    HOST_RESULT_MAX
}
/// Directional host input schemas; all scientific bodies remain original document references.
pub fn header_schemas() -> Vec<(&'static str, Value)> {
    let mut schemas = Vec::new();
    macro_rules! header {
        ($ty:ty) => {
            schemas.push((
                stringify!($ty),
                schemars::generate::SchemaSettings::draft2020_12()
                    .for_deserialize()
                    .into_generator()
                    .into_root_schema_for::<$ty>()
                    .to_value(),
            ));
        };
    }
    header!(Resources);
    header!(Source);
    header!(Preparation);
    header!(ModelingComparison);
    header!(Deployment);
    header!(Metadata);
    header!(Select);
    header!(Revision);
    header!(RevisionPreparation);
    header!(Bundle);
    header!(Diagnostic);
    header!(DiagnosticProcesses);
    header!(DiagnosticCheckpoint);
    header!(Evidence);
    header!(Invocation);
    header!(Result);
    header!(PhysicalInspection);
    header!(ResultInspection);
    header!(EvidenceCompare);
    header!(ProfileCaptureRequest);
    header!(ConfiguredCapabilities);
    header!(DocumentReference);
    header!(DocumentField);
    header!(ContinuationField);
    header!(EffectFacts);
    header!(ExtensionRequirement);
    header!(OperationDeclaration);
    header!(OperationCatalogue);
    schemas
}

/// Finite product operations; supplied-only evidence is contributed by its tooling owner.
const OPERATIONS: &[OperationDeclaration] = &[
        OperationDeclaration {
            operation: "telemetry_profile_capture",
            title: "Capture an explicitly selected native telemetry suspect",
            envelope: EnvelopeKind::ProfileCaptureRequest,
            documents: &[DocumentField::of::<pse_runtime::tooling::capture::CaptureRequest>("request"), DocumentField::of::<pse_runtime::observation::CheckpointIdentity>("expected")],
            receiver: "pse.diagnostics.read_checkpoint + capture + CaptureSession.wait/stop",
            description: "Associate a selected bounded telemetry observation with an explicit original native CaptureRequest and CheckpointIdentity. Start only the requested finite Timeline/CPU capture against its exact checkpoint/OperationId, then return the intact original CaptureDocument. The original native owner validates operation/liveness; preflight and postcheck preserve process/start association but are not atomic expected-identity admission. Elapsed telemetry time is not CPU time, and late attachment cannot recover preceding samples. Cancelling this attached helper drains capture only; it never cancels or restarts science, starts telemetry services, changes permissions/tuning or launches a profiler UI. Existing output directories refuse. Capture failure, symbols, overlap and loss remain independent of scientific outcomes.",
            domain: "diagnostics",
            effects: "Explicit bounded native Timeline/CPU capture and fresh retained output only; native capture owns samplers/exporters and drains them independently of target science.",
            effect_facts: EffectFacts::CAPTURE,
            extension: Some(ExtensionRequirement { mode: ExtensionMode::InstalledCorrectness, validated: true }),
            output_document: Some(<pse_runtime::tooling::capture::CaptureDocument as RegisteredDocument>::REFERENCE),
            output_many: false,
            worker: WorkerFamily::Native,
            prerequisites: Prerequisites::Extension,
            prepare_import: true,
            invocation_lifetime: InvocationLifetime::Attached,
            preserve_document: true,
            read_eligibility: ReadEligibility::Dedicated,
            public_binding: false,
            context_fields: &[],
            continuations: &[],
        },
        OperationDeclaration {
            operation: "model_declarations",
            title: "Read native declaration page",
            envelope: EnvelopeKind::Preparation,
            documents: &[DocumentField::of::<pse_runtime::workflow::ProjectDraftDocuments>("physical"), DocumentField::of::<pse_runtime::workflow::ProjectDraftDocuments>("documents"), DocumentField::of::<pse_runtime::workflow::ModelingDeclarationRequest>("request")],
            receiver: "pse.ModelingPackage.declaration_page",
            description: "Admit finite supplied scientific source and read an exact selected declaration page; local revision handles expire with the child.",
            domain: "projects",
            effects: "Native source admission and selected declaration observation; no solve.",
            effect_facts: EffectFacts::SOURCE_ADMISSION,
            extension: Some(ExtensionRequirement { mode: ExtensionMode::InstalledCorrectness, validated: true }),
            output_document: Some(<pse_runtime::workflow::ModelingDeclarationPage as RegisteredDocument>::REFERENCE),
            output_many: false,
            worker: WorkerFamily::Native,
            prerequisites: Prerequisites::Extension,
            prepare_import: true,
            invocation_lifetime: InvocationLifetime::Attached,
            preserve_document: true,
            read_eligibility: ReadEligibility::ExactContext,
            public_binding: true,
            context_fields: &["resources", "physical", "documents"],
            continuations: &[ContinuationField { input: &["request", "offset"], output: &["document", "next_offset"], kind: ContinuationKind::Offset }],
        },
        OperationDeclaration {
            operation: "model_inspect",
            title: "Inspect native declared model",
            envelope: EnvelopeKind::Preparation,
            documents: &[DocumentField::of::<pse_runtime::workflow::ProjectDraftDocuments>("physical"), DocumentField::of::<pse_runtime::workflow::ProjectDraftDocuments>("documents"), DocumentField::of::<pse_runtime::workflow::ModelingInspectionRequest>("request")],
            receiver: "pse.ModelingPackage.inspect",
            description: "Inspect selected instantiated lineage under an explicit native preparation policy; contextual admission can prepare providers, but no procedure is dispatched.",
            domain: "projects",
            effects: "Native source/provider admission and selected observation, not universally passive.",
            effect_facts: EffectFacts::SOURCE_ADMISSION,
            extension: Some(ExtensionRequirement { mode: ExtensionMode::InstalledCorrectness, validated: true }),
            output_document: Some(<pse_runtime::workflow::ModelingInspection as RegisteredDocument>::REFERENCE),
            output_many: false,
            worker: WorkerFamily::Native,
            prerequisites: Prerequisites::Extension,
            prepare_import: true,
            invocation_lifetime: InvocationLifetime::Attached,
            preserve_document: true,
            read_eligibility: ReadEligibility::ExactContext,
            public_binding: true,
            context_fields: &["resources", "physical", "documents"],
            continuations: &[ContinuationField { input: &["request", "offset"], output: &["document", "next_offset"], kind: ContinuationKind::Offset }],
        },
        OperationDeclaration {
            operation: "model_compare",
            title: "Compare native authored model sources",
            envelope: EnvelopeKind::ModelingComparison,
            documents: &[DocumentField::of::<pse_runtime::workflow::ProjectDraftDocuments>("physical"), DocumentField::of::<pse_runtime::workflow::ProjectDraftDocuments>("documents"), DocumentField::of::<pse_runtime::workflow::ProjectDraftDocuments>("other_physical"), DocumentField::of::<pse_runtime::workflow::ProjectDraftDocuments>("other_documents"), DocumentField::of::<pse_runtime::workflow::ModelingComparisonRequest>("request")],
            receiver: "pse.ModelingPackage.comparison_observation",
            description: "Admit two finite sources in one Runtime and compare selected native changes/details; source-only mode preserves unknown scientific impact across physical contexts.",
            domain: "projects",
            effects: "Native source admission and finite comparison; no solve or numerical equivalence claim.",
            effect_facts: EffectFacts::SOURCE_ADMISSION,
            extension: Some(ExtensionRequirement { mode: ExtensionMode::InstalledCorrectness, validated: true }),
            output_document: Some(<pse_runtime::workflow::ModelingComparisonObservation as RegisteredDocument>::REFERENCE),
            output_many: false,
            worker: WorkerFamily::Native,
            prerequisites: Prerequisites::Extension,
            prepare_import: true,
            invocation_lifetime: InvocationLifetime::Attached,
            preserve_document: true,
            read_eligibility: ReadEligibility::ExactContext,
            public_binding: true,
            context_fields: &["resources", "physical", "documents", "other_physical", "other_documents"],
            continuations: &[ContinuationField { input: &["request", "offset"], output: &["document", "next_offset"], kind: ContinuationKind::Offset }],
        },
        OperationDeclaration {
            operation: "source_admit",
            title: "Admit original authored source",
            envelope: EnvelopeKind::Source,
            documents: &[DocumentField::of::<pse_runtime::workflow::ProjectDraftDocuments>("physical"), DocumentField::of::<pse_runtime::workflow::ProjectDraftDocuments>("documents")],
            receiver: "pse.Runtime.physical_from_documents + modeling_from_documents",
            description: "Admit original raw package bytes through the native physical and modeling source loaders. JSON schemas describe the byte container; authoring.schema is guidance for applicable authored TOML sections, not a parser for TOML/.pse/Parquet. The returned local revision is an observation and cannot be reopened after this call.",
            domain: "projects",
            effects: "Prepare native import providers through their existing owner as needed, then admit ephemeral source; no canonical database or publication.",
            effect_facts: EffectFacts::SOURCE_ADMISSION,
            extension: Some(ExtensionRequirement { mode: ExtensionMode::InstalledCorrectness, validated: true }),
            output_document: None,
            output_many: false,
            worker: WorkerFamily::Native,
            prerequisites: Prerequisites::Extension,
            prepare_import: true,
            invocation_lifetime: InvocationLifetime::Attached,
            preserve_document: false,
            read_eligibility: ReadEligibility::ExactContext,
            public_binding: true,
            context_fields: &["resources", "physical", "documents"],
            continuations: &[],
        },
        OperationDeclaration {
            operation: "preparation_discover",
            title: "Inspect preparation availability",
            envelope: EnvelopeKind::Preparation,
            documents: &[DocumentField::of::<pse_runtime::workflow::ProjectDraftDocuments>("physical"), DocumentField::of::<pse_runtime::workflow::ProjectDraftDocuments>("documents"), DocumentField::of::<pse_runtime::workflow::PreparationDiscoveryRequest>("request")],
            receiver: "pse.ModelingPackage.preparation_discovery",
            description: "Admit original supplied source and inspect exact preparation availability within the same Runtime lifetime. This passively observes availability without preparing mathematics or opening canonical storage.",
            domain: "projects",
            effects: "Prepare native import providers through their existing owner as needed, then admit source and passively observe preparations.",
            effect_facts: EffectFacts::SOURCE_ADMISSION,
            extension: Some(ExtensionRequirement { mode: ExtensionMode::InstalledCorrectness, validated: true }),
            output_document: Some(<pse_runtime::workflow::PreparationDiscoveryReport as RegisteredDocument>::REFERENCE),
            output_many: false,
            worker: WorkerFamily::Native,
            prerequisites: Prerequisites::Extension,
            prepare_import: true,
            invocation_lifetime: InvocationLifetime::Attached,
            preserve_document: false,
            read_eligibility: ReadEligibility::ExactContext,
            public_binding: true,
            context_fields: &["resources", "physical", "documents"],
            continuations: &[ContinuationField { input: &["request", "durable_after"], output: &["preparation", "durable", "next"], kind: ContinuationKind::Opaque }],
        },
        OperationDeclaration {
            operation: "project_select",
            title: "Select exact project source",
            envelope: EnvelopeKind::Select,
            documents: &[DocumentField::of::<pse_runtime::workflow::ProjectDraftDocuments>("physical"), DocumentField::of::<pse_runtime::workflow::ProjectSelectRequest>("request")],
            receiver: "pse.Runtime.project_select",
            description: "Select a branch or bookmark through its actual native receiver, freezing source and physical identities. Uses the explicitly selected existing canonical database and original physical source bytes.",
            domain: "projects",
            effects: "Read selected canonical store; native physical admission, no publication.",
            effect_facts: EffectFacts::SOURCE_ADMISSION,
            extension: Some(ExtensionRequirement { mode: ExtensionMode::InstalledCorrectness, validated: true }),
            output_document: Some(<pse_runtime::workflow::ProjectSelection as RegisteredDocument>::REFERENCE),
            output_many: false,
            worker: WorkerFamily::Native,
            prerequisites: Prerequisites::ExtensionAndCanonical,
            prepare_import: false,
            invocation_lifetime: InvocationLifetime::Detached,
            preserve_document: false,
            read_eligibility: ReadEligibility::ExactContext,
            public_binding: true,
            context_fields: &["resources", "state", "database", "physical"],
            continuations: &[],
        },
        OperationDeclaration {
            operation: "project_metadata",
            title: "Read project metadata page",
            envelope: EnvelopeKind::Metadata,
            documents: &[DocumentField::of::<pse_operations::canonical_projects::ProjectMetadataRequest>("request")],
            receiver: "pse.Runtime.project_metadata",
            description: "Read the original bounded project/draft/bookmark/savepoint/diagnostic metadata page, preserving the issuing store and owner-bound continuation.",
            domain: "projects",
            effects: "Read explicitly selected existing canonical database; no provisioning.",
            effect_facts: EffectFacts::OBSERVATION,
            extension: Some(ExtensionRequirement { mode: ExtensionMode::InstalledCorrectness, validated: true }),
            output_document: Some(<pse_operations::canonical_projects::ProjectMetadataPage as RegisteredDocument>::REFERENCE),
            output_many: false,
            worker: WorkerFamily::Native,
            prerequisites: Prerequisites::ExtensionAndCanonical,
            prepare_import: false,
            invocation_lifetime: InvocationLifetime::Detached,
            preserve_document: false,
            read_eligibility: ReadEligibility::ExactContext,
            public_binding: true,
            context_fields: &["resources", "state", "database"],
            continuations: &[ContinuationField { input: &["request", "cursor"], output: &["document", "next"], kind: ContinuationKind::Opaque }],
        },
        OperationDeclaration {
            operation: "revision_source",
            title: "Inspect exact retained source revision",
            envelope: EnvelopeKind::Revision,
            documents: &[DocumentField::of::<pse_runtime::workflow::ProjectDraftDocuments>("physical")],
            receiver: "pse.Runtime.modeling_revision + ModelingPackage.project_source_detail",
            description: "Reopen the explicitly retained canonical revision with its original physical closure and read selected logical source detail. Source and binary identities remain native; a local revision token from an earlier ephemeral call cannot be used here.",
            domain: "projects",
            effects: "Protected canonical source read and native source admission; no source edit or publication.",
            effect_facts: EffectFacts::SOURCE_ADMISSION,
            extension: Some(ExtensionRequirement { mode: ExtensionMode::InstalledCorrectness, validated: true }),
            output_document: None,
            output_many: false,
            worker: WorkerFamily::Native,
            prerequisites: Prerequisites::ExtensionAndCanonical,
            prepare_import: false,
            invocation_lifetime: InvocationLifetime::Detached,
            preserve_document: false,
            read_eligibility: ReadEligibility::ExactContext,
            public_binding: true,
            context_fields: &["resources", "state", "database", "physical", "revision"],
            continuations: &[],
        },
        OperationDeclaration {
            operation: "revision_preparation_discover",
            title: "Inspect retained revision preparations",
            envelope: EnvelopeKind::RevisionPreparation,
            documents: &[DocumentField::of::<pse_runtime::workflow::ProjectDraftDocuments>("physical"), DocumentField::of::<pse_runtime::workflow::PreparationDiscoveryRequest>("request")],
            receiver: "pse.Runtime.modeling_revision + ModelingPackage.preparation_discovery",
            description: "Reopen an exact retained canonical revision and passively inspect its preparation/publication headers. Durable presence preserves unchecked receiving eligibility; no numerical preparation is requested.",
            domain: "projects",
            effects: "Protected canonical source/preparation metadata read, no numerical preparation.",
            effect_facts: EffectFacts::SOURCE_ADMISSION,
            extension: Some(ExtensionRequirement { mode: ExtensionMode::InstalledCorrectness, validated: true }),
            output_document: Some(<pse_runtime::workflow::PreparationDiscoveryReport as RegisteredDocument>::REFERENCE),
            output_many: false,
            worker: WorkerFamily::Native,
            prerequisites: Prerequisites::ExtensionAndCanonical,
            prepare_import: false,
            invocation_lifetime: InvocationLifetime::Detached,
            preserve_document: false,
            read_eligibility: ReadEligibility::ExactContext,
            public_binding: true,
            context_fields: &["resources", "state", "database", "physical", "revision"],
            continuations: &[ContinuationField { input: &["request", "durable_after"], output: &["document", "durable", "next"], kind: ContinuationKind::Opaque }],
        },
        OperationDeclaration {
            operation: "project_bundle_inspect",
            title: "Inspect supplied project pack offline",
            envelope: EnvelopeKind::Bundle,
            documents: &[],
            receiver: "pse.ProjectBundle.manifest + digest",
            description: "Verify and inspect a supplied copied project closure through the ordinary native bundle owner. Reads its original manifest and digest without opening a canonical database or provisioning storage.",
            domain: "projects",
            effects: "Read and digest-check supplied project pack only.",
            effect_facts: EffectFacts::OBSERVATION,
            extension: Some(ExtensionRequirement { mode: ExtensionMode::InstalledCorrectness, validated: true }),
            output_document: Some(<pse_runtime::workflow::ProjectManifest as RegisteredDocument>::REFERENCE),
            output_many: false,
            worker: WorkerFamily::Native,
            prerequisites: Prerequisites::Extension,
            prepare_import: false,
            invocation_lifetime: InvocationLifetime::Detached,
            preserve_document: false,
            read_eligibility: ReadEligibility::Dedicated,
            public_binding: true,
            context_fields: &["path", "maximum_document_bytes"],
            continuations: &[],
        },
        OperationDeclaration {
            operation: "research_read",
            title: "Read original scientific research metadata",
            envelope: EnvelopeKind::Metadata,
            documents: &[DocumentField::of::<pse_runtime::workflow::ResearchReadRequest>("request")],
            receiver: "pse.Runtime.research_read",
            description: "Read bounded immutable scientific research dataset, selection or publication metadata through the original native owner. Metadata grants no measurement or result lifetime.",
            domain: "projects",
            effects: "Bounded read of explicitly selected canonical research metadata.",
            effect_facts: EffectFacts::OBSERVATION,
            extension: Some(ExtensionRequirement { mode: ExtensionMode::InstalledCorrectness, validated: true }),
            output_document: Some(<pse_runtime::workflow::ResearchReadResult as RegisteredDocument>::REFERENCE),
            output_many: false,
            worker: WorkerFamily::Native,
            prerequisites: Prerequisites::ExtensionAndCanonical,
            prepare_import: false,
            invocation_lifetime: InvocationLifetime::Detached,
            preserve_document: false,
            read_eligibility: ReadEligibility::ExactContext,
            public_binding: true,
            context_fields: &["resources", "state", "database"],
            continuations: &[],
        },
        OperationDeclaration {
            operation: "diagnostic_processes",
            title: "Discover published native diagnostic processes",
            envelope: EnvelopeKind::DiagnosticProcesses,
            documents: &[DocumentField::of::<pse_runtime::tooling::inspection::DiagnosticListRequest>("request")],
            receiver: "pse.diagnostics.list_processes",
            description: "Read one bounded page of native process checkpoint summaries from the selected diagnostic directory through the original native reader. Retains process/start locators, recorded target runtime_metrics and operation IDs when available. Directory discovery is not an atomic snapshot or live-process/attachment authority; no Runtime construction, service startup or capture.",
            domain: "diagnostics",
            effects: "Read bounded selected native diagnostic process summaries only; no subject execution or settlement.",
            effect_facts: EffectFacts::OBSERVATION,
            extension: Some(ExtensionRequirement { mode: ExtensionMode::InstalledCorrectness, validated: true }),
            output_document: Some(<pse_runtime::tooling::inspection::DiagnosticProcessPage as RegisteredDocument>::REFERENCE),
            output_many: false,
            worker: WorkerFamily::Native,
            prerequisites: Prerequisites::Extension,
            prepare_import: false,
            invocation_lifetime: InvocationLifetime::Detached,
            preserve_document: true,
            read_eligibility: ReadEligibility::Dedicated,
            public_binding: true,
            context_fields: &["directory"],
            continuations: &[ContinuationField { input: &["request", "after"], output: &["document", "next"], kind: ContinuationKind::Opaque }],
        },
        OperationDeclaration {
            operation: "diagnostic_checkpoint",
            title: "Read an exact native diagnostic checkpoint",
            envelope: EnvelopeKind::DiagnosticCheckpoint,
            documents: &[DocumentField::of::<pse_runtime::observation::CheckpointIdentity>("expected")],
            receiver: "pse.diagnostics.read_checkpoint",
            description: "Decode one exact bounded published native checkpoint, including target runtime_metrics and registration coverage, without requiring an operation ID. Optional expected identity validates the same decoded image's original diagnostic process/start association. Recorded observations do not prove current liveness or grant attachment, cancellation or settlement authority. No Runtime construction, service startup or capture.",
            domain: "diagnostics",
            effects: "Read supplied native checkpoint and optionally validate its exact original association; no execution or publication.",
            effect_facts: EffectFacts::OBSERVATION,
            extension: Some(ExtensionRequirement { mode: ExtensionMode::InstalledCorrectness, validated: true }),
            output_document: Some(<pse_runtime::observation::ProcessCheckpoint as RegisteredDocument>::REFERENCE),
            output_many: false,
            worker: WorkerFamily::Native,
            prerequisites: Prerequisites::Extension,
            prepare_import: false,
            invocation_lifetime: InvocationLifetime::Detached,
            preserve_document: true,
            read_eligibility: ReadEligibility::Dedicated,
            public_binding: true,
            context_fields: &["checkpoint", "expected"],
            continuations: &[],
        },
        OperationDeclaration {
            operation: "diagnostic_inspect",
            title: "Inspect published diagnostic checkpoint",
            envelope: EnvelopeKind::Diagnostic,
            documents: &[DocumentField::of::<pse_runtime::tooling::inspection::InspectionRequest>("request")],
            receiver: "pse.diagnostics.inspect_checkpoint",
            description: "Inspect an exact published checkpoint through native decoding and interpretation, including target runtime_metrics with original target locator, executor role and registration coverage when present. These are recorded target observations, not calling/helper-process metrics. Does not construct a Runtime, start a service, or cancel the observed subject.",
            domain: "diagnostics",
            effects: "Read supplied native checkpoint only.",
            effect_facts: EffectFacts::OBSERVATION,
            extension: Some(ExtensionRequirement { mode: ExtensionMode::InstalledCorrectness, validated: true }),
            output_document: Some(<pse_runtime::tooling::inspection::InspectionDocument as RegisteredDocument>::REFERENCE),
            output_many: false,
            worker: WorkerFamily::Native,
            prerequisites: Prerequisites::Extension,
            prepare_import: false,
            invocation_lifetime: InvocationLifetime::Detached,
            preserve_document: true,
            read_eligibility: ReadEligibility::Dedicated,
            public_binding: true,
            context_fields: &["checkpoint"],
            continuations: &[],
        },
        OperationDeclaration {
            operation: "evidence_inspect",
            title: "Inspect supplied frozen evidence",
            envelope: EnvelopeKind::Evidence,
            documents: &[DocumentField::of::<pse_runtime::tooling::frozen_inspection::FrozenInspectionRequest>("request")],
            receiver: "pse.diagnostics.read_evidence + inspect_frozen",
            description: "Read a supplied bounded saved evidence set and interpret its immutable members at recorded source time. Embedded paths do not authorize lookup; canonical storage is not acquired.",
            domain: "diagnostics",
            effects: "Read and interpret supplied frozen evidence only.",
            effect_facts: EffectFacts::OBSERVATION,
            extension: Some(ExtensionRequirement { mode: ExtensionMode::InstalledCorrectness, validated: true }),
            output_document: Some(<pse_runtime::tooling::frozen_inspection::FrozenInspectionDocument as RegisteredDocument>::REFERENCE),
            output_many: false,
            worker: WorkerFamily::Native,
            prerequisites: Prerequisites::Extension,
            prepare_import: false,
            invocation_lifetime: InvocationLifetime::Detached,
            preserve_document: false,
            read_eligibility: ReadEligibility::Dedicated,
            public_binding: true,
            context_fields: &["path"],
            continuations: &[],
        },
        OperationDeclaration {
            operation: "invocation_read",
            title: "Read original native invocation evidence",
            envelope: EnvelopeKind::Invocation,
            documents: &[],
            receiver: "pse.diagnostics.read_invocation",
            description: "Decode descriptive invocation facts through the native saved-evidence reader. This grants no live result handle and does not replay the invocation.",
            domain: "diagnostics",
            effects: "Read supplied invocation evidence only.",
            effect_facts: EffectFacts::OBSERVATION,
            extension: Some(ExtensionRequirement { mode: ExtensionMode::InstalledCorrectness, validated: true }),
            output_document: Some(<pse_runtime::invocation::InvocationEvidenceDocument as RegisteredDocument>::REFERENCE),
            output_many: false,
            worker: WorkerFamily::Native,
            prerequisites: Prerequisites::Extension,
            prepare_import: false,
            invocation_lifetime: InvocationLifetime::Detached,
            preserve_document: false,
            read_eligibility: ReadEligibility::Dedicated,
            public_binding: true,
            context_fields: &["path"],
            continuations: &[],
        },
        OperationDeclaration {
            operation: "result_read",
            title: "Read exact stored scientific result window",
            envelope: EnvelopeKind::Result,
            documents: &[],
            receiver: "pse.Runtime.results",
            description: "Read a bounded original run/attempt/relation row window through the protected native Arrow reader. Returns Arrow IPC stream bytes, preserving IEEE payloads, units and schema metadata. A detail bound asks for a narrower exact window; it never silently substitutes rows or attempts.",
            domain: "projects",
            effects: "Protected read of explicitly selected canonical results; no scientific execution or publication.",
            effect_facts: EffectFacts::OBSERVATION,
            extension: Some(ExtensionRequirement { mode: ExtensionMode::InstalledCorrectness, validated: true }),
            output_document: None,
            output_many: false,
            worker: WorkerFamily::Native,
            prerequisites: Prerequisites::ExtensionAndCanonical,
            prepare_import: false,
            invocation_lifetime: InvocationLifetime::Detached,
            preserve_document: true,
            read_eligibility: ReadEligibility::ExactContext,
            public_binding: true,
            context_fields: &["resources", "state", "database", "run", "attempt", "relation"],
            continuations: &[],
        },
        OperationDeclaration {
            operation: "physical_inspect",
            title: "Describe an admitted original physical context",
            envelope: EnvelopeKind::PhysicalInspection,
            documents: &[DocumentField::of::<pse_runtime::workflow::ProjectDraftDocuments>("physical"), DocumentField::of::<pse_runtime::workflow::PhysicalInspectionRequest>("request")],
            receiver: "pse.PhysicalContext.describe",
            description: "Admit the supplied physical document closure through the ordinary native owner and describe a bounded quantity or named reference-state page. Names, typed identities, exact dimension, basis, datum, subject and affine point/difference conversion come from that same Rust contract.",
            domain: "projects",
            effects: "Native physical admission and bounded observation; no canonical publication or scientific execution.",
            effect_facts: EffectFacts::SOURCE_ADMISSION,
            extension: Some(ExtensionRequirement { mode: ExtensionMode::InstalledCorrectness, validated: true }),
            output_document: Some(<pse_runtime::workflow::PhysicalInspectionDocument as RegisteredDocument>::REFERENCE),
            output_many: false,
            worker: WorkerFamily::Native,
            prerequisites: Prerequisites::Extension,
            prepare_import: false,
            invocation_lifetime: InvocationLifetime::Detached,
            preserve_document: true,
            read_eligibility: ReadEligibility::ExactContext,
            public_binding: true,
            context_fields: &["resources", "physical"],
            continuations: &[ContinuationField { input: &["request", "offset"], output: &["document", "next"], kind: ContinuationKind::Offset }],
        },
        OperationDeclaration {
            operation: "result_inspect",
            title: "Inspect protected scientific result shape or readable selected values",
            envelope: EnvelopeKind::ResultInspection,
            documents: &[DocumentField::of::<pse_runtime::workflow::ResultInspectionRequest>("request")],
            receiver: "pse.Runtime.inspect_result",
            description: "Inspect an exact protected run/attempt/relation. Schema view does not acquire data batches. Values view selects bounded fields and rows natively, retains original identity, units, basis and quality context, and represents null/nonfinite/unsupported cells explicitly. Exact original Arrow IPC remains native_result_read. No scientific usability is inferred.",
            domain: "projects",
            effects: "Protected read of the explicitly selected existing canonical result; no execution, provision or publication.",
            effect_facts: EffectFacts::OBSERVATION,
            extension: Some(ExtensionRequirement { mode: ExtensionMode::InstalledCorrectness, validated: true }),
            output_document: Some(<pse_runtime::workflow::ResultInspectionDocument as RegisteredDocument>::REFERENCE),
            output_many: false,
            worker: WorkerFamily::Native,
            prerequisites: Prerequisites::ExtensionAndCanonical,
            prepare_import: false,
            invocation_lifetime: InvocationLifetime::Detached,
            preserve_document: true,
            read_eligibility: ReadEligibility::ExactContext,
            public_binding: true,
            context_fields: &["resources", "state", "database"],
            continuations: &[ContinuationField { input: &["request", "cursor"], output: &["document", "next_cursor"], kind: ContinuationKind::Opaque }],
        },
        OperationDeclaration {
            operation: "configured_capabilities",
            title: "Inspect actual linked or configured native capability",
            envelope: EnvelopeKind::ConfiguredCapabilities,
            documents: &[DocumentField::of::<pse_backend_native::execution::BackendSettings>("settings")],
            receiver: "pse.Runtime.capabilities + capability_for",
            description: "Read actual linked native adapter rows, or select one backend with its original typed BackendSettings to inspect that configured adapter. This creates no scientific model, prepares no numerical product and opens no canonical store. A capability row does not establish model eligibility or solve outcome.",
            domain: "projects",
            effects: "Observe the existing linked or configured native adapter capability only; no numerical execution or canonical publication.",
            effect_facts: EffectFacts::OBSERVATION,
            extension: Some(ExtensionRequirement { mode: ExtensionMode::InstalledCorrectness, validated: true }),
            output_document: Some(<pse_model::generated::runtime::solver_capabilities::Row as RegisteredDocument>::REFERENCE),
            output_many: true,
            worker: WorkerFamily::Native,
            prerequisites: Prerequisites::Extension,
            prepare_import: false,
            invocation_lifetime: InvocationLifetime::Detached,
            preserve_document: false,
            read_eligibility: ReadEligibility::ExactContext,
            public_binding: true,
            context_fields: &["resources"],
            continuations: &[],
        },
];
/// The finite source-owned operation declarations.
pub fn operations() -> &'static [OperationDeclaration] {
    OPERATIONS
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn host_operations_keep_typed_direction_prerequisite_and_preservation_distinct() {
        let operations = operations();
        assert_eq!(operations.len(), 21);
        let mut identities = std::collections::BTreeSet::new();
        for operation in operations {
            assert!(identities.insert(operation.operation));
            assert!(operation.documents.iter().all(|field| field.document.input));
            assert!(
                operation
                    .output_document
                    .is_none_or(|reference| reference.output)
            );
            assert!(!matches!(operation.worker, WorkerFamily::Evidence));
        }
        let capture = operations
            .iter()
            .find(|op| op.operation == "telemetry_profile_capture")
            .unwrap();
        assert!(capture.preserve_document);
        assert!(matches!(
            capture.read_eligibility,
            ReadEligibility::Dedicated
        ));
        let capability = operations
            .iter()
            .find(|op| op.operation == "configured_capabilities")
            .unwrap();
        assert!(matches!(capability.prerequisites, Prerequisites::Extension));
        assert_eq!(capability.documents[0].document.slug, "backend-settings");
        assert_eq!(
            capability.output_document.unwrap().slug,
            "solver-capability"
        );
        assert!(capability.output_many);
        assert!(!capability.preserve_document);
        assert!(matches!(
            capability.read_eligibility,
            ReadEligibility::ExactContext
        ));
    }
}

/// Existing typed project operator protocol and public host projection declarations.
pub mod project;
