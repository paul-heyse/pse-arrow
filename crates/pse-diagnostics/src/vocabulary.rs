// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! One declaration of platform classes and detailed diagnostic codes.

macro_rules! vocabulary {
    ($name:ident { $($variant:ident => ($text:literal, $description:literal)),* $(,)? }) => {
        #[doc = "Stable platform diagnostic vocabulary."]
        #[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
        pub enum $name { $(#[doc = $description] $variant),* }
        impl $name {
            /// All declarations in stable order.
            pub const ALL: &'static [Self] = &[$(Self::$variant),*];
            /// Registry spelling.
            pub const fn as_str(self) -> &'static str { match self { $(Self::$variant => $text),* } }
            /// Declared description.
            pub const fn description(self) -> &'static str { match self { $(Self::$variant => $description),* } }
        }
    };
}

vocabulary! { FailureClass {
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
        #[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
        pub enum DiagnosticCode { $(#[doc = $description] $variant),* }
        impl DiagnosticCode {
            /// All detailed codes.
            pub const ALL: &'static [Self] = &[$(Self::$variant),*];
            /// Stable dotted registry spelling.
            pub const fn as_str(self) -> &'static str { match self { $(Self::$variant => $text),* } }
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
