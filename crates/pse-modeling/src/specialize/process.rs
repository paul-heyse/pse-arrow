// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Independent process state, one connection occurrence and original-space inventory meaning.
use super::rewrite::ResolvedPath;
use super::*;

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
    /// Original physical expression.
    pub expression: Expr,
    /// Complete physical convention.
    pub ty: Type,
    /// Physical agreement budget.
    pub tolerance: Value,
}
/// Admitted state at an actual indexed instance.
#[derive(Clone, Debug, PartialEq)]
pub struct StateSpecification {
    /// Specification occurrence.
    pub id: SemanticId,
    /// Independent coordinate slots aliasing original members.
    pub coordinates: BTreeMap<StateKey, SemanticId>,
    /// Original dependent reconstruction rows and their physical tolerances.
    pub reconstructions: Vec<(Row, Value)>,
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
    /// Canonical physical closure tolerance.
    pub tolerance: Value,
    /// Actual guard member occurrence to original pre-event transfer expression.
    pub transfers: BTreeMap<SemanticId, Expr>,
    /// Conserved subject attribution.
    pub lineage: Lineage,
}

impl Engine<'_, '_> {
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
        at: DeclarationId,
        env: &Environment,
        role: &str,
        names: impl Iterator<Item = &'a str>,
    ) -> Result<Vec<Vec<(String, Value)>>> {
        let role = format!("{role}.indices.domain");
        let mut output = vec![Vec::new()];
        for (position, name) in names.enumerate() {
            let mut next = Vec::new();
            for row in output {
                self.checkpoint()?;
                let local = coordinates_env(env, &row);
                let Value::Set(values) = Evaluator {
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
    fn process_type(&self, expression: &Expr, at: DeclarationId) -> Result<Type> {
        let types = self
            .model
            .symbols
            .iter()
            .map(|(id, s)| (symbol_name(*id), s.ty.clone()))
            .collect();
        crate::expression::infer(
            expression,
            &types,
            &self.model.function_contracts(self.p),
            self.c,
            at,
            None,
        )
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
                    let target = if let Some((owner, declaration, coordinates)) = parameter {
                        symbol_expr(self.symbol(owner, declaration, &coordinates, &[at])?)
                    } else {
                        self.rewrite(instance, &original, &scope, &[at])?
                    };
                    let symbol = symbol_reference(&target).ok_or_else(|| {
                        invalid(
                            at,
                            "independent state coordinate must alias a scalar member",
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
                    at,
                    &local,
                    &format!("state_specification.reconstructions.{position}"),
                    reconstruction.indices.iter().map(|i| i.name.as_str()),
                )? {
                    let scope = coordinates_env(&local, &indices);
                    let equation = self.rewrite_equation(
                        instance,
                        &self
                            .p
                            .equation_at(
                                at,
                                "state_specification.reconstructions.equation",
                                position,
                            )?
                            .clone(),
                        &scope,
                        &[at],
                    )?;
                    let EquationKind::Relation {
                        lhs,
                        sense: EquationSense::Eq,
                        ..
                    } = &equation.kind
                    else {
                        return Err(invalid(at, "state reconstruction must be an equality"));
                    };
                    let ty = self.process_type(lhs, at)?;
                    let tolerance = self.process_value(
                        at,
                        "state_specification.reconstructions.tolerance",
                        position,
                        &scope,
                        Some(&self.tolerance_type(&ty, at)?),
                    )?;
                    if tolerance.scalar(at)? <= 0.0 {
                        return Err(invalid(at, "positive reconstruction tolerance required"));
                    }
                    let key = Self::state_key(&reconstruction.name, &indices);
                    specification.reconstructions.push((
                        Row {
                            id: key.id(id),
                            equation,
                            lineage: specification.lineage.clone(),
                        },
                        tolerance,
                    ));
                }
            }
            for (position, transport) in value.transports.iter().enumerate() {
                for indices in self.process_coordinates(
                    at,
                    &local,
                    &format!("state_specification.transports.{position}"),
                    transport.indices.iter().map(|i| i.name.as_str()),
                )? {
                    let scope = coordinates_env(&local, &indices);
                    let expression = self.rewrite(
                        instance,
                        &self
                            .p
                            .expression_at(
                                at,
                                "state_specification.transports.expression",
                                position,
                            )?
                            .clone(),
                        &scope,
                        &[at],
                    )?;
                    let ty = self.process_type(&expression, at)?;
                    let tolerance = self.process_value(
                        at,
                        "state_specification.transports.tolerance",
                        position,
                        &scope,
                        Some(&self.tolerance_type(&ty, at)?),
                    )?;
                    if tolerance.scalar(at)? <= 0.0 {
                        return Err(invalid(
                            at,
                            "positive transport agreement tolerance required",
                        ));
                    }
                    let key = Self::state_key(&transport.name, &indices);
                    if specification
                        .transports
                        .insert(
                            key,
                            TransportObservation {
                                expression,
                                ty,
                                tolerance,
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
                    self.model.equations.push(Row {
                        id: row_id,
                        equation: Equation {
                            kind: EquationKind::Relation {
                                lhs: symbol_expr(sa),
                                sense: EquationSense::Eq,
                                rhs: symbol_expr(sb),
                            },
                            span: Span::default(),
                        },
                        lineage: lineage.clone(),
                    });
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
                    let tolerance =
                        if transport.tolerance.scalar(at)? <= other.tolerance.scalar(at)? {
                            transport.tolerance.clone()
                        } else {
                            other.tolerance.clone()
                        };
                    let closure = pse_ids::named_id(key.id(id), "transport-agreement");
                    self.observation_closure(
                        closure,
                        transport.ty.clone(),
                        tolerance,
                        transport.expression.clone(),
                        other.expression.clone(),
                        lineage.clone(),
                    );
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
                let lhs = self.rewrite(instance, &source, &local, &[at])?;
                let rhs = self.rewrite(instance, &target, &local, &[at])?;
                let sa = symbol_reference(&lhs)
                    .ok_or_else(|| invalid(at, "connection scalar symbol"))?;
                let sb = symbol_reference(&rhs)
                    .ok_or_else(|| invalid(at, "connection scalar symbol"))?;
                if self.model.symbols[&sa].ty != self.model.symbols[&sb].ty {
                    return Err(invalid(at, "connection physical types differ"));
                }
                self.model.equations.push(Row {
                    id,
                    equation: Equation {
                        kind: EquationKind::Relation {
                            lhs,
                            sense: EquationSense::Eq,
                            rhs,
                        },
                        span: Span::default(),
                    },
                    lineage: lineage.clone(),
                });
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
        tolerance: Value,
        lhs: Expr,
        rhs: Expr,
        lineage: Lineage,
    ) {
        self.model.closures.insert(
            id,
            Closure {
                id,
                mode: Mode::Conservation,
                observation_only: true,
                ty: Self::difference_type(&ty),
                boundary: None,
                tolerance,
                terms: vec![Contribution {
                    id: pse_ids::named_id(id, "difference"),
                    role: Role::Positive,
                    expression: Expr {
                        kind: ExprKind::Binary {
                            op: BinaryOp::Sub,
                            lhs: Box::new(lhs),
                            rhs: Box::new(rhs),
                        },
                        span: Span::default(),
                    },
                    lineage: lineage.clone(),
                }],
                lineage,
            },
        );
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
            for (row, tolerance) in specification.reconstructions {
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
                let ty = self.process_type(&lhs, row.lineage.declaration)?;
                self.observation_closure(
                    pse_ids::named_id(row.id, "consistency"),
                    ty,
                    tolerance,
                    lhs,
                    rhs,
                    row.lineage,
                );
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
            .map_err(|error| invalid(at, error.to_string()))?;
        let integrated = self.model.integrated.contains_key(&mesh.id);
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
            let inventory = self.rewrite(instance, &inventory_source, &local, &[at])?;
            let flux = self.rewrite(instance, &flux_source, &local, &[at])?;
            let ty = self.process_type(&inventory, at)?;
            let tolerance = self.process_value(
                at,
                "inventory_balance.tolerance",
                0,
                &local,
                Some(&self.tolerance_type(&ty, at)?),
            )?;
            if tolerance.scalar(at)? <= 0.0 {
                return Err(invalid(
                    at,
                    "positive physical inventory tolerance required",
                ));
            }
            let lineage = self.lineage(instance, row, &[at]);
            let mut transfers = BTreeMap::new();
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
                let expression = self.rewrite(
                    instance,
                    &self
                        .p
                        .expression_at(at, "inventory_balance.transfers.expression", position)?
                        .clone(),
                    &local,
                    &[at],
                )?;
                if self.process_type(&expression, at)? != self.tolerance_type(&ty, at)? {
                    return Err(invalid(at, "event transfer inventory convention differs"));
                }
                if transfers.insert(event, expression).is_some() {
                    return Err(invalid(at, "duplicate permitted event transfer"));
                }
            }
            let derivative_source = Expr {
                kind: ExprKind::Derivative {
                    body: Box::new(inventory_source.clone()),
                    wrt: Path {
                        segments: vec![PathSegment {
                            name: binder.clone(),
                            indices: vec![],
                        }],
                    },
                },
                span: Span::default(),
            };
            let source_equation = Equation {
                kind: EquationKind::Relation {
                    lhs: derivative_source,
                    sense: EquationSense::Eq,
                    rhs: flux_source.clone(),
                },
                span: Span::default(),
            };
            let expanded_inventory =
                self.inline_evidence_members(&inventory, &mut BTreeSet::new(), 0)?;
            // An independently authored differential system may expose a composite conserved
            // observation. Never add a dependent stock/rate system on top of that system.
            let existing_dynamics = integrated
                && expanded_inventory.free_paths().iter().any(|path| {
                    let name = dsl::render_path(path);
                    self.model
                        .derivatives
                        .keys()
                        .any(|id| name == symbol_name(*id))
                });
            let mut differential_state = None;
            if !existing_dynamics && self.equation_defined(&source_equation, &local)? {
                let body = if integrated
                    && symbol_reference(&inventory)
                        .is_none_or(|symbol| self.model.symbols[&symbol].role != Kind::Variable)
                {
                    let state = pse_ids::named_id(id, "inventory-state");
                    if !self.model.symbols.contains_key(&state) {
                        let mut initial = expanded_inventory.clone();
                        let substitutions = &self.inventory_initial_sources;
                        let mut replaced = BTreeSet::new();
                        let mut coordinate_starts = BTreeMap::new();
                        initial.try_walk_mut(|node| -> Result<()> {
                            let Some(symbol)=symbol_reference(node) else{return Ok(());};
                            if self.model.symbols.get(&symbol).is_some_and(|s|s.role==Kind::Variable) {
                                let (source,value,source_lineage)=substitutions.get(&symbol).ok_or_else(||invalid(at,"composite inventory requires explicit initial conditions for its coordinates"))?;
                                coordinate_starts.insert(symbol, (*source, value.clone(), source_lineage.clone()));
                                replaced.insert(*source); *node=value.clone();
                            }
                            Ok(())
                        })?;
                        for (target, (source, expression, source_lineage)) in coordinate_starts {
                            self.model.inventory_initial_conditions.insert(
                                target,
                                InventoryInitialCondition {
                                    row: source,
                                    lineage: source_lineage.clone(),
                                },
                            );
                            // Before lowering, the authored initial row took precedence over
                            // a start guess. Preserve that meaning and its original attribution.
                            self.model.annotations.retain(|annotation| {
                                !(annotation.target == target
                                    && matches!(
                                        annotation.value,
                                        crate::annotation::AnnotationValue::Start(_)
                                    ))
                            });
                            self.model.annotations.push(crate::annotation::Annotation {
                                target,
                                value: crate::annotation::AnnotationValue::Start(expression),
                                lineage: source_lineage,
                            });
                        }
                        self.model.annotations.push(crate::annotation::Annotation {
                            target: state,
                            value: crate::annotation::AnnotationValue::Start(initial.clone()),
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
                        self.model.equations.push(Row {
                            id: initial_row,
                            equation: Equation {
                                kind: EquationKind::Relation {
                                    lhs: symbol_expr(state),
                                    sense: EquationSense::Eq,
                                    rhs: initial,
                                },
                                span: Span::default(),
                            },
                            lineage: lineage.clone(),
                        });
                        self.model.symbols.insert(
                            state,
                            Symbol {
                                id: state,
                                ty: ty.clone(),
                                role: Kind::Variable,
                                domain: Domain::Continuous,
                                expression: None,
                                initial: None,
                                lineage: lineage.clone(),
                            },
                        );
                        self.model.equations.push(Row {
                            id: pse_ids::named_id(id, "inventory-definition"),
                            equation: Equation {
                                kind: EquationKind::Relation {
                                    lhs: symbol_expr(state),
                                    sense: EquationSense::Eq,
                                    rhs: inventory.clone(),
                                },
                                span: Span::default(),
                            },
                            lineage: lineage.clone(),
                        });
                    }
                    symbol_expr(state)
                } else {
                    inventory_source.clone()
                };
                let derivative = self.derivative(
                    instance,
                    &body,
                    &Path {
                        segments: vec![PathSegment {
                            name: binder.clone(),
                            indices: vec![],
                        }],
                    },
                    &local,
                    &[at],
                    Some(&self.process_type(&flux, at)?),
                )?;
                if integrated {
                    differential_state = self
                        .model
                        .derivatives
                        .values()
                        .find(|d| symbol_expr(d.rate) == derivative)
                        .map(|d| d.state);
                }
                self.model.equations.push(Row {
                    id: pse_ids::named_id(member_id(instance, at, &coordinates), "inventory-rate"),
                    equation: Equation {
                        kind: EquationKind::Relation {
                            lhs: derivative,
                            sense: EquationSense::Eq,
                            rhs: flux.clone(),
                        },
                        span: Span::default(),
                    },
                    lineage: lineage.clone(),
                });
            }
            if integrated || local.get(&binder) == Some(&last) {
                let integral = self.integral(
                    instance,
                    &dsl::Binder {
                        var: binder.clone(),
                        domain: domain.clone(),
                        filter: None,
                    },
                    &flux_source,
                    &local,
                    &[at],
                    Some(&self.tolerance_type(&ty, at)?),
                )?;
                if integrated {
                    let old = symbol_reference(&integral)
                        .ok_or_else(|| invalid(at, "original flux integral missing"))?;
                    if old != flux_id {
                        let mut declared = self
                            .model
                            .integrals
                            .remove(&old)
                            .ok_or_else(|| invalid(at, "flux quadrature missing"))?;
                        declared.result = flux_id;
                        self.model.integrals.insert(flux_id, declared);
                        let mut symbol = self
                            .model
                            .symbols
                            .remove(&old)
                            .ok_or_else(|| invalid(at, "flux integral symbol missing"))?;
                        symbol.id = flux_id;
                        self.model.symbols.insert(flux_id, symbol);
                    }
                } else {
                    let mut baseline = local.clone();
                    baseline.insert(binder.clone(), first.clone());
                    let replica = self.replica(instance, local[&binder].identity());
                    let initial = self.shifted(
                        instance,
                        &inventory_source,
                        &baseline,
                        &binder,
                        &first,
                        replica.as_ref(),
                        &[at],
                    )?;
                    let rhs = Expr {
                        kind: ExprKind::Binary {
                            op: BinaryOp::Add,
                            lhs: Box::new(initial),
                            rhs: Box::new(integral),
                        },
                        span: Span::default(),
                    };
                    self.observation_closure(
                        pse_ids::named_id(id, "temporal-closure"),
                        ty.clone(),
                        tolerance.clone(),
                        inventory.clone(),
                        rhs,
                        lineage.clone(),
                    );
                }
                self.model.inventory_balances.insert(
                    id,
                    InventoryBalance {
                        id,
                        inventory_id,
                        flux_id,
                        state: differential_state,
                        inventory,
                        flux,
                        ty,
                        tolerance,
                        transfers,
                        lineage,
                    },
                );
            }
        }
        Ok(())
    }
}
