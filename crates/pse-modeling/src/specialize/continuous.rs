// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::rewrite::RewriteContext;
use super::*;
use crate::continuous::Mesh;
use crate::resolved::{Source, construction::Rewritten};

impl Engine<'_, '_> {
    pub(super) fn realize_domains(
        &mut self,
        instance: InstanceId,
        path: &str,
        members: &BTreeMap<String, DeclarationId>,
        env: &mut Environment,
    ) -> Result<()> {
        let mut policies = BTreeMap::new();
        for member in members.values() {
            if let Some(policy) = &self.p.declarations[member].value.discretization {
                let target = self
                    .p
                    .resolve(*member, &policy.target)
                    .ok_or_else(|| invalid(*member, "mesh target absent"))?;
                if policies.insert(target, (*member, policy.clone())).is_some() {
                    return Err(invalid(*member, "competing discretizations for one domain"));
                }
            }
        }
        for member in members.values() {
            let row = self.p.declarations[member].clone();
            let Some(_axis) = &row.value.continuous else {
                continue;
            };
            let Type::Continuous(_, ty) = &self.p.types[member] else {
                return Err(invalid(*member, "continuous physical contract absent"));
            };
            let ty = *ty.clone();
            let lower = self.eval_field(*member, env, "continuous.lower", 0, Some(&ty))?;
            let upper = self.eval_field(*member, env, "continuous.upper", 0, Some(&ty))?;
            let a = lower.scalar(*member)?;
            let b = upper.scalar(*member)?;
            if !a.is_finite() || !b.is_finite() || a >= b || !(b - a).is_finite() {
                return Err(invalid(
                    *member,
                    "continuous interval must be finite and increasing",
                ));
            }
            let (policy_id, policy) = policies.get(member).ok_or_else(|| {
                invalid(
                    *member,
                    "continuous domain needs an explicit discretization",
                )
            })?;
            let count = self.eval_field(
                *policy_id,
                env,
                "discretization.elements",
                0,
                Some(&Type::Integer),
            )?;
            let order = self.eval_field(
                *policy_id,
                env,
                "discretization.order",
                0,
                Some(&Type::Integer),
            )?;
            let (Value::Integer(count), Value::Integer(order)) = (count, order) else {
                return Err(invalid(*policy_id, "exact integer mesh sizes required"));
            };
            let count = usize::try_from(count)
                .ok()
                .filter(|n| *n > 0)
                .ok_or_else(|| invalid(*policy_id, "positive element count required"))?;
            let order = usize::try_from(order)
                .ok()
                .filter(|n| *n > 0)
                .ok_or_else(|| invalid(*policy_id, "positive interpolation order required"))?;
            if policy.scheme == "stationary" {
                if count != 1
                    || order != 1
                    || env.get(&crate::analysis::Fact::Dynamic.path())
                        != Some(&Value::Boolean(false))
                {
                    return Err(invalid(
                        *policy_id,
                        "stationary realization requires the steady route, elements=1 and order=1",
                    ));
                }
                let quantity = crate::temporal::time_quantity(&ty, self.c, *member)?;
                let id = member_id(instance, *member, &[]);
                let point = Value::Coordinate {
                    id: pse_ids::named_id(id, "stationary-coordinate"),
                    bits: a.to_bits(),
                    quantity,
                };
                self.reserve(1)?;
                self.model.meshes.insert(
                    id,
                    Mesh {
                        id,
                        points: vec![point.clone()],
                        derivative: vec![vec![]],
                        integral: vec![0.],
                        continuity: vec![],
                    },
                );
                env.insert(row.name.clone(), Value::Set(vec![point]));
                continue;
            }
            if policy.scheme == "integrated" {
                if count != 1 || order != 1 {
                    return Err(invalid(
                        *policy_id,
                        "integrated realization requires elements=1 and order=1; native integration controls belong to the analysis",
                    ));
                }
                let Type::Quantity(scheme) = &ty else {
                    return Err(invalid(
                        *member,
                        "integrated axis must be a physical time coordinate",
                    ));
                };
                let quantity = scheme
                    .resolve_with_evidence(
                        self.c.quantities,
                        &BTreeMap::new(),
                        self.c.preconditions,
                    )
                    .map_err(|e| invalid(*member, e.to_string()))?;
                let q = self
                    .c
                    .quantities
                    .quantity_type(quantity)
                    .map_err(|e| invalid(*member, e.to_string()))?;
                let unit = self
                    .c
                    .quantities
                    .unit(q.canonical_unit)
                    .map_err(|e| invalid(*member, e.to_string()))?;
                if unit.dimension
                    != pse_quantity::DimensionVector::base(pse_quantity::BaseDimension::Time)
                    || unit.is_affine
                {
                    return Err(invalid(
                        *member,
                        "integrated axis must have a non-affine time unit",
                    ));
                }
                if !self.model.integrated.is_empty() {
                    return Err(invalid(
                        *member,
                        "one integrated time axis per selected model is required",
                    ));
                }
                let id = member_id(instance, *member, &[]);
                let coordinate = pse_ids::named_id(id, "integrated-coordinate");
                let time = pse_ids::named_id(id, "time-value");
                let point = Value::Coordinate {
                    id: coordinate,
                    bits: a.to_bits(),
                    quantity,
                };
                self.model.integrated.insert(
                    id,
                    crate::continuous::IntegratedAxis {
                        id,
                        coordinate,
                        time,
                        quantity,
                        lower: a,
                        upper: b,
                    },
                );
                self.model.symbols.insert(
                    time,
                    Symbol {
                        definition: None,
                        id: time,
                        ty: ty.clone(),
                        role: Kind::Parameter,
                        domain: Domain::Continuous,
                        expression: None,
                        initial: Some(Value::Number {
                            bits: a.to_bits(),
                            quantity,
                        }),
                        lineage: Lineage {
                            declaration: *member,
                            instance,
                            path: format!("{path}.{}", row.name),
                            demand: vec![*member],
                            default_owner: row
                                .parent_id
                                .filter(|id| self.p.declarations[id].value.kind == Kind::Interface),
                            is_override: row.is_override,
                            presets: self.preset_stack.clone(),
                        },
                    },
                );
                self.model.meshes.insert(
                    id,
                    Mesh {
                        id,
                        points: vec![point.clone()],
                        derivative: vec![vec![]],
                        integral: vec![0.],
                        continuity: vec![],
                    },
                );
                env.insert(row.name.clone(), Value::Set(vec![point]));
                self.reserve(3)?;
                continue;
            }
            if count
                .checked_mul(order + 1)
                .is_none_or(|n| n > self.limits.members)
            {
                return Err(ModelingError::Budget("continuous mesh extent".into()));
            }
            let scheme = crate::continuous::scheme(self.p, *policy_id, &policy.scheme)?;
            let stencil = self.discretizer.element(scheme, order, *policy_id)?;
            stencil.validate(*policy_id)?;
            if stencil.nodes.last() != Some(&1.0) {
                return Err(invalid(*policy_id, "element must include both boundaries"));
            }
            let width = (b - a) / count as f64;
            let stride = stencil.nodes.len() - 1;
            let extent = count
                .checked_mul(stride)
                .and_then(|n| n.checked_add(1))
                .filter(|n| *n <= self.limits.members)
                .ok_or_else(|| ModelingError::Budget("continuous mesh extent".into()))?;
            self.reserve(extent)?;
            let Type::Quantity(scheme) = ty else {
                return Err(invalid(*member, "physical axis required"));
            };
            let quantity = scheme
                .resolve_with_evidence(self.c.quantities, &BTreeMap::new(), self.c.preconditions)
                .map_err(|e| invalid(*member, e.to_string()))?;
            let id = member_id(instance, *member, &[]);
            let mut mesh = Mesh {
                id,
                points: Vec::with_capacity(extent),
                derivative: vec![vec![]; extent],
                integral: vec![0.0; extent],
                continuity: Vec::new(),
            };
            for element in 0..count {
                self.checkpoint()?;
                if let Some(weights) = &stencil.endpoint {
                    mesh.continuity.push((
                        (element + 1) * stride,
                        weights
                            .iter()
                            .enumerate()
                            .map(|(j, w)| (element * stride + j, *w))
                            .collect(),
                    ));
                }
                for (local, node) in stencil.nodes.iter().enumerate() {
                    let index = element * stride + local;
                    if mesh.points.len() == index {
                        let mut h = FramedHasher::new(pse_ids::Frame::ModelingMeshCoordinateV1);
                        h.id(&id)
                            .id(&policy_id.as_id())
                            .id(&self
                                .p
                                .resolve(*policy_id, &policy.scheme)
                                .ok_or_else(|| invalid(*policy_id, "scheme absent"))?
                                .as_id())
                            .u64(count as u64)
                            .u64(order as u64)
                            .u64(index as u64);
                        for node in &stencil.nodes {
                            h.u64(node.to_bits());
                        }
                        let value = if index == extent - 1 {
                            b
                        } else {
                            a + width * (element as f64 + node)
                        };
                        mesh.points.push(Value::Coordinate {
                            id: h.finish_id(),
                            bits: value.to_bits(),
                            quantity,
                        });
                    }
                    mesh.integral[index] += width * stencil.integral[local];
                    if !stencil.derivative[local].is_empty() {
                        // At a shared boundary the scheme determines the owning element.
                        mesh.derivative[index] = stencil.derivative[local]
                            .iter()
                            .map(|(j, w)| (element * stride + j, w / width))
                            .collect();
                    }
                }
            }
            if let Some(weights) = &stencil.lattice {
                let spacing = width / stride as f64;
                for (i, derivative) in mesh.derivative.iter_mut().enumerate().take(extent) {
                    let row = weights
                        .iter()
                        .map(|(offset, weight)| {
                            let j = i.checked_add_signed(*offset as isize)?;
                            (j < extent).then_some((j, *weight / spacing))
                        })
                        .collect::<Option<Vec<_>>>();
                    *derivative = row.unwrap_or_default();
                }
            }
            env.insert(row.name.clone(), Value::Set(mesh.points.clone()));
            self.model.meshes.insert(id, mesh);
        }
        Ok(())
    }

    pub(super) fn coordinate_mesh(&self, value: &Value) -> Result<(&Mesh, usize)> {
        let Value::Coordinate { id, .. } = value else {
            return Err(invalid(
                SemanticId::NIL,
                "derivative requires a realized continuous coordinate",
            ));
        };
        self.model
            .meshes
            .values()
            .find_map(|mesh| {
                mesh.points
                    .iter()
                    .position(|p| p.identity() == *id)
                    .map(|i| (mesh, i))
            })
            .ok_or_else(|| invalid(*id, "coordinate mesh absent"))
    }

    /// Distinguish an explicit lower-endpoint constraint from a relation applying
    /// throughout time before coordinate references are reduced to scalar symbols.
    pub(super) fn initial_equation(
        &self,
        instance: InstanceId,
        at: DeclarationId,
        equation: &Equation,
        env: &Environment,
    ) -> Result<bool> {
        if self.model.integrated.is_empty() {
            return Ok(false);
        }
        let (lhs, rhs) = match &equation.kind {
            EquationKind::Relation { lhs, rhs, .. } => (lhs, rhs),
            EquationKind::Conditional {
                guard,
                then,
                otherwise,
            } => {
                let selected = Evaluator {
                    lexical_bases: Some(&self.lexical_bases),
                    package: self.p,
                    physical: self.c,
                    at,
                    env,
                    limit: self.limits.members,
                    stack: vec![],
                    reader: self.reader,
                    selections: Some(&self.selection_collector),
                }
                .predicate(guard)?;
                return self.initial_equation(
                    instance,
                    at,
                    if selected { then } else { otherwise },
                    env,
                );
            }
        };
        let mut endpoint = false;
        let mut varying = false;
        for path in lhs.paths().into_iter().chain(rhs.paths()) {
            if let Some(Value::Coordinate { id, .. }) = env.get(&dsl::render_path(path))
                && self.model.integrated.values().any(|a| a.coordinate == *id)
            {
                varying = true;
            }
            if self.resolve_path(instance, at, path, env, false).is_err() {
                continue;
            }
            let mut owner = instance;
            for (position, segment) in path.segments.iter().enumerate() {
                if segment.name == "parent" && segment.indices.is_empty() {
                    owner = self.states[&owner]
                        .parent
                        .ok_or_else(|| invalid(at, "root has no parent"))?;
                    continue;
                }
                let member = *self.states[&owner]
                    .members
                    .get(&segment.name)
                    .ok_or_else(|| invalid(at, "integrated member absent"))?;
                let values = segment
                    .indices
                    .iter()
                    .map(|index| self.eval_ast_with(at, env, index, None))
                    .collect::<Result<Vec<_>>>()?;
                let coordinates = self.member_coordinates(owner, member, values.clone())?;
                for (value, (_, coordinate)) in values.iter().zip(&coordinates) {
                    let Value::Coordinate { id, .. } = coordinate else {
                        continue;
                    };
                    if !self.model.integrated.values().any(|a| a.coordinate == *id) {
                        continue;
                    }
                    match value {
                        Value::Number { .. } => endpoint = true,
                        Value::Coordinate { .. } => varying = true,
                        _ => return Err(invalid(instance, "integrated coordinate selection")),
                    }
                }
                if position + 1 < path.segments.len() {
                    owner = self.child_instance(owner, at, segment, env)?;
                }
            }
        }
        if endpoint && varying {
            return Err(invalid(
                instance,
                "a time equation cannot mix initial-endpoint and evolving state references",
            ));
        }
        Ok(endpoint)
    }

    pub(super) fn equation_defined(&self, equation: &Equation, env: &Environment) -> Result<bool> {
        let mut defined = true;
        let mut check = |e: &Expr| {
            e.walk(|e| {
                if let ExprKind::Derivative { wrt, .. } = &e.kind
                    && let Some(value) = env.get(&dsl::render_path(wrt))
                    && let Ok((mesh, index)) = self.coordinate_mesh(value)
                {
                    defined &= self.model.integrated.contains_key(&mesh.id)
                        || !mesh.derivative[index].is_empty();
                }
            });
        };
        match &equation.kind {
            EquationKind::Relation { lhs, rhs, .. } => {
                check(lhs);
                check(rhs);
            }
            EquationKind::Conditional {
                then, otherwise, ..
            } => {
                return Ok(
                    self.equation_defined(then, env)? && self.equation_defined(otherwise, env)?
                );
            }
        }
        Ok(defined)
    }

    pub(super) fn derivative(
        &mut self,
        rewrite: RewriteContext<'_>,
        body: &Expr,
        wrt: &Path,
        boundary: Option<&Type>,
        source: Source,
    ) -> Result<Rewritten> {
        let RewriteContext {
            instance,
            env,
            chain,
        } = rewrite;
        let at = chain
            .last()
            .copied()
            .unwrap_or(self.states[&instance].definition);
        let name = dsl::render_path(wrt);
        let value = env
            .get(&name)
            .ok_or_else(|| invalid(at, "derivative coordinate is not bound"))?;
        let (mesh, index) = self.coordinate_mesh(value)?;
        if let Some(axis) = self.model.integrated.get(&mesh.id).cloned() {
            let original = self.rewrite(instance, body, env, chain)?;
            return self.integrated_derivative_of(
                instance,
                &original,
                &axis,
                boundary,
                (at, source),
                chain,
            );
        }
        let points = mesh.points.clone();
        let mesh_id = mesh.id;
        let continuity = mesh.continuity.clone();
        let weights = mesh.derivative[index].clone();
        if weights.is_empty() {
            return Err(invalid(
                at,
                "derivative is undefined at this scheme boundary",
            ));
        }
        let replica = self.replica(instance, value.identity());
        let Type::Quantity(axis) = value_type(value).ok_or_else(|| invalid(at, "axis type"))?
        else {
            return Err(invalid(at, "axis type"));
        };
        let scalar = self
            .c
            .quantities
            .neutral_dimensionless()
            .ok_or_else(|| invalid(at, "neutral scalar type"))?;
        let coefficient = Type::Quantity(pse_quantity::scheme::Scheme::Quotient(
            Box::new(pse_quantity::scheme::Scheme::Concrete(scalar)),
            Box::new(pse_quantity::scheme::Scheme::Delta(Box::new(axis))),
        ));
        let baseline = self.rewrite(instance, body, env, chain)?;
        let mut hash = FramedHasher::new(pse_ids::Frame::ModelingContinuityV2);
        match &replica {
            Some(replica) => {
                hash.id(&replica.owner.as_id()).str(&replica.name);
                for (position, id) in replica.key.iter().enumerate() {
                    if position != replica.position {
                        hash.id(id);
                    }
                }
                for (name, key) in &replica.descent {
                    hash.str(name);
                    for id in key {
                        hash.id(id);
                    }
                }
            }
            None => {
                hash.id(&instance.as_id());
            }
        }
        hash.id(&at.as_id()).id(&mesh_id);
        // One continuity family spans the active mesh coordinate. Construct its
        // actual typed body at one mesh point using the same coordinate/replica
        // substitution as the stencil. Other bindings and fixed point references
        // retain their original identities.
        let family = self.shifted(
            RewriteContext { instance, env, chain },
            body,
            &name,
            &points[0],
            replica.as_ref(),
        )?;
        self.frame_current_roots(&[family.root], &mut hash)?;
        for (coordinate, value) in env {
            if coordinate != &name
                && let Value::Coordinate { id, .. } = value
            {
                hash.str(coordinate).id(id);
            }
        }
        let key = hash.finish_id();
        if self.continuity_done.insert(key) {
            for (element, (endpoint, coefficients)) in continuity.into_iter().enumerate() {
                self.reserve(1)?;
                let endpoint = self.shifted(
                    RewriteContext {
                        instance,
                        env,
                        chain,
                    },
                    body,
                    &name,
                    &points[endpoint],
                    replica.as_ref(),
                )?;
                let rhs = self.generated_binary(
                    BinaryOp::Sub,
                    endpoint,
                    baseline.clone(),
                    source.clone(),
                )?;
                let mut terms = Vec::new();
                for (point, weight) in coefficients {
                    let shifted = self.shifted(
                        RewriteContext {
                            instance,
                            env,
                            chain,
                        },
                        body,
                        &name,
                        &points[point],
                        replica.as_ref(),
                    )?;
                    let delta = self.generated_binary(
                        BinaryOp::Sub,
                        shifted,
                        baseline.clone(),
                        source.clone(),
                    )?;
                    let factor = self.constant(
                        weight,
                        &Type::Quantity(pse_quantity::scheme::Scheme::Concrete(scalar)),
                        at,
                        source.clone(),
                    )?;
                    terms.push(self.generated_binary(
                        BinaryOp::Mul,
                        factor,
                        delta,
                        source.clone(),
                    )?);
                }
                let lhs = self.generated_sum(terms, source.clone())?;
                let equation = Self::generated_relation(lhs, EquationSense::Eq, rhs, source.span);
                let id = self.model.derivations.publish(
                    crate::derived::Relationship::ContinuityRow {
                        family: key,
                        element: element as u64,
                    },
                )?;
                self.retain_row(
                    id,
                    equation,
                    self.lineage(instance, &self.p.declarations[&at], chain),
                )?;
            }
        }
        let mut terms = Vec::new();
        for (point, weight) in weights {
            let shifted = self.shifted(
                RewriteContext {
                    instance,
                    env,
                    chain,
                },
                body,
                &name,
                &points[point],
                replica.as_ref(),
            )?;
            let delta =
                self.generated_binary(BinaryOp::Sub, shifted, baseline.clone(), source.clone())?;
            let factor = self.constant(weight, &coefficient, at, source.clone())?;
            terms.push(self.generated_binary(BinaryOp::Mul, factor, delta, source.clone())?);
        }
        self.generated_sum(terms, source)
    }
    pub(super) fn integrated_derivative_of(
        &mut self,
        instance: InstanceId,
        original: &Rewritten,
        axis: &crate::continuous::IntegratedAxis,
        boundary: Option<&Type>,
        origin: (DeclarationId, Source),
        chain: &[DeclarationId],
    ) -> Result<Rewritten> {
        let (at, source) = origin;
        let state = self
            .symbol_of_root(original.root)
            .ok_or_else(|| invalid(at, "integrated derivative must select a scalar variable"))?;
        let symbol = self.model.symbols[&state].clone();
        if symbol.role != Kind::Variable || symbol.definition.is_some() {
            return Err(invalid(
                at,
                "integrated state must be an independent variable",
            ));
        }
        if let Some(existing) = self.model.derivatives.get(&state) {
            if existing.axis != axis.id {
                return Err(invalid(at, "state derivative has conflicting time axes"));
            }
            return self.selected_scalar(existing.rate, source);
        }
        let Type::Quantity(state_ty) = symbol.ty else {
            return Err(invalid(at, "integrated state physical type"));
        };
        let scheme = pse_quantity::scheme::Scheme::Quotient(
            Box::new(pse_quantity::scheme::Scheme::Delta(Box::new(state_ty))),
            Box::new(pse_quantity::scheme::Scheme::Delta(Box::new(
                pse_quantity::scheme::Scheme::Concrete(axis.quantity),
            ))),
        );
        let quantity =
            self.integrated_quantity_boundary(&scheme, boundary, at, "integrated derivative")?;
        let rate = self
            .model
            .derivations
            .publish(crate::derived::Relationship::IntegratedRateCoordinate { state })?;
        let lineage = self.lineage(instance, &self.p.declarations[&at], chain);
        self.model.symbols.insert(
            rate,
            Symbol {
                definition: None,
                id: rate,
                ty: Type::Quantity(pse_quantity::scheme::Scheme::Concrete(quantity)),
                role: Kind::Variable,
                domain: Domain::Continuous,
                expression: None,
                initial: None,
                lineage: lineage.clone(),
            },
        );
        self.model.derivatives.insert(
            state,
            crate::continuous::IntegratedDerivative {
                state,
                rate,
                axis: axis.id,
                lineage,
            },
        );
        self.reserve(2)?;
        self.selected_scalar(rate, source)
    }
    /// The same original body is constructed at the actual shifted coordinate.
    pub(super) fn shifted(
        &mut self,
        rewrite: RewriteContext<'_>,
        body: &Expr,
        name: &str,
        point: &Value,
        replica: Option<&Replica>,
    ) -> Result<Rewritten> {
        let RewriteContext {
            instance,
            env,
            chain,
        } = rewrite;
        let Some(replica) = replica else {
            let mut local = env.clone();
            local.insert(name.to_owned(), point.clone());
            return self.rewrite(instance, body, &local, chain);
        };
        let sibling = self.replica_at(replica, point.identity())?;
        let own = &self.states[&instance].env;
        let mut local = self.states[&sibling].env.clone();
        for (key, value) in env {
            if own.get(key) != Some(value) {
                local.insert(key.clone(), value.clone());
            }
        }
        self.rewrite(sibling, body, &local, chain)
    }

    /// The indexed child occurrence that replicated `instance`, or its nearest such
    /// ancestor, at the continuous coordinate `coordinate`.
    pub(super) fn replica(&self, instance: InstanceId, coordinate: SemanticId) -> Option<Replica> {
        let mut descent = Vec::new();
        let mut current = instance;
        while let Some(owner) = self.states.get(&current).and_then(|s| s.parent) {
            let (name, key) = self.states[&owner]
                .children
                .iter()
                .find(|(_, child)| **child == current)
                .map(|(key, _)| key.clone())?;
            if let Some(position) = key.iter().position(|id| *id == coordinate) {
                descent.reverse();
                return Some(Replica {
                    owner,
                    name,
                    key,
                    position,
                    descent,
                });
            }
            descent.push((name, key));
            current = owner;
        }
        None
    }
    /// The instance corresponding to a replica's instance in the replica at `point`.
    fn replica_at(&self, replica: &Replica, point: SemanticId) -> Result<InstanceId> {
        let absent = || {
            invalid(
                self.states[&replica.owner].definition,
                "replica at a stencil coordinate is absent",
            )
        };
        let mut key = replica.key.clone();
        key[replica.position] = point;
        let mut current = *self.states[&replica.owner]
            .children
            .get(&(replica.name.clone(), key))
            .ok_or_else(absent)?;
        for step in &replica.descent {
            current = *self
                .states
                .get(&current)
                .and_then(|s| s.children.get(step))
                .ok_or_else(absent)?;
        }
        Ok(current)
    }
    /// Whether an equation differentiates along a meshed coordinate at which its instance
    /// was replicated; such stencils read sibling replicas that may not exist yet.
    pub(super) fn differentiates_replicas(
        &self,
        instance: InstanceId,
        equation: &Equation,
        env: &Environment,
    ) -> bool {
        let mut found = false;
        let mut visit = |e: &Expr| {
            e.walk(|e| {
                if let ExprKind::Derivative { wrt, .. } = &e.kind
                    && let Some(value) = env.get(&dsl::render_path(wrt))
                    && let Ok((mesh, _)) = self.coordinate_mesh(value)
                    && !self.model.integrated.contains_key(&mesh.id)
                    && self.replica(instance, value.identity()).is_some()
                {
                    found = true;
                }
            });
        };
        fn equations<'e>(equation: &'e Equation, out: &mut Vec<&'e Expr>) {
            match &equation.kind {
                EquationKind::Relation { lhs, rhs, .. } => {
                    out.push(lhs);
                    out.push(rhs);
                }
                EquationKind::Conditional {
                    then, otherwise, ..
                } => {
                    equations(then, out);
                    equations(otherwise, out);
                }
            }
        }
        let mut sides = Vec::new();
        equations(equation, &mut sides);
        for side in sides {
            visit(side);
        }
        found
    }

    pub(super) fn integral(
        &mut self,
        rewrite: RewriteContext<'_>,
        binder: &dsl::Binder,
        body: &Expr,
        boundary: Option<&Type>,
        source: Source,
    ) -> Result<Rewritten> {
        let RewriteContext {
            instance,
            env,
            chain,
        } = rewrite;
        let at = chain
            .last()
            .copied()
            .unwrap_or(self.states[&instance].definition);
        if binder.filter.is_some() {
            return Err(invalid(
                at,
                "continuous integrals require an explicit subdomain, not a discontinuous filter",
            ));
        }
        let Value::Set(points) = self.eval_ast_with(
            at,
            env,
            &Expr {
                kind: ExprKind::Path(binder.domain.clone()),
                span: Span::default(),
            },
            None,
        )?
        else {
            return Err(invalid(at, "realized integral domain absent"));
        };
        let first = points
            .first()
            .ok_or_else(|| invalid(at, "empty integral domain"))?;
        let (mesh, _) = self.coordinate_mesh(first)?;
        if let Some(axis) = self.model.integrated.get(&mesh.id).cloned() {
            let mut local = env.clone();
            local.insert(binder.var.clone(), first.clone());
            // Source type inspection precedes construction. No final rewritten AST is inferred.
            let types = self.source_types(at, &local)?;
            let Type::Quantity(value_type) =
                crate::expression::infer(body, &types, self.p, self.c, at, None)?
            else {
                return Err(invalid(at, "physical integrand required"));
            };
            let value_type = value_type
                .resolve_with_evidence(self.c.quantities, &BTreeMap::new(), self.c.preconditions)
                .map_err(|error| invalid(at, error.to_string()))?;
            let integrand = self.rewrite(instance, body, &local, chain)?;
            let scheme = pse_quantity::scheme::Scheme::Product(
                Box::new(pse_quantity::scheme::Scheme::Concrete(value_type)),
                Box::new(pse_quantity::scheme::Scheme::Delta(Box::new(
                    pse_quantity::scheme::Scheme::Concrete(axis.quantity),
                ))),
            );
            let result_type =
                self.integrated_quantity_boundary(&scheme, boundary, at, "integrated integral")?;
            let mut identity = FramedHasher::new(pse_ids::Frame::ModelingDefiniteIntegralV2);
            identity
                .id(&instance.as_id())
                .id(&at.as_id())
                .id(&axis.id)
                .id(&result_type.as_id());
            self.frame_current_roots(&[integrand.root], &mut identity)?;
            let result = self.model.derivations.publish(
                crate::derived::Relationship::ContinuousIntegral {
                    content: identity.finish_hash(),
                },
            )?;
            if self.model.integrals.contains_key(&result) {
                return self.selected_scalar(result, source);
            }
            self.reserve(3)?;
            let flux = self
                .model
                .derivations
                .publish(crate::derived::Relationship::IntegralFlux { result })?;
            let lineage = self.lineage(instance, &self.p.declarations[&at], chain);
            self.model.symbols.insert(
                flux,
                Symbol {
                    definition: Some(integrand.root),
                    id: flux,
                    ty: Type::Quantity(pse_quantity::scheme::Scheme::Concrete(value_type)),
                    role: Kind::Let,
                    domain: Domain::Continuous,
                    expression: Some(integrand.inspection),
                    initial: None,
                    lineage: lineage.clone(),
                },
            );
            self.model.symbols.insert(
                result,
                Symbol {
                    definition: None,
                    id: result,
                    ty: Type::Quantity(pse_quantity::scheme::Scheme::Concrete(result_type)),
                    role: Kind::Parameter,
                    domain: Domain::Continuous,
                    expression: None,
                    initial: None,
                    lineage,
                },
            );
            self.model.integrals.insert(
                result,
                crate::continuous::IntegratedIntegral {
                    result,
                    integrand: flux,
                    axis: axis.id,
                },
            );
            return self.selected_scalar(result, source);
        }
        let weights = mesh.integral.clone();
        let replica = self.replica(instance, first.identity());
        let points = if replica.is_some() {
            mesh.points.clone()
        } else {
            points.clone()
        };
        let Type::Quantity(axis) = value_type(first).ok_or_else(|| invalid(at, "axis type"))?
        else {
            return Err(invalid(at, "axis type"));
        };
        let coefficient = Type::Quantity(pse_quantity::scheme::Scheme::Delta(Box::new(axis)));
        let mut terms = Vec::new();
        for (point, weight) in points.into_iter().zip(weights) {
            if weight == 0.0 {
                continue;
            }
            let mut local = env.clone();
            local.insert(binder.var.clone(), point.clone());
            let value = self.shifted(
                RewriteContext {
                    instance,
                    env: &local,
                    chain,
                },
                body,
                &binder.var,
                &point,
                replica.as_ref(),
            )?;
            let factor = self.constant(weight, &coefficient, at, source.clone())?;
            terms.push(self.generated_binary(BinaryOp::Mul, factor, value, source.clone())?);
        }
        self.generated_sum(terms, source)
    }
    /// Conservation supplies an already checked named physical result boundary.
    /// Generic derivatives and integrals still require their own named result.
    fn integrated_quantity_boundary(
        &self,
        scheme: &pse_quantity::scheme::Scheme,
        boundary: Option<&Type>,
        at: DeclarationId,
        operation: &str,
    ) -> Result<pse_quantity::QuantityTypeId> {
        let actual = scheme
            .resolve_contract_with_evidence(
                self.c.quantities,
                &BTreeMap::new(),
                self.c.preconditions,
            )
            .map_err(|e| invalid(at, e.to_string()))?;
        let Some(Type::Quantity(expected)) = boundary else {
            return actual
                .require_named()
                .map_err(|e| invalid(at, e.to_string()));
        };
        let expected = expected
            .resolve_with_evidence(self.c.quantities, &BTreeMap::new(), self.c.preconditions)
            .map_err(|e| invalid(at, e.to_string()))?;
        let (named, _coordinate_scale) = actual
            .at_boundary(expected, self.c.quantities)
            .map_err(|e| invalid(at, format!("{operation}: {e}")))?;
        // Generated rate variables and integrals use this named canonical unit.
        // The native program's rate/state and flux/inventory unit ratios retain the
        // conversion; multiplying the original physical source again would double it.
        named
            .require_named()
            .map_err(|e| invalid(at, e.to_string()))
    }
}
/// An indexed child occurrence replicating an instance over a continuous coordinate:
/// the owner, the child key with the coordinate's position, and the child keys from the
/// replica down to the replicated instance.
pub(super) struct Replica {
    owner: InstanceId,
    name: String,
    key: Vec<SemanticId>,
    position: usize,
    descent: Vec<(String, Vec<SemanticId>)>,
}
