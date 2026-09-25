// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Compile-time specialization delegates all mathematical typing to the compiler.
use super::{CompositionDeclarations, invalid, port_id, symbol_id, unsupported};
use crate::workflow::{ModelDeclaration, PhysicalContext, ProviderBinding, WorkflowError};
use pse_authoring::dsl::{self, Expr, ExprKind, Path, Predicate, PredicateKind};
use pse_ids::{SemanticId, named_id};
use pse_kernels::Port;
use pse_relations::generated::{
    authored::{computation_models as m, instances, template_symbols},
    enums::*,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, atomic::AtomicBool},
};

/// Inspectable source-to-scalar binding.
#[derive(Clone, Debug)]
pub struct Binding {
    /// Owning instantiated template.
    pub instance: SemanticId,
    /// Original symbol declaration.
    pub declaration: SemanticId,
    /// Ordered finite index members.
    pub members: Vec<SemanticId>,
    /// Derived scalar identity.
    pub scalar: SemanticId,
}
#[derive(Default)]
pub(crate) struct Lowered {
    pub flows: BTreeMap<SemanticId, pse_structural::flowsheet::Declaration>,
    pub bindings: Vec<Binding>,
    pub balances: Vec<crate::workflow::BalanceDeclaration>,
    pub numerics: Vec<pse_model::numerics::NumericalRequirement>,
}
#[derive(Clone, Debug, PartialEq)]
enum Value {
    Bool(bool),
    Number(f64),
    Text(String),
}
#[derive(Default)]
struct Context {
    constants: BTreeMap<String, Value>,
    domains: BTreeMap<String, Vec<(SemanticId, String)>>,
    slots: BTreeMap<(String, Vec<SemanticId>), usize>,
    formals: Vec<m::AuthoredComputationModelsFieldDefinitionsItemFormalsItem>,
    ports: Vec<Port>,
}

pub(crate) fn lower(
    source: &CompositionDeclarations,
    model: &mut ModelDeclaration,
    physical: &PhysicalContext,
    providers: &BTreeMap<String, ProviderBinding>,
    registry: &pse_schema::Registry,
) -> Result<Lowered, WorkflowError> {
    let Some(root) = source.roots.first() else {
        if let Some(spec) = source.case_specs.first() {
            return Err(unsupported(
                [spec.case_id, spec.spec_id],
                "template case specifications require a selected composition",
            ));
        }
        return Ok(Lowered::default());
    };
    pse_math::initialize().map_err(crate::workflow::math)?;
    let mut lowered = Lowered::default();
    let mut used_specs = BTreeSet::new();
    let mut contexts = BTreeMap::new();
    for instance in &source.instances {
        if let Some(template) = source
            .templates
            .iter()
            .find(|t| t.template_id == instance.template_id)
        {
            if template.default_scaler_template_id.is_some()
                || template.default_initializer_template_id.is_some()
            {
                return Err(unsupported(
                    [instance.instance_id, template.template_id],
                    "selected default scaling/initialization strategy requires an admitted numerical policy",
                ));
            }
            if matches!(
                template.kind,
                TemplateKind::Scaler
                    | TemplateKind::Initializer
                    | TemplateKind::ConnectionRule
                    | TemplateKind::Law
            ) {
                return Err(unsupported(
                    [instance.instance_id, template.template_id],
                    "selected template kind is not a process instance",
                ));
            }
        }
        if !source
            .templates
            .iter()
            .any(|t| t.template_id == instance.template_id)
        {
            return Err(invalid(
                [instance.instance_id, instance.template_id],
                "selected template is missing",
            ));
        }
        if instance.property_package_id.is_some() || instance.reaction_package_id.is_some() {
            return Err(unsupported(
                [instance.instance_id],
                "package binding requires an admitted native material/provider declaration",
            ));
        }
        let mut context = context(source, instance, registry)?;
        for symbol in source
            .symbols
            .iter()
            .filter(|s| s.template_id == instance.template_id)
        {
            if !active(source, &context, symbol.guard_id, symbol.symbol_decl_id)? {
                continue;
            }
            if !matches!(symbol.role, SymbolRole::Variable | SymbolRole::Parameter)
                || symbol.reference_to.is_some()
                || symbol.wrt_domain.is_some()
            {
                return Err(unsupported(
                    [instance.instance_id, symbol.symbol_decl_id],
                    "selected symbol role requires a declared expression or dynamic binding",
                ));
            }
            let quantity = physical
                .quantities
                .quantity_type(symbol.quantity_type_id.into())
                .map_err(crate::workflow::math)?;
            if quantity.key.shape.len() != symbol.indexed_by.len() {
                return Err(invalid(
                    [symbol.symbol_decl_id],
                    "symbol axes differ from its physical shape",
                ));
            }
            for (axis, kind) in symbol.indexed_by.iter().zip(&quantity.key.shape) {
                let domain = source
                    .template_domains
                    .iter()
                    .find(|d| d.template_id == instance.template_id && d.name == *axis)
                    .ok_or_else(|| invalid([symbol.symbol_decl_id], "undeclared symbol axis"))?;
                if domain.kind.as_str() != kind.as_str() {
                    return Err(invalid(
                        [symbol.symbol_decl_id],
                        "symbol domain kind differs from quantity shape",
                    ));
                }
            }
            let mut scalar_key = quantity.key.clone();
            scalar_key.shape.clear();
            let scalar = physical
                .quantities
                .quantity_types()
                .find(|q| q.key == scalar_key)
                .ok_or_else(|| {
                    invalid(
                        [symbol.symbol_decl_id],
                        "indexed quantity has no declared scalar member type",
                    )
                })?;
            for members in tuples(&context, &symbol.indexed_by, symbol.symbol_decl_id)? {
                let id = symbol_id(instance.instance_id, symbol.symbol_decl_id, &members);
                let slot = context.ports.len();
                if context
                    .slots
                    .insert((symbol.name.clone(), members.clone()), slot)
                    .is_some()
                {
                    return Err(invalid(
                        [symbol.symbol_decl_id],
                        "duplicate symbol name/member",
                    ));
                }
                let port = Port {
                    id,
                    quantity: scalar.id,
                    unit: scalar.canonical_unit,
                };
                context.formals.push(
                    m::AuthoredComputationModelsFieldDefinitionsItemFormalsItem {
                        path: format!("s{slot}"),
                        quantity_id: scalar.id.as_id(),
                    },
                );
                context.ports.push(port.clone());
                lowered.bindings.push(Binding {
                    instance: instance.instance_id,
                    declaration: symbol.symbol_decl_id,
                    members: members.clone(),
                    scalar: id,
                });
                for case in &mut model.cases {
                    add_symbol(
                        case,
                        symbol,
                        &port,
                        source,
                        instance.instance_id,
                        &members,
                        physical,
                        &mut used_specs,
                        model.model_id,
                        &mut lowered.numerics,
                    )?;
                }
            }
        }
        contexts.insert(instance.instance_id, context);
    }
    for instance in &source.instances {
        let context = &contexts[&instance.instance_id];
        validate_children(source, instance, context)?;
        let mut sources = vec![];
        let mut senses = vec![];
        let mut rows = vec![];
        for equation in source
            .equations
            .iter()
            .filter(|e| e.template_id == instance.template_id)
        {
            if !active(
                source,
                context,
                equation.guard_id,
                equation.equation_decl_id,
            )? {
                continue;
            }
            for members in tuples(context, &equation.indexed_by, equation.equation_decl_id)? {
                let constants = index_constants(context, &equation.indexed_by, &members)?;
                if let Some(filter) = &equation.filter {
                    let predicate = dsl::parse_predicate(filter)
                        .map_err(|e| invalid([equation.equation_decl_id], e.to_string()))?;
                    if !predicate_value(&predicate, context, &constants, equation.equation_decl_id)?
                    {
                        continue;
                    }
                }
                let parsed = dsl::parse_equation(&equation.expression)
                    .map_err(|e| invalid([equation.equation_decl_id], e.to_string()))?;
                let (expression, sense) =
                    equation_body(parsed, context, &constants, equation.equation_decl_id)?;
                let expected = match equation.sense {
                    Sense::Eq => dsl::EquationSense::Eq,
                    Sense::Le => dsl::EquationSense::Le,
                    Sense::Ge => dsl::EquationSense::Ge,
                    _ => {
                        return Err(unsupported(
                            [equation.equation_decl_id],
                            "selected equation sense",
                        ));
                    }
                };
                if sense != expected {
                    return Err(invalid(
                        [equation.equation_decl_id],
                        "equation text and declared sense disagree",
                    ));
                }
                sources.push(dsl::render_expr(&expression));
                senses.push(sense);
                rows.push(symbol_id(
                    instance.instance_id,
                    equation.equation_decl_id,
                    &members,
                ));
            }
        }
        if !sources.is_empty() {
            add_body(
                model, instance, context, sources, senses, rows, physical, providers,
            )?;
        }
        lower_laws(
            source,
            model,
            instance,
            context,
            physical,
            providers,
            registry,
            &mut lowered.balances,
        )?;
    }
    for spec in &source.case_specs {
        if !used_specs.contains(&spec.spec_id) {
            return Err(invalid(
                [spec.case_id, spec.spec_id],
                "case target has no active scalar in the selected composition",
            ));
        }
    }
    let graph = connections(source, model, physical, &contexts)?;
    // Pure graph admission validates quantity conversion, ownership and target multiplicity.
    pse_structural::flowsheet::FlowGraph::admit(
        graph.clone(),
        &physical.quantities,
        pse_structural::projection::GraphLimits {
            nodes: 4096,
            edges: 16384,
        },
    )
    .map_err(|e| invalid([model.model_id], e.to_string()))?;
    lowered.flows.insert(root.root_instance_id, graph);
    Ok(lowered)
}

fn context(
    source: &CompositionDeclarations,
    instance: &instances::Row,
    registry: &pse_schema::Registry,
) -> Result<Context, WorkflowError> {
    let mut c = Context::default();
    let id = instance.instance_id;
    let mut provided = BTreeMap::new();
    for p in &instance.param_values {
        if provided.insert(p.name.clone(), p.value.clone()).is_some() {
            return Err(invalid([id], "duplicate parameter binding"));
        }
    }
    for p in source
        .parameters
        .iter()
        .filter(|p| p.template_id == instance.template_id)
    {
        let value = provided.remove(&p.name).or_else(|| p.default.clone());
        let Some(value) = value else {
            if p.required {
                return Err(invalid([id], format!("required parameter {}", p.name)));
            }
            continue;
        };
        if p.domain_spec.is_some() {
            return Err(unsupported(
                [id],
                format!("parameter {} domain constraint", p.name),
            ));
        }
        let logical = registry
            .logical_types()
            .iter()
            .find(|t| t.id == p.logical_type_id)
            .ok_or_else(|| invalid([id, p.logical_type_id], "parameter logical type is missing"))?;
        let ty: DataType = serde_json::from_str(&logical.arrow_storage)
            .map_err(|e| invalid([id], e.to_string()))?;
        use datafusion::arrow::datatypes::DataType;
        let value = match ty {
            DataType::Boolean => Value::Bool(
                value
                    .parse()
                    .map_err(|_| invalid([id], "parameter requires a Boolean"))?,
            ),
            DataType::Float64 => Value::Number(
                value
                    .parse::<f64>()
                    .ok()
                    .filter(|v| v.is_finite())
                    .ok_or_else(|| invalid([id], "parameter requires a finite real"))?,
            ),
            DataType::Int16 | DataType::Int32 | DataType::Int64 => {
                let number = value
                    .parse::<i64>()
                    .map_err(|_| invalid([id], "parameter requires an integer"))?;
                if (ty == DataType::Int16 && i16::try_from(number).is_err())
                    || (ty == DataType::Int32 && i32::try_from(number).is_err())
                    || number.unsigned_abs() > (1u64 << 53)
                {
                    return Err(invalid(
                        [id],
                        "integer parameter exceeds its exact declared representation",
                    ));
                }
                Value::Number(number as f64)
            }
            DataType::Utf8 | DataType::FixedSizeBinary(16) => Value::Text(value),
            _ => {
                return Err(unsupported(
                    [id, p.logical_type_id],
                    "selected structured parameter type",
                ));
            }
        };
        if let Some(enum_id) = p.enum_id {
            let domain = registry
                .enums()
                .iter()
                .find(|e| e.id == enum_id)
                .ok_or_else(|| invalid([id, enum_id], "parameter enum is missing"))?;
            if !matches!(&value, Value::Text(v) if domain.members.iter().any(|m| m.name == v)) {
                return Err(invalid([id], "parameter is outside its enum domain"));
            }
        }
        c.constants.insert(p.name.clone(), value);
    }
    if !provided.is_empty() {
        return Err(invalid([id], "unknown template parameter"));
    }
    let mut provided = BTreeMap::new();
    for p in &instance.feature_values {
        if provided.insert(p.name.clone(), p.value.clone()).is_some() {
            return Err(invalid([id], "duplicate feature binding"));
        }
    }
    for p in source
        .features
        .iter()
        .filter(|p| p.template_id == instance.template_id)
    {
        if p.inherit_from.is_some() {
            return Err(unsupported(
                [id],
                "feature inheritance requires an explicit resolved binding",
            ));
        }
        let value = provided
            .remove(&p.name)
            .or_else(|| p.default.clone())
            .ok_or_else(|| invalid([id], format!("unresolved feature {}", p.name)))?;
        let value = match p.kind {
            FeatureKind::Bool => Value::Bool(
                value
                    .parse()
                    .map_err(|_| invalid([id], "feature requires a Boolean"))?,
            ),
            FeatureKind::Enum | FeatureKind::Choice => {
                let domain = p
                    .enum_id
                    .and_then(|id| registry.enums().iter().find(|e| e.id == id))
                    .ok_or_else(|| invalid([id], "feature enum is missing"))?;
                if !domain.members.iter().any(|m| m.name == value) {
                    return Err(invalid([id], "feature is outside its enum domain"));
                }
                Value::Text(value)
            }
        };
        if c.constants.insert(p.name.clone(), value).is_some() {
            return Err(invalid([id], "feature and parameter names conflict"));
        }
    }
    if !provided.is_empty() {
        return Err(invalid([id], "unknown template feature"));
    }
    for rule in source
        .feature_rules
        .iter()
        .filter(|r| r.template_id == instance.template_id)
    {
        let a = dsl::parse_predicate(&rule.antecedent).map_err(|e| invalid([id], e.to_string()))?;
        let b = dsl::parse_predicate(&rule.consequent).map_err(|e| invalid([id], e.to_string()))?;
        let a = predicate_value(&a, &c, &BTreeMap::new(), id)?;
        let b = predicate_value(&b, &c, &BTreeMap::new(), id)?;
        if a && match rule.rule {
            FeatureRuleKind::Excludes => b,
            _ => !b,
        } {
            return Err(invalid([id], "feature rule is unsatisfied"));
        }
    }
    for domain in source
        .template_domains
        .iter()
        .filter(|d| d.template_id == instance.template_id)
    {
        if domain.continuous || domain.bounds.is_some() || domain.members_from.is_some() {
            return Err(unsupported(
                [id],
                format!("domain {} requires an explicit finite binding", domain.name),
            ));
        }
        let local = source
            .instance_domains
            .iter()
            .find(|d| d.instance_id == id && d.domain_name == domain.name)
            .map(|d| d.domain_id);
        let declared = source
            .template_domain_bindings
            .iter()
            .find(|d| d.template_id == instance.template_id && d.name == domain.name);
        let inherited = declared
            .map(|d| match d.source.kind {
                DomainBindingSource::Domain => d
                    .source
                    .domain
                    .as_ref()
                    .map(|d| d.domain_id)
                    .ok_or_else(|| invalid([id], "missing domain binding arm")),
                DomainBindingSource::Parameter => {
                    let p = d
                        .source
                        .parameter
                        .as_ref()
                        .ok_or_else(|| invalid([id], "missing parameter domain arm"))?;
                    if let Some(Value::Text(v)) = c.constants.get(&p.name) {
                        SemanticId::parse_hex(v)
                            .map_err(|_| invalid([id], "domain parameter is not a semantic ID"))
                    } else {
                        Err(invalid([id], "domain parameter is missing"))
                    }
                }
                DomainBindingSource::Species
                | DomainBindingSource::Phase
                | DomainBindingSource::PhaseSpecies
                | DomainBindingSource::Element => Err(unsupported(
                    [id],
                    "material-derived domain requires an admitted material-system binding",
                )),
            })
            .transpose()?;
        let domain_id = local
            .or(inherited)
            .ok_or_else(|| invalid([id], format!("unbound domain {}", domain.name)))?;
        let actual = source
            .domains
            .iter()
            .find(|d| d.domain_id == domain_id)
            .ok_or_else(|| invalid([id, domain_id], "domain is missing"))?;
        if actual.parent_domain_id.is_some() {
            return Err(unsupported(
                [id, domain_id],
                "parent-dependent domain requires explicit ragged tuple bindings",
            ));
        }
        if actual.continuous || actual.kind != domain.kind || actual.unit_id != domain.unit_id {
            return Err(invalid(
                [id, domain_id],
                "finite domain kind/unit contract differs from template",
            ));
        }
        let members: Vec<_> = source
            .members
            .iter()
            .filter(|m| m.domain_id == domain_id)
            .map(|m| (m.member_id, m.label.clone()))
            .collect();
        if members.len() > 4096
            || members.iter().map(|m| m.0).collect::<BTreeSet<_>>().len() != members.len()
            || members.iter().map(|m| &m.1).collect::<BTreeSet<_>>().len() != members.len()
        {
            return Err(invalid(
                [domain_id],
                "domain member identities/labels are duplicate or exceed allowance",
            ));
        }
        if c.domains.insert(domain.name.clone(), members).is_some() {
            return Err(invalid([id], "duplicate domain name"));
        }
    }
    Ok(c)
}

fn tuples(
    c: &Context,
    axes: &[String],
    id: SemanticId,
) -> Result<Vec<Vec<SemanticId>>, WorkflowError> {
    let mut tuples = vec![vec![]];
    for axis in axes {
        let members = c
            .domains
            .get(axis)
            .ok_or_else(|| invalid([id], format!("unknown index domain {axis}")))?;
        if tuples
            .len()
            .checked_mul(members.len())
            .is_none_or(|n| n > 4096)
        {
            return Err(unsupported(
                [id],
                "finite specialization exceeds 4096 members",
            ));
        }
        tuples = tuples
            .into_iter()
            .flat_map(|t| {
                members.iter().map(move |(id, _)| {
                    let mut t = t.clone();
                    t.push(*id);
                    t
                })
            })
            .collect();
    }
    Ok(tuples)
}
fn index_constants(
    c: &Context,
    axes: &[String],
    members: &[SemanticId],
) -> Result<BTreeMap<String, Value>, WorkflowError> {
    axes.iter()
        .zip(members)
        .map(|(axis, member)| {
            let label = c.domains[axis]
                .iter()
                .find(|m| m.0 == *member)
                .ok_or_else(|| invalid([*member], "missing index member"))?
                .1
                .clone();
            Ok((axis.clone(), Value::Text(label)))
        })
        .collect()
}
fn value(
    e: &Expr,
    c: &Context,
    local: &BTreeMap<String, Value>,
    id: SemanticId,
) -> Result<Value, WorkflowError> {
    match &e.kind {
        ExprKind::Number(n) if n.unit.is_none() => Ok(Value::Number(n.value)),
        ExprKind::Path(p) if p.segments.len() == 1 && p.segments[0].indices.is_empty() => {
            let name = &p.segments[0].name;
            match name.as_str() {
                "true" => Ok(Value::Bool(true)),
                "false" => Ok(Value::Bool(false)),
                _ => local
                    .get(name)
                    .or_else(|| c.constants.get(name))
                    .cloned()
                    .ok_or_else(|| invalid([id], format!("unresolved compile-time name {name}"))),
            }
        }
        ExprKind::Neg(e) => match value(e, c, local, id)? {
            Value::Number(v) => Ok(Value::Number(-v)),
            _ => Err(invalid([id], "negated nonnumeric parameter")),
        },
        _ => Err(unsupported(
            [id],
            "formulation guard must use resolved compile-time values",
        )),
    }
}
fn predicate_value(
    p: &Predicate,
    c: &Context,
    local: &BTreeMap<String, Value>,
    id: SemanticId,
) -> Result<bool, WorkflowError> {
    use dsl::CompareOp;
    Ok(match &p.kind {
        PredicateKind::Bool(v) => *v,
        PredicateKind::Null => return Err(invalid([id], "unresolved null formulation guard")),
        PredicateKind::Atom(e) => match value(e, c, local, id)? {
            Value::Bool(v) => v,
            _ => return Err(invalid([id], "guard needs a Boolean")),
        },
        PredicateKind::Not(p) => !predicate_value(p, c, local, id)?,
        PredicateKind::And(a, b) => {
            predicate_value(a, c, local, id)? && predicate_value(b, c, local, id)?
        }
        PredicateKind::Or(a, b) => {
            predicate_value(a, c, local, id)? || predicate_value(b, c, local, id)?
        }
        PredicateKind::Compare { op, lhs, rhs } => {
            let a = value(lhs, c, local, id)?;
            let b = value(rhs, c, local, id)?;
            match op {
                CompareOp::Eq => a == b,
                CompareOp::NotEq => a != b,
                _ => {
                    let (Value::Number(a), Value::Number(b)) = (a, b) else {
                        return Err(invalid([id], "ordered guard comparison requires numbers"));
                    };
                    match op {
                        CompareOp::Lt => a < b,
                        CompareOp::Le => a <= b,
                        CompareOp::Gt => a > b,
                        CompareOp::Ge => a >= b,
                        CompareOp::Eq => a == b,
                        CompareOp::NotEq => a != b,
                    }
                }
            }
        }
        PredicateKind::In { expr, domain } => {
            let Value::Text(label) = value(expr, c, local, id)? else {
                return Err(invalid([id], "domain membership needs a member label"));
            };
            c.domains
                .get(&dsl::render_path(domain))
                .ok_or_else(|| invalid([id], "unknown membership domain"))?
                .iter()
                .any(|(_, name)| *name == label)
        }
    })
}
fn active(
    source: &CompositionDeclarations,
    c: &Context,
    guard: Option<SemanticId>,
    id: SemanticId,
) -> Result<bool, WorkflowError> {
    let Some(guard) = guard else {
        return Ok(true);
    };
    let guard = source
        .guards
        .iter()
        .find(|g| g.guard_id == guard)
        .ok_or_else(|| invalid([id, guard], "missing formulation guard"))?;
    let predicate =
        dsl::parse_predicate(&guard.predicate).map_err(|e| invalid([id], e.to_string()))?;
    predicate_value(&predicate, c, &BTreeMap::new(), id)
}
fn plain(name: String) -> Expr {
    Expr {
        kind: ExprKind::Path(Path {
            segments: vec![dsl::PathSegment {
                name,
                indices: vec![],
            }],
        }),
        span: Default::default(),
    }
}
fn specialize(
    mut e: Expr,
    c: &Context,
    local: &BTreeMap<String, Value>,
    id: SemanticId,
) -> Result<Expr, WorkflowError> {
    e.kind = match e.kind {
        ExprKind::Path(p) if p.segments.len() == 1 => {
            let part = &p.segments[0];
            if let Some(Value::Number(v)) = local
                .get(&part.name)
                .or_else(|| c.constants.get(&part.name))
            {
                if !part.indices.is_empty() {
                    return Err(invalid([id], "indexed scalar parameter"));
                }
                ExprKind::Number(dsl::Number {
                    value: *v,
                    unit: None,
                })
            } else {
                let indices = part
                    .indices
                    .iter()
                    .map(|i| value(i, c, local, id))
                    .collect::<Result<Vec<_>, _>>()?;
                let slot = c.slots.iter().find_map(|((name,members),slot)| {
                    if name != &part.name || indices.len() != members.len() { return None; }
                    let matches = indices.iter().zip(members).all(|(v,id)| matches!(v, Value::Text(label) if c.domains.values().flatten().any(|(member,name)| member == id && name == label)));
                    matches.then_some(*slot)
                });
                if let Some(slot) = slot {
                    plain(format!("s{slot}")).kind
                } else {
                    ExprKind::Path(p)
                }
            }
        }
        ExprKind::Neg(v) => ExprKind::Neg(Box::new(specialize(*v, c, local, id)?)),
        ExprKind::Binary { op, lhs, rhs } => ExprKind::Binary {
            op,
            lhs: Box::new(specialize(*lhs, c, local, id)?),
            rhs: Box::new(specialize(*rhs, c, local, id)?),
        },
        ExprKind::Call { function, args } => ExprKind::Call {
            function,
            args: args
                .into_iter()
                .map(|v| specialize(v, c, local, id))
                .collect::<Result<_, _>>()?,
        },
        ExprKind::Kernel { name, args } => ExprKind::Kernel {
            name,
            args: args
                .into_iter()
                .map(|v| specialize(v, c, local, id))
                .collect::<Result<_, _>>()?,
        },
        ExprKind::Conditional {
            guard,
            then,
            otherwise,
        } => {
            return specialize(
                if predicate_value(&guard, c, local, id)? {
                    *then
                } else {
                    *otherwise
                },
                c,
                local,
                id,
            );
        }
        ExprKind::Reduce { kind, binder, body } => {
            if kind == dsl::ReduceKind::Integral {
                return Err(unsupported(
                    [id],
                    "continuous integral needs dynamic formulation",
                ));
            }
            let domain = c
                .domains
                .get(&dsl::render_path(&binder.domain))
                .ok_or_else(|| invalid([id], "unknown reduction domain"))?;
            let mut terms = Vec::new();
            for (_, label) in domain {
                let mut local = local.clone();
                local.insert(binder.var.clone(), Value::Text(label.clone()));
                if let Some(filter) = &binder.filter
                    && !predicate_value(filter, c, &local, id)?
                {
                    continue;
                }
                terms.push(specialize(*body.clone(), c, &local, id)?);
            }
            let mut terms = terms.into_iter();
            let mut out = terms
                .next()
                .ok_or_else(|| invalid([id], "empty reduction needs an explicit typed identity"))?;
            for term in terms {
                out = Expr {
                    kind: ExprKind::Binary {
                        op: if kind == dsl::ReduceKind::Sum {
                            dsl::BinaryOp::Add
                        } else {
                            dsl::BinaryOp::Mul
                        },
                        lhs: Box::new(out),
                        rhs: Box::new(term),
                    },
                    span: e.span,
                };
            }
            return Ok(out);
        }
        ExprKind::Derivative { .. } => {
            return Err(unsupported(
                [id],
                "derivative expression needs a dynamic case binding",
            ));
        }
        ExprKind::Let { bindings, body } => ExprKind::Let {
            bindings: bindings
                .into_iter()
                .map(|(n, v)| Ok((n, specialize(v, c, local, id)?)))
                .collect::<Result<_, WorkflowError>>()?,
            body: Box::new(specialize(*body, c, local, id)?),
        },
        kind => kind,
    };
    Ok(e)
}
fn equation_body(
    e: dsl::Equation,
    c: &Context,
    local: &BTreeMap<String, Value>,
    id: SemanticId,
) -> Result<(Expr, dsl::EquationSense), WorkflowError> {
    match e.kind {
        dsl::EquationKind::Relation { lhs, sense, rhs } => Ok((
            Expr {
                kind: ExprKind::Binary {
                    op: dsl::BinaryOp::Sub,
                    lhs: Box::new(specialize(lhs, c, local, id)?),
                    rhs: Box::new(specialize(rhs, c, local, id)?),
                },
                span: e.span,
            },
            sense,
        )),
        dsl::EquationKind::Conditional {
            guard,
            then,
            otherwise,
        } => equation_body(
            if predicate_value(&guard, c, local, id)? {
                *then
            } else {
                *otherwise
            },
            c,
            local,
            id,
        ),
    }
}

fn bound(
    value: &Option<pse_relations::generated::extension_values::Bound>,
    id: SemanticId,
) -> Result<Option<f64>, WorkflowError> {
    match value {
        None => Ok(None),
        Some(v) => match (v.kind, v.value) {
            (BoundKind::Finite, Some(v)) if v.is_finite() => Ok(Some(v)),
            (BoundKind::Unbounded, None) => Ok(None),
            _ => Err(invalid([id], "invalid bound alternative")),
        },
    }
}
fn wire_port(port: &Port) -> m::AuthoredComputationModelsFieldCasesItemParametersItem {
    m::AuthoredComputationModelsFieldCasesItemParametersItem {
        symbol_id: port.id,
        quantity_id: port.quantity.as_id(),
        unit_id: port.unit.as_id(),
    }
}
fn targets_symbol(
    spec: &pse_relations::generated::authored::case_specs::Row,
    instance: SemanticId,
    symbol: &template_symbols::Row,
    members: &[SemanticId],
    source: &CompositionDeclarations,
) -> Result<bool, WorkflowError> {
    use pse_authoring::targets::IndexSelector;
    let at = spec
        .source_span
        .clone()
        .try_into()
        .map_err(|e: pse_authoring::AuthoringError| invalid([spec.spec_id], e.to_string()))?;
    let target = pse_authoring::targets::parse(&spec.target, at)
        .map_err(|e| invalid([spec.spec_id], e.to_string()))?;
    let mut ancestry = vec![];
    let mut current = Some(instance);
    let mut seen = BTreeSet::new();
    while let Some(id) = current {
        if !seen.insert(id) {
            return Err(invalid([id], "cyclic instance parent chain"));
        }
        let Some(i) = source.instances.iter().find(|i| i.instance_id == id) else {
            break;
        };
        ancestry.push(i.name.clone());
        current = i.parent_instance_id;
    }
    ancestry.reverse();
    if !target.instance_wildcard {
        ancestry.push(symbol.name.clone());
    }
    if target.names != ancestry {
        return Ok(false);
    }
    if target.indices.is_empty() {
        return Ok(true);
    }
    if target.indices.len() != members.len() {
        return Err(invalid(
            [spec.spec_id],
            "case target index arity differs from symbol",
        ));
    }
    Ok(target.indices.iter().zip(members).all(|(index, id)| {
        let Some(member) = source.members.iter().find(|m| m.member_id == *id) else {
            return false;
        };
        match index {
            IndexSelector::Wildcard => true,
            IndexSelector::Label(label) => *label == member.label,
            IndexSelector::Value(value) => {
                *value == member.label
                    || SemanticId::parse_hex(value).is_ok_and(|v| v == *id)
                    || value.parse::<f64>().is_ok_and(|v| {
                        member.coordinate.is_some_and(|c| {
                            pse_ids::canonical_f64_bits(c) == pse_ids::canonical_f64_bits(v)
                        })
                    })
            }
        }
    }))
}

#[expect(
    clippy::too_many_arguments,
    reason = "explicit authored symbol, resolved binding and selected case inputs"
)]
fn add_symbol(
    case: &mut m::AuthoredComputationModelsFieldCasesItem,
    symbol: &template_symbols::Row,
    port: &Port,
    source: &CompositionDeclarations,
    instance: SemanticId,
    members: &[SemanticId],
    physical: &PhysicalContext,
    used_specs: &mut BTreeSet<SemanticId>,
    model_id: SemanticId,
    numerics: &mut Vec<pse_model::numerics::NumericalRequirement>,
) -> Result<(), WorkflowError> {
    let mut initial = symbol.default_initial;
    let mut lower = bound(&symbol.default_lower, symbol.symbol_decl_id)?;
    let mut upper = bound(&symbol.default_upper, symbol.symbol_decl_id)?;
    let mut fixed = symbol.role == SymbolRole::Parameter;
    let mut applicable = vec![];
    for spec in source
        .case_specs
        .iter()
        .filter(|s| s.case_id == case.case_id)
    {
        if targets_symbol(spec, instance, symbol, members, source)? {
            used_specs.insert(spec.spec_id);
            applicable.push(spec);
        }
    }
    applicable.sort_by_key(|s| s.priority);
    if applicable
        .windows(2)
        .any(|w| w[0].priority == w[1].priority)
    {
        return Err(invalid(
            [case.case_id, symbol.symbol_decl_id],
            "overlapping case specifications have equal priority",
        ));
    }
    for spec in applicable {
        if let Some(scale) = spec.scaling_factor {
            if !scale.is_finite() || scale <= 0.0 {
                return Err(invalid(
                    [spec.spec_id],
                    "case scaling must be finite and positive",
                ));
            }
            numerics.push(pse_model::numerics::NumericalRequirement {
                requirement_id: named_id(spec.spec_id, &format!("numerical.{}", port.id)),
                model_id,
                case_id: Some(case.case_id),
                target_id: port.id,
                target_kind: NumericalTarget::Variable,
                nominal: None,
                scaling_factor: Some(scale),
                absolute_tolerance: None,
                relative_tolerance: None,
                unit_id: spec.unit_id,
                coordinates: NumericalCoordinates::Physical,
                priority: spec.priority,
                required: true,
                provenance: format!("selected case specification {}", spec.spec_id),
            });
        }
        if let Some(treatment) = spec.treatment {
            match (symbol.role, treatment) {
                (SymbolRole::Variable, Treatment::Fixed) => fixed = true,
                (SymbolRole::Variable, Treatment::Free) => fixed = false,
                (SymbolRole::Parameter, Treatment::Parameter | Treatment::Fixed) => (),
                _ => {
                    return Err(invalid(
                        [spec.spec_id],
                        "case treatment changes the declared symbol role",
                    ));
                }
            }
        }
        let convert = |v: f64| -> Result<f64, WorkflowError> {
            let unit = spec.unit_id.map_or(port.unit, Into::into);
            let source = Port {
                unit,
                ..port.clone()
            };
            let conversion =
                pse_math::binding::SlotBinding::new(&source, port, &physical.quantities)
                    .map_err(crate::workflow::math)?;
            Ok(v * conversion.scale() + conversion.offset())
        };
        if let Some(v) = spec.value.or(spec.initial) {
            initial = Some(convert(v)?);
        }
        if spec.lower.is_some() {
            lower = bound(&spec.lower, spec.spec_id)?.map(convert).transpose()?;
        }
        if spec.upper.is_some() {
            upper = bound(&spec.upper, spec.spec_id)?.map(convert).transpose()?;
        }
    }
    let initial = initial.filter(|v| v.is_finite()).ok_or_else(|| {
        invalid(
            [case.case_id, symbol.symbol_decl_id],
            "selected scalar has no finite initial/parameter value",
        )
    })?;
    if symbol.role == SymbolRole::Parameter {
        case.parameters.push(wire_port(port));
    } else {
        case.variables
            .push(m::AuthoredComputationModelsFieldCasesItemVariablesItem {
                port: m::AuthoredComputationModelsFieldCasesItemVariablesItemPort {
                    symbol_id: port.id,
                    quantity_id: port.quantity.as_id(),
                    unit_id: port.unit.as_id(),
                },
                fixed,
                domain: NativeVariableDomain::Continuous,
                lower,
                upper,
            });
    }
    case.values
        .push(m::AuthoredComputationModelsFieldCasesItemValuesItem {
            symbol_id: port.id,
            value: initial,
        });
    Ok(())
}

#[expect(
    clippy::too_many_arguments,
    reason = "typed admission takes distinct immutable contracts and mutable output declarations"
)]
fn add_body(
    model: &mut ModelDeclaration,
    instance: &instances::Row,
    c: &Context,
    sources: Vec<String>,
    senses: Vec<dsl::EquationSense>,
    rows: Vec<SemanticId>,
    physical: &PhysicalContext,
    providers: &BTreeMap<String, ProviderBinding>,
) -> Result<Vec<pse_quantity::QuantityTypeId>, WorkflowError> {
    let id = named_id(instance.instance_id, "template-equations");
    let formals = c
        .formals
        .iter()
        .map(|f| pse_compiler::typed_math::Formal {
            path: f.path.clone(),
            quantity: f.quantity_id.into(),
        })
        .collect::<Vec<_>>();
    let calls = providers
        .iter()
        .map(|(name, p)| {
            (
                name.clone(),
                pse_compiler::typed_math::ProviderCall {
                    descriptor: p.registration.descriptor(),
                    output: p.output,
                },
            )
        })
        .collect();
    let units = physical
        .quantities
        .units()
        .map(|u| (u.symbol.clone(), u.id))
        .collect();
    let expressions = sources
        .iter()
        .map(|s| dsl::parse_expr(s).map_err(|e| invalid([id], e.to_string())))
        .collect::<Result<Vec<_>, _>>()?;
    let request = pse_compiler::typed_math::Request {
        definition: id,
        expressions: &expressions,
        formals: &formals,
        domains: &BTreeMap::new(),
        groups: &BTreeMap::new(),
        providers: &calls,
        units: &units,
        literals: &BTreeMap::new(),
        limits: Default::default(),
        physical: physical.key,
        structure: pse_ids::ContentHash::from_bytes([0; 32]),
    };
    let admitted = request
        .admit(
            &physical.quantities,
            physical.preconditions.as_ref(),
            &Arc::new(AtomicBool::new(false)),
        )
        .map_err(crate::workflow::math)?;
    model
        .definitions
        .push(m::AuthoredComputationModelsFieldDefinitionsItem {
            definition_id: id,
            sources,
            formals: c.formals.clone(),
            domains: vec![],
            groups: vec![],
            providers: providers.keys().cloned().collect(),
            units: units
                .into_iter()
                .map(|(spelling, unit_id)| {
                    m::AuthoredComputationModelsFieldDefinitionsItemUnitsItem {
                        spelling,
                        unit_id: unit_id.as_id(),
                    }
                })
                .collect(),
            literals: vec![],
        });
    for case in &mut model.cases {
        for ((row, quantity), sense) in rows.iter().zip(&admitted.quantities).zip(&senses) {
            case.rows
                .push(m::AuthoredComputationModelsFieldCasesItemRowsItem {
                    row_id: *row,
                    quantity_id: quantity.as_id(),
                    lower: matches!(sense, dsl::EquationSense::Eq | dsl::EquationSense::Ge)
                        .then_some(0.0),
                    upper: matches!(sense, dsl::EquationSense::Eq | dsl::EquationSense::Le)
                        .then_some(0.0),
                });
        }
        case.instances
            .push(m::AuthoredComputationModelsFieldCasesItemInstancesItem {
                instance_id: named_id(instance.instance_id, "equation-occurrence"),
                definition_id: id,
                slots: c
                    .ports
                    .iter()
                    .map(
                        |p| m::AuthoredComputationModelsFieldCasesItemInstancesItemSlotsItem {
                            source_id: p.id,
                            formal_quantity_id: p.quantity.as_id(),
                            formal_unit_id: p.unit.as_id(),
                        },
                    )
                    .collect(),
                contributions: rows
                    .iter()
                    .enumerate()
                    .map(|(output, id)| {
                        m::AuthoredComputationModelsFieldCasesItemInstancesItemContributionsItem {
                            output: output as i64,
                            row_id: Some(*id),
                            scale: 1.0,
                        }
                    })
                    .collect(),
            });
    }
    Ok(admitted.quantities.to_vec())
}

#[expect(
    clippy::too_many_arguments,
    reason = "conservation lowering shares the existing source, physical and provider contracts"
)]
fn lower_laws(
    source: &CompositionDeclarations,
    model: &mut ModelDeclaration,
    instance: &instances::Row,
    context: &Context,
    physical: &PhysicalContext,
    providers: &BTreeMap<String, ProviderBinding>,
    registry: &pse_schema::Registry,
    balances: &mut Vec<crate::workflow::BalanceDeclaration>,
) -> Result<(), WorkflowError> {
    use pse_relations::generated::authored::physical_balances as b;
    let mut consumed = BTreeSet::new();
    for law in source
        .laws
        .iter()
        .filter(|l| l.template_id == instance.template_id)
    {
        if !active(source, context, law.guard_id, law.law_instance_decl_id)? {
            continue;
        }
        let id = law.law_instance_decl_id;
        let contract = source
            .law_contracts
            .iter()
            .find(|l| l.law_instance_decl_id == id)
            .ok_or_else(|| invalid([id], "law quantity contract missing"))?;
        let binding = source
            .law_bindings
            .iter()
            .find(|l| l.law_template_id == law.law_template_id)
            .ok_or_else(|| invalid([id, law.law_template_id], "law expansion binding missing"))?;
        if !contract.indexed_by.is_empty()
            || contract.coordinates.member.is_some()
            || contract.coordinates.phase.is_some()
            || contract.default_balance.is_some()
            || binding.expansion != LawExpansion::Conservation
            || binding.subject_projection != LawSubjectProjection::Identity
            || binding.family != binding.source_family
            || !matches!(
                binding.subject_kind,
                ContributionSubjectKind::Total
                    | ContributionSubjectKind::Energy
                    | ContributionSubjectKind::Momentum
            )
        {
            return Err(unsupported(
                [id],
                "selected law requires an explicitly supported scalar conservation contract",
            ));
        }
        if binding.balance_enum_id != contract.balance_enum_id
            || !registry.enums().iter().any(|e| {
                e.id == binding.balance_enum_id
                    && e.members.iter().any(|m| m.name == binding.balance_member)
            })
        {
            return Err(invalid([id], "law balance enum contract"));
        }
        let tolerance = law
            .options
            .iter()
            .find(|o| o.key == "tolerance")
            .and_then(|o| o.value.parse::<f64>().ok())
            .filter(|v| v.is_finite() && *v > 0.0)
            .ok_or_else(|| {
                invalid(
                    [id],
                    "law requires an explicit positive canonical tolerance",
                )
            })?;
        if law.options.len() != 1 {
            return Err(unsupported([id], "unknown or duplicate law options"));
        }
        let mut terms = vec![];
        for contribution in source.contributions.iter().filter(|c| {
            c.template_id == instance.template_id
                && c.scope == law.scope
                && c.law_family == binding.source_family
        }) {
            let cid = contribution.contribution_decl_id;
            if !active(source, context, contribution.guard_id, cid)? {
                continue;
            }
            if !consumed.insert(cid) {
                return Err(invalid(
                    [id, cid],
                    "contribution has multiple conservation owners",
                ));
            }
            if contribution.orientation == Orientation::Accumulation {
                return Err(unsupported(
                    [cid],
                    "dynamic accumulation requires explicit conserved state binding",
                ));
            }
            let expr = dsl::parse_expr(&contribution.expression)
                .map_err(|e| invalid([cid], e.to_string()))?;
            let expression = specialize(expr, context, &BTreeMap::new(), cid)?;
            let mut occurrence = instance.clone();
            occurrence.instance_id = symbol_id(instance.instance_id, cid, &[]);
            let quantities = add_body(
                model,
                &occurrence,
                context,
                vec![dsl::render_expr(&expression)],
                vec![],
                vec![],
                physical,
                providers,
            )?;
            if quantities
                != [pse_quantity::QuantityTypeId::from_id(
                    contract.quantity_type_id,
                )]
            {
                return Err(invalid(
                    [cid, id],
                    "contribution quantity differs from law quantity",
                ));
            }
            terms.push(b::AuthoredPhysicalBalancesFieldTermsItem {
                multiplier: 1.0,
                source_id: occurrence.instance_id,
                role: match contribution.orientation {
                    Orientation::IntoScope => BalanceRole::Inlet,
                    Orientation::OutOfScope => BalanceRole::Outlet,
                    _ => BalanceRole::Generation,
                },
                transfer_id: None,
                mode: None,
                instance_id: named_id(occurrence.instance_id, "equation-occurrence"),
                output: 0,
            });
        }
        if terms.is_empty() {
            return Err(invalid([id], "selected law has no active contributions"));
        }
        for case in &model.cases {
            balances.push(b::Row {
                balance_id: symbol_id(instance.instance_id, id, &[case.case_id]),
                model_id: model.model_id,
                case_id: case.case_id,
                quantity_id: contract.quantity_type_id,
                accumulation: None,
                tolerance,
                integral_tolerance: None,
                provenance: format!(
                    "authored law {} in instance {}",
                    id.to_hex(),
                    instance.instance_id.to_hex()
                ),
                terms: terms.clone(),
                impulses: vec![],
            });
        }
    }
    for contribution in source
        .contributions
        .iter()
        .filter(|c| c.template_id == instance.template_id)
    {
        if active(
            source,
            context,
            contribution.guard_id,
            contribution.contribution_decl_id,
        )? && !consumed.contains(&contribution.contribution_decl_id)
        {
            return Err(unsupported(
                [instance.instance_id, contribution.contribution_decl_id],
                "active contribution has no selected conservation law",
            ));
        }
    }
    Ok(())
}

fn validate_children(
    source: &CompositionDeclarations,
    instance: &instances::Row,
    c: &Context,
) -> Result<(), WorkflowError> {
    for row in source
        .submodels
        .iter()
        .filter(|s| s.template_id == instance.template_id)
    {
        let enabled = active(source, c, row.guard_id, instance.instance_id)?;
        let children: Vec<_> = source
            .instances
            .iter()
            .filter(|i| i.parent_instance_id == Some(instance.instance_id) && i.name == row.name)
            .collect();
        if !enabled {
            if !children.is_empty() {
                return Err(invalid(
                    [instance.instance_id],
                    "disabled submodel has an active instance",
                ));
            }
            continue;
        }
        if row.multiplicity_domain.is_some() {
            return Err(unsupported(
                [instance.instance_id],
                "submodel multiplicity requires explicit scalar child declarations",
            ));
        }
        if children.len() != 1 {
            return Err(invalid(
                [instance.instance_id],
                format!("submodel {} needs one child binding", row.name),
            ));
        }
        let template = match (&row.child_template_id, &row.child_from_param) {
            (Some(id), None) => *id,
            (None, Some(name)) => match c.constants.get(name) {
                Some(Value::Text(v)) => SemanticId::parse_hex(v).map_err(|_| {
                    invalid(
                        [instance.instance_id],
                        "child template parameter is not an ID",
                    )
                })?,
                _ => return Err(invalid([instance.instance_id], "unresolved child template")),
            },
            _ => {
                return Err(invalid(
                    [instance.instance_id],
                    "submodel needs exactly one template selector",
                ));
            }
        };
        if children[0].template_id != template {
            return Err(invalid(
                [children[0].instance_id],
                "child template differs from declaration",
            ));
        }
        for binding in &row.bindings {
            let child = children[0]
                .param_values
                .iter()
                .find(|p| p.name == binding.child_param)
                .ok_or_else(|| {
                    invalid(
                        [children[0].instance_id],
                        "missing explicit child parameter binding",
                    )
                })?;
            if child.value != binding.value {
                return Err(invalid(
                    [children[0].instance_id],
                    "child binding differs from submodel declaration",
                ));
            }
        }
    }
    Ok(())
}

fn connections(
    source: &CompositionDeclarations,
    model: &mut ModelDeclaration,
    physical: &PhysicalContext,
    contexts: &BTreeMap<SemanticId, Context>,
) -> Result<pse_structural::flowsheet::Declaration, WorkflowError> {
    use pse_structural::flowsheet::{Connection, Decision, Declaration, Node, Policy};
    let mut graph = Declaration {
        nodes: vec![],
        connections: vec![],
        decisions: vec![],
    };
    let mut ports = BTreeMap::new();
    for i in &source.instances {
        let c = &contexts[&i.instance_id];
        let mut node = Node {
            id: i.instance_id,
            ports: vec![],
        };
        for p in source
            .ports
            .iter()
            .filter(|p| p.template_id == i.template_id)
        {
            if !active(source, c, p.guard_id, i.instance_id)? {
                continue;
            }
            let names: Vec<_> = source
                .port_members
                .iter()
                .filter(|m| m.template_id == i.template_id && m.symbol_group == p.bound_to)
                .map(|m| m.symbol_decl_id)
                .collect();
            let ordered_names: Vec<_> = if names.is_empty() {
                source
                    .symbols
                    .iter()
                    .filter(|s| s.template_id == i.template_id && s.name == p.bound_to)
                    .map(|s| s.name.as_str())
                    .collect()
            } else {
                names
                    .iter()
                    .map(|id| {
                        source
                            .symbols
                            .iter()
                            .find(|s| s.template_id == i.template_id && s.symbol_decl_id == *id)
                            .map(|s| s.name.as_str())
                            .ok_or_else(|| invalid([*id], "port member symbol missing"))
                    })
                    .collect::<Result<_, _>>()?
            };
            let mut members = vec![];
            for name in ordered_names {
                for ((slot_name, index), slot) in &c.slots {
                    if slot_name != name {
                        continue;
                    }
                    let actual = &c.ports[*slot];
                    let flow = Port {
                        id: symbol_id(port_id(i.instance_id, &p.name), actual.id, index),
                        ..actual.clone()
                    };
                    node.ports.push(flow.clone());
                    members.push((flow, actual.clone()));
                }
            }
            if members.is_empty() {
                return Err(invalid(
                    [i.instance_id],
                    format!("port {} has no active typed members", p.name),
                ));
            }
            ports.insert(port_id(i.instance_id, &p.name), (i.instance_id, p, members));
        }
        graph.nodes.push(node);
    }
    let mut outgoing = BTreeSet::new();
    let mut incoming = BTreeSet::new();
    for connection in &source.connections {
        let rule = source
            .connection_rules
            .iter()
            .find(|r| r.rule_template_id == connection.rule_template_id)
            .ok_or_else(|| {
                invalid(
                    [connection.connection_id, connection.rule_template_id],
                    "missing connection rule binding",
                )
            })?;
        if rule.expansion != ConnectionExpansion::Equality {
            return Err(unsupported(
                [connection.connection_id],
                "selected connection expansion",
            ));
        }
        if connection
            .tear_cost
            .is_some_and(|v| !v.is_finite() || v < 0.0)
        {
            return Err(invalid(
                [connection.connection_id],
                "tear cost must be finite and nonnegative",
            ));
        }
        let from = ports.get(&connection.from_port_id).ok_or_else(|| {
            invalid(
                [connection.connection_id, connection.from_port_id],
                "source port is outside selected composition",
            )
        })?;
        let to = ports.get(&connection.to_port_id).ok_or_else(|| {
            invalid(
                [connection.connection_id, connection.to_port_id],
                "destination port is outside selected composition",
            )
        })?;
        if from.1.direction != Direction::Outlet
            || to.1.direction != Direction::Inlet
            || from.1.kind != to.1.kind
            || from.2.len() != to.2.len()
        {
            return Err(invalid(
                [connection.connection_id],
                "connection direction/kind/member contract",
            ));
        }
        if !incoming.insert(connection.to_port_id) {
            return Err(invalid(
                [connection.connection_id, connection.to_port_id],
                "multiple assignments to one inlet",
            ));
        }
        let extensive = from.2.iter().try_fold(false, |found, (_, p)| {
            let q = physical
                .quantities
                .quantity_type(p.quantity)
                .map_err(crate::workflow::math)?;
            Ok::<_, WorkflowError>(
                found
                    || physical
                        .quantities
                        .kind(q.key.kind)
                        .map_err(crate::workflow::math)?
                        .extensive,
            )
        })?;
        if from.1.kind != PortKind::Signal && extensive && !outgoing.insert(connection.from_port_id)
        {
            return Err(invalid(
                [connection.connection_id, connection.from_port_id],
                "extensive-flow fan-out requires an explicit splitter conservation formulation",
            ));
        }
        let mut bindings = vec![];
        for ((a, source), (b, target)) in from.2.iter().zip(&to.2) {
            pse_math::binding::SlotBinding::new(source, target, &physical.quantities)
                .map_err(crate::workflow::math)?;
            bindings.push((a.id, b.id));
            // Connections are ordinary typed equality definitions, consumed by the same compiler.
            let id = symbol_id(connection.connection_id, source.id, &[target.id]);
            let c = Context {
                formals: vec![
                    m::AuthoredComputationModelsFieldDefinitionsItemFormalsItem {
                        path: "s0".into(),
                        quantity_id: source.quantity.as_id(),
                    },
                    m::AuthoredComputationModelsFieldDefinitionsItemFormalsItem {
                        path: "s1".into(),
                        quantity_id: target.quantity.as_id(),
                    },
                ],
                ports: vec![source.clone(), target.clone()],
                ..Default::default()
            };
            let instance = instances::Row {
                instance_id: id,
                parent_instance_id: None,
                template_id: connection.rule_template_id,
                name: "connection".into(),
                param_values: vec![],
                feature_values: vec![],
                property_package_id: None,
                reaction_package_id: None,
                doc: String::new(),
            };
            add_body(
                model,
                &instance,
                &c,
                vec!["s0 - s1".into()],
                vec![dsl::EquationSense::Eq],
                vec![id],
                physical,
                &BTreeMap::new(),
            )?;
        }
        if from.1.kind != PortKind::Signal {
            let decision = connection.tear_group.unwrap_or(connection.connection_id);
            let policy = match connection.tear_policy {
                Some(TearPolicy::Mandatory) => Policy::Mandatory,
                Some(TearPolicy::Forbidden) => Policy::Forbidden,
                _ => Policy::Free,
            };
            let cost = connection.tear_cost.unwrap_or(1.0);
            if let Some(previous) = graph.decisions.iter().find(|d| d.id == decision) {
                if previous.cost != cost || previous.policy != policy {
                    return Err(invalid(
                        [connection.connection_id, decision],
                        "grouped tear members must declare the same group cost and policy",
                    ));
                }
            } else {
                graph.decisions.push(Decision {
                    id: decision,
                    cost,
                    policy,
                });
            }
            graph.connections.push(Connection {
                id: connection.connection_id,
                from: from.0,
                to: to.0,
                decision,
                bindings,
            });
        }
    }
    Ok(graph)
}
