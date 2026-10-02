// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bind evidence to actual selected records and physical arguments before lowering.
use super::*;
use pse_authoring::dsl::Predicate;
use pse_model::applicability::{Claim, Node, Permission, Region};
use pse_model::generated::enums::{
    ModelingApplicabilityKind as Kind, ModelingPermissionTarget as Target,
};

impl Engine<'_, '_> {
    fn claim_owner_lineage(&self, owner: DeclarationId) -> Result<Vec<SemanticId>> {
        let mut lineage = Vec::new();
        let mut current = Some(owner);
        let mut seen = BTreeSet::new();
        while let Some(id) = current {
            if !seen.insert(id) {
                return Err(invalid(owner, "cyclic scientific owner kind ancestry"));
            }
            lineage.push(id.as_id());
            current = self.p.kinds.get(&id).and_then(|kind| kind.base);
        }
        Ok(lineage)
    }

    fn evidence_permissions(&self, instance: InstanceId) -> Result<Vec<Permission>> {
        let mut permissions = Vec::new();
        let mut current = Some(instance);
        let mut seen = BTreeSet::new();
        while let Some(id) = current {
            let state = &self.states[&id];
            for member in state.members.values() {
                if !seen.insert(*member) {
                    continue;
                }
                let declaration = &self.p.declarations[member];
                let Some(v) = &declaration.value.permission else {
                    continue;
                };
                let mut targets = BTreeSet::new();
                for (position, target) in v.targets.iter().enumerate() {
                    match v.target_kind {
                        Target::Families => {
                            let family = self
                                .p
                                .resolve(*member, target)
                                .ok_or_else(|| invalid(*member, "permission family is absent"))?;
                            if !matches!(
                                self.p.types.get(&family),
                                Some(
                                    Type::Entity(_)
                                        | Type::Definition(_)
                                        | Type::Interface(_)
                                        | Type::Row(_)
                                        | Type::Function { .. }
                                )
                            ) && !self.p.tables.contains_key(&family)
                            {
                                return Err(invalid(
                                    *member,
                                    "permission targets a declared scientific family",
                                ));
                            }
                            targets.insert(family.as_id());
                        }
                        Target::Records => {
                            let value = self.eval_field(
                                *member,
                                &state.env,
                                "permission.targets",
                                position,
                                None,
                            )?;
                            let Value::Entity { id, .. } = value else {
                                return Err(invalid(
                                    *member,
                                    "record permission names individual scientific records",
                                ));
                            };
                            targets.insert(id.as_id());
                        }
                    }
                }
                permissions.push(Permission {
                    id: member.as_id(),
                    scope: declaration.parent_id.unwrap_or(*member).as_id(),
                    target_kind: v.target_kind,
                    targets: targets.into_iter().collect(),
                    allow_unknown: v.allow_unknown,
                    allow_extrapolation: v.allow_extrapolation,
                });
            }
            current = state.parent;
        }
        Ok(permissions)
    }
    pub(super) fn specialize_applicability(
        &mut self,
        instance: InstanceId,
        form: &crate::Function,
        env: &Environment,
    ) -> Result<Vec<crate::applicability::Use>> {
        let permissions = self.evidence_permissions(instance)?;
        let mut uses = Vec::new();
        for application in &form.applicability {
            if let ExprKind::NamedCall { name, args } = &application.kind
                && name.is_ident("applicability_interval")
            {
                if args.len() != 2 {
                    return Err(invalid(
                        form.id,
                        "interval applicability takes exactly two endpoint claim applications",
                    ));
                }
                let (first, last) = (&args[0], &args[1]);
                let ExprKind::NamedCall {
                    name: first_name,
                    args: first_args,
                } = &first.kind
                else {
                    return Err(invalid(
                        form.id,
                        "interval endpoints are named claim applications",
                    ));
                };
                let ExprKind::NamedCall {
                    name: last_name,
                    args: last_args,
                } = &last.kind
                else {
                    return Err(invalid(
                        form.id,
                        "interval endpoints are named claim applications",
                    ));
                };
                let first_claim = self.eval_ast(
                    form.id,
                    env,
                    &Expr {
                        kind: ExprKind::Path(first_name.clone()),
                        span: Span::default(),
                    },
                )?;
                let last_claim = self.eval_ast(
                    form.id,
                    env,
                    &Expr {
                        kind: ExprKind::Path(last_name.clone()),
                        span: Span::default(),
                    },
                )?;
                let Value::Function(claim) = first_claim else {
                    return Err(invalid(
                        form.id,
                        "interval endpoint is not a declared claim",
                    ));
                };
                if last_claim != Value::Function(claim) {
                    return Err(invalid(
                        form.id,
                        "interval endpoints must use the same named claim",
                    ));
                }
                let declared = self.p.declarations[&claim]
                    .value
                    .applicability
                    .as_ref()
                    .ok_or_else(|| invalid(claim, "interval endpoint is not a claim"))?;
                if declared.claim_kind != Kind::Interval {
                    return Err(invalid(
                        claim,
                        "endpoint reduction requires an explicitly declared interval region; arbitrary regions need a whole-interval claim",
                    ));
                }
                let signature = self.p.functions[&claim].clone();
                let axis = signature
                    .arguments
                    .iter()
                    .position(|(name, ty)| {
                        Some(name.as_str()) == declared.axis.as_deref()
                            && ty.quantity_scheme().is_some()
                    })
                    .ok_or_else(|| {
                        invalid(
                            claim,
                            "endpoint reduction requires a single numerical axis formal",
                        )
                    })?;
                if first_args.len() != signature.arguments.len()
                    || last_args.len() != signature.arguments.len()
                {
                    return Err(invalid(claim, "interval endpoint arity differs"));
                }
                for (index, (_, ty)) in signature.arguments.iter().enumerate() {
                    if index == axis {
                        continue;
                    }
                    if ty.quantity_scheme().is_some() {
                        let left = self.rewrite(instance, &first_args[index], env, &[form.id])?;
                        let right = self.rewrite(instance, &last_args[index], env, &[form.id])?;
                        if dsl::render_expr(&left) != dsl::render_expr(&right) {
                            return Err(invalid(
                                claim,
                                "interval endpoints differ on a nonaxis argument",
                            ));
                        }
                    } else if self.eval_ast_with(form.id, env, &first_args[index], Some(ty))?
                        != self.eval_ast_with(form.id, env, &last_args[index], Some(ty))?
                    {
                        return Err(invalid(
                            claim,
                            "interval endpoints use different record or evidence contexts",
                        ));
                    }
                }
                let mut coverage =
                    FramedHasher::new(pse_ids::Frame::ModelingApplicabilityCoverageV1);
                coverage
                    .str("declared-interval-coverage-v1")
                    .id(&form.id.as_id())
                    .id(&claim.as_id())
                    .str(&dsl::render_expr(application));
                let coverage = coverage.finish_id();
                for endpoint in [first, last] {
                    let mut predicates = Vec::new();
                    let mut inputs = Vec::new();
                    let mut node = self.bind_claim(
                        instance,
                        form.id,
                        form.id,
                        endpoint,
                        env,
                        &permissions,
                        &mut predicates,
                        &mut inputs,
                        &mut Vec::new(),
                    )?;
                    node.claim.coverage = Some(coverage);
                    uses.push(crate::applicability::Use {
                        node,
                        predicates,
                        inputs,
                    });
                }
                continue;
            }
            let mut predicates = Vec::new();
            let mut inputs = Vec::new();
            let node = self.bind_claim(
                instance,
                form.id,
                form.id,
                application,
                env,
                &permissions,
                &mut predicates,
                &mut inputs,
                &mut Vec::new(),
            )?;
            uses.push(crate::applicability::Use {
                node,
                predicates,
                inputs,
            });
        }
        // Selection edges are required scientific dependencies. Named fit unions occur
        // only inside claim declarations; distinct reached records are never merged.
        let edges = self
            .selection_collector
            .current_consumer_frame()
            .values()
            .flat_map(|c| c.edges.iter())
            .map(|(a, b)| (a.clone(), b.clone()))
            .collect::<Vec<_>>();
        for usage in &mut uses {
            let mut visited = BTreeSet::new();
            node_records(&usage.node, &mut visited);
            self.selected_dependencies(
                instance,
                form,
                env,
                &permissions,
                &edges,
                &mut visited,
                usage,
            )?;
        }
        Ok(uses)
    }
    /// Attach implicit evidence to the exact numerical read, so inactive branches and
    /// undemanded members never acquire evidence obligations from another path.
    pub(super) fn scientific_value_expression(
        &mut self,
        instance: InstanceId,
        path: &Path,
        value: &Value,
        at: DeclarationId,
        env: &Environment,
    ) -> Result<Expr> {
        let expression = self.value_expression(value, at)?;
        if !matches!(value, Value::Number { .. })
            || path.segments.len() < 2
            || self
                .p
                .functions
                .get(&at)
                .is_some_and(|f| f.result == Type::Applicability)
        {
            return Ok(expression);
        }
        let receiver = Path {
            segments: path.segments[..path.segments.len() - 1].to_vec(),
        };
        let Ok(Value::Entity { id, kind }) = self.eval_ast_with(
            at,
            env,
            &Expr {
                kind: ExprKind::Path(receiver.clone()),
                span: Span::default(),
            },
            None,
        ) else {
            return Ok(expression);
        };
        let selected = self.selection_collector.current_consumer_frame();
        let scientific=selected.values().any(|c|c.records.contains(&Value::Entity {id,kind})) || self.p.entities.get(&id).is_some_and(|record|record.values.values().any(|value|matches!(value,Value::Function(f) if self.p.functions.get(f).is_some_and(|f|f.result==Type::Applicability))));
        if !scientific {
            return Ok(expression);
        }
        let result = value_type(value)
            .ok_or_else(|| invalid(at, "scientific numerical read has no physical type"))?;
        let mut form = crate::Function {
            applicability: Vec::new(),
            applicability_uses: Vec::new(),
            prerequisites: Vec::new(),
            physical_admissions: BTreeMap::new(),
            physical_operation: None,
            reduction: None,
            validity: None,
            envelopes: Vec::new(),
            validity_reads: Default::default(),
            external: None,
            continuity: None,
            id: at,
            variables: BTreeSet::new(),
            arguments: self
                .function_types
                .iter()
                .map(|(n, t)| (n.clone(), t.clone()))
                .collect(),
            result: result.clone(),
            body: None,
        };
        let permissions = self.evidence_permissions(instance)?;
        let mut usage =
            self.selected_record_claim(instance, &form, id.as_id(), env, &permissions)?;
        let edges = selected
            .values()
            .flat_map(|c| c.edges.iter())
            .map(|(a, b)| (a.clone(), b.clone()))
            .collect::<Vec<_>>();
        let mut visited = BTreeSet::new();
        node_records(&usage.node, &mut visited);
        self.selected_dependencies(
            instance,
            &form,
            env,
            &permissions,
            &edges,
            &mut visited,
            &mut usage,
        )?;
        let mut arguments = BTreeMap::new();
        for (name, ty) in &self.function_types {
            if ty.quantity_scheme().is_some()
                && let Some(actual) = self.lexical.get(name)
                && let ExprKind::Path(p) = &actual.kind
            {
                arguments.insert(dsl::render_path(p), (ty.clone(), actual.clone()));
            }
        }
        let mut identity = FramedHasher::new(pse_ids::Frame::ModelingApplicabilityCallV1);
        identity
            .str("demanded-scientific-record-read-v1")
            .id(&instance.as_id())
            .id(&at.as_id())
            .str(&dsl::render_path(path))
            .u64(self.local_serial as u64);
        self.local_serial += 1;
        usage.frame(&mut identity);
        identity.str(&dsl::render_expr(&expression));
        let call = identity.finish_id();
        usage.node.claim.call = call;
        form.id = DeclarationId::from(call);
        form.arguments = arguments
            .iter()
            .map(|(name, (ty, _))| (name.clone(), ty.clone()))
            .collect();
        form.body = Some(expression);
        form.applicability_uses = vec![usage];
        let name = format!("f_{}", call.to_hex());
        self.reserve(1)?;
        self.model.functions.insert(name.clone(), form);
        Ok(Expr {
            kind: ExprKind::NamedCall {
                name: Path::single(name),
                args: arguments.into_values().map(|(_, actual)| actual).collect(),
            },
            span: Span::default(),
        })
    }
    fn selected_record_claim(
        &mut self,
        instance: InstanceId,
        form: &crate::Function,
        record_id: SemanticId,
        env: &Environment,
        permissions: &[Permission],
    ) -> Result<crate::applicability::Use> {
        let id = DeclarationId::from(record_id);
        let record = self
            .p
            .entities
            .get(&id)
            .cloned()
            .ok_or_else(|| invalid(form.id, "selected applicability record absent"))?;
        let candidates = record
            .values
            .values()
            .filter_map(|value| match value {
                Value::Function(id)
                    if self
                        .p
                        .functions
                        .get(id)
                        .is_some_and(|f| f.result == Type::Applicability) =>
                {
                    Some(*id)
                }
                _ => None,
            })
            .collect::<BTreeSet<_>>();
        let mut environment = env.clone();
        let mut expressions = Vec::new();
        let mut function = None;
        if candidates.len() == 1 {
            let claim = *candidates
                .iter()
                .next()
                .ok_or_else(|| invalid(form.id, "claim candidate absent"))?;
            let signature = self.p.functions[&claim].clone();
            let mut available = true;
            for (index, (name, ty)) in signature.arguments.iter().enumerate() {
                if matches!(ty,Type::Entity(kind) if self.p.refines(record.kind,*kind)) {
                    let binding = format!("__applicability_record_{index}");
                    environment.insert(
                        binding.clone(),
                        Value::Entity {
                            id,
                            kind: record.kind,
                        },
                    );
                    expressions.push(Expr {
                        kind: ExprKind::Path(Path::single(binding)),
                        span: Span::default(),
                    });
                } else if self.lexical.contains_key(name) && ty.quantity_scheme().is_some()
                    || environment
                        .get(name)
                        .is_some_and(|v| value::conforms(v, ty, self.p))
                {
                    expressions.push(Expr {
                        kind: ExprKind::Path(Path::single(name.clone())),
                        span: Span::default(),
                    });
                } else {
                    available = false;
                    break;
                }
            }
            if available {
                function = Some(claim);
            }
        }
        let mut predicates = Vec::new();
        let mut inputs = Vec::new();
        let node = if let Some(claim) = function {
            let binding = "__applicability_callable".to_owned();
            environment.insert(binding.clone(), Value::Function(claim));
            let expression = Expr {
                kind: ExprKind::NamedCall {
                    name: Path::single(binding),
                    args: expressions,
                },
                span: Span::default(),
            };
            self.bind_claim(
                instance,
                form.id,
                form.id,
                &expression,
                &environment,
                permissions,
                &mut predicates,
                &mut inputs,
                &mut Vec::new(),
            )?
        } else {
            let mut identity = FramedHasher::new(pse_ids::Frame::ModelingApplicabilityCallV1);
            identity
                .str("missing-selected-record-claim")
                .id(&form.id.as_id())
                .id(&record_id);
            let mut input_indices = Vec::new();
            for (name, ty) in &form.arguments {
                let Some(scheme) = ty.quantity_scheme() else {
                    continue;
                };
                let Some(actual) = self.lexical.get(name).cloned() else {
                    continue;
                };
                let quantity = scheme
                    .resolve_with_evidence(
                        self.c.quantities,
                        &BTreeMap::new(),
                        self.c.preconditions,
                    )
                    .map_err(|e| invalid(form.id, e.to_string()))?;
                identity
                    .str(name)
                    .str(&dsl::render_expr(&actual))
                    .id(&quantity.as_id());
                input_indices.push((name.clone(), inputs.len(), quantity.as_id()));
                inputs.push(actual);
            }
            Node {claim:Claim {id:None,coverage:None,owner:record.kind.as_id(),owner_lineage:self.claim_owner_lineage(record.kind)?,evidence:self.p.provenance(record.origin).map(|p|p.source.as_id()),form:form.id.as_id(),call:identity.finish_id(),records:vec![record_id],dependencies:Vec::new(),layer:pse_model::generated::enums::ModelingValidityLayer::Data,basis:None,reason:Some("Selected scientific record has no claim applicable to this actual form signature".into())},region:Region::Unknown,dependencies:Vec::new(),inputs:input_indices,permissions:permissions.to_vec()}
        };
        Ok(crate::applicability::Use {
            node,
            predicates,
            inputs,
        })
    }
    #[allow(
        clippy::too_many_arguments,
        reason = "selection dependency edges retain their original consumer and admission frame"
    )]
    fn selected_dependencies(
        &mut self,
        instance: InstanceId,
        form: &crate::Function,
        env: &Environment,
        permissions: &[Permission],
        edges: &[(Value, BTreeSet<Value>)],
        visited: &mut BTreeSet<SemanticId>,
        usage: &mut crate::applicability::Use,
    ) -> Result<()> {
        let saved = self.lexical.clone();
        for (name, index, _) in &usage.node.inputs {
            if let Some(input) = usage.inputs.get(*index) {
                self.lexical.insert(name.clone(), input.clone());
            }
        }
        let mut dependencies = Vec::new();
        for record in &usage.node.claim.records {
            for (source, targets) in edges {
                if !matches!(source,Value::Entity {id,..} if id.as_id()==*record) {
                    continue;
                }
                for target in targets {
                    let Value::Entity { id, .. } = target else {
                        continue;
                    };
                    if !visited.insert(id.as_id()) {
                        continue;
                    }
                    let mut dependency =
                        self.selected_record_claim(instance, form, id.as_id(), env, permissions)?;
                    self.selected_dependencies(
                        instance,
                        form,
                        env,
                        permissions,
                        edges,
                        visited,
                        &mut dependency,
                    )?;
                    remap_node(
                        &mut dependency.node,
                        usage.predicates.len(),
                        usage.inputs.len(),
                    );
                    usage.predicates.extend(dependency.predicates);
                    usage.inputs.extend(dependency.inputs);
                    dependencies.push(dependency.node);
                }
            }
        }
        usage.node.claim.dependencies.extend(
            dependencies
                .iter()
                .map(|n| n.claim.id.unwrap_or(n.claim.records[0])),
        );
        usage.node.dependencies.extend(dependencies);
        self.lexical = saved;
        Ok(())
    }
    #[allow(
        clippy::too_many_arguments,
        reason = "one bounded claim binding carries its immutable authorization and numerical capture frame"
    )]
    fn bind_claim(
        &mut self,
        instance: InstanceId,
        form: DeclarationId,
        at: DeclarationId,
        application: &Expr,
        env: &Environment,
        permissions: &[Permission],
        predicates: &mut Vec<Predicate>,
        inputs: &mut Vec<Expr>,
        stack: &mut Vec<DeclarationId>,
    ) -> Result<Node> {
        self.checkpoint()?;
        let ExprKind::NamedCall { name, args } = &application.kind else {
            return Err(invalid(
                at,
                "applicability metadata is a typed named claim application",
            ));
        };
        let value = self
            .eval_ast(
                at,
                env,
                &Expr {
                    kind: ExprKind::Path(name.clone()),
                    span: Span::default(),
                },
            )
            .map_err(|error| {
                invalid(
                    at,
                    format!(
                        "claim callable {name} at {at} resolves {:?}: {error}",
                        self.p.resolve_segments(at, &name.segments)
                    ),
                )
            })?;
        let Value::Function(id) = value else {
            return Err(invalid(
                at,
                "applicability callable is not a named evidence claim",
            ));
        };
        if stack.contains(&id) || stack.len() >= self.limits.depth {
            return Err(invalid(
                id,
                "recursive or excessive applicability dependency expansion",
            ));
        }
        let declaration = self.p.declarations[&id].clone();
        let v =
            declaration.value.applicability.as_ref().ok_or_else(|| {
                invalid(id, "Applicability output belongs to a claim declaration")
            })?;
        let signature = self.p.functions[&id].clone();
        if args.len() != signature.arguments.len() {
            return Err(invalid(id, "claim application argument arity differs"));
        }
        stack.push(id);
        let mut statics = Environment::new();
        let mut lexical = BTreeMap::new();
        let mut input_indices = Vec::new();
        let mut records = BTreeSet::new();
        for ((name, ty), arg) in signature.arguments.iter().zip(args) {
            if ty.quantity_scheme().is_some() {
                let actual = self.rewrite(instance, arg, env, &[at])?;
                let quantity = ty
                    .quantity_scheme()
                    .ok_or_else(|| invalid(id, "claim input is not physical"))?
                    .resolve_with_evidence(
                        self.c.quantities,
                        &BTreeMap::new(),
                        self.c.preconditions,
                    )
                    .map_err(|e| invalid(id, e.to_string()))?;
                input_indices.push((name.clone(), inputs.len(), quantity.as_id()));
                inputs.push(actual.clone());
                lexical.insert(name.clone(), actual);
            } else {
                let value = self.eval_ast_with(at, env, arg, Some(ty))?;
                collect_records(&value, &mut records);
                statics.insert(name.clone(), value);
            }
        }
        let owner = self
            .p
            .resolve(id, &v.owner)
            .ok_or_else(|| invalid(id, "claim owner absent"))?;
        let evidence = self.eval_field(id, &statics, "applicability.evidence", 0, None)?;
        let Value::Entity { id: evidence, kind } = evidence else {
            return Err(invalid(id, "claim evidence must be a source record"));
        };
        if !self.p.kinds.get(&kind).is_some_and(|k| k.provenance) {
            return Err(invalid(
                id,
                "claim evidence lacks provenance source authority",
            ));
        }
        let saved_lexical = std::mem::replace(&mut self.lexical, lexical);
        let mut identity = FramedHasher::new(pse_ids::Frame::ModelingApplicabilityCallV1);
        identity
            .str("applicability-call-v1")
            .id(&form.as_id())
            .id(&id.as_id());
        for (_, index, _) in &input_indices {
            identity.str(&dsl::render_expr(&inputs[*index]));
        }
        for record in &records {
            identity.id(record);
        }
        for (name, value) in &statics {
            identity.str(name);
            value.frame(&mut identity);
        }
        let mut dependencies = Vec::new();
        for position in 0..v.dependencies.len() {
            let e = self
                .p
                .expression_at(id, "applicability.dependencies", position)?
                .clone();
            dependencies.push(self.bind_claim(
                instance,
                form,
                id,
                &e,
                &statics,
                permissions,
                predicates,
                inputs,
                stack,
            )?);
        }
        let region = match v.claim_kind {
            Kind::Unknown => Region::Unknown,
            Kind::Unrestricted => Region::Unrestricted,
            Kind::Region | Kind::Interval => {
                let predicate = if v.claim_kind == Kind::Interval {
                    crate::applicability::interval_predicate(self.p, id)?
                } else {
                    self.p
                        .predicate_at(id, "applicability.predicate", 0)?
                        .clone()
                };
                let mut predicate =
                    self.rewrite_predicate(instance, &predicate, &statics, &[id])?;
                predicate.strip_spans();
                let index = predicates.len();
                predicates.push(predicate);
                Region::Predicate(index)
            }
            Kind::Union => {
                let mut alternatives = Vec::new();
                for position in 0..v.alternatives.len() {
                    let e = self
                        .p
                        .expression_at(id, "applicability.alternatives", position)?
                        .clone();
                    let child = self.bind_claim(
                        instance,
                        form,
                        id,
                        &e,
                        &statics,
                        permissions,
                        predicates,
                        inputs,
                        stack,
                    )?;
                    if child.claim.owner != owner.as_id() {
                        return Err(invalid(
                            id,
                            "a declared union combines claims from its own scientific family only",
                        ));
                    }
                    alternatives.push(child);
                }
                Region::Union(alternatives)
            }
        };
        self.lexical = saved_lexical;
        stack.pop();
        Ok(Node {
            claim: Claim {
                id: Some(id.as_id()),
                coverage: None,
                owner: owner.as_id(),
                owner_lineage: self.claim_owner_lineage(owner)?,
                evidence: Some(evidence.as_id()),
                form: form.as_id(),
                call: identity.finish_id(),
                records: records.into_iter().collect(),
                dependencies: dependencies.iter().filter_map(|n| n.claim.id).collect(),
                layer: v.scope,
                basis: v.basis,
                reason: v.reason.clone(),
            },
            region,
            dependencies,
            inputs: input_indices,
            permissions: permissions.to_vec(),
        })
    }
}
fn collect_records(value: &Value, records: &mut BTreeSet<SemanticId>) {
    match value {
        Value::Entity { id, .. } => {
            records.insert(id.as_id());
        }
        Value::Set(values) | Value::Tuple(values) => {
            for v in values {
                collect_records(v, records);
            }
        }
        _ => {}
    }
}

fn node_records(node: &Node, records: &mut BTreeSet<SemanticId>) {
    records.extend(&node.claim.records);
    for child in &node.dependencies {
        node_records(child, records);
    }
    if let Region::Union(children) = &node.region {
        for child in children {
            node_records(child, records);
        }
    }
}
fn remap_node(node: &mut Node, predicates: usize, inputs: usize) {
    match &mut node.region {
        Region::Predicate(index) => *index += predicates,
        Region::Union(children) => {
            for child in children {
                remap_node(child, predicates, inputs);
            }
        }
        _ => {}
    }
    for (_, index, _) in &mut node.inputs {
        *index += inputs;
    }
    for child in &mut node.dependencies {
        remap_node(child, predicates, inputs);
    }
}
