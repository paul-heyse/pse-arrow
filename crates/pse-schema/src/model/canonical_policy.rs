// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Inert native canonical field policies, independent of database/type/context admission.
use super::{CanonicalFieldContract, EnumSpec, FieldContract, field_facets::{CANONICAL_KEY, CANONICAL_READONLY, CANONICAL_RECORD_KEY, CANONICAL_ASSERT, CANONICAL_DEFAULT, CANONICAL_TRANSITION, CANONICAL_TRANSITION_EXCEPTION, CANONICAL_SEALED, CANONICAL_SEAL_REFUSAL, CANONICAL_REFERENCE_KIND}};
use crate::{SchemaError, catalog::canonical_refusals::RefusalCode};
use std::{collections::HashMap, sync::Arc};
use surrealdb_types::ToSql as _;

/// Original concrete parser refusal with deterministic source-input identity.
/// SurrealDB's ParseError is Clone but not Eq; equality here identifies the exact
/// failed input and parser error variant under the fixed stored-definition parser.
#[derive(Clone, Debug, thiserror::Error)]
#[error("{source}")]
pub struct CanonicalExpressionParseError {
    input: Arc<str>,
    #[source]
    source: surrealdb_syn::ParseError,
}
impl CanonicalExpressionParseError {
    /// Exact original input, never a substituted rendered diagnostic.
    pub fn input(&self) -> &str { &self.input }
    /// Original typed SurrealDB parser cause.
    pub fn parse_error(&self) -> &surrealdb_syn::ParseError { &self.source }
}
impl PartialEq for CanonicalExpressionParseError {
    fn eq(&self, other: &Self) -> bool { self.input == other.input && std::mem::discriminant(&self.source) == std::mem::discriminant(&other.source) }
}
impl Eq for CanonicalExpressionParseError {}
pse_diagnostics::impl_diagnostic! {CanonicalExpressionParseError,
    code(_this){Some(pse_diagnostics::DiagnosticCode::SchemaInvalidDeclaration)},
    forward(_this){None},help(_this){None},related(_this){None},source(_this){None}
}

/// A complete inert SurrealQL expression; parsing grants no execution or context validity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanonicalExpression(surrealdb_sql::Expr);
impl CanonicalExpression {
    fn parse(input: &str, path: &str, facet: &'static str) -> Result<Self, SchemaError> {
        // Exact 3.3.4 entry parses field-definition context and assert_finished:
        // https://docs.rs/surrealdb-syn/3.3.4/src/surrealdb_syn/lib.rs.html
        surrealdb_syn::expr_for_definition(input).map(Self).map_err(|source| SchemaError::CanonicalExpression {
            context: path.into(), facet,
            source: CanonicalExpressionParseError { input: input.into(), source },
        })
    }
    /// Canonical inert SQL text from the native AST formatter, not Debug or evaluation.
    pub fn canonical(&self) -> String { self.0.to_sql() }
    /// Parsed native syntax; datastore, type, capability and variable context remain separate.
    pub fn expression(&self) -> &surrealdb_sql::Expr { &self.0 }
}

/// Canonical policy carried by the actual field's named source facets.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanonicalFieldPolicy {
    /// Existing key/readonly/record-key meaning, composed without a second parser.
    pub contract: Option<CanonicalFieldContract>,
    /// Source ASSERT expression.
    pub assertion: Option<CanonicalExpression>,
    /// Source DEFAULT expression.
    pub default: Option<CanonicalExpression>,
    /// Exact initial enum member; never parsed as an expression.
    pub initial: Option<String>,
    /// Exceptional transition predicate.
    pub transition_exception: Option<CanonicalExpression>,
    /// Original before-row sealing predicate.
    pub sealed: Option<CanonicalExpression>,
    /// Existing closed native refusal identity.
    pub seal_refusal: Option<RefusalCode>,
    /// Exact referenced kind member; membership requires its resolved native enum.
    pub reference_kind: Option<String>,
}
impl CanonicalFieldPolicy {
    /// Parse complete selected field policy from the same source metadata carrier as Arrow.
    /// # Errors
    /// Unknown policy facets, malformed expressions, unknown refusals or incoherent policy
    /// refuse. Syntax success does not establish database/type/variable-context validity.
    pub fn parse_metadata(metadata: &HashMap<String, String>, path: &str) -> Result<Option<Self>, SchemaError> {
        const KEYS: &[&str] = &[CANONICAL_KEY, CANONICAL_READONLY, CANONICAL_RECORD_KEY, CANONICAL_ASSERT, CANONICAL_DEFAULT, CANONICAL_TRANSITION, CANONICAL_TRANSITION_EXCEPTION, CANONICAL_SEALED, CANONICAL_SEAL_REFUSAL, CANONICAL_REFERENCE_KIND];
        if let Some(key) = metadata.keys().filter(|key| key.starts_with("pse.domain.canonical.") && !KEYS.contains(&key.as_str())).min() {
            return Err(crate::checks::invalid(path, format!("unknown canonical field policy facet {key}")));
        }
        if !KEYS.iter().any(|key| metadata.contains_key(*key)) { return Ok(None); }
        let get = |key: &str| metadata.get(key).map(String::as_str);
        let expression = |key| get(key).map(|text| CanonicalExpression::parse(text, path, key)).transpose();
        let contract = CanonicalFieldContract::parse_metadata(get(CANONICAL_KEY), get(CANONICAL_READONLY), get(CANONICAL_RECORD_KEY), path)?;
        if contract.is_some_and(|contract| contract.record_key && contract.readonly != Some(true)) {
            return Err(crate::checks::invalid(path, "canonical record-key projection requires explicit readonly policy"));
        }
        let member = |key: &str| get(key).map(|text| if text.is_empty() { Err(crate::checks::invalid(path, format!("{key} requires a nonempty native enum member"))) } else { Ok(text.to_owned()) }).transpose();
        let initial = member(CANONICAL_TRANSITION)?;
        let reference_kind = member(CANONICAL_REFERENCE_KIND)?;
        let seal_refusal = get(CANONICAL_SEAL_REFUSAL).map(|value| RefusalCode::parse(value).ok_or_else(|| crate::checks::invalid(path, "unknown canonical sealed-field refusal identity"))).transpose()?;
        let policy = Self { contract, assertion: expression(CANONICAL_ASSERT)?, default: expression(CANONICAL_DEFAULT)?, initial,
            transition_exception: expression(CANONICAL_TRANSITION_EXCEPTION)?, sealed: expression(CANONICAL_SEALED)?, seal_refusal, reference_kind };
        if policy.transition_exception.is_some() && policy.initial.is_none() { return Err(crate::checks::invalid(path, "canonical transition exception requires an initial lifecycle member")); }
        if policy.seal_refusal.is_some() != policy.sealed.is_some() { return Err(crate::checks::invalid(path, "canonical sealing predicate and native refusal must be declared together")); }
        Ok(Some(policy))
    }
    /// Apply native physical/association facts unavailable to the metadata-only parser.
    /// # Errors
    /// Initial members without an enum or referenced kinds without a foreign key refuse.
    pub fn validate_field(&self, field: &FieldContract, path: &str) -> Result<(), SchemaError> {
        if let Some(contract) = self.contract { contract.validate_field(field, path)?; }
        if self.initial.is_some() && field.enum_name().is_none() { return Err(crate::checks::invalid(path, "canonical initial member requires an actual native enum field")); }
        if self.reference_kind.is_some() && field.checked_fk()?.is_none() { return Err(crate::checks::invalid(path, "canonical reference-kind member requires an actual native foreign-key association")); }
        Ok(())
    }
    /// Validate the initial value against its separately resolved native enumeration.
    /// # Errors
    /// An initial member outside that actual enum refuses.
    pub fn validate_initial(&self, domain: &EnumSpec, path: &str) -> Result<(), SchemaError> { Self::validate_member(self.initial.as_deref(), domain, path, "initial") }
    /// Validate a reference-kind member against the referenced row's actual kind enum.
    /// # Errors
    /// No inferred or global kind vocabulary substitutes for the resolved native owner.
    pub fn validate_reference_kind(&self, domain: &EnumSpec, path: &str) -> Result<(), SchemaError> { Self::validate_member(self.reference_kind.as_deref(), domain, path, "reference kind") }
    fn validate_member(member: Option<&str>, domain: &EnumSpec, path: &str, role: &str) -> Result<(), SchemaError> {
        if member.is_some_and(|member| !domain.members.iter().any(|value| value.name == member)) {
            return Err(crate::checks::invalid(path, format!("canonical {role} is not a member of native enum {}", domain.name)));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn metadata(values: &[(&str, &str)]) -> HashMap<String, String> { values.iter().map(|(key, value)| ((*key).into(), (*value).into())).collect() }
    fn domain() -> EnumSpec {
        EnumSpec { id: pse_ids::SemanticId::NIL, name: "Phase", idaes_source: None, source: None, default_member: None, transitions: vec![], projections: vec![], members: vec![super::super::EnumMember { name: "ready", idaes_name: None, deprecated: false, doc: "test source member" }] }
    }
    #[test]
    fn canonical_policy_uses_native_ast_and_composes_existing_contract() {
        assert!(CanonicalFieldPolicy::parse_metadata(&HashMap::new(), "source.field").unwrap().is_none());
        let policy = CanonicalFieldPolicy::parse_metadata(&metadata(&[(CANONICAL_KEY, "run"), (CANONICAL_READONLY, "false"), (CANONICAL_ASSERT, "$value>0"), (CANONICAL_DEFAULT, "1+2")]), "source.field").unwrap().unwrap();
        assert_eq!(policy.contract.unwrap().key, Some(super::super::CanonicalKeyKind::Run));
        assert_eq!(policy.contract.unwrap().readonly, Some(false));
        let assertion = policy.assertion.unwrap();
        assert_eq!(assertion.expression(), &surrealdb_syn::expr_for_definition("$value > 0").unwrap());
        assert_eq!(surrealdb_syn::expr_for_definition(&assertion.canonical()).unwrap(), *assertion.expression());
        assert_eq!(surrealdb_syn::expr_for_definition(&policy.default.unwrap().canonical()).unwrap(), surrealdb_syn::expr_for_definition("1+2").unwrap(), "formatting does not evaluate the expression");
    }
    #[test]
    fn canonical_policy_refuses_incomplete_and_trailing_expression_with_typed_source() {
        use std::error::Error as _;
        for facet in [CANONICAL_ASSERT, CANONICAL_DEFAULT, CANONICAL_SEALED, CANONICAL_TRANSITION_EXCEPTION] {
            for input in ["$value >", "true false", "true; RETURN true"] {
                let error = CanonicalFieldPolicy::parse_metadata(&metadata(&[(facet, input), (CANONICAL_TRANSITION, "ready")]), "runtime.row.field").unwrap_err();
                let SchemaError::CanonicalExpression { context, facet: actual, source } = &error else { panic!("typed original parser refusal: {error}"); };
                assert_eq!(context, "runtime.row.field"); assert_eq!(*actual, facet);
                assert_eq!(source.input(), input);
                assert!(matches!(source.parse_error(), surrealdb_syn::ParseError::InvalidQuery(_)));
                assert!(source.source().unwrap().downcast_ref::<surrealdb_syn::ParseError>().is_some());
                assert_eq!(error, error.clone(), "registry error memoization retains deterministic identity");
            }
        }
    }
    #[test]
    fn canonical_policy_validates_native_refusal_and_intrinsic_coherence() {
        let refusal = RefusalCode::ExecutionTerminalSeedManifestImmutable;
        let policy = CanonicalFieldPolicy::parse_metadata(&metadata(&[(CANONICAL_SEALED, "$before.phase IN ['terminal']"), (CANONICAL_SEAL_REFUSAL, refusal.wire())]), "source.seed").unwrap().unwrap();
        assert_eq!(policy.seal_refusal, Some(refusal));
        for values in [
            vec![(CANONICAL_SEALED, "true"), (CANONICAL_SEAL_REFUSAL, "PSE_REFUSAL:unknown")],
            vec![(CANONICAL_SEAL_REFUSAL, refusal.wire())],
            vec![(CANONICAL_SEALED, "true")],
            vec![(CANONICAL_TRANSITION_EXCEPTION, "true")],
            vec![(CANONICAL_RECORD_KEY, "true"), (CANONICAL_READONLY, "false")],
            vec![("pse.domain.canonical.unknown", "true")],
        ] { assert!(CanonicalFieldPolicy::parse_metadata(&metadata(&values), "source.field").is_err()); }
    }
    #[test]
    fn canonical_policy_checks_initial_and_reference_members_separately_from_syntax() {
        let field = FieldContract::enumeration("Phase").with_canonical_initial("ready").with_canonical_transition_exception("$this.retry = true");
        let policy = field.canonical_policy().unwrap().unwrap();
        policy.validate_initial(&domain(), "source.phase").unwrap();
        let unknown = FieldContract::enumeration("Phase").with_canonical_initial("absent").canonical_policy().unwrap().unwrap();
        assert!(unknown.validate_initial(&domain(), "source.phase").is_err());
        assert!(FieldContract::native(arrow_schema::DataType::Utf8).with_canonical_initial("ready").canonical_policy().is_err());
        assert!(FieldContract::native(arrow_schema::DataType::Utf8).with_canonical_reference_kind("ready").canonical_policy().is_err());
        let field = FieldContract::native(arrow_schema::DataType::Utf8).with_fk("runtime.target", "id").with_canonical_reference_kind("ready");
        let policy = field.canonical_policy().unwrap().unwrap();
        policy.validate_reference_kind(&domain(), "source.target").unwrap();
        assert!(field.with_canonical_reference_kind("missing").canonical_policy().unwrap().unwrap().validate_reference_kind(&domain(), "source.target").is_err());
    }
    #[test]
    fn canonical_policy_is_checked_by_native_field_admission_with_owning_path() {
        let field = FieldContract::native(arrow_schema::DataType::Boolean).with_canonical_default("true trailing");
        assert!(matches!(field.validate_facets("runtime.original.flag"), Err(SchemaError::CanonicalExpression { context, facet: CANONICAL_DEFAULT, .. }) if context == "runtime.original.flag"));
    }
    #[test]
    fn canonical_policy_reuses_opaque_key_physical_coherence_for_incoming_fields() {
        use super::super::CanonicalKeyKind;
        let valid = FieldContract::native(arrow_schema::DataType::Utf8).with_canonical_key(CanonicalKeyKind::Run);
        valid.canonical_policy().unwrap().unwrap().validate_field(&valid, "runtime.original.id").unwrap();
        for field in [
            FieldContract::native(arrow_schema::DataType::UInt64).with_canonical_key(CanonicalKeyKind::Run),
            FieldContract::enumeration("Phase").with_canonical_key(CanonicalKeyKind::Run),
            valid.with_identity("OriginalIdentity"),
        ] {
            let policy = CanonicalFieldPolicy::parse_metadata(field.field().metadata(), "runtime.original.id").unwrap().unwrap();
            assert!(policy.validate_field(&field, "runtime.original.id").is_err());
            assert!(field.validate_facets("runtime.original.id").is_err());
        }
    }
    #[test]
    fn canonical_policy_parses_actual_canonical_catalog_population() {
        let registry = crate::registry().unwrap();
        let mut expressions = 0;
        for relation in registry.relations() {
            for field in &relation.columns {
                if let Some(policy) = CanonicalFieldPolicy::parse_metadata(field.field().metadata(), &format!("{}.{}", relation.key, field.name())).unwrap() {
                    policy.validate_field(field, field.name()).unwrap();
                    expressions += [&policy.assertion, &policy.default, &policy.sealed, &policy.transition_exception].into_iter().flatten().count();
                    if let Some(name) = field.enum_name() { policy.validate_initial(registry.enum_spec(name).unwrap(), field.name()).unwrap(); }
                }
            }
        }
        assert!(expressions > 0, "the selected real source population was exercised");
    }
}
