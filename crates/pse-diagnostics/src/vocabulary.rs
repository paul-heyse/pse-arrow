// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
#![allow(
    missing_docs,
    reason = "strum's generated const into_str method is exposed through documented as_str contracts"
)]

//! One declaration of platform classes and detailed diagnostic codes.

/// A spelling outside a closed, source-owned vocabulary (ADR-0115 Outcome 3).
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum VocabularyError {
    /// The spelling names no member of the vocabulary.
    #[error("`{value}` is not a member of `{vocabulary}`")]
    UnknownMember {
        /// The vocabulary's registry name.
        vocabulary: &'static str,
        /// The refused spelling.
        value: String,
    },
}
crate::impl_diagnostic! {
    VocabularyError,
    code(_this) { Some(DiagnosticCode::SchemaEnumMember) },
    forward(_this) { None }, help(_this) { None }, related(_this) { None }, source(_this) { None }
}

/// The `postgres` feature maps each vocabulary to the store's ENUM type of the same
/// snake-case name, member by member (ADR-0117 Outcome 4, ADR-0114 Outcome 25).
macro_rules! vocabulary {
    ($name:ident as $sql:literal { $($variant:ident => ($text:literal, $description:literal)),* $(,)? }) => {
        #[doc = "Stable platform diagnostic vocabulary."]
        #[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, strum::EnumString, strum::Display, strum::VariantArray, strum::IntoStaticStr, serde::Serialize, serde::Deserialize)]
        #[serde(try_from = "String", into = "String")]
        #[strum(const_into_str, parse_err_ty = VocabularyError, parse_err_fn = Self::unknown_member)]
        #[cfg_attr(feature = "postgres", derive(postgres_types::ToSql, postgres_types::FromSql), postgres(name = $sql))]
        pub enum $name { $(#[doc = $description] #[strum(serialize = $text)] #[cfg_attr(feature = "postgres", postgres(name = $text))] $variant),* }
        impl $name {
            /// All declarations in stable order.
            pub const ALL: &'static [Self] = <Self as strum::VariantArray>::VARIANTS;
            /// Registry spelling.
            pub const fn as_str(self) -> &'static str { self.into_str() }
            /// Declared description.
            pub const fn description(self) -> &'static str { match self { $(Self::$variant => $description),* } }
            /// The member with this registry spelling, if any.
            pub fn parse(value: &str) -> Option<Self> {
                value.parse().ok()
            }
        }
        impl schemars::JsonSchema for $name {
            fn schema_name() -> std::borrow::Cow<'static,str> { stringify!($name).into() }
            fn schema_id() -> std::borrow::Cow<'static,str> { concat!(module_path!(),"::",stringify!($name)).into() }
            fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema { schemars::json_schema!({"type":"string","enum":Self::ALL.iter().map(|v|v.as_str()).collect::<Vec<_>>()}) }
        }
        impl From<$name> for String { fn from(value: $name) -> Self { value.as_str().to_owned() } }
        impl TryFrom<String> for $name { type Error = VocabularyError; fn try_from(value: String) -> Result<Self, Self::Error> { value.parse() } }
        impl $name {
            fn unknown_member(value: &str) -> VocabularyError {
                VocabularyError::UnknownMember { vocabulary: stringify!($name), value: value.to_owned() }
            }
        }
    };
}

vocabulary! { FailureClass as "failure_class" {
    AuthoringParse => ("authoring.parse", "A syntax error or an unknown key."),
    AuthoringReference => ("authoring.reference", "An unknown path or a derived write."),
    ValidationInvariant => ("validation.invariant", "A declared invariant does not hold."),
    CompileFeature => ("compile.feature", "An incompatible feature combination."),
    CompileProperty => ("compile.property", "An unsupported or ambiguous property."),
    CompileLaw => ("compile.law", "An unsupported balance binding."),
    CompileMath => ("compile.math", "Incompatible physical contracts or a cyclic expression."),
    KernelUnboundParameter => ("kernel.unbound_parameter", "A selected kernel lacks its actual executable or parameter binding."),
    CompileDiscretization => ("compile.discretization", "A mixed derivative or a missing policy."),
    CapabilityBackend => ("capability.backend", "An unsupported opcode or missing derivative."),
    PlanInitialization => ("plan.initialization", "A structural singularity or a failed postcheck."),
    SolveInfeasible => ("solve.infeasible", "The problem is infeasible."),
    SolveLocallyInfeasible => ("solve.locally_infeasible", "The solver converged to local infeasibility."),
    SolveUnbounded => ("solve.unbounded", "The objective is unbounded."),
    SolveLimit => ("solve.limit", "An iteration, time or evaluation limit."),
    SolveEvaluationError => ("solve.evaluation_error", "A function evaluation failed."),
    SolveSolverError => ("solve.solver_error", "The solver reported an internal failure."),
    RuntimeCancelled => ("runtime.cancelled", "A cancellation token fired."),
    RuntimeTimeout => ("runtime.timeout", "A wall-clock limit fired."),
    RuntimeResourceLimit => ("runtime.resource_limit", "A reservation or size limit was exceeded."),
    RuntimeInfrastructure => ("runtime.infrastructure", "Store input/output or an integrity failure."),
    ConfigInvalid => ("config.invalid", "An invalid engine or platform configuration key."),
    InternalInvariant => ("internal.invariant", "A pass postcondition failed."),
    UserModel => ("user.model", "An authored assertion or user equation failed."),
} }

macro_rules! codes {
    ($($variant:ident => ($text:literal, $class:ident, $description:literal)),* $(,)?) => {
        /// Detailed diagnostic identity; several codes can share a failure class.
        #[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, strum::VariantArray, strum::IntoStaticStr, serde::Serialize, serde::Deserialize)]
        #[strum(const_into_str)]
        #[serde(try_from = "String", into = "String")]
        #[cfg_attr(feature = "postgres", derive(postgres_types::ToSql, postgres_types::FromSql), postgres(name = "diagnostic_code"))]
        pub enum DiagnosticCode { $(#[doc = $description] #[strum(serialize = $text)] #[cfg_attr(feature = "postgres", postgres(name = $text))] $variant),* }
        impl DiagnosticCode {
            /// All detailed codes.
            pub const ALL: &'static [Self] = <Self as strum::VariantArray>::VARIANTS;
            /// Stable dotted registry spelling.
            pub const fn as_str(self) -> &'static str { self.into_str() }
            /// Coarse failure classification.
            pub const fn class(self) -> FailureClass { match self { $(Self::$variant => FailureClass::$class),* } }
            /// Human-readable description.
            pub const fn description(self) -> &'static str { match self { $(Self::$variant => $description),* } }
            /// Read a declared dotted or miette path spelling.
            pub fn parse(value: &str) -> Option<Self> {
                let dotted = value.replace("::", ".");
                match dotted.as_str() { $($text => Some(Self::$variant),)* _ => None }
            }
        }
    };
}

codes! {
    AuthoringParse => ("authoring.parse", AuthoringParse, "A syntax error or an unknown key."),
    AuthoringParseAmbiguousUnaryPower => ("authoring.parse.ambiguous_unary_power", AuthoringParse, "authoring parse ambiguous unary power."),
    AuthoringParseBudget => ("authoring.parse.budget", AuthoringParse, "authoring parse budget."),
    AuthoringParseDocumentIo => ("authoring.parse.document_io", AuthoringParse, "authoring parse document io."),
    AuthoringParseMissingId => ("authoring.parse.missing_id", AuthoringParse, "authoring parse missing id."),
    AuthoringParseNonfiniteNumber => ("authoring.parse.nonfinite_number", AuthoringParse, "authoring parse nonfinite number."),
    AuthoringParseSyntax => ("authoring.parse.syntax", AuthoringParse, "authoring parse syntax."),
    AuthoringParseUnknownKey => ("authoring.parse.unknown_key", AuthoringParse, "authoring parse unknown key."),
    AuthoringParseUnresolvedTarget => ("authoring.parse.unresolved_target", AuthoringParse, "authoring parse unresolved target."),
    AuthoringPkgUnresolved => ("authoring.pkg.unresolved", AuthoringReference, "authoring pkg unresolved."),
    AuthoringPkgVersionConflict => ("authoring.pkg.version_conflict", AuthoringReference, "authoring pkg version conflict."),
    AuthoringReference => ("authoring.reference", AuthoringReference, "An unknown path or a derived write."),
    AuthoringReferenceContract => ("authoring.reference.contract", AuthoringReference, "authoring reference contract."),
    AuthoringReferenceDerivedWrite => ("authoring.reference.derived_write", AuthoringReference, "authoring reference derived write."),
    AuthoringReferenceRenameNamed => ("authoring.reference.rename_named", AuthoringReference, "authoring reference rename named."),
    AuthoringReferenceUnknownRowKey => ("authoring.reference.unknown_row_key", AuthoringReference, "authoring reference unknown row key."),
    CapabilityBackend => ("capability.backend", CapabilityBackend, "An unsupported opcode or missing derivative."),
    CompileDiscretization => ("compile.discretization", CompileDiscretization, "A mixed derivative or a missing policy."),
    CompileFeature => ("compile.feature", CompileFeature, "An incompatible feature combination."),
    CompileLaw => ("compile.law", CompileLaw, "An unsupported balance binding."),
    CompileMath => ("compile.math", CompileMath, "Incompatible physical contracts or a cyclic expression."),
    CompileMathCyclicExpression => ("compile.math.cyclic_expression", CompileMath, "compile math cyclic expression."),
    CompileMathDomainViolationStatic => ("compile.math.domain_violation_static", CompileMath, "compile math domain violation static."),
    CompileMathQuantityOperationUnsupported => ("compile.math.quantity_operation_unsupported", CompileMath, "compile math quantity operation unsupported."),
    CompileMathUnitInconsistent => ("compile.math.unit_inconsistent", CompileMath, "compile math unit inconsistent."),
    CompileProperty => ("compile.property", CompileProperty, "An unsupported or ambiguous property."),
    ConfigInvalid => ("config.invalid", ConfigInvalid, "An invalid engine or platform configuration key."),
    InternalInvariant => ("internal.invariant", InternalInvariant, "A pass postcondition failed."),
    KernelUnboundParameter => ("kernel.unbound_parameter", KernelUnboundParameter, "A selected kernel lacks its actual executable or parameter binding."),
    PlanInitialization => ("plan.initialization", PlanInitialization, "A structural singularity or a failed postcheck."),
    RuleFloatKey => ("rule.float_key", ValidationInvariant, "rule float key."),
    RuleHeadSchemaMismatch => ("rule.head_schema_mismatch", ValidationInvariant, "rule head schema mismatch."),
    RuntimeCancelled => ("runtime.cancelled", RuntimeCancelled, "A cancellation token fired."),
    RuntimeInfrastructure => ("runtime.infrastructure", RuntimeInfrastructure, "Store input/output or an integrity failure."),
    RuntimeResourceLimit => ("runtime.resource_limit", RuntimeResourceLimit, "A reservation or size limit was exceeded."),
    RuntimeTimeout => ("runtime.timeout", RuntimeTimeout, "A wall-clock limit fired."),
    SchemaAdmission => ("schema.admission", ValidationInvariant, "schema admission."),
    SchemaArrow => ("schema.arrow", ValidationInvariant, "schema arrow."),
    SchemaCodegen => ("schema.codegen", ValidationInvariant, "schema codegen."),
    SchemaContractMismatch => ("schema.contract_mismatch", ValidationInvariant, "schema contract mismatch."),
    SchemaDuplicateDeclaration => ("schema.duplicate_declaration", ValidationInvariant, "schema duplicate declaration."),
    SchemaEnumMember => ("schema.enum_member", ValidationInvariant, "schema enum member."),
    SchemaExtensionMetadata => ("schema.extension_metadata", ValidationInvariant, "schema extension metadata."),
    SchemaExtensionType => ("schema.extension_type", ValidationInvariant, "schema extension type."),
    SchemaFingerprintMismatch => ("schema.fingerprint_mismatch", ValidationInvariant, "schema fingerprint mismatch."),
    SchemaInvalidDeclaration => ("schema.invalid_declaration", ValidationInvariant, "schema invalid declaration."),
    SchemaInvalidKey => ("schema.invalid_key", ValidationInvariant, "schema invalid key."),
    SchemaMissingGranularity => ("schema.missing_granularity", ValidationInvariant, "schema missing granularity."),
    SchemaMissingSnapshotClass => ("schema.missing_snapshot_class", ValidationInvariant, "schema missing snapshot class."),
    SchemaNullability => ("schema.nullability", ValidationInvariant, "schema nullability."),
    SchemaOrdinalRange => ("schema.ordinal_range", ValidationInvariant, "schema ordinal range."),
    SchemaRuleFloatKey => ("schema.rule_float_key", ValidationInvariant, "schema rule float key."),
    SchemaRuleStratification => ("schema.rule_stratification", ValidationInvariant, "schema rule stratification."),
    SchemaStorage => ("schema.storage", ValidationInvariant, "schema storage."),
    SchemaUnknownMetadata => ("schema.unknown_metadata", ValidationInvariant, "schema unknown metadata."),
    SchemaUnknownReference => ("schema.unknown_reference", ValidationInvariant, "schema unknown reference."),
    SchemaUnknownRegistry => ("schema.unknown_registry", ValidationInvariant, "schema unknown registry."),
    SchemaUnsupportedLayout => ("schema.unsupported_layout", ValidationInvariant, "schema unsupported layout."),
    SchemaVersionMismatch => ("schema.version_mismatch", ValidationInvariant, "schema version mismatch."),
    SolveEvaluationError => ("solve.evaluation_error", SolveEvaluationError, "A function evaluation failed."),
    SolveInfeasible => ("solve.infeasible", SolveInfeasible, "The problem is infeasible."),
    SolveLimit => ("solve.limit", SolveLimit, "An iteration, time or evaluation limit."),
    SolveLocallyInfeasible => ("solve.locally_infeasible", SolveLocallyInfeasible, "The solver converged to local infeasibility."),
    SolveSolverError => ("solve.solver_error", SolveSolverError, "The solver reported an internal failure."),
    SolveUnbounded => ("solve.unbounded", SolveUnbounded, "The objective is unbounded."),
    TemplateGuardUndecidable => ("template.guard_undecidable", ValidationInvariant, "template guard undecidable."),
    UserModel => ("user.model", UserModel, "An authored assertion or user equation failed."),
    ValidationInvariant => ("validation.invariant", ValidationInvariant, "A declared invariant does not hold."),
    AuthoringBudget => ("authoring.budget", AuthoringParse, "authoring budget diagnostic contract."),
    AuthoringContract => ("authoring.contract", AuthoringReference, "authoring contract diagnostic contract."),
    AuthoringDerivedWrite => ("authoring.derived_write", AuthoringReference, "authoring derived write diagnostic contract."),
    AuthoringDocumentIo => ("authoring.document_io", AuthoringParse, "authoring document io diagnostic contract."),
    AuthoringMissingId => ("authoring.missing_id", AuthoringParse, "authoring missing id diagnostic contract."),
    AuthoringPackageUnresolved => ("authoring.package_unresolved", AuthoringReference, "authoring package unresolved diagnostic contract."),
    AuthoringPackageVersionConflict => ("authoring.package_version_conflict", AuthoringReference, "authoring package version conflict diagnostic contract."),
    AuthoringRenameNamed => ("authoring.rename_named", AuthoringReference, "authoring rename named diagnostic contract."),
    AuthoringSchemaVersion => ("authoring.schema_version", ValidationInvariant, "authoring schema version diagnostic contract."),
    AuthoringSyntax => ("authoring.syntax", AuthoringParse, "authoring syntax diagnostic contract."),
    AuthoringUnknownKey => ("authoring.unknown_key", AuthoringParse, "authoring unknown key diagnostic contract."),
    AuthoringUnknownRowKey => ("authoring.unknown_row_key", AuthoringReference, "authoring unknown row key diagnostic contract."),
    AuthoringUnresolvedTarget => ("authoring.unresolved_target", AuthoringParse, "authoring unresolved target diagnostic contract."),
    CandidateEvaluationFailed => ("candidate.evaluation_failed", SolveEvaluationError, "candidate evaluation failed diagnostic contract."),
    CompilerCancelled => ("compiler.cancelled", RuntimeCancelled, "compiler cancelled diagnostic contract."),
    CompilerLimit => ("compiler.limit", RuntimeResourceLimit, "compiler limit diagnostic contract."),
    CompilerMath => ("compiler.math", CompileMath, "compiler math diagnostic contract."),
    CompilerMissing => ("compiler.missing", CompileMath, "compiler missing diagnostic contract."),
    CompilerModeling => ("compiler.modeling", CompileMath, "compiler modeling diagnostic contract."),
    CompilerStructure => ("compiler.structure", CompileMath, "compiler structure diagnostic contract."),
    CompilerSyntax => ("compiler.syntax", CompileMath, "compiler syntax diagnostic contract."),
    DomainPotentialEvaluationError => ("domain.potential_evaluation_error", ValidationInvariant, "domain potential evaluation error diagnostic contract."),
    EquationCancelingTerms => ("equation.canceling_terms", ValidationInvariant, "equation canceling terms diagnostic contract."),
    EquationLargeResidual => ("equation.large_residual", ValidationInvariant, "equation large residual diagnostic contract."),
    EquationMismatchedTerm => ("equation.mismatched_term", ValidationInvariant, "equation mismatched term diagnostic contract."),
    EquationTermEvaluationFailed => ("equation.term_evaluation_failed", ValidationInvariant, "equation term evaluation failed diagnostic contract."),
    FitCandidateValidation => ("fit.candidate_validation", ValidationInvariant, "fit candidate validation diagnostic contract."),
    FitFinalEvaluation => ("fit.final_evaluation", SolveEvaluationError, "fit final evaluation diagnostic contract."),
    FitObjectiveOverflow => ("fit.objective_overflow", SolveEvaluationError, "fit objective overflow diagnostic contract."),
    FitResponseRank => ("fit.response_rank", ValidationInvariant, "fit response rank diagnostic contract."),
    JacobianAnalysisInconclusive => ("jacobian.analysis_inconclusive", ValidationInvariant, "jacobian analysis inconclusive diagnostic contract."),
    JacobianConditionEstimate => ("jacobian.condition_estimate", ValidationInvariant, "jacobian condition estimate diagnostic contract."),
    JacobianExtremeColumn => ("jacobian.extreme_column", ValidationInvariant, "jacobian extreme column diagnostic contract."),
    JacobianExtremeEntry => ("jacobian.extreme_entry", ValidationInvariant, "jacobian extreme entry diagnostic contract."),
    JacobianExtremeRow => ("jacobian.extreme_row", ValidationInvariant, "jacobian extreme row diagnostic contract."),
    JacobianNumericalRankDeficiency => ("jacobian.numerical_rank_deficiency", ValidationInvariant, "jacobian numerical rank deficiency diagnostic contract."),
    JacobianParallelColumns => ("jacobian.parallel_columns", ValidationInvariant, "jacobian parallel columns diagnostic contract."),
    JacobianParallelRows => ("jacobian.parallel_rows", ValidationInvariant, "jacobian parallel rows diagnostic contract."),
    MathApplicability => ("math.applicability", SolveEvaluationError, "math applicability diagnostic contract."),
    MathCancelled => ("math.cancelled", RuntimeCancelled, "math cancelled diagnostic contract."),
    MathCoefficientRange => ("math.coefficient_range", RuntimeInfrastructure, "math coefficient range diagnostic contract."),
    MathContract => ("math.contract", CompileMath, "math contract diagnostic contract."),
    MathDomain => ("math.domain", SolveEvaluationError, "math domain diagnostic contract."),
    MathEvaluation => ("math.evaluation", SolveEvaluationError, "math evaluation diagnostic contract."),
    MathLibrary => ("math.library", RuntimeInfrastructure, "math library diagnostic contract."),
    MathLimit => ("math.limit", RuntimeResourceLimit, "math limit diagnostic contract."),
    MathNative => ("math.native", ValidationInvariant, "math native diagnostic contract."),
    MathProvider => ("math.provider", SolveEvaluationError, "math provider diagnostic contract."),
    MathQuantity => ("math.quantity", CompileMath, "math quantity diagnostic contract."),
    MathRange => ("math.range", SolveEvaluationError, "math range diagnostic contract."),
    MathValidity => ("math.validity", SolveEvaluationError, "math validity diagnostic contract."),
    ModelingBudget => ("modeling.budget", RuntimeResourceLimit, "modeling budget diagnostic contract."),
    ModelingCancelled => ("modeling.cancelled", RuntimeCancelled, "modeling cancelled diagnostic contract."),
    ModelingCapability => ("modeling.capability", CapabilityBackend, "modeling capability diagnostic contract."),
    ModelingConditionalUnitAdmissionInvalidModel => ("modeling.conditional_unit.admission.invalid_model", ValidationInvariant, "modeling conditional unit admission invalid model diagnostic contract."),
    ModelingConditionalUnitAdmissionNumerical => ("modeling.conditional_unit.admission.numerical", SolveSolverError, "modeling conditional unit admission numerical diagnostic contract."),
    ModelingConditionalUnitAdmissionTrialRejected => ("modeling.conditional_unit.admission.trial_rejected", ValidationInvariant, "modeling conditional unit admission trial rejected diagnostic contract."),
    ModelingContract => ("modeling.contract", CompileMath, "modeling contract diagnostic contract."),
    ModelingDiagnosticSamples => ("modeling.diagnostic_samples", ValidationInvariant, "modeling diagnostic samples diagnostic contract."),
    ModelingDiagnostics => ("modeling.diagnostics", ValidationInvariant, "modeling diagnostics diagnostic contract."),
    ModelingDomain => ("modeling.domain", ValidationInvariant, "modeling domain diagnostic contract."),
    ModelingDomainTightened => ("modeling.domain.tightened", ValidationInvariant, "modeling domain tightened diagnostic contract."),
    ModelingDynamicAlgebraicResetUnsupported => ("modeling.dynamic.algebraic_reset.unsupported", ValidationInvariant, "modeling dynamic algebraic reset unsupported diagnostic contract."),
    ModelingDynamicInventoryTransferEventUnsupported => ("modeling.dynamic.inventory_transfer.event.unsupported", ValidationInvariant, "modeling dynamic inventory transfer event unsupported diagnostic contract."),
    ModelingInitializationAttemptLimit => ("modeling.initialization.attempt_limit", RuntimeResourceLimit, "modeling initialization attempt limit diagnostic contract."),
    ModelingInitializationIncomplete => ("modeling.initialization.incomplete", ValidationInvariant, "modeling initialization incomplete diagnostic contract."),
    ModelingInitializationMinimumStep => ("modeling.initialization.minimum_step", RuntimeResourceLimit, "modeling initialization minimum step diagnostic contract."),
    ModelingInitializationOutcome => ("modeling.initialization.outcome", ValidationInvariant, "modeling initialization outcome diagnostic contract."),
    ModelingInitializationStepPrecision => ("modeling.initialization.step_precision", RuntimeResourceLimit, "modeling initialization step precision diagnostic contract."),
    ModelingNonlinearAttemptLimit => ("modeling.nonlinear.attempt_limit", RuntimeResourceLimit, "modeling nonlinear attempt limit diagnostic contract."),
    ModelingNonlinearCancelled => ("modeling.nonlinear.cancelled", RuntimeCancelled, "modeling nonlinear cancelled diagnostic contract."),
    ModelingNonlinearInitialInconclusive => ("modeling.nonlinear.initial_inconclusive", ValidationInvariant, "modeling nonlinear initial inconclusive diagnostic contract."),
    ModelingNonlinearTimeLimit => ("modeling.nonlinear.time_limit", RuntimeResourceLimit, "modeling nonlinear time limit diagnostic contract."),
    ModelingObjective => ("modeling.objective", ValidationInvariant, "modeling objective diagnostic contract."),
    ModelingProvenance => ("modeling.provenance", ValidationInvariant, "modeling provenance diagnostic contract."),
    ModelingQualificationInfeasibilityContradicted => ("modeling.qualification.infeasibility_contradicted", ValidationInvariant, "modeling qualification infeasibility contradicted diagnostic contract."),
    ModelingQualificationRejected => ("modeling.qualification.rejected", ValidationInvariant, "modeling qualification rejected diagnostic contract."),
    ModelingRealization => ("modeling.realization", CapabilityBackend, "modeling realization diagnostic contract."),
    ModelingStagedStart => ("modeling.staged.start", ValidationInvariant, "modeling staged start diagnostic contract."),
    ModelingStudyInitialization => ("modeling.study.initialization", ValidationInvariant, "modeling study initialization diagnostic contract."),
    ModelingStudyPredecessor => ("modeling.study.predecessor", ValidationInvariant, "modeling study predecessor diagnostic contract."),
    ModelingStudyProcedure => ("modeling.study.procedure", ValidationInvariant, "modeling study procedure diagnostic contract."),
    ModelingTrajectoryIncomplete => ("modeling.trajectory.incomplete", ValidationInvariant, "modeling trajectory incomplete diagnostic contract."),
    ModelingTrajectoryRejected => ("modeling.trajectory.rejected", ValidationInvariant, "modeling trajectory rejected diagnostic contract."),
    NativeCancelled => ("native.cancelled", RuntimeCancelled, "native cancelled diagnostic contract."),
    NativeContract => ("native.contract", CompileMath, "native contract diagnostic contract."),
    NativeInternal => ("native.internal", InternalInvariant, "native internal diagnostic contract."),
    NativeLimit => ("native.limit", RuntimeResourceLimit, "native limit diagnostic contract."),
    NativeNumerical => ("native.numerical", SolveSolverError, "native numerical diagnostic contract."),
    NativeReuse => ("native.reuse", CapabilityBackend, "native reuse diagnostic contract."),
    NativeRouteRefused => ("native.route_refused", CapabilityBackend, "native route refused diagnostic contract."),
    NativeStructural => ("native.structural", ValidationInvariant, "native structural diagnostic contract."),
    NativeUnavailable => ("native.unavailable", CapabilityBackend, "native unavailable diagnostic contract."),
    NativeUnsupported => ("native.unsupported", CapabilityBackend, "native unsupported diagnostic contract."),
    ObjectivePriorityOptimizationIncomplete => ("objective.priority_optimization_incomplete", ValidationInvariant, "objective priority optimization incomplete diagnostic contract."),
    ShootingCandidateValidation => ("shooting.candidate_validation", ValidationInvariant, "shooting candidate validation diagnostic contract."),
    ShootingEvaluation => ("shooting.evaluation", SolveEvaluationError, "shooting evaluation diagnostic contract."),
    StructuralOverdetermined => ("structural.overdetermined", ValidationInvariant, "structural overdetermined diagnostic contract."),
    StructuralUnderdetermined => ("structural.underdetermined", ValidationInvariant, "structural underdetermined diagnostic contract."),
    VariableFixedZero => ("variable.fixed_zero", ValidationInvariant, "variable fixed zero diagnostic contract."),
    VariableLargeValue => ("variable.large_value", ValidationInvariant, "variable large value diagnostic contract."),
    VariableMissingValue => ("variable.missing_value", ValidationInvariant, "variable missing value diagnostic contract."),
    VariableNearLowerBound => ("variable.near_lower_bound", ValidationInvariant, "variable near lower bound diagnostic contract."),
    VariableNearUpperBound => ("variable.near_upper_bound", ValidationInvariant, "variable near upper bound diagnostic contract."),
    VariableNonfinite => ("variable.nonfinite", ValidationInvariant, "variable nonfinite diagnostic contract."),
    VariableOnlyInInequalities => ("variable.only_in_inequalities", ValidationInvariant, "variable only in inequalities diagnostic contract."),
    VariableOutsideLowerBound => ("variable.outside_lower_bound", ValidationInvariant, "variable outside lower bound diagnostic contract."),
    VariableOutsideUpperBound => ("variable.outside_upper_bound", ValidationInvariant, "variable outside upper bound diagnostic contract."),
    VariableSmallValue => ("variable.small_value", ValidationInvariant, "variable small value diagnostic contract."),
    VariableUnused => ("variable.unused", ValidationInvariant, "variable unused diagnostic contract."),
    WorkflowContract => ("workflow.contract", CompileMath, "workflow contract diagnostic contract."),
    WorkflowEphemeralPublication => ("workflow.ephemeral_publication", ValidationInvariant, "workflow ephemeral publication diagnostic contract."),
    WorkflowExportExpired => ("workflow.export_expired", ValidationInvariant, "workflow export expired diagnostic contract."),
    WorkflowJobPayloadVersion => ("workflow.job_payload_version", ValidationInvariant, "workflow job payload version diagnostic contract."),
    WorkflowLegacyWorkspace => ("workflow.legacy_workspace", ValidationInvariant, "workflow legacy workspace diagnostic contract."),
    WorkflowOperations => ("workflow.operations", RuntimeInfrastructure, "workflow operations diagnostic contract."),
    WorkflowPublicationUnresolved => ("workflow.publication_unresolved", RuntimeInfrastructure, "workflow publication unresolved diagnostic contract."),
    WorkflowUnclassified => ("workflow.unclassified", InternalInvariant, "workflow unclassified diagnostic contract."),
    QuantityUnknownId => ("quantity.unknown_id", CompileMath, "quantity unknown id diagnostic contract."),
    QuantityContractMismatch => ("quantity.contract_mismatch", CompileMath, "quantity contract mismatch diagnostic contract."),
    QuantityUnitConversion => ("quantity.unit_conversion", CompileMath, "quantity unit conversion diagnostic contract."),
    QuantityIncompatible => ("quantity.incompatible", CompileMath, "quantity incompatible diagnostic contract."),
    QuantityNonfinite => ("quantity.nonfinite", CompileMath, "quantity nonfinite diagnostic contract."),
    WorkflowInput => ("workflow.input", CompileMath, "workflow input diagnostic contract."),
    WorkflowInternal => ("workflow.internal", InternalInvariant, "workflow internal diagnostic contract."),
    WorkflowPanic => ("workflow.panic", InternalInvariant, "workflow panic diagnostic contract."),
    DiagnosticAggregate => ("diagnostic.aggregate", InternalInvariant, "diagnostic aggregate diagnostic contract."),
    StudyBindingTarget => ("study.binding.target", AuthoringReference, "study binding target refusal."),
    StudyBindingDuplicate => ("study.binding.duplicate", ValidationInvariant, "study binding duplicate refusal."),
    StudyBindingPhysical => ("study.binding.physical", CompileMath, "study binding physical refusal."),
    StudyBindingRevision => ("study.binding.revision", AuthoringReference, "study binding revision refusal."),
    StudyDependencyUnusable => ("study.dependency.unusable", ValidationInvariant, "study dependency unusable refusal."),
    StudySeedUnavailable => ("study.seed.unavailable", ValidationInvariant, "study seed unavailable refusal."),
    StudySeedIncompatible => ("study.seed.incompatible", ValidationInvariant, "study seed incompatible refusal."),
    StudySeedInternal => ("study.seed.internal", InternalInvariant, "study seed internal refusal."),
    StudyOperationUnsupported => ("study.operation.unsupported", CapabilityBackend, "study operation unsupported refusal."),
    ModelingConditionalUnitAdmissionUnsupported => ("modeling.conditional_unit.admission.unsupported", CapabilityBackend, "Conditional-unit admission unsupported disposition."),
    ModelingConditionalUnitAdmissionCancelled => ("modeling.conditional_unit.admission.cancelled", RuntimeCancelled, "Conditional-unit admission cancelled disposition."),
    ModelingConditionalUnitAdmissionResourceLimit => ("modeling.conditional_unit.admission.resource_limit", RuntimeResourceLimit, "Conditional-unit admission resource limit disposition."),
    ModelingConditionalUnitAdmissionNonfinite => ("modeling.conditional_unit.admission.nonfinite", SolveEvaluationError, "Conditional-unit admission nonfinite disposition."),
    ModelingConditionalUnitAdmissionInfrastructure => ("modeling.conditional_unit.admission.infrastructure", RuntimeInfrastructure, "Conditional-unit admission infrastructure disposition."),
    ModelingConditionalUnitAdmissionConflict => ("modeling.conditional_unit.admission.conflict", RuntimeInfrastructure, "Conditional-unit admission conflict disposition."),
    ModelingConditionalUnitAdmissionIncompatible => ("modeling.conditional_unit.admission.incompatible", RuntimeInfrastructure, "Conditional-unit admission incompatible disposition."),
    ModelingConditionalUnitAdmissionInternal => ("modeling.conditional_unit.admission.internal", InternalInvariant, "Conditional-unit admission internal disposition."),
    ModelingConditionalUnitAdmissionInconclusive => ("modeling.conditional_unit.admission.inconclusive", SolveSolverError, "Conditional-unit admission inconclusive disposition."),
    StudyPolicyAdmission => ("study.policy.admission", ValidationInvariant, "Invalid admitted study graph or occurrence fact."),
}

impl schemars::JsonSchema for DiagnosticCode {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "DiagnosticCode".into()
    }
    fn schema_id() -> std::borrow::Cow<'static, str> {
        "pse_diagnostics::DiagnosticCode".into()
    }
    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({"type":"string","enum":Self::ALL.iter().map(|v|v.as_str()).collect::<Vec<_>>()})
    }
}

impl From<DiagnosticCode> for String {
    fn from(value: DiagnosticCode) -> Self {
        value.as_str().to_owned()
    }
}
impl TryFrom<String> for DiagnosticCode {
    type Error = VocabularyError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl std::str::FromStr for DiagnosticCode {
    type Err = VocabularyError;
    /// Reads the dotted registry spelling, or the miette path spelling `Display` writes.
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value).ok_or_else(|| VocabularyError::UnknownMember {
            vocabulary: "DiagnosticCode",
            value: value.to_owned(),
        })
    }
}

impl std::fmt::Display for DiagnosticCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (index, part) in self.as_str().split('.').enumerate() {
            if index > 0 {
                f.write_str("::")?;
            }
            f.write_str(part)?;
        }
        Ok(())
    }
}

vocabulary! { DiagnosticRule as "diagnostic_rule" {
    AuthoringBudget => ("authoring.budget", "authoring budget diagnostic contract."),
    AuthoringContract => ("authoring.contract", "authoring contract diagnostic contract."),
    AuthoringDerivedWrite => ("authoring.derived_write", "authoring derived write diagnostic contract."),
    AuthoringDocumentIo => ("authoring.document_io", "authoring document io diagnostic contract."),
    AuthoringMissingId => ("authoring.missing_id", "authoring missing id diagnostic contract."),
    AuthoringPackageUnresolved => ("authoring.package_unresolved", "authoring package unresolved diagnostic contract."),
    AuthoringPackageVersionConflict => ("authoring.package_version_conflict", "authoring package version conflict diagnostic contract."),
    AuthoringRenameNamed => ("authoring.rename_named", "authoring rename named diagnostic contract."),
    AuthoringSchemaVersion => ("authoring.schema_version", "authoring schema version diagnostic contract."),
    AuthoringSyntax => ("authoring.syntax", "authoring syntax diagnostic contract."),
    AuthoringUnknownKey => ("authoring.unknown_key", "authoring unknown key diagnostic contract."),
    AuthoringUnknownRowKey => ("authoring.unknown_row_key", "authoring unknown row key diagnostic contract."),
    AuthoringUnresolvedTarget => ("authoring.unresolved_target", "authoring unresolved target diagnostic contract."),
    CandidateEvaluationFailed => ("candidate.evaluation_failed", "candidate evaluation failed diagnostic contract."),
    CompilerCancelled => ("compiler.cancelled", "compiler cancelled diagnostic contract."),
    CompilerLimit => ("compiler.limit", "compiler limit diagnostic contract."),
    CompilerMath => ("compiler.math", "compiler math diagnostic contract."),
    CompilerMissing => ("compiler.missing", "compiler missing diagnostic contract."),
    CompilerModeling => ("compiler.modeling", "compiler modeling diagnostic contract."),
    CompilerStructure => ("compiler.structure", "compiler structure diagnostic contract."),
    CompilerSyntax => ("compiler.syntax", "compiler syntax diagnostic contract."),
    DomainPotentialEvaluationError => ("domain.potential_evaluation_error", "domain potential evaluation error diagnostic contract."),
    EquationCancelingTerms => ("equation.canceling_terms", "equation canceling terms diagnostic contract."),
    EquationLargeResidual => ("equation.large_residual", "equation large residual diagnostic contract."),
    EquationMismatchedTerm => ("equation.mismatched_term", "equation mismatched term diagnostic contract."),
    EquationTermEvaluationFailed => ("equation.term_evaluation_failed", "equation term evaluation failed diagnostic contract."),
    FitCandidateValidation => ("fit.candidate_validation", "fit candidate validation diagnostic contract."),
    FitFinalEvaluation => ("fit.final_evaluation", "fit final evaluation diagnostic contract."),
    FitObjectiveOverflow => ("fit.objective_overflow", "fit objective overflow diagnostic contract."),
    FitResponseRank => ("fit.response_rank", "fit response rank diagnostic contract."),
    JacobianAnalysisInconclusive => ("jacobian.analysis_inconclusive", "jacobian analysis inconclusive diagnostic contract."),
    JacobianConditionEstimate => ("jacobian.condition_estimate", "jacobian condition estimate diagnostic contract."),
    JacobianExtremeColumn => ("jacobian.extreme_column", "jacobian extreme column diagnostic contract."),
    JacobianExtremeEntry => ("jacobian.extreme_entry", "jacobian extreme entry diagnostic contract."),
    JacobianExtremeRow => ("jacobian.extreme_row", "jacobian extreme row diagnostic contract."),
    JacobianNumericalRankDeficiency => ("jacobian.numerical_rank_deficiency", "jacobian numerical rank deficiency diagnostic contract."),
    JacobianParallelColumns => ("jacobian.parallel_columns", "jacobian parallel columns diagnostic contract."),
    JacobianParallelRows => ("jacobian.parallel_rows", "jacobian parallel rows diagnostic contract."),
    MathApplicability => ("math.applicability", "math applicability diagnostic contract."),
    MathCancelled => ("math.cancelled", "math cancelled diagnostic contract."),
    MathCoefficientRange => ("math.coefficient_range", "math coefficient range diagnostic contract."),
    MathContract => ("math.contract", "math contract diagnostic contract."),
    MathDomain => ("math.domain", "math domain diagnostic contract."),
    MathEvaluation => ("math.evaluation", "math evaluation diagnostic contract."),
    MathLibrary => ("math.library", "math library diagnostic contract."),
    MathLimit => ("math.limit", "math limit diagnostic contract."),
    MathNative => ("math.native", "math native diagnostic contract."),
    MathProvider => ("math.provider", "math provider diagnostic contract."),
    MathQuantity => ("math.quantity", "math quantity diagnostic contract."),
    MathRange => ("math.range", "math range diagnostic contract."),
    MathValidity => ("math.validity", "math validity diagnostic contract."),
    ModelingBudget => ("modeling.budget", "modeling budget diagnostic contract."),
    ModelingCancelled => ("modeling.cancelled", "modeling cancelled diagnostic contract."),
    ModelingCapability => ("modeling.capability", "modeling capability diagnostic contract."),
    ModelingConditionalUnitAdmissionInvalidModel => ("modeling.conditional_unit.admission.invalid_model", "modeling conditional unit admission invalid model diagnostic contract."),
    ModelingConditionalUnitAdmissionNumerical => ("modeling.conditional_unit.admission.numerical", "modeling conditional unit admission numerical diagnostic contract."),
    ModelingConditionalUnitAdmissionTrialRejected => ("modeling.conditional_unit.admission.trial_rejected", "modeling conditional unit admission trial rejected diagnostic contract."),
    ModelingContract => ("modeling.contract", "modeling contract diagnostic contract."),
    ModelingDiagnosticSamples => ("modeling.diagnostic_samples", "modeling diagnostic samples diagnostic contract."),
    ModelingDiagnostics => ("modeling.diagnostics", "modeling diagnostics diagnostic contract."),
    ModelingDomain => ("modeling.domain", "modeling domain diagnostic contract."),
    ModelingDomainTightened => ("modeling.domain.tightened", "modeling domain tightened diagnostic contract."),
    ModelingDynamicAlgebraicResetUnsupported => ("modeling.dynamic.algebraic_reset.unsupported", "modeling dynamic algebraic reset unsupported diagnostic contract."),
    ModelingDynamicInventoryTransferEventUnsupported => ("modeling.dynamic.inventory_transfer.event.unsupported", "modeling dynamic inventory transfer event unsupported diagnostic contract."),
    ModelingInitializationAttemptLimit => ("modeling.initialization.attempt_limit", "modeling initialization attempt limit diagnostic contract."),
    ModelingInitializationIncomplete => ("modeling.initialization.incomplete", "modeling initialization incomplete diagnostic contract."),
    ModelingInitializationMinimumStep => ("modeling.initialization.minimum_step", "modeling initialization minimum step diagnostic contract."),
    ModelingInitializationOutcome => ("modeling.initialization.outcome", "modeling initialization outcome diagnostic contract."),
    ModelingInitializationStepPrecision => ("modeling.initialization.step_precision", "modeling initialization step precision diagnostic contract."),
    ModelingNonlinearAttemptLimit => ("modeling.nonlinear.attempt_limit", "modeling nonlinear attempt limit diagnostic contract."),
    ModelingNonlinearCancelled => ("modeling.nonlinear.cancelled", "modeling nonlinear cancelled diagnostic contract."),
    ModelingNonlinearInitialInconclusive => ("modeling.nonlinear.initial_inconclusive", "modeling nonlinear initial inconclusive diagnostic contract."),
    ModelingNonlinearTimeLimit => ("modeling.nonlinear.time_limit", "modeling nonlinear time limit diagnostic contract."),
    ModelingObjective => ("modeling.objective", "modeling objective diagnostic contract."),
    ModelingProvenance => ("modeling.provenance", "modeling provenance diagnostic contract."),
    ModelingQualificationInfeasibilityContradicted => ("modeling.qualification.infeasibility_contradicted", "modeling qualification infeasibility contradicted diagnostic contract."),
    ModelingQualificationRejected => ("modeling.qualification.rejected", "modeling qualification rejected diagnostic contract."),
    ModelingRealization => ("modeling.realization", "modeling realization diagnostic contract."),
    ModelingStagedStart => ("modeling.staged.start", "modeling staged start diagnostic contract."),
    ModelingStudyInitialization => ("modeling.study.initialization", "modeling study initialization diagnostic contract."),
    ModelingStudyPredecessor => ("modeling.study.predecessor", "modeling study predecessor diagnostic contract."),
    ModelingStudyProcedure => ("modeling.study.procedure", "modeling study procedure diagnostic contract."),
    ModelingTrajectoryIncomplete => ("modeling.trajectory.incomplete", "modeling trajectory incomplete diagnostic contract."),
    ModelingTrajectoryRejected => ("modeling.trajectory.rejected", "modeling trajectory rejected diagnostic contract."),
    NativeCancelled => ("native.cancelled", "native cancelled diagnostic contract."),
    NativeContract => ("native.contract", "native contract diagnostic contract."),
    NativeInternal => ("native.internal", "native internal diagnostic contract."),
    NativeLimit => ("native.limit", "native limit diagnostic contract."),
    NativeNumerical => ("native.numerical", "native numerical diagnostic contract."),
    NativeReuse => ("native.reuse", "native reuse diagnostic contract."),
    NativeRouteRefused => ("native.route_refused", "native route refused diagnostic contract."),
    NativeStructural => ("native.structural", "native structural diagnostic contract."),
    NativeUnavailable => ("native.unavailable", "native unavailable diagnostic contract."),
    NativeUnsupported => ("native.unsupported", "native unsupported diagnostic contract."),
    ObjectivePriorityOptimizationIncomplete => ("objective.priority_optimization_incomplete", "objective priority optimization incomplete diagnostic contract."),
    ShootingCandidateValidation => ("shooting.candidate_validation", "shooting candidate validation diagnostic contract."),
    ShootingEvaluation => ("shooting.evaluation", "shooting evaluation diagnostic contract."),
    StructuralOverdetermined => ("structural.overdetermined", "structural overdetermined diagnostic contract."),
    StructuralUnderdetermined => ("structural.underdetermined", "structural underdetermined diagnostic contract."),
    VariableFixedZero => ("variable.fixed_zero", "variable fixed zero diagnostic contract."),
    VariableLargeValue => ("variable.large_value", "variable large value diagnostic contract."),
    VariableMissingValue => ("variable.missing_value", "variable missing value diagnostic contract."),
    VariableNearLowerBound => ("variable.near_lower_bound", "variable near lower bound diagnostic contract."),
    VariableNearUpperBound => ("variable.near_upper_bound", "variable near upper bound diagnostic contract."),
    VariableNonfinite => ("variable.nonfinite", "variable nonfinite diagnostic contract."),
    VariableOnlyInInequalities => ("variable.only_in_inequalities", "variable only in inequalities diagnostic contract."),
    VariableOutsideLowerBound => ("variable.outside_lower_bound", "variable outside lower bound diagnostic contract."),
    VariableOutsideUpperBound => ("variable.outside_upper_bound", "variable outside upper bound diagnostic contract."),
    VariableSmallValue => ("variable.small_value", "variable small value diagnostic contract."),
    VariableUnused => ("variable.unused", "variable unused diagnostic contract."),
    WorkflowContract => ("workflow.contract", "workflow contract diagnostic contract."),
    WorkflowEphemeralPublication => ("workflow.ephemeral_publication", "workflow ephemeral publication diagnostic contract."),
    WorkflowExportExpired => ("workflow.export_expired", "workflow export expired diagnostic contract."),
    WorkflowJobPayloadVersion => ("workflow.job_payload_version", "workflow job payload version diagnostic contract."),
    WorkflowLegacyWorkspace => ("workflow.legacy_workspace", "workflow legacy workspace diagnostic contract."),
    WorkflowOperations => ("workflow.operations", "workflow operations diagnostic contract."),
    WorkflowPublicationUnresolved => ("workflow.publication_unresolved", "workflow publication unresolved diagnostic contract."),
    WorkflowUnclassified => ("workflow.unclassified", "workflow unclassified diagnostic contract."),
    QuantityUnknownId => ("quantity.unknown_id", "quantity unknown id diagnostic contract."),
    QuantityContractMismatch => ("quantity.contract_mismatch", "quantity contract mismatch diagnostic contract."),
    QuantityUnitConversion => ("quantity.unit_conversion", "quantity unit conversion diagnostic contract."),
    QuantityIncompatible => ("quantity.incompatible", "quantity incompatible diagnostic contract."),
    QuantityNonfinite => ("quantity.nonfinite", "quantity nonfinite diagnostic contract."),
    WorkflowInput => ("workflow.input", "workflow input diagnostic contract."),
    WorkflowInternal => ("workflow.internal", "workflow internal diagnostic contract."),
    WorkflowPanic => ("workflow.panic", "workflow panic diagnostic contract."),
    DiagnosticAggregate => ("diagnostic.aggregate", "diagnostic aggregate diagnostic contract."),
    StudyBindingTarget => ("study.binding.target", "study binding target refusal."),
    StudyBindingDuplicate => ("study.binding.duplicate", "study binding duplicate refusal."),
    StudyBindingPhysical => ("study.binding.physical", "study binding physical refusal."),
    StudyBindingRevision => ("study.binding.revision", "study binding revision refusal."),
    StudyDependencyUnusable => ("study.dependency.unusable", "study dependency unusable refusal."),
    StudySeedUnavailable => ("study.seed.unavailable", "study seed unavailable refusal."),
    StudySeedIncompatible => ("study.seed.incompatible", "study seed incompatible refusal."),
    StudySeedInternal => ("study.seed.internal", "study seed internal refusal."),
    StudyOperationUnsupported => ("study.operation.unsupported", "study operation unsupported refusal."),
    ModelingConditionalUnitAdmissionUnsupported => ("modeling.conditional_unit.admission.unsupported", "Conditional-unit admission unsupported disposition."),
    ModelingConditionalUnitAdmissionCancelled => ("modeling.conditional_unit.admission.cancelled", "Conditional-unit admission cancelled disposition."),
    ModelingConditionalUnitAdmissionResourceLimit => ("modeling.conditional_unit.admission.resource_limit", "Conditional-unit admission resource limit disposition."),
    ModelingConditionalUnitAdmissionNonfinite => ("modeling.conditional_unit.admission.nonfinite", "Conditional-unit admission nonfinite disposition."),
    ModelingConditionalUnitAdmissionInfrastructure => ("modeling.conditional_unit.admission.infrastructure", "Conditional-unit admission infrastructure disposition."),
    ModelingConditionalUnitAdmissionConflict => ("modeling.conditional_unit.admission.conflict", "Conditional-unit admission conflict disposition."),
    ModelingConditionalUnitAdmissionIncompatible => ("modeling.conditional_unit.admission.incompatible", "Conditional-unit admission incompatible disposition."),
    ModelingConditionalUnitAdmissionInternal => ("modeling.conditional_unit.admission.internal", "Conditional-unit admission internal disposition."),
    ModelingConditionalUnitAdmissionInconclusive => ("modeling.conditional_unit.admission.inconclusive", "Conditional-unit admission inconclusive disposition."),
    StudyPolicyAdmission => ("study.policy.admission", "Invalid admitted study graph or occurrence fact."),
} }

vocabulary! { DiagnosticObservationKind as "diagnostic_observation_kind" {
    Missing => ("missing", "Evidence was unavailable."),
    Real => ("real", "Finite numerical evidence."),
    Nonfinite => ("nonfinite", "Explicitly tagged nonfinite numerical failure evidence."),
    Physical => ("physical", "Finite physical evidence with immutable quantity/unit/context contracts."),
    Integer => ("integer", "An exact count or status."),
    Boolean => ("boolean", "A checked predicate."),
    Text => ("text", "Native status or explanatory detail."),
    Contracts => ("contracts", "Ordered complete quantity/free-index operand contracts."),
} }

vocabulary! { DiagnosticStage as "diagnostic_stage" {
    Workflow => ("workflow", "workflow diagnostic contract."),
    Modeling => ("modeling", "modeling diagnostic contract."),
    Routing => ("routing", "routing diagnostic contract."),
    Native => ("native", "native diagnostic contract."),
    Implicit => ("implicit", "implicit diagnostic contract."),
    Presolve => ("presolve", "presolve diagnostic contract."),
    Evaluation => ("evaluation", "evaluation diagnostic contract."),
    Property => ("property", "property diagnostic contract."),
    Fit => ("fit", "fit diagnostic contract."),
    Shooting => ("shooting", "shooting diagnostic contract."),
    Simulation => ("simulation", "simulation diagnostic contract."),
    Initialization => ("initialization", "initialization diagnostic contract."),
    ModelingAdmission => ("modeling.admission", "modeling admission diagnostic contract."),
    ModelingConditionalUnitAdmission => ("modeling.conditional_unit.admission", "modeling conditional unit admission diagnostic contract."),
    ModelingDiagnostics => ("modeling.diagnostics", "modeling diagnostics diagnostic contract."),
    ModelingDiagnosticSamples => ("modeling.diagnostic_samples", "modeling diagnostic samples diagnostic contract."),
    ModelingNonlinear => ("modeling.nonlinear", "modeling nonlinear diagnostic contract."),
    ModelingQualification => ("modeling.qualification", "modeling qualification diagnostic contract."),
    ModelingStaged => ("modeling.staged", "modeling staged diagnostic contract."),
    ModelingStudy => ("modeling.study", "modeling study diagnostic contract."),
    ModelingTrajectory => ("modeling.trajectory", "modeling trajectory diagnostic contract."),
    ModelingDynamic => ("modeling.dynamic", "modeling dynamic diagnostic contract."),
    DynamicTest => ("dynamic-test", "dynamic-test diagnostic contract."),
    Fitting => ("fitting", "fitting diagnostic contract."),
    Horizon => ("horizon", "horizon diagnostic contract."),
    Compiler => ("compiler", "compiler diagnostic contract."),
    Authoring => ("authoring", "authoring diagnostic contract."),
    Quantity => ("quantity", "quantity diagnostic contract."),
    Diagnostic => ("diagnostic", "diagnostic diagnostic contract."),
    Binding => ("binding", "binding diagnostic contract."),
    Study => ("study", "study diagnostic contract."),
    Continuation => ("continuation", "continuation diagnostic contract."),
    StudyAdmission => ("study.admission", "study admission boundary."),
    StudyPolicy => ("study.policy", "study policy boundary."),
    StudyBinding => ("study.binding", "study binding boundary."),
    Applicability => ("applicability", "applicability boundary."),
    ModelingConservationTransfer => ("modeling-conservation-transfer", "modeling conservation transfer boundary."),
    ModelingEventReset => ("modeling-event-reset", "modeling event reset boundary."),
    NonlinearExplanation => ("nonlinear-explanation", "nonlinear explanation boundary."),
    ObjectiveLevels => ("objective-levels", "objective levels boundary."),
    ShootingTrajectoryProjection => ("shooting-trajectory-projection", "shooting trajectory projection boundary."),
    Test => ("test", "test boundary."),
} }

impl DiagnosticRule {
    /// Detailed code declared by the checking rule; never reconstructed from a coarse class.
    pub const fn code(self) -> DiagnosticCode {
        match self {
            Self::AuthoringBudget => DiagnosticCode::AuthoringBudget,
            Self::AuthoringContract => DiagnosticCode::AuthoringContract,
            Self::AuthoringDerivedWrite => DiagnosticCode::AuthoringDerivedWrite,
            Self::AuthoringDocumentIo => DiagnosticCode::AuthoringDocumentIo,
            Self::AuthoringMissingId => DiagnosticCode::AuthoringMissingId,
            Self::AuthoringPackageUnresolved => DiagnosticCode::AuthoringPackageUnresolved,
            Self::AuthoringPackageVersionConflict => {
                DiagnosticCode::AuthoringPackageVersionConflict
            }
            Self::AuthoringRenameNamed => DiagnosticCode::AuthoringRenameNamed,
            Self::AuthoringSchemaVersion => DiagnosticCode::AuthoringSchemaVersion,
            Self::AuthoringSyntax => DiagnosticCode::AuthoringSyntax,
            Self::AuthoringUnknownKey => DiagnosticCode::AuthoringUnknownKey,
            Self::AuthoringUnknownRowKey => DiagnosticCode::AuthoringUnknownRowKey,
            Self::AuthoringUnresolvedTarget => DiagnosticCode::AuthoringUnresolvedTarget,
            Self::CandidateEvaluationFailed => DiagnosticCode::CandidateEvaluationFailed,
            Self::CompilerCancelled => DiagnosticCode::CompilerCancelled,
            Self::CompilerLimit => DiagnosticCode::CompilerLimit,
            Self::CompilerMath => DiagnosticCode::CompilerMath,
            Self::CompilerMissing => DiagnosticCode::CompilerMissing,
            Self::CompilerModeling => DiagnosticCode::CompilerModeling,
            Self::CompilerStructure => DiagnosticCode::CompilerStructure,
            Self::CompilerSyntax => DiagnosticCode::CompilerSyntax,
            Self::DomainPotentialEvaluationError => DiagnosticCode::DomainPotentialEvaluationError,
            Self::EquationCancelingTerms => DiagnosticCode::EquationCancelingTerms,
            Self::EquationLargeResidual => DiagnosticCode::EquationLargeResidual,
            Self::EquationMismatchedTerm => DiagnosticCode::EquationMismatchedTerm,
            Self::EquationTermEvaluationFailed => DiagnosticCode::EquationTermEvaluationFailed,
            Self::FitCandidateValidation => DiagnosticCode::FitCandidateValidation,
            Self::FitFinalEvaluation => DiagnosticCode::FitFinalEvaluation,
            Self::FitObjectiveOverflow => DiagnosticCode::FitObjectiveOverflow,
            Self::FitResponseRank => DiagnosticCode::FitResponseRank,
            Self::JacobianAnalysisInconclusive => DiagnosticCode::JacobianAnalysisInconclusive,
            Self::JacobianConditionEstimate => DiagnosticCode::JacobianConditionEstimate,
            Self::JacobianExtremeColumn => DiagnosticCode::JacobianExtremeColumn,
            Self::JacobianExtremeEntry => DiagnosticCode::JacobianExtremeEntry,
            Self::JacobianExtremeRow => DiagnosticCode::JacobianExtremeRow,
            Self::JacobianNumericalRankDeficiency => {
                DiagnosticCode::JacobianNumericalRankDeficiency
            }
            Self::JacobianParallelColumns => DiagnosticCode::JacobianParallelColumns,
            Self::JacobianParallelRows => DiagnosticCode::JacobianParallelRows,
            Self::MathApplicability => DiagnosticCode::MathApplicability,
            Self::MathCancelled => DiagnosticCode::MathCancelled,
            Self::MathCoefficientRange => DiagnosticCode::MathCoefficientRange,
            Self::MathContract => DiagnosticCode::MathContract,
            Self::MathDomain => DiagnosticCode::MathDomain,
            Self::MathEvaluation => DiagnosticCode::MathEvaluation,
            Self::MathLibrary => DiagnosticCode::MathLibrary,
            Self::MathLimit => DiagnosticCode::MathLimit,
            Self::MathNative => DiagnosticCode::MathNative,
            Self::MathProvider => DiagnosticCode::MathProvider,
            Self::MathQuantity => DiagnosticCode::MathQuantity,
            Self::MathRange => DiagnosticCode::MathRange,
            Self::MathValidity => DiagnosticCode::MathValidity,
            Self::ModelingBudget => DiagnosticCode::ModelingBudget,
            Self::ModelingCancelled => DiagnosticCode::ModelingCancelled,
            Self::ModelingCapability => DiagnosticCode::ModelingCapability,
            Self::ModelingConditionalUnitAdmissionInvalidModel => {
                DiagnosticCode::ModelingConditionalUnitAdmissionInvalidModel
            }
            Self::ModelingConditionalUnitAdmissionNumerical => {
                DiagnosticCode::ModelingConditionalUnitAdmissionNumerical
            }
            Self::ModelingConditionalUnitAdmissionTrialRejected => {
                DiagnosticCode::ModelingConditionalUnitAdmissionTrialRejected
            }
            Self::ModelingContract => DiagnosticCode::ModelingContract,
            Self::ModelingDiagnosticSamples => DiagnosticCode::ModelingDiagnosticSamples,
            Self::ModelingDiagnostics => DiagnosticCode::ModelingDiagnostics,
            Self::ModelingDomain => DiagnosticCode::ModelingDomain,
            Self::ModelingDomainTightened => DiagnosticCode::ModelingDomainTightened,
            Self::ModelingDynamicAlgebraicResetUnsupported => {
                DiagnosticCode::ModelingDynamicAlgebraicResetUnsupported
            }
            Self::ModelingDynamicInventoryTransferEventUnsupported => {
                DiagnosticCode::ModelingDynamicInventoryTransferEventUnsupported
            }
            Self::ModelingInitializationAttemptLimit => {
                DiagnosticCode::ModelingInitializationAttemptLimit
            }
            Self::ModelingInitializationIncomplete => {
                DiagnosticCode::ModelingInitializationIncomplete
            }
            Self::ModelingInitializationMinimumStep => {
                DiagnosticCode::ModelingInitializationMinimumStep
            }
            Self::ModelingInitializationOutcome => DiagnosticCode::ModelingInitializationOutcome,
            Self::ModelingInitializationStepPrecision => {
                DiagnosticCode::ModelingInitializationStepPrecision
            }
            Self::ModelingNonlinearAttemptLimit => DiagnosticCode::ModelingNonlinearAttemptLimit,
            Self::ModelingNonlinearCancelled => DiagnosticCode::ModelingNonlinearCancelled,
            Self::ModelingNonlinearInitialInconclusive => {
                DiagnosticCode::ModelingNonlinearInitialInconclusive
            }
            Self::ModelingNonlinearTimeLimit => DiagnosticCode::ModelingNonlinearTimeLimit,
            Self::ModelingObjective => DiagnosticCode::ModelingObjective,
            Self::ModelingProvenance => DiagnosticCode::ModelingProvenance,
            Self::ModelingQualificationInfeasibilityContradicted => {
                DiagnosticCode::ModelingQualificationInfeasibilityContradicted
            }
            Self::ModelingQualificationRejected => DiagnosticCode::ModelingQualificationRejected,
            Self::ModelingRealization => DiagnosticCode::ModelingRealization,
            Self::ModelingStagedStart => DiagnosticCode::ModelingStagedStart,
            Self::ModelingStudyInitialization => DiagnosticCode::ModelingStudyInitialization,
            Self::ModelingStudyPredecessor => DiagnosticCode::ModelingStudyPredecessor,
            Self::ModelingStudyProcedure => DiagnosticCode::ModelingStudyProcedure,
            Self::ModelingTrajectoryIncomplete => DiagnosticCode::ModelingTrajectoryIncomplete,
            Self::ModelingTrajectoryRejected => DiagnosticCode::ModelingTrajectoryRejected,
            Self::NativeCancelled => DiagnosticCode::NativeCancelled,
            Self::NativeContract => DiagnosticCode::NativeContract,
            Self::NativeInternal => DiagnosticCode::NativeInternal,
            Self::NativeLimit => DiagnosticCode::NativeLimit,
            Self::NativeNumerical => DiagnosticCode::NativeNumerical,
            Self::NativeReuse => DiagnosticCode::NativeReuse,
            Self::NativeRouteRefused => DiagnosticCode::NativeRouteRefused,
            Self::NativeStructural => DiagnosticCode::NativeStructural,
            Self::NativeUnavailable => DiagnosticCode::NativeUnavailable,
            Self::NativeUnsupported => DiagnosticCode::NativeUnsupported,
            Self::ObjectivePriorityOptimizationIncomplete => {
                DiagnosticCode::ObjectivePriorityOptimizationIncomplete
            }
            Self::ShootingCandidateValidation => DiagnosticCode::ShootingCandidateValidation,
            Self::ShootingEvaluation => DiagnosticCode::ShootingEvaluation,
            Self::StructuralOverdetermined => DiagnosticCode::StructuralOverdetermined,
            Self::StructuralUnderdetermined => DiagnosticCode::StructuralUnderdetermined,
            Self::VariableFixedZero => DiagnosticCode::VariableFixedZero,
            Self::VariableLargeValue => DiagnosticCode::VariableLargeValue,
            Self::VariableMissingValue => DiagnosticCode::VariableMissingValue,
            Self::VariableNearLowerBound => DiagnosticCode::VariableNearLowerBound,
            Self::VariableNearUpperBound => DiagnosticCode::VariableNearUpperBound,
            Self::VariableNonfinite => DiagnosticCode::VariableNonfinite,
            Self::VariableOnlyInInequalities => DiagnosticCode::VariableOnlyInInequalities,
            Self::VariableOutsideLowerBound => DiagnosticCode::VariableOutsideLowerBound,
            Self::VariableOutsideUpperBound => DiagnosticCode::VariableOutsideUpperBound,
            Self::VariableSmallValue => DiagnosticCode::VariableSmallValue,
            Self::VariableUnused => DiagnosticCode::VariableUnused,
            Self::WorkflowContract => DiagnosticCode::WorkflowContract,
            Self::WorkflowEphemeralPublication => DiagnosticCode::WorkflowEphemeralPublication,
            Self::WorkflowExportExpired => DiagnosticCode::WorkflowExportExpired,
            Self::WorkflowJobPayloadVersion => DiagnosticCode::WorkflowJobPayloadVersion,
            Self::WorkflowLegacyWorkspace => DiagnosticCode::WorkflowLegacyWorkspace,
            Self::WorkflowOperations => DiagnosticCode::WorkflowOperations,
            Self::WorkflowPublicationUnresolved => DiagnosticCode::WorkflowPublicationUnresolved,
            Self::WorkflowUnclassified => DiagnosticCode::WorkflowUnclassified,
            Self::QuantityUnknownId => DiagnosticCode::QuantityUnknownId,
            Self::QuantityContractMismatch => DiagnosticCode::QuantityContractMismatch,
            Self::QuantityUnitConversion => DiagnosticCode::QuantityUnitConversion,
            Self::QuantityIncompatible => DiagnosticCode::QuantityIncompatible,
            Self::QuantityNonfinite => DiagnosticCode::QuantityNonfinite,
            Self::WorkflowInput => DiagnosticCode::WorkflowInput,
            Self::WorkflowInternal => DiagnosticCode::WorkflowInternal,
            Self::WorkflowPanic => DiagnosticCode::WorkflowPanic,
            Self::DiagnosticAggregate => DiagnosticCode::DiagnosticAggregate,
            Self::StudyBindingTarget => DiagnosticCode::StudyBindingTarget,
            Self::StudyBindingDuplicate => DiagnosticCode::StudyBindingDuplicate,
            Self::StudyBindingPhysical => DiagnosticCode::StudyBindingPhysical,
            Self::StudyBindingRevision => DiagnosticCode::StudyBindingRevision,
            Self::StudyDependencyUnusable => DiagnosticCode::StudyDependencyUnusable,
            Self::StudySeedUnavailable => DiagnosticCode::StudySeedUnavailable,
            Self::StudySeedIncompatible => DiagnosticCode::StudySeedIncompatible,
            Self::StudySeedInternal => DiagnosticCode::StudySeedInternal,
            Self::StudyOperationUnsupported => DiagnosticCode::StudyOperationUnsupported,
            Self::ModelingConditionalUnitAdmissionUnsupported => {
                DiagnosticCode::ModelingConditionalUnitAdmissionUnsupported
            }
            Self::ModelingConditionalUnitAdmissionCancelled => {
                DiagnosticCode::ModelingConditionalUnitAdmissionCancelled
            }
            Self::ModelingConditionalUnitAdmissionResourceLimit => {
                DiagnosticCode::ModelingConditionalUnitAdmissionResourceLimit
            }
            Self::ModelingConditionalUnitAdmissionNonfinite => {
                DiagnosticCode::ModelingConditionalUnitAdmissionNonfinite
            }
            Self::ModelingConditionalUnitAdmissionInfrastructure => {
                DiagnosticCode::ModelingConditionalUnitAdmissionInfrastructure
            }
            Self::ModelingConditionalUnitAdmissionConflict => {
                DiagnosticCode::ModelingConditionalUnitAdmissionConflict
            }
            Self::ModelingConditionalUnitAdmissionIncompatible => {
                DiagnosticCode::ModelingConditionalUnitAdmissionIncompatible
            }
            Self::ModelingConditionalUnitAdmissionInternal => {
                DiagnosticCode::ModelingConditionalUnitAdmissionInternal
            }
            Self::ModelingConditionalUnitAdmissionInconclusive => {
                DiagnosticCode::ModelingConditionalUnitAdmissionInconclusive
            }
            Self::StudyPolicyAdmission => DiagnosticCode::StudyPolicyAdmission,
        }
    }
}
