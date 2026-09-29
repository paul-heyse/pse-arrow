// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::{Declaration, Selected};
use crate::AuthoringError;
use pse_model::generated::identities::DeclarationId;
use std::collections::{BTreeMap, BTreeSet};

fn quoted(text: &str) -> String {
    format!("\"{}\"", text.replace('\\', "\\\\").replace('"', "\\\""))
}
/// A name as a plain identifier when it is one, else quoted.
fn name(text: &str) -> String {
    let mut chars = text.chars();
    if chars
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        text.into()
    } else {
        quoted(text)
    }
}
fn list(values: &[String], open: &str, close: &str) -> String {
    if values.is_empty() {
        String::new()
    } else {
        format!("{open}{}{close}", values.join(", "))
    }
}
fn indices<'a>(pairs: impl Iterator<Item = (&'a str, &'a str)>) -> String {
    list(
        &pairs
            .map(|(n, d)| format!("{n} in {d}"))
            .collect::<Vec<_>>(),
        "[",
        "]",
    )
}
/// The canonical spelling of a declared type (ADR-0123 Outcome 1).
fn ty(nodes: &[super::TypeNode]) -> Result<String, AuthoringError> {
    Ok(super::render_type(nodes)?)
}
fn parameters<'a>(
    items: impl Iterator<Item = (&'a str, &'a [super::TypeNode], Option<&'a str>)>,
) -> Result<String, AuthoringError> {
    Ok(list(
        &items
            .map(|(n, t, d)| {
                Ok(format!(
                    "{n}: {}{}",
                    ty(t)?,
                    d.map_or(String::new(), |d| format!(" = {d}"))
                ))
            })
            .collect::<Result<Vec<_>, AuthoringError>>()?,
        "(",
        ")",
    ))
}
/// A fixture's execution policy clause, settings and options in their canonical order.
fn fixture_policy(
    policy: &super::AuthoredModelingDeclarationsFieldValueScopeFixturePolicy,
) -> String {
    fn options<const N: usize>(values: [(&str, Option<String>); N]) -> Option<String> {
        let written = values
            .into_iter()
            .filter_map(|(name, value)| value.map(|v| format!("{name}({v})")))
            .collect::<Vec<_>>();
        (!written.is_empty()).then(|| written.join(" "))
    }
    let mut settings = Vec::new();
    if let Some(backend) = policy.backend {
        settings.push(format!("backend {};", backend.as_str()));
    }
    if let Some(presolve) = policy.presolve {
        settings.push(format!("presolve {};", presolve.as_str()));
    }
    if let Some(derivatives) = options([
        ("step", policy.derivative_step.map(|v| format!("{v:?}"))),
        ("tolerance", policy.derivative_tolerance.map(|v| format!("{v:?}"))),
        ("cells", policy.derivative_cells.map(|v| v.to_string())),
    ]) {
        settings.push(format!("derivatives {derivatives};"));
    }
    if let Some(limits) = options([
        ("items", policy.items.map(|v| v.to_string())),
        ("body_occurrences", policy.body_occurrences.map(|v| v.to_string())),
        ("body_slots", policy.body_slots.map(|v| v.to_string())),
        ("foreign_bytes", policy.foreign_bytes.map(|v| v.to_string())),
    ]) {
        settings.push(format!("limits {limits};"));
    }
    format!("policy {{ {} }}", settings.join(" "))
}
fn bad(reason: impl Into<String>) -> AuthoringError {
    AuthoringError::Contract {
        at: None,
        reason: reason.into(),
    }
}

/// Canonically print declarations, preserving explicit IDs and semantic order.
/// # Errors
/// Malformed tagged payloads, duplicate/dangling identities or recursive parent ownership.
pub fn render(rows: &[Declaration]) -> Result<String, AuthoringError> {
    let ids = rows
        .iter()
        .map(|r| r.declaration_id)
        .collect::<BTreeSet<_>>();
    if ids.len() != rows.len() {
        return Err(bad("duplicate declaration identity"));
    }
    let mut children = BTreeMap::<Option<DeclarationId>, Vec<&Declaration>>::new();
    for row in rows {
        if row.parent_id.is_some_and(|id| !ids.contains(&id)) {
            return Err(bad("missing parent"));
        }
        children.entry(row.parent_id).or_default().push(row);
    }
    for values in children.values_mut() {
        values.sort_by_key(|r| (r.ordinal, r.declaration_id));
    }
    let mut visited = BTreeSet::new();
    let mut out = String::new();
    print_block(None, &children, &mut visited, 0, &mut out)?;
    if visited.len() != rows.len() {
        return Err(bad("cyclic or unreachable parent ownership"));
    }
    Ok(out)
}
fn print_block(
    parent: Option<DeclarationId>,
    children: &BTreeMap<Option<DeclarationId>, Vec<&Declaration>>,
    visited: &mut BTreeSet<DeclarationId>,
    depth: usize,
    out: &mut String,
) -> Result<(), AuthoringError> {
    if depth > 64 {
        return Err(bad("render depth exceeds 64"));
    }
    for row in children.get(&parent).into_iter().flatten() {
        if !visited.insert(row.declaration_id) {
            return Err(bad("cyclic declaration ownership"));
        }
        let pad = "  ".repeat(depth);
        out.push_str(&pad);
        out.push_str(&format!(
            "@id({}) ",
            quoted(&row.declaration_id.to_string())
        ));
        if row.is_override {
            out.push_str("override ");
        }
        let n = crate::grammar::render_name(&row.name);
        let mut block = false;
        let value = row.value.selected().map_err(|e| bad(e.to_string()))?;
        let text = match value {
            Selected::Relaxation(v) => format!("relax {n} on {} nominal {};", v.target, v.nominal),
            Selected::Continuation(v) => format!(
                "continue {n} on {} from {} to {};",
                v.target, v.start, v.end
            ),
            Selected::Continuous(v) => format!(
                "domain {n}: {} from {} to {};",
                ty(&v.r#type)?, v.lower, v.upper
            ),
            Selected::DifferenceScheme(v) => {
                let list = |values: Vec<String>| values.join(", ");
                format!("difference {n} order({}) offsets({}) weights({}) quadrature({});",
                    v.order, list(v.offsets.iter().map(ToString::to_string).collect()),
                    list(v.weights.iter().map(ToString::to_string).collect()),
                    list(v.quadrature.iter().map(ToString::to_string).collect()))
            }
            Selected::CollocationScheme(v) => format!(
                "collocation {n} alpha({}) beta({}) right({});", v.alpha, v.beta, v.right_endpoint,
            ),
            Selected::Discretization(v) => format!(
                "discretize {n} on {} using {}(elements = {}, order = {});",
                v.target,
                v.scheme.as_str(),
                v.elements,
                v.order
            ),
            Selected::Realization(v) => {
                use pse_model::generated::enums::ModelingRealizationPolicy as Policy;
                if v.function.is_some() != (v.policy == Policy::Smooth) {
                    return Err(bad("a smoothing function belongs to exactly smooth"));
                }
                let policy = match (v.policy, v.argument.as_deref()) {
                    (Policy::Smooth, Some(width)) => {
                        format!("smooth({}, {width})", v.function.as_deref().unwrap_or_default())
                    }
                    (Policy::Smooth, None) => return Err(bad("smooth needs its width")),
                    (Policy::PenaltyL1, None) => "penalty(l1)".into(),
                    (Policy::BigM, Some(m)) => format!("bigm({m})"),
                    (Policy::DerivedBigM, None) => "bigm(derived)".into(),
                    (Policy::DerivedBigM, Some(margin)) => format!("bigm(derived, {margin})"),
                    (Policy::Hull, Some(epsilon)) => format!("hull({epsilon})"),
                    (Policy::BigM, None) => return Err(bad("an authored big-M needs its value")),
                    (policy, None) => policy.as_str().into(),
                    (_, Some(_)) => return Err(bad("this realization takes no argument")),
                };
                format!("realize {n} on {} using {policy}{};", v.target,
                    v.accelerator.as_ref().map_or(String::new(), |id| format!("({})", quoted(id))))
            }
            Selected::Sos1(v) | Selected::Sos2(v) => format!(
                "{} {n}{}: {} weight {};",
                if matches!(value, Selected::Sos1(_)) { "sos1" } else { "sos2" },
                indices(v.indices.iter().map(|i| (i.name.as_str(), i.domain.as_str()))),
                v.member,
                v.weight
            ),
            Selected::Atmost(v) | Selected::Atleast(v) | Selected::Exactly(v) => format!(
                "{} {n}{}: {} of {};",
                match value {
                    Selected::Atmost(_) => "atmost",
                    Selected::Atleast(_) => "atleast",
                    _ => "exactly",
                },
                indices(v.indices.iter().map(|i| (i.name.as_str(), i.domain.as_str()))),
                v.count,
                v.member
            ),
            Selected::Piecewise(v) => format!(
                "piecewise {n}{}: {} == {} at ({}, {});",
                indices(v.indices.iter().map(|i| (i.name.as_str(), i.domain.as_str()))),
                v.output,
                v.input,
                v.abscissa,
                v.ordinate
            ),
            Selected::Complementarity(v) => format!(
                "complements {n}{}: ({} >= 0, {} >= 0);",
                indices(v.indices.iter().map(|i| (i.name.as_str(), i.domain.as_str()))),
                v.first,
                v.second
            ),
            Selected::Logic(v) => format!(
                "logic {n}{}: {};",
                indices(v.indices.iter().map(|i| (i.name.as_str(), i.domain.as_str()))),
                v.proposition
            ),
            Selected::Package(v)
            | Selected::EntityKind(v)
            | Selected::Interface(v)
            | Selected::Definition(v)
            | Selected::Case(v)
            | Selected::Test(v)
            | Selected::Stage(v)
            | Selected::Regime(v)
            | Selected::Disjunction(v)
            | Selected::Alternative(v)
            | Selected::Implicit(v) => {
                block = true;
                let keyword = match value {
                    Selected::Package(_) => "package",
                    Selected::EntityKind(_) => "entity kind",
                    Selected::Interface(_) => "interface",
                    Selected::Definition(_) => "def",
                    Selected::Case(_) => "case",
                    Selected::Test(_) => "test",
                    Selected::Stage(_) => "stage",
                    Selected::Regime(_) => "regime",
                    Selected::Disjunction(_) => "disjunction",
                    Selected::Alternative(_) => "alternative",
                    _ => "implicit",
                };
                format!(
                    "{keyword} {n}{}{}{}{}{}{}{} {{",
                    list(&v.type_parameters, "<", ">"),
                    parameters(v.parameters.iter().map(|p| (
                        p.name.as_str(),
                        p.r#type.as_slice(),
                        p.default_value.as_deref()
                    )))?,
                    if v.bases.is_empty() {
                        String::new()
                    } else {
                        format!(" : {}", v.bases.join(", "))
                    },
                    v.selection.as_ref().map_or_else(String::new,|s|format!(" select minimum({}, {})",s.criterion,s.tolerance)),
                    v.eligibility.as_ref().map_or_else(String::new,|e|format!(" eligible({e})")),
                    v.oracle.as_ref().map_or_else(String::new,|o|format!(" source {} revision {}",quoted(&o.reference),quoted(&o.revision))),
                    v.fixture.as_ref().map_or_else(String::new, |f| {
                        let mut statements = vec![format!("dof {};", f.degrees_of_freedom)];
                        if let Some(execution) = f.execution { statements.push(format!("run {};", execution.as_str())); }
                        if let Some(intent) = f.intent { statements.push(format!("intent {};", intent.as_str())); }
                        if let Some(policy) = &f.policy { statements.push(fixture_policy(policy)); }
                        if !f.stages.is_empty() { statements.push(format!("stages({});", f.stages.iter().map(|s| quoted(s)).collect::<Vec<_>>().join(", "))); }
                        if let Some(policy) = &f.initialization { statements.push(format!("initialize homotopy({}) step({}) minimum({}) growth({}) attempts({}) seconds({});", policy.homotopy, policy.initial_step, policy.minimum_step, policy.growth, policy.maximum_attempts, policy.time_limit_seconds)); }
                        if let Some(integration) = &f.integration {
                            let quadrature = integration.quadrature_relative_tolerance.map_or_else(String::new, |relative| format!(" quadrature_relative({relative}) quadrature_absolute({})", integration.quadratures.iter().map(|q|format!("{}={}",q.target,q.absolute_tolerance)).collect::<Vec<_>>().join(", ")));
                            statements.push(format!("integrate samples({}) relative({}) normalized_absolute({}) step({}){quadrature};", integration.samples.join(", "), integration.relative_tolerance, integration.normalized_absolute_tolerance, integration.initial_step));
                            statements.extend(integration.schedules.iter().map(|s| format!(
                                "schedule {} at({}) values({}){}{}{};",
                                s.target,
                                s.times.join(", "),
                                s.values.join(", "),
                                if s.free { " free" } else { "" },
                                s.lower.as_ref().map_or_else(String::new, |v| format!(" lower({v})")),
                                s.upper.as_ref().map_or_else(String::new, |v| format!(" upper({v})"))
                            )));
                        }
                        if let Some(shooting) = &f.shooting {
                            statements.push(format!(
                                "shoot {}{};",
                                shooting.method.as_str(),
                                if shooting.nodes.is_empty() { String::new() } else { format!(" nodes({})", shooting.nodes.join(", ")) }
                            ));
                        }
                        for mode in &f.modes {
                            let facts = mode.facts.iter().map(|f| format!("{}.{} = {}", f.namespace.as_str(), f.name, f.value)).collect::<Vec<_>>();
                            statements.push(format!("mode {}{};", name(&mode.name), if facts.is_empty() { String::new() } else { format!(" facts({})", facts.join(", ")) }));
                            for e in &mode.events {
                                let reset = e.reset.iter().map(|r| format!("{} = {}", r.target, r.expression)).collect::<Vec<_>>();
                                statements.push(format!(
                                    "event {} direction({}) tolerance({}){} {};",
                                    e.guard,
                                    e.direction.as_str(),
                                    e.tolerance,
                                    if reset.is_empty() { String::new() } else { format!(" reset({})", reset.join(", ")) },
                                    e.next.as_ref().map_or_else(|| "terminal".into(), |n| format!("next({})", name(n)))
                                ));
                            }
                        }
                        if let Some(failure) = &f.expected_failure { statements.push(format!("failure {} {};", failure.class.as_str(), quoted(&failure.rule))); }
                        statements.extend(f.specifications.iter().map(|s|format!("{} {}{};",s.kind.as_str(),s.target,s.expression.as_ref().map_or_else(String::new,|e|format!(" = {e}")))));
                        format!(" fixture {{ {} }}", statements.join(" "))
                    })
                )
            }
            Selected::Parameter(v)
            | Selected::Variable(v)
            | Selected::Let(v)
            | Selected::Alias(v)
            | Selected::Attribute(v)
            | Selected::Set(v)
            | Selected::Child(v)
            | Selected::Port(v)
            | Selected::Preset(v)
            | Selected::ScopeValue(v) => {
                let keyword = match value {
                    Selected::Parameter(_) => "param",
                    Selected::Variable(_) => "var",
                    Selected::Let(_) => "let",
                    Selected::Alias(_) => "alias",
                    Selected::Attribute(_) => "attribute",
                    Selected::Set(_) => "set",
                    Selected::Child(_) => "child",
                    Selected::Port(_) => "port",
                    Selected::Preset(_) => "preset",
                    _ => "scope",
                };
                format!(
                    "{keyword} {n}{}{}{}{}{};",
                    indices(
                        v.indices
                            .iter()
                            .map(|i| (i.name.as_str(), i.domain.as_str()))
                    ),
                    v.r#type
                        .as_deref()
                        .map(ty)
                        .transpose()?
                        .map_or(String::new(), |t| format!(": {t}")),
                    // Continuous is the default and prints without a facet.
                    v.domain
                        .filter(|d| d.is_discrete())
                        .map_or(String::new(), |d| format!(" in {}", d.as_str())),
                    v.expression
                        .as_ref()
                        .map_or(String::new(), |e| format!(" = {e}")),
                    v.defined_by
                        .as_ref()
                        .map_or(String::new(), |e| format!(" defined by {e}"))
                )
            }
            Selected::Function(v) => format!(
                "fn {n}{}{} -> {}{}{}{}{};",
                list(&v.type_parameters, "<", ">"),
                if v.arguments.is_empty() {
                    "()".into()
                } else {
                    parameters(v.arguments.iter().map(|p| {
                        (
                            p.name.as_str(),
                            p.r#type.as_slice(),
                            p.default_value.as_deref(),
                        )
                    }))?
                },
                ty(&v.return_type)?,
                v.validity.as_ref().map_or(String::new(),|p|format!(" valid({p})")),
                v.continuity
                    .map_or(String::new(), |order| format!(" piecewise {order}")),
                v.external.as_ref().map_or(String::new(),|v|format!(" external {} revision {} data {} output {} derivatives {} source {} smoothness {}",quoted(&v.implementation),quoted(&v.revision),quoted(&v.data),v.output,v.derivatives,v.derivative_source.as_str(),v.smoothness)),
                v.body.as_ref().map_or(String::new(), |b| format!(" = {b}"))
            ),
            Selected::Import(v) => format!(
                "use {n} @ {}{};",
                // Exact is the only admitted operator (§6.1); it prints unprefixed.
                quoted(&super::requirement_version(&v.version)),
                v.alias
                    .as_ref()
                    .map_or(String::new(), |a| format!(" as {a}"))
            ),
            Selected::Enum(v) => format!("enum {n} {{ {} }}", v.members.join(", ")),
            Selected::Entity(v) => format!(
                "entity {} {n} {{ {} }}",
                v.kind_name,
                v.attributes
                    .iter()
                    .map(|a| format!("{} = {}", a.name, a.expression))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Selected::Equation(v) => format!(
                "eq {n}{}{}: {};",
                indices(
                    v.indices
                        .iter()
                        .map(|i| (i.name.as_str(), i.domain.as_str()))
                ),
                v.condition.as_ref().map_or_else(String::new, |c| format!(
                    " when {}{}",
                    if c.active { "" } else { "not " },
                    c.variable
                )),
                v.expression
            ),
            Selected::When(v) => {
                block = true;
                format!("when {} {{", v.predicate)
            }
            Selected::Requirement(v) => {
                format!("require {} : {};", v.predicate, quoted(&v.message))
            }
            Selected::Connection(v) => format!("connect {} -> {};", v.from, v.to),
            Selected::Accumulator(v) => format!(
                "accumulate {n}{}: {} {} tolerance {};",
                indices(
                    v.indices
                        .iter()
                        .map(|i| (i.name.as_str(), i.domain.as_str()))
                ),
                ty(&v.r#type)?,
                v.mode.as_str(),
                v.tolerance
            ),
            Selected::Contribution(v) => format!(
                "contribute {}{} role {}{} = {};",
                indices(
                    v.indices
                        .iter()
                        .map(|i| (i.name.as_str(), i.domain.as_str()))
                ),
                v.target,
                v.role.as_str(),
                match (&v.transfer_id, &v.transfer_side) {
                    (Some(id), Some(side)) => format!(" transfer {} {side}", quoted(id)),
                    (None, None) => String::new(),
                    _ => return Err(bad("incomplete transfer")),
                },
                v.expression
            ),
            Selected::Table(v) => format!(
                "table {n}{}: {} missing {}{};",
                list(
                    &v.keys
                        .iter()
                        .map(|k| Ok(format!("{}: {}", k.name, ty(&k.r#type)?)))
                        .collect::<Result<Vec<_>, AuthoringError>>()?,
                    "[",
                    "]"
                ),
                match (&v.value_type, v.columns.is_empty()) {
                    (Some(value), true) => ty(value)?,
                    (None, false) => format!(
                        "{{{}}}",
                        v.columns
                            .iter()
                            .map(|c| Ok(format!("{}: {}", c.name, ty(&c.r#type)?)))
                            .collect::<Result<Vec<_>, AuthoringError>>()?
                            .join(", ")
                    ),
                    _ => return Err(bad("a table declares a value type exactly when it has no columns")),
                },
                v.missing_policy.as_str(),
                v.default_value
                    .as_ref()
                    .map_or(String::new(), |d| format!(" {d}"))
            ),
            Selected::Dataset(v) => format!(
                "dataset {n}: {} source {} {{ {} }}",
                v.table,
                quoted(&v.source),
                v.rows
                    .iter()
                    .map(|r| format!("[{}] = [{}];", r.keys.join(", "), r.values.join(", ")))
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
            Selected::Annotation(v) => {
                use pse_model::generated::enums::ModelingAnnotationKind as A;
                let typed = (
                    v.objective.is_some(),
                    v.extrapolation.is_some(),
                    v.scheme.is_some(),
                    v.connectivity.is_some(),
                );
                let expected = (
                    v.kind == A::Objective,
                    v.kind == A::Valid,
                    v.kind == A::Scale,
                    v.kind == A::Connectivity,
                );
                if typed != expected {
                    return Err(bad("an annotation carries exactly the typed members of its kind"));
                }
                let maximum = |m: Option<i64>| m.map_or_else(|| "many".to_owned(), |m| m.to_string());
                let arguments = match (v.kind, v.arguments.as_slice()) {
                    // ADR-0111: the sense, then the declared members in one canonical order.
                    (A::Objective, []) => {
                        let o = v.objective.as_ref().ok_or_else(|| bad("objective members"))?;
                        let mut members = vec![o.sense.as_str().to_owned()];
                        members.extend(o.priority.map(|p| format!("priority = {p}")));
                        for (name, value) in [
                            ("weight", &o.weight),
                            ("normalization", &o.normalization),
                            ("absolute_tolerance", &o.absolute_tolerance),
                            ("relative_tolerance", &o.relative_tolerance),
                        ] {
                            members.extend(value.as_ref().map(|v| format!("{name} = {v}")));
                        }
                        members.join(", ")
                    }
                    (A::Valid, [lower, upper]) => format!(
                        "{lower}, {upper}, {}",
                        v.extrapolation.map_or("", |p| p.as_str())
                    ),
                    (A::Scale, []) => v.scheme.map_or("", |s| s.as_str()).to_owned(),
                    (A::Connectivity, []) => {
                        let c = v.connectivity.as_ref().ok_or_else(|| bad("connectivity maxima"))?;
                        format!("{}, {}", maximum(c.incoming), maximum(c.outgoing))
                    }
                    (A::Start | A::Nominal | A::Bounds | A::Report | A::Check, arguments) => {
                        arguments.join(", ")
                    }
                    _ => return Err(bad("an annotation's arguments disagree with its kind")),
                };
                format!(
                    "annotation {} {}({arguments});",
                    v.kind.as_str(),
                    v.target,
                )
            }
            Selected::Expectation(v) => format!(
                "expect {} == {} tolerance {}{};",
                v.actual, v.expected, v.tolerance,
                v.relative_tolerance.as_ref().map_or_else(String::new, |v| format!(" relative {v}"))
            ),
        };
        out.push_str(&text);
        out.push('\n');
        if block {
            print_block(Some(row.declaration_id), children, visited, depth + 1, out)?;
            out.push_str(&pad);
            out.push_str("}\n");
        } else if children.contains_key(&Some(row.declaration_id)) {
            return Err(bad("non-scope declaration owns children"));
        }
    }
    Ok(())
}
