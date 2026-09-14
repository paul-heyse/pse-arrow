// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! One typed mapping from actual error leaves to declared terminal failure classes.
use crate::{CompilerError, passes::dag::invalid};
use miette::Diagnostic;
use pse_catalog::CatalogError;
use pse_relations::RecordBatch;
use pse_rules::RuleError;
use pse_schema::Registry;

type Visitor<'a, 'v> = dyn FnMut(&'static str, &'a dyn Diagnostic, Option<&'a [RecordBatch]>) -> Result<(), CompilerError>
    + 'v;

pub(super) fn visit<'a>(
    error: &'a CompilerError,
    reg: &Registry,
    visitor: &mut Visitor<'a, '_>,
) -> Result<(), CompilerError> {
    match error {
        CompilerError::Cancelled { findings } => {
            visitor("runtime.cancelled", error, Some(findings))
        }
        CompilerError::PassFailure { source, findings } => {
            let mut primary = None;
            visit(source, reg, &mut |class, diagnostic, batches| {
                if primary.is_none() {
                    primary = Some(class);
                }
                visitor(class, diagnostic, batches)
            })?;
            if !findings.is_empty() {
                visitor(
                    primary.ok_or_else(|| invalid("pass failure has no diagnostic class"))?,
                    error,
                    Some(findings),
                )?;
            }
            Ok(())
        }
        CompilerError::TerminalRecording { errors, .. } => {
            for error in errors {
                visit(error, reg, visitor)?;
            }
            Ok(())
        }
        CompilerError::AttemptFailed { source, .. }
        | CompilerError::SuccessRecording { source, .. }
        | CompilerError::CommitPublication { source, .. }
        | CompilerError::AuxiliaryHint { source, .. } => visit(source, reg, visitor),
        CompilerError::Authoring(error) => authoring(error, visitor),
        CompilerError::Rule(error) => rule(error, reg, visitor),
        CompilerError::Catalog(error) => catalog(error, reg, visitor),
        CompilerError::Canon(error) => visitor(canonical(error), error, None),
        CompilerError::Relation(error) => relation(error, visitor),
        CompilerError::Quantity(error) => visitor(quantity(error), error, None),
        CompilerError::MathIr(error) => visitor(math(error), error, None),
        CompilerError::Schema(_) => visitor("validation.invariant", error, None),
        CompilerError::ResourceLimit { .. } => visitor("runtime.resource_limit", error, None),
        CompilerError::Infrastructure { .. } => visitor("runtime.infrastructure", error, None),
        CompilerError::Postcondition { findings, .. } => {
            visitor("internal.invariant", error, Some(findings))
        }
        CompilerError::StageGraph { .. } | CompilerError::Internal { .. } => {
            visitor("internal.invariant", error, None)
        }
    }
}
fn authoring<'a>(
    error: &'a pse_authoring::AuthoringError,
    visitor: &mut Visitor<'a, '_>,
) -> Result<(), CompilerError> {
    use pse_authoring::AuthoringError as E;
    let class = match error {
        E::Resource(_) => "runtime.resource_limit",
        E::Allocation(error) => return visitor(canonical(error), error, None),
        E::Relation(error) => return relation(error, visitor),
        E::DocumentIo { .. }
        | E::Syntax { .. }
        | E::UnknownKey { .. }
        | E::MissingId { .. }
        | E::UnresolvedTarget { .. }
        | E::Budget { .. } => "authoring.parse",
        E::Contract { .. }
        | E::DerivedWrite { .. }
        | E::RenameNamed { .. }
        | E::UnknownRowKey { .. }
        | E::PackageUnresolved { .. }
        | E::PackageVersionConflict { .. } => "authoring.reference",
        E::SchemaVersionMismatch { .. } => "validation.invariant",
    };
    visitor(class, error, None)
}
fn canonical(error: &pse_ids::CanonError) -> &'static str {
    use pse_ids::CanonError as E;
    match error {
        E::Cancelled => "runtime.cancelled",
        E::Envelope { .. } | E::Reservation(_) => "runtime.resource_limit",
        E::Arrow(_) => "runtime.infrastructure",
        E::UnsupportedLayout { .. }
        | E::ContractMismatch { .. }
        | E::UnregisteredMetadata { .. }
        | E::InvalidKey { .. }
        | E::NullKey { .. }
        | E::DuplicateKey { .. }
        | E::DictionaryOutOfBounds { .. }
        | E::EnumMember { .. } => "validation.invariant",
        _ => "internal.invariant",
    }
}
fn quantity(error: &pse_quantity::QuantityError) -> &'static str {
    use pse_quantity::QuantityError as E;
    match error {
        E::Registry { .. } | E::Dimension(_) | E::UnknownId { .. } => "validation.invariant",
        E::InferencePrecondition { .. }
        | E::OperationUnsupported { .. }
        | E::UnregisteredResultType { .. }
        | E::Incompatible { .. }
        | E::AmbiguousLiteral { .. }
        | E::UnitConvertMismatch { .. }
        | E::StaticDomain { .. }
        | E::ContractMismatch { .. } => "compile.math",
        _ => "internal.invariant", // A future unclassified typed failure is an implementation defect.
    }
}
fn math(error: &pse_mathir::MathIrError) -> &'static str {
    use pse_mathir::MathIrError as E;
    match error {
        E::Cycle { .. }
        | E::Quantity { .. }
        | E::StaticDomain { .. }
        | E::NonFiniteLiteral { .. }
        | E::NonCanonicalUnitEscapes { .. }
        | E::GuardNotBoolean { .. }
        | E::UnboundIndex { .. }
        | E::UnknownBinding { .. }
        | E::UnresolvedValue { .. }
        | E::UnresolvedDomain { .. } => "compile.math",
        _ => "internal.invariant",
    }
}
fn rule<'a>(
    error: &'a RuleError,
    reg: &Registry,
    visitor: &mut Visitor<'a, '_>,
) -> Result<(), CompilerError> {
    match error {
        RuleError::Execution { source, .. } => rule(source, reg, visitor),
        RuleError::Collection { errors } if !errors.is_empty() => {
            for error in errors {
                rule(error, reg, visitor)?;
            }
            Ok(())
        }
        RuleError::InvariantViolations { findings, .. } => {
            visitor("validation.invariant", error, Some(findings))
        }
        RuleError::Catalog(error) => catalog(error, reg, visitor),
        RuleError::Relation(error) => relation(error, visitor),
        RuleError::ResourceLimit { .. } => visitor("runtime.resource_limit", error, None),
        RuleError::Infrastructure { .. } => visitor("runtime.infrastructure", error, None),
        RuleError::ConfigInvalid { .. } => visitor("config.invalid", error, None),
        RuleError::FloatKey { .. }
        | RuleError::HeadSchemaMismatch { .. }
        | RuleError::Internal { .. }
        | RuleError::Collection { .. } => visitor("internal.invariant", error, None),
    }
}
fn relation<'a>(
    error: &'a pse_relations::RelationError,
    visitor: &mut Visitor<'a, '_>,
) -> Result<(), CompilerError> {
    use pse_relations::RelationError as E;
    if let E::Validation { errors } = error {
        for error in errors {
            relation(error, visitor)?;
        }
        return Ok(());
    }
    visitor(
        if matches!(error, E::Arrow(_)) {
            "runtime.infrastructure"
        } else {
            "validation.invariant"
        },
        error,
        None,
    )
}
fn catalog<'a>(
    error: &'a CatalogError,
    reg: &Registry,
    visitor: &mut Visitor<'a, '_>,
) -> Result<(), CompilerError> {
    match error {
        CatalogError::Semantic(source) => {
            let source: &dyn std::error::Error = source.as_ref();
            if let Some(error) = source.downcast_ref::<RuleError>() {
                return rule(error, reg, visitor);
            }
            if let Some(error) = source.downcast_ref::<CompilerError>() {
                return visit(error, reg, visitor);
            }
            if let Some(error) = source.downcast_ref::<CatalogError>() {
                return catalog(error, reg, visitor);
            }
            visitor(exact_class(error, reg)?, error, None)
        }
        CatalogError::Multiple { errors } if !errors.is_empty() => {
            for error in errors {
                catalog(error, reg, visitor)?;
            }
            Ok(())
        }
        CatalogError::Canon(error) => visitor(canonical(error), error, None),
        CatalogError::ResourceLimit { .. } | CatalogError::Reserve(_) => {
            visitor("runtime.resource_limit", error, None)
        }
        CatalogError::Cancelled => visitor("runtime.cancelled", error, None),
        CatalogError::Infrastructure { .. }
        | CatalogError::CorruptObject { .. }
        | CatalogError::RefConflict { .. } => visitor("runtime.infrastructure", error, None),
        CatalogError::ConfigInvalid { .. } => visitor("config.invalid", error, None),
        CatalogError::UserModel { .. } => visitor("user.model", error, None),
        CatalogError::EvaluationError { .. } => visitor("solve.evaluation_error", error, None),
        CatalogError::CompileProperty { .. } => visitor("compile.property", error, None),
        CatalogError::Internal { .. } | CatalogError::Multiple { .. } => {
            visitor("internal.invariant", error, None)
        }
        CatalogError::ManifestInvalid { .. }
        | CatalogError::UnknownRegistry { .. }
        | CatalogError::UnknownVersion { .. }
        | CatalogError::Admission { .. }
        | CatalogError::ForeignSource { .. }
        | CatalogError::Sealed
        | CatalogError::LogicalHashMismatch { .. }
        | CatalogError::Membership { .. }
        | CatalogError::Snapshot(_) => visitor("validation.invariant", error, None),
        _ => visitor(exact_class(error, reg)?, error, None),
    }
}
fn exact_class(error: &dyn Diagnostic, reg: &Registry) -> Result<&'static str, CompilerError> {
    // Rust diagnostic paths and the registry use their documented separator spellings.
    // Only an exact declared class matches; unknown subcodes are never guessed by prefix.
    let code = error
        .code()
        .ok_or_else(|| invalid("unclassified semantic diagnostic"))?
        .to_string()
        .replace("::", ".");
    reg.enum_spec("FailureClass")
        .and_then(|spec| spec.members.iter().find(|member| member.name == code))
        .map(|member| member.name)
        .ok_or_else(|| invalid("semantic diagnostic has no declared terminal failure class"))
}
