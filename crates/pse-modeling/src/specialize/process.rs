// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Independent process state, one connection occurrence and original-space inventory meaning.
use super::rewrite::ResolvedPath;
use super::rewrite::RewriteContext;
use super::*;
use crate::resolved::{NodeIx, Source, construction::Rewritten};

/// A contractual slot and its semantic index values; binder spelling and vector position are absent.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct StateKey {
    /// Authored slot role.
    pub name: String,
    /// Actual species, phase or other semantic indices.
    pub indices: Vec<Value>,
}
impl StateKey {
    fn id(&self, owner: SemanticId) -> SemanticId {
        let mut hash = FramedHasher::new(pse_ids::Frame::ModelingProcessSlotV1);
        hash.id(&owner)
            .str(&self.name)
            .u64(self.indices.len() as u64);
        for value in &self.indices {
            value.frame(&mut hash);
        }
        hash.finish_id()
    }
}
/// One derived transported observation, independent of the coordinates binding a state.
#[derive(Clone, Debug, PartialEq)]
pub struct TransportObservation {
    /// Sole executable transported observation.
    pub root: NodeIx,
    /// Original physical expression.
    pub expression: Expr,
    /// Complete physical convention.
    pub ty: Type,
    /// Original physical agreement intent and named scalar context.
    pub requirement: ClosureRequirement,
}
/// Admitted state at an actual indexed instance.
#[derive(Clone, Debug, PartialEq)]
pub struct StateSpecification {
    /// Specification occurrence.
    pub id: SemanticId,
    /// Independent coordinate slots aliasing original members.
    pub coordinates: BTreeMap<StateKey, SemanticId>,
    /// Original dependent reconstruction rows and their physical tolerances.
    pub reconstructions: Vec<(Row, ClosureRequirement)>,
    /// Full supplied boundary state retains consistency observations.
    pub supplied: bool,
    /// Original derived transports.
    pub transports: BTreeMap<StateKey, TransportObservation>,
    /// Specification owner.
    pub lineage: Lineage,
}
/// One material boundary whose coordinates remain scalar numerical projections.
#[derive(Clone, Debug, PartialEq)]
pub struct MaterialPort {
    /// Actual boundary occurrence.
    pub id: SemanticId,
    /// One admitted state meaning.
    pub specification: SemanticId,
    /// Scalar graph coordinate aliases keyed by semantic identity.
    pub coordinates: BTreeMap<StateKey, SemanticId>,
    /// Actual equipment owner.
    pub lineage: Lineage,
}
/// Original coordinate initial condition retained by composite inventory lowering.
#[derive(Clone, Debug, PartialEq)]
pub struct InventoryInitialCondition {
    /// Original initial equation, retained as a hard consistency obligation.
    pub row: SemanticId,
    /// Original equation attribution; its Start projection retains the isolated RHS.
    pub lineage: Lineage,
}
/// Original conserved inventory, signed flux and allowed state-dependent event transfers.
#[derive(Clone, Debug, PartialEq)]
pub struct InventoryBalance {
    /// Sole executable original inventory.
    pub inventory_root: NodeIx,
    /// Sole executable original signed flux.
    pub flux_root: NodeIx,
    /// Exact executable pre-event transfers keyed by actual event.
    pub transfer_roots: BTreeMap<SemanticId, NodeIx>,
    /// Stable conserved subject occurrence.
    pub id: SemanticId,
    /// Physical inventory expression identity.
    pub inventory_id: SemanticId,
    /// Independent flux integral identity.
    pub flux_id: SemanticId,
    /// Integrated differential coordinate, including a generated stock for a composite inventory.
    pub state: Option<SemanticId>,
    /// Original inventory, never the solved accumulation variable.
    pub inventory: Expr,
    /// Original signed transport/source expression.
    pub flux: Expr,
    /// Canonical physical inventory type.
    pub ty: Type,
    /// Original physical inventory closure intent and named scalar context.
    pub requirement: ClosureRequirement,
    /// Actual guard member occurrence to original pre-event transfer expression.
    pub transfers: BTreeMap<SemanticId, Expr>,
    /// Conserved subject attribution.
    pub lineage: Lineage,
}

impl Engine<'_, '_> {
    pub(super) fn closure_requirement(
        &self,
        at: DeclarationId,
        role: &str,
        position: usize,
        env: &Environment,
        expected: &Type,
        context: Option<SemanticId>,
    ) -> Result<ClosureRequirement> {
        let expression = self.p.expression_at(at, role, position)?;
        let value = self.process_value(at, role, position, env, Some(expected))?;
        let magnitude = value.scalar(at)?;
        if !magnitude.is_finite() || magnitude <= 0.0 {
            return Err(invalid(
                at,
                "positive finite physical closure tolerance required",
            ));
        }
        let rule = if let ExprKind::Path(path) = &expression.kind {
            let name = dsl::render_path(path);
            let unbound = !env.contains_key(&name)
                && path
                    .segments
                    .first()
                    .is_some_and(|first| !env.contains_key(&first.name));
            if unbound
                && path
                    .segments
                    .iter()
                    .all(|segment| segment.indices.is_empty())
            {
                self.p
                    .resolve_segments(at, &path.segments)
                    .and_then(|constant| {
                        self.model
                            .engineering_rules
                            .iter()
                            .find(|rule| rule.id == constant.as_id())
                    })
            } else {
                None
            }
        } else {
            None
        };
        Ok(ClosureRequirement {
            tolerance: rule.map_or(ClosureTolerance::Explicit(value), |rule| {
                ClosureTolerance::EngineeringRule {
                    rule_id: rule.id,
                    marker: rule.marker,
                }
            }),
            context,
            endpoint: None,
        })
    }
    // Keep a named member's owner even when its parameter value will be folded.
    // An arbitrary expression does not acquire a guessed engineering context.
    fn closure_context(
        &mut self,
        instance: InstanceId,
        at: DeclarationId,
        expression: &Expr,
        env: &Environment,
    ) -> Result<Option<SemanticId>> {
        let ExprKind::Path(path) = &expression.kind else {
            return Ok(None);
        };
        let Ok((owner, declaration, coordinates)) =
            self.resolve_path(instance, at, path, env, true)
        else {
            return Ok(None);
        };
        if !matches!(
            self.p.declarations[&declaration].value.kind,
            Kind::Variable | Kind::Parameter | Kind::Let | Kind::Alias
        ) {
            return Ok(None);
        }
        self.symbol(owner, declaration, &coordinates, &[at])
            .map(Some)
    }
    fn process_value(
        &self,
        at: DeclarationId,
        role: &str,
        position: usize,
        env: &Environment,
        expected: Option<&Type>,
    ) -> Result<Value> {
        self.checkpoint()?;
        let value = Evaluator {
            lexical_bases: Some(&self.lexical_bases),
            package: self.p,
            physical: self.c,
            at,
            env,
            limit: self.limits.members,
            stack: Vec::new(),
            reader: self.reader,
            selections: Some(&self.selection_collector),
        }
        .expr(self.p.expression_at(at, role, position)?, expected, 0)?;
        if let Some(expected) = expected
            && !value::conforms(&value, expected, self.p)
        {
            return Err(invalid(
                at,
                format!("static value does not satisfy {expected:?}"),
            ));
        }
        Ok(value)
    }
    fn process_coordinates<'a>(
        &self,
        instance: InstanceId,
        at: DeclarationId,
        env: &Environment,
        role: &str,
        names: impl Iterator<Item = &'a str>,
    ) -> Result<Vec<Vec<(String, Value)>>> {
        let role = format!("{role}.indices.domain");
        let family = self.finite_family(instance, at, &role, env);
        let mut output = vec![Vec::new()];
        for (position, name) in names.enumerate() {
            let mut next = Vec::new();
            for row in output {
                self.checkpoint()?;
                let local = coordinates_env(env, &row);
                let Value::Set(values) = Evaluator {
                    lexical_bases: Some(&self.lexical_bases),
                    package: self.p,
                    physical: self.c,
                    at,
                    env: &local,
                    limit: self.limits.members,
                    stack: Vec::new(),
                    reader: self.reader,
                    selections: Some(&self.selection_collector),
                }
                .syntax(self.p.static_at(at, &role, position)?, None, 0)?
                else {
                    return Err(invalid(at, "finite domain required"));
                };
                self.finite_domain(
                    &family,
                    position,
                    &row,
                    self.finite_domain_type(at, &local, &role, position)?,
                    &values,
                )?;
                if next
                    .len()
                    .checked_add(values.len())
                    .is_none_or(|n| n > self.limits.members)
                {
                    return Err(ModelingError::Budget("indexed extent".into()));
                }
                for value in values {
                    let mut row = row.clone();
                    row.push((name.into(), value));
                    next.push(row);
                }
            }
            output = next;
        }
        self.finite_collector.family(family, &output)?;
        Ok(output)
    }
    fn difference_type(ty: &Type) -> Type {
        match ty {
            Type::Quantity(scheme) => Type::Quantity(pse_quantity::scheme::Scheme::Delta(
                Box::new(scheme.clone()),
            )),
            _ => ty.clone(),
        }
    }
    fn tolerance_type(&self, ty: &Type, at: DeclarationId) -> Result<Type> {
        match Self::difference_type(ty) {
            Type::Quantity(scheme) => Ok(Type::Quantity(pse_quantity::scheme::Scheme::Concrete(
                scheme
                    .resolve_with_evidence(
                        self.c.quantities,
                        &BTreeMap::new(),
                        self.c.preconditions,
                    )
                    .map_err(|e| invalid(at, e.to_string()))?,
            ))),
            other => Ok(other),
        }
    }
    fn process_type(&self, root: NodeIx, _at: DeclarationId) -> Result<Type> {
        self.constructed_type(root)
    }
    fn state_key(name: &str, coordinates: &[(String, Value)]) -> StateKey {
        StateKey {
            name: name.into(),
            indices: coordinates.iter().map(|(_, v)| v.clone()).collect(),
        }
    }
    pub(super) fn state_specification(
        &mut self,
        instance: InstanceId,
        row: &Declaration,
        env: &Environment,
    ) -> Result<()> {
        let at = row.declaration_id;
        let value = row
            .value
            .state_specification
            .as_ref()
            .ok_or_else(|| invalid(at, "state specification required"))?
            .clone();
        for coordinates in self.process_coordinates(
            instance,
            at,
            env,
            "state_specification",
            value.indices.iter().map(|i| i.name.as_str()),
        )? {
            let id = member_id(instance, at, &coordinates);
            if self.model.state_specifications.contains_key(&id) {
                continue;
            }
            let local = coordinates_env(env, &coordinates);
            let Value::Boolean(supplied) = self.process_value(
                at,
                "state_specification.supplied",
                0,
                &local,
                Some(&Type::Boolean),
            )?
            else {
                return Err(invalid(at, "state supplied responsibility must be Boolean"));
            };
            let mut specification = StateSpecification {
                id,
                coordinates: BTreeMap::new(),
                reconstructions: Vec::new(),
                supplied,
                transports: BTreeMap::new(),
                lineage: self.lineage(instance, row, &[at]),
            };
            for (position, coordinate) in value.coordinates.iter().enumerate() {
                for indices in self.process_coordinates(
                    instance,
                    at,
                    &local,
                    &format!("state_specification.coordinates.{position}"),
                    coordinate.indices.iter().map(|i| i.name.as_str()),
                )? {
                    let original = self
                        .p
                        .expression_at(at, "state_specification.coordinates.target", position)?
                        .clone();
                    let scope = coordinates_env(&local, &indices);
                    // Static parameter folding is useful in equations, but a boundary
                    // coordinate must keep the identity of the authored fixed member.
                    let parameter = if let ExprKind::Path(path) = &original.kind {
                        let (owner, declaration, coordinates) =
                            self.resolve_path(instance, at, path, &scope, true)?;
                        (self.p.declarations[&declaration].value.kind == Kind::Parameter)
                            .then_some((owner, declaration, coordinates))
                    } else {
                        None
                    };
                    let target = self.with_original_expression(
                        at,
                        "state_specification.coordinates.target",
                        position,
                        &original,
                        |engine| {
                            if let Some((owner, declaration, coordinates)) = parameter {
                                let id = engine.symbol(owner, declaration, &coordinates, &[at])?;
                                engine.selected_scalar(
                                    id,
                                    engine.original_expression_source(&original, at)?,
                                )
                            } else {
                                engine.rewrite(instance, &original, &scope, &[at])
                            }
                        },
                    )?;
                    let symbol = self.symbol_of_root(target.root).ok_or_else(|| {
                        invalid(
                            at,
                            "independent state coordinate must alias an actual scalar",
                        )
                    })?;
                    let key = Self::state_key(&coordinate.name, &indices);
                    if specification.coordinates.insert(key, symbol).is_some() {
                        return Err(invalid(at, "duplicate independent state slot"));
                    }
                }
            }
            for (position, reconstruction) in value.reconstructions.iter().enumerate() {
                for indices in self.process_coordinates(
                    instance,
                    at,
                    &local,
                    &format!("state_specification.reconstructions.{position}"),
                    reconstruction.indices.iter().map(|i| i.name.as_str()),
                )? {
                    let scope = coordinates_env(&local, &indices);
                    let original = self
                        .p
                        .equation_at(at, "state_specification.reconstructions.equation", position)?
                        .clone();
                    let context = match &original.kind {
                        EquationKind::Relation { lhs, .. } => {
                            self.closure_context(instance, at, lhs, &scope)?
                        }
                        _ => None,
                    };
                    let equation = self.with_original_equation(
                        at,
                        "state_specification.reconstructions.equation",
                        position,
                        &original,
                        |engine| engine.rewrite_equation(instance, &original, &scope, &[at]),
                    )?;
                    if equation.sense != EquationSense::Eq {
                        return Err(invalid(at, "state reconstruction must be an equality"));
                    }
                    let ty = self.process_type(equation.lhs, at)?;
                    let requirement = self.closure_requirement(
                        at,
                        "state_specification.reconstructions.tolerance",
                        position,
                        &scope,
                        &self.tolerance_type(&ty, at)?,
                        context,
                    )?;
                    let key = Self::state_key(&reconstruction.name, &indices);
                    let row_id = key.id(id);
                    self.model
                        .equation_roots
                        .insert(row_id, equation.relation());
                    specification.reconstructions.push((
                        Row {
                            id: row_id,
                            equation: equation.inspection,
                            lineage: specification.lineage.clone(),
                        },
                        requirement,
                    ));
                }
            }
            for (position, transport) in value.transports.iter().enumerate() {
                for indices in self.process_coordinates(
                    instance,
                    at,
                    &local,
                    &format!("state_specification.transports.{position}"),
                    transport.indices.iter().map(|i| i.name.as_str()),
                )? {
                    let scope = coordinates_env(&local, &indices);
                    let original = self
                        .p
                        .expression_at(at, "state_specification.transports.expression", position)?
                        .clone();
                    let context = self.closure_context(instance, at, &original, &scope)?;
                    let expression = self.with_original_expression(
                        at,
                        "state_specification.transports.expression",
                        position,
                        &original,
                        |engine| engine.rewrite(instance, &original, &scope, &[at]),
                    )?;
                    let ty = self.process_type(expression.root, at)?;
                    let requirement = self.closure_requirement(
                        at,
                        "state_specification.transports.tolerance",
                        position,
                        &scope,
                        &self.tolerance_type(&ty, at)?,
                        context,
                    )?;
                    let key = Self::state_key(&transport.name, &indices);
                    if specification
                        .transports
                        .insert(
                            key,
                            TransportObservation {
                                root: expression.root,
                                expression: expression.inspection,
                                ty,
                                requirement,
                            },
                        )
                        .is_some()
                    {
                        return Err(invalid(at, "duplicate transported observation"));
                    }
                }
            }
            if value.extends.is_some() {
                let (owner, declaration, coordinates) = self.process_endpoint(
                    instance,
                    at,
                    self.p.expression_at(at, "state_specification.extends", 0)?,
                    &local,
                    Kind::StateSpecification,
                )?;
                let inherited_id = member_id(owner, declaration, &coordinates);
                if inherited_id == id {
                    return Err(invalid(at, "recursive state specification composition"));
                }
                if !self.model.state_specifications.contains_key(&inherited_id) {
                    self.state_specification(
                        owner,
                        &self.p.declarations[&declaration].clone(),
                        &self.states[&owner].env.clone(),
                    )?;
                }
                let inherited = self.model.state_specifications[&inherited_id].clone();
                for (key, symbol) in inherited.coordinates {
                    if specification.coordinates.insert(key, symbol).is_some() {
                        return Err(invalid(at, "composed independent state slot duplicated"));
                    }
                }
                specification
                    .reconstructions
                    .extend(inherited.reconstructions);
                for (key, observation) in inherited.transports {
                    if specification.transports.insert(key, observation).is_some() {
                        return Err(invalid(at, "composed transported observation duplicated"));
                    }
                }
            }
            let mut symbols = BTreeSet::new();
            if specification
                .coordinates
                .values()
                .any(|s| !symbols.insert(*s))
            {
                return Err(invalid(
                    at,
                    "one independent coordinate cannot occupy multiple state slots",
                ));
            }
            if specification.coordinates.is_empty() {
                return Err(invalid(at, "state requires independent coordinates"));
            }
            if specification.transports.is_empty() {
                return Err(invalid(
                    at,
                    "material state requires transported observations",
                ));
            }
            self.reserve(specification.coordinates.len() + specification.transports.len() + 1)?;
            self.model.state_specifications.insert(id, specification);
        }
        Ok(())
    }
    fn process_endpoint(
        &self,
        instance: InstanceId,
        at: DeclarationId,
        expression: &Expr,
        env: &Environment,
        kind: Kind,
    ) -> Result<ResolvedPath> {
        let ExprKind::Path(path) = &expression.kind else {
            return Err(invalid(
                at,
                "process endpoint must name a declared occurrence",
            ));
        };
        let (owner, declaration, coordinates) = self.resolve_path(instance, at, path, env, true)?;
        if self.p.declarations[&declaration].value.kind != kind {
            return Err(invalid(
                at,
                "process endpoint declaration has the wrong role",
            ));
        }
        Ok((owner, declaration, coordinates))
    }
    pub(super) fn material_port(
        &mut self,
        instance: InstanceId,
        row: &Declaration,
        env: &Environment,
    ) -> Result<()> {
        let at = row.declaration_id;
        let value = row
            .value
            .state_port
            .as_ref()
            .ok_or_else(|| invalid(at, "material port required"))?
            .clone();
        for indices in self.process_coordinates(
            instance,
            at,
            env,
            "state_port",
            value.indices.iter().map(|i| i.name.as_str()),
        )? {
            let id = member_id(instance, at, &indices);
            if self.model.material_ports.contains_key(&id) {
                continue;
            }
            let local = coordinates_env(env, &indices);
            let (owner, declaration, coordinates) = self.process_endpoint(
                instance,
                at,
                self.p.expression_at(at, "state_port.specification", 0)?,
                &local,
                Kind::StateSpecification,
            )?;
            let specification = member_id(owner, declaration, &coordinates);
            if !self.model.state_specifications.contains_key(&specification) {
                self.state_specification(
                    owner,
                    &self.p.declarations[&declaration].clone(),
                    &self.states[&owner].env.clone(),
                )?;
            }
            let state = self
                .model
                .state_specifications
                .get(&specification)
                .ok_or_else(|| invalid(at, "state specification occurrence missing"))?
                .clone();
            let lineage = self.lineage(instance, row, &[at]);
            let mut bindings = BTreeMap::new();
            for (key, symbol) in state.coordinates {
                let coordinate = key.id(id);
                self.model.ports.insert(
                    coordinate,
                    Port {
                        id: coordinate,
                        symbol,
                        lineage: lineage.clone(),
                    },
                );
                bindings.insert(key, coordinate);
            }
            self.model.material_ports.insert(
                id,
                MaterialPort {
                    id,
                    specification,
                    coordinates: bindings,
                    lineage,
                },
            );
        }
        Ok(())
    }
    /// A stage replaces endpoints while retaining the original connection occurrence.
    pub(super) fn connection_occurrence_id(
        &self,
        instance: InstanceId,
        row: &Declaration,
        indices: &[(String, Value)],
    ) -> SemanticId {
        let at = row.declaration_id;
        let identity_at = if row.is_override {
            self.p
                .members
                .get(&self.states[&instance].definition)
                .and_then(|members| members.get(&row.name))
                .copied()
                .unwrap_or(at)
        } else {
            at
        };
        member_id(instance, identity_at, indices)
    }
    pub(super) fn process_connection(
        &mut self,
        instance: InstanceId,
        row: &Declaration,
        env: &Environment,
    ) -> Result<()> {
        let at = row.declaration_id;
        let value = row
            .value
            .connection
            .as_ref()
            .ok_or_else(|| invalid(at, "connection required"))?
            .clone();
        for indices in self.process_coordinates(
            instance,
            at,
            env,
            "connection",
            value.indices.iter().map(|i| i.name.as_str()),
        )? {
            let local = coordinates_env(env, &indices);
            let source = self.p.expression_at(at, "connection.from", 0)?.clone();
            let target = self.p.expression_at(at, "connection.to", 0)?.clone();
            let endpoint = |engine: &Self, e: &Expr| -> Result<ResolvedPath> {
                let ExprKind::Path(p) = &e.kind else {
                    return Err(invalid(at, "connection endpoint must name a declared port"));
                };
                engine.resolve_path(instance, at, p, &local, true)
            };
            let (a, da, ca) = endpoint(self, &source)?;
            let (b, db, cb) = endpoint(self, &target)?;
            let from = member_id(a, da, &ca);
            let to = member_id(b, db, &cb);
            let id = self.connection_occurrence_id(instance, row, &indices);
            let lineage = self.lineage(instance, row, &[at]);
            let material = self.p.declarations[&da].value.kind == Kind::StatePort;
            let mut bindings = Vec::new();
            let mut rows = Vec::new();
            if material {
                if self.p.declarations[&db].value.kind != Kind::StatePort {
                    return Err(invalid(
                        at,
                        "material state cannot connect to a scalar port",
                    ));
                }
                for (owner, declaration) in [(a, da), (b, db)] {
                    self.material_port(
                        owner,
                        &self.p.declarations[&declaration].clone(),
                        &self.states[&owner].env.clone(),
                    )?;
                }
                let left = self.model.material_ports[&from].clone();
                let right = self.model.material_ports[&to].clone();
                if left.coordinates.keys().ne(right.coordinates.keys()) {
                    return Err(invalid(
                        at,
                        "independent state/species correspondence differs; explicit translator required",
                    ));
                }
                for (key, pa) in &left.coordinates {
                    let pb = right.coordinates[key];
                    let sa = self.model.ports[pa].symbol;
                    let sb = self.model.ports[&pb].symbol;
                    if self.model.symbols[&sa].ty != self.model.symbols[&sb].ty {
                        return Err(invalid(
                            at,
                            "state coordinate physical conventions differ; explicit translator required",
                        ));
                    }
                    let row_id = key.id(id);
                    let source = Source {
                        field: crate::expression::occurrences::OccurrenceKey {
                            declaration: at,
                            role: "derived.state_connection".into(),
                            position: rows.len(),
                        },
                        span: Span::default(),
                        coordinate: None,
                    };
                    let lhs = self.selected_scalar(sa, source.clone())?;
                    let rhs = self.selected_scalar(sb, source.clone())?;
                    self.retain_row(
                        row_id,
                        Self::generated_relation(lhs, EquationSense::Eq, rhs, source.span),
                        lineage.clone(),
                    )?;
                    bindings.push((*pa, pb));
                    rows.push(row_id);
                }
                let source = self.model.state_specifications[&left.specification].clone();
                let target = self.model.state_specifications[&right.specification].clone();
                if source.transports.keys().ne(target.transports.keys()) {
                    return Err(invalid(
                        at,
                        "transport/species coverage differs; explicit admitted embedding required",
                    ));
                }
                for (key, transport) in &source.transports {
                    let other = &target.transports[key];
                    if transport.ty != other.ty {
                        return Err(invalid(
                            at,
                            "transport basis/reference conventions differ; explicit translator required",
                        ));
                    }
                    let mut source_requirement = transport.requirement.clone();
                    source_requirement.endpoint = Some(left.id);
                    let mut target_requirement = other.requirement.clone();
                    target_requirement.endpoint = Some(right.id);
                    let closure = pse_ids::named_id(key.id(id), "transport-agreement");
                    self.observation_closure(
                        closure,
                        transport.ty.clone(),
                        vec![source_requirement, target_requirement],
                        Rewritten {
                            root: transport.root,
                            inspection: transport.expression.clone(),
                        },
                        Rewritten {
                            root: other.root,
                            inspection: other.expression.clone(),
                        },
                        lineage.clone(),
                    )?;
                }
            } else {
                if self.p.declarations[&da].value.kind != Kind::Port
                    || self.p.declarations[&db].value.kind != Kind::Port
                {
                    return Err(invalid(
                        at,
                        "connection endpoints must name scalar or material ports of the same role",
                    ));
                }
                let lhs =
                    self.with_original_expression(at, "connection.from", 0, &source, |engine| {
                        engine.rewrite(instance, &source, &local, &[at])
                    })?;
                let rhs =
                    self.with_original_expression(at, "connection.to", 0, &target, |engine| {
                        engine.rewrite(instance, &target, &local, &[at])
                    })?;
                let sa = self
                    .symbol_of_root(lhs.root)
                    .ok_or_else(|| invalid(at, "connection requires an actual scalar"))?;
                let sb = self
                    .symbol_of_root(rhs.root)
                    .ok_or_else(|| invalid(at, "connection requires an actual scalar"))?;
                if self.model.symbols[&sa].ty != self.model.symbols[&sb].ty {
                    return Err(invalid(at, "connection physical types differ"));
                }
                self.retain_row(
                    id,
                    Self::generated_relation(lhs, EquationSense::Eq, rhs, Span::default()),
                    lineage.clone(),
                )?;
                bindings.push((from, to));
                rows.push(id);
            }
            self.model.connections.insert(
                id,
                Connection {
                    id,
                    from,
                    to,
                    bindings,
                    rows,
                    lineage,
                },
            );
        }
        Ok(())
    }
    fn observation_closure(
        &mut self,
        id: SemanticId,
        ty: Type,
        requirements: Vec<ClosureRequirement>,
        lhs: Rewritten,
        rhs: Rewritten,
        lineage: Lineage,
    ) -> Result<()> {
        let source = Source {
            field: crate::expression::occurrences::OccurrenceKey {
                declaration: lineage.declaration,
                role: "derived.observation_difference".into(),
                position: 0,
            },
            span: Span::default(),
            coordinate: None,
        };
        let expression = self.generated_binary(BinaryOp::Sub, lhs, rhs, source)?;
        let term = pse_ids::named_id(id, "difference");
        self.model.contribution_roots.insert(term, expression.root);
        self.model.closures.insert(
            id,
            Closure {
                id,
                mode: Mode::Conservation,
                observation_only: true,
                ty: Self::difference_type(&ty),
                boundary: None,
                requirements,
                terms: vec![Contribution {
                    id: term,
                    role: Role::Positive,
                    expression: expression.inspection,
                    lineage: lineage.clone(),
                }],
                lineage,
            },
        );
        Ok(())
    }
    pub(super) fn finish_process_states(&mut self) -> Result<()> {
        let receiving = self
            .model
            .connections
            .values()
            .filter_map(|c| {
                self.model
                    .material_ports
                    .get(&c.to)
                    .map(|p| p.specification)
            })
            .collect::<BTreeSet<_>>();
        let mut generated = BTreeSet::new();
        for specification in self
            .model
            .state_specifications
            .values()
            .cloned()
            .collect::<Vec<_>>()
        {
            for (row, requirement) in specification.reconstructions {
                if (!specification.supplied || receiving.contains(&specification.id))
                    && generated.insert(row.id)
                {
                    self.model.equations.push(row.clone());
                }
                let EquationKind::Relation { lhs, rhs, .. } = row.equation.kind else {
                    return Err(invalid(
                        row.lineage.declaration,
                        "state reconstruction relation required",
                    ));
                };
                let relation = *self.model.equation_roots.get(&row.id).ok_or_else(|| {
                    invalid(
                        row.lineage.declaration,
                        "state reconstruction lacks its actual relation roots",
                    )
                })?;
                let ty = self.process_type(relation.lhs, row.lineage.declaration)?;
                let lhs = Rewritten {
                    root: relation.lhs,
                    inspection: lhs,
                };
                let rhs = Rewritten {
                    root: relation.rhs,
                    inspection: rhs,
                };
                self.observation_closure(
                    pse_ids::named_id(row.id, "consistency"),
                    ty,
                    vec![requirement],
                    lhs,
                    rhs,
                    row.lineage,
                )?;
            }
        }
        Ok(())
    }
    pub(super) fn inventory_balance(
        &mut self,
        instance: InstanceId,
        row: &Declaration,
        env: &Environment,
    ) -> Result<()> {
        let at = row.declaration_id;
        let value = row
            .value
            .inventory_balance
            .as_ref()
            .ok_or_else(|| invalid(at, "inventory declaration required"))?
            .clone();
        let inventory_source = self
            .p
            .expression_at(at, "inventory_balance.inventory", 0)?
            .clone();
        let flux_source = self
            .p
            .expression_at(at, "inventory_balance.flux", 0)?
            .clone();
        let axis = self
            .p
            .expression_at(at, "inventory_balance.axis", 0)?
            .clone();
        let ExprKind::Path(domain) = axis.kind else {
            return Err(invalid(
                at,
                "inventory axis must name its continuous domain",
            ));
        };
        let binder = value
            .indices
            .iter()
            .find(|i| i.domain.replace(' ', "") == value.axis.replace(' ', ""))
            .map(|i| i.name.clone())
            .ok_or_else(|| {
                invalid(
                    at,
                    "inventory requires an explicit index over its declared time axis",
                )
            })?;
        let points = match self.process_value(at, "inventory_balance.axis", 0, env, None)? {
            Value::Set(points) => points,
            _ => return Err(invalid(at, "inventory axis is not realized")),
        };
        let coordinate = points
            .first()
            .ok_or_else(|| invalid(at, "inventory axis is empty"))?
            .clone();
        let (mesh, _) = self
            .coordinate_mesh(&coordinate)
            .map_err(|error| match error {
                ModelingError::Contract {
                    reason, message, ..
                } => crate::invalid_reason(at, reason, message),
                other => other,
            })?;
        let mesh_id = mesh.id;
        let integrated = self.model.integrated.contains_key(&mesh_id);
        let first = mesh
            .points
            .first()
            .ok_or_else(|| invalid(at, "inventory mesh is empty"))?
            .clone();
        let last = mesh
            .points
            .last()
            .ok_or_else(|| invalid(at, "inventory mesh is empty"))?
            .clone();
        if !integrated && !value.transfers.is_empty() {
            return Err(ModelingError::Unsupported {
                declaration: at.as_id(),
                capability: "event-bearing simultaneous conservation".into(),
            });
        }
        for coordinates in self.process_coordinates(
            instance,
            at,
            env,
            "inventory_balance",
            value.indices.iter().map(|i| i.name.as_str()),
        )? {
            let local = coordinates_env(env, &coordinates);
            let subject_coordinates = coordinates
                .iter()
                .filter(|(name, _)| name != &binder)
                .cloned()
                .collect::<Vec<_>>();
            let id = member_id(instance, at, &subject_coordinates);
            let inventory_id = pse_ids::named_id(id, "inventory-observation");
            let flux_id = pse_ids::named_id(id, "original-flux-integral");
            let inventory = self.with_original_expression(
                at,
                "inventory_balance.inventory",
                0,
                &inventory_source,
                |engine| engine.rewrite(instance, &inventory_source, &local, &[at]),
            )?;
            let flux = self.with_original_expression(
                at,
                "inventory_balance.flux",
                0,
                &flux_source,
                |engine| engine.rewrite(instance, &flux_source, &local, &[at]),
            )?;
            let ty = self.process_type(inventory.root, at)?;
            let context = self.closure_context(instance, at, &inventory_source, &local)?;
            let requirement = self.closure_requirement(
                at,
                "inventory_balance.tolerance",
                0,
                &local,
                &self.tolerance_type(&ty, at)?,
                context,
            )?;
            let lineage = self.lineage(instance, row, &[at]);
            let mut transfers = BTreeMap::new();
            let mut transfer_roots = BTreeMap::new();
            for (position, _) in value.transfers.iter().enumerate() {
                let ExprKind::Path(event) = &self
                    .p
                    .expression_at(at, "inventory_balance.transfers.event", position)?
                    .kind
                else {
                    return Err(invalid(
                        at,
                        "inventory transfer event must name a guard path",
                    ));
                };
                let (owner, member, coordinates) =
                    self.resolve_path(instance, at, event, &local, false)?;
                let event = self.symbol(owner, member, &coordinates, &[at])?;
                let expression = self.rewrite_field(
                    instance,
                    at,
                    "inventory_balance.transfers.expression",
                    position,
                    &local,
                    &[at],
                )?;
                if self.process_type(expression.root, at)? != self.tolerance_type(&ty, at)? {
                    return Err(invalid(at, "event transfer inventory convention differs"));
                }
                transfer_roots.insert(event, expression.root);
                if transfers.insert(event, expression.inspection).is_some() {
                    return Err(invalid(at, "duplicate permitted event transfer"));
                }
            }
            let reads = self
                .construction
                .reads(&self.model.symbols, inventory.root)?;
            let existing_dynamics = integrated
                && reads
                    .numeric
                    .iter()
                    .any(|id| self.model.derivatives.contains_key(id));
            let derivative_defined = {
                let (mesh, index) = self.coordinate_mesh(
                    local
                        .get(&binder)
                        .ok_or_else(|| invalid(at, "inventory coordinate is unbound"))?,
                )?;
                integrated || !mesh.derivative[index].is_empty()
            };
            let origin = Source {
                field: crate::expression::occurrences::OccurrenceKey {
                    declaration: at,
                    role: "derived.inventory_balance".into(),
                    position: 0,
                },
                span: Span::default(),
                coordinate: None,
            };
            let mut differential_state = None;
            if !existing_dynamics && derivative_defined {
                let body = if integrated
                    && self
                        .symbol_of_root(inventory.root)
                        .is_none_or(|symbol| self.model.symbols[&symbol].role != Kind::Variable)
                {
                    let state = pse_ids::named_id(id, "inventory-state");
                    if !self.model.symbols.contains_key(&state) {
                        let mut replacements = BTreeMap::new();
                        let mut starts = Vec::new();
                        let mut replaced = BTreeSet::new();
                        for coordinate in &reads.numeric {
                            if self
                                .model
                                .symbols
                                .get(coordinate)
                                .is_some_and(|symbol| symbol.role == Kind::Variable)
                            {
                                let (row,root,inspection,lineage)=self.inventory_initial_sources.get(coordinate).cloned().ok_or_else(||invalid(at,"composite inventory requires explicit initial conditions for its coordinates"))?;
                                replacements.insert(*coordinate, root);
                                replaced.insert(row);
                                starts.push((*coordinate, row, root, inspection, lineage));
                            }
                        }
                        let initial_root =
                            self.substitute_root(inventory.root, &replacements, at)?;
                        for (target, row, root, inspection, lineage) in starts {
                            self.model.inventory_initial_conditions.insert(
                                target,
                                InventoryInitialCondition {
                                    row,
                                    lineage: lineage.clone(),
                                },
                            );
                            self.model.annotations.retain(|annotation| {
                                !(annotation.target == target
                                    && matches!(
                                        annotation.value,
                                        crate::annotation::AnnotationValue::Start(_)
                                    ))
                            });
                            self.model.annotations.push(crate::annotation::Annotation {
                                target,
                                value: crate::annotation::AnnotationValue::Start(inspection),
                                roots: crate::annotation::AnnotationRoots::Scalar { value: root },
                                lineage,
                            });
                        }
                        self.model.symbols.insert(
                            state,
                            Symbol {
                                definition: None,
                                id: state,
                                ty: ty.clone(),
                                role: Kind::Variable,
                                domain: Domain::Continuous,
                                expression: None,
                                initial: None,
                                lineage: lineage.clone(),
                            },
                        );
                        self.model.annotations.push(crate::annotation::Annotation {
                            target: state,
                            value: crate::annotation::AnnotationValue::Start(
                                inventory.inspection.clone(),
                            ),
                            roots: crate::annotation::AnnotationRoots::Scalar {
                                value: initial_root,
                            },
                            lineage: lineage.clone(),
                        });
                        let initial_row = pse_ids::named_id(id, "inventory-initial");
                        self.model
                            .equations
                            .retain(|row| !replaced.contains(&row.id));
                        self.model
                            .initial_equations
                            .retain(|row| !replaced.contains(row));
                        self.model.initial_equations.insert(initial_row);
                        let state_value = self.selected_scalar(state, origin.clone())?;
                        let initial = Rewritten {
                            root: initial_root,
                            inspection: inventory.inspection.clone(),
                        };
                        self.retain_row(
                            initial_row,
                            Self::generated_relation(
                                state_value.clone(),
                                EquationSense::Eq,
                                initial,
                                origin.span,
                            ),
                            lineage.clone(),
                        )?;
                        let definition = pse_ids::named_id(id, "inventory-definition");
                        self.retain_row(
                            definition,
                            Self::generated_relation(
                                state_value,
                                EquationSense::Eq,
                                inventory.clone(),
                                origin.span,
                            ),
                            lineage.clone(),
                        )?;
                    }
                    self.selected_scalar(state, origin.clone())?
                } else {
                    inventory.clone()
                };
                let flux_type = self.process_type(flux.root, at)?;
                let derivative = if integrated {
                    let axis = self
                        .model
                        .integrated
                        .get(&mesh_id)
                        .cloned()
                        .ok_or_else(|| invalid(at, "integrated inventory has no actual axis"))?;
                    self.integrated_derivative_of(
                        instance,
                        &body,
                        &axis,
                        Some(&flux_type),
                        (at, origin.clone()),
                        &[at],
                    )?
                } else {
                    self.with_original_expression(
                        at,
                        "inventory_balance.inventory",
                        0,
                        &inventory_source,
                        |engine| {
                            engine.derivative(
                                RewriteContext {
                                    instance,
                                    env: &local,
                                    chain: &[at],
                                },
                                &inventory_source,
                                &Path::single(binder.clone()),
                                Some(&flux_type),
                                origin.clone(),
                            )
                        },
                    )?
                };
                if integrated {
                    let rate = self.symbol_of_root(derivative.root).ok_or_else(|| {
                        invalid(at, "inventory derivative has no actual rate scalar")
                    })?;
                    differential_state = self
                        .model
                        .derivatives
                        .values()
                        .find(|value| value.rate == rate)
                        .map(|value| value.state);
                }
                self.retain_row(
                    pse_ids::named_id(member_id(instance, at, &coordinates), "inventory-rate"),
                    Self::generated_relation(
                        derivative,
                        EquationSense::Eq,
                        flux.clone(),
                        origin.span,
                    ),
                    lineage.clone(),
                )?;
            }
            if integrated || local.get(&binder) == Some(&last) {
                let tolerance_type = self.tolerance_type(&ty, at)?;
                let integral = self.with_original_expression(
                    at,
                    "inventory_balance.flux",
                    0,
                    &flux_source,
                    |engine| {
                        engine.integral(
                            RewriteContext {
                                instance,
                                env: &local,
                                chain: &[at],
                            },
                            &dsl::Binder {
                                var: binder.clone(),
                                domain: domain.clone(),
                                filter: None,
                            },
                            &flux_source,
                            Some(&tolerance_type),
                            origin.clone(),
                        )
                    },
                )?;
                if integrated {
                    let old = self
                        .symbol_of_root(integral.root)
                        .ok_or_else(|| invalid(at, "original flux integral missing"))?;
                    if old != flux_id {
                        let mut declared = self
                            .model
                            .integrals
                            .remove(&old)
                            .ok_or_else(|| invalid(at, "flux quadrature missing"))?;
                        declared.result = flux_id;
                        self.model.integrals.insert(flux_id, declared);
                        let slot = self
                            .model
                            .symbols
                            .slot(old)
                            .ok_or_else(|| invalid(at, "flux integral symbol missing"))?;
                        self.model.symbols.rebind(slot, flux_id)?;
                        self.finite_collector.remap(old, flux_id);
                    }
                } else {
                    let mut baseline = local.clone();
                    baseline.insert(binder.clone(), first.clone());
                    let replica = self.replica(instance, local[&binder].identity());
                    let initial = self.with_original_expression(
                        at,
                        "inventory_balance.inventory",
                        0,
                        &inventory_source,
                        |engine| {
                            engine.shifted(
                                RewriteContext {
                                    instance,
                                    env: &baseline,
                                    chain: &[at],
                                },
                                &inventory_source,
                                &binder,
                                &first,
                                replica.as_ref(),
                            )
                        },
                    )?;
                    let rhs =
                        self.generated_binary(BinaryOp::Add, initial, integral, origin.clone())?;
                    self.observation_closure(
                        pse_ids::named_id(id, "temporal-closure"),
                        ty.clone(),
                        vec![requirement.clone()],
                        inventory.clone(),
                        rhs,
                        lineage.clone(),
                    )?;
                }
                self.model.inventory_balances.insert(
                    id,
                    InventoryBalance {
                        id,
                        inventory_id,
                        flux_id,
                        state: differential_state,
                        inventory_root: inventory.root,
                        inventory: inventory.inspection,
                        flux_root: flux.root,
                        flux: flux.inspection,
                        transfer_roots,
                        ty,
                        requirement,
                        transfers,
                        lineage,
                    },
                );
            }
        }
        Ok(())
    }
}
