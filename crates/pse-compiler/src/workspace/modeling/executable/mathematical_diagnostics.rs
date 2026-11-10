// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Original resolved-source admission for exact and numerical diagnostic products.
use super::*;
use pse_math::mathematical_diagnostics::{
    ArithmeticSourceAdmission, DiagnosticClaim, DiagnosticRequest, PreparedDiagnostic,
};
use pse_modeling::resolved::{Node, ResolvedSyntax};

fn arithmetic_source(syntax: &ResolvedSyntax) -> Result<()> {
    fn integral(syntax: &ResolvedSyntax, node: pse_modeling::resolved::NodeIx) -> bool {
        match &syntax.nodes[node] {
            Node::Number(number) => number.integer().is_some(),
            Node::Neg(value) => {
                matches!(&syntax.nodes[*value], Node::Number(number) if number.integer().is_some())
            }
            _ => false,
        }
    }
    // SelectedRegion retains the original reached syntax. Scan that owner, never
    // supplier atoms: sin(x)-sin(x) must remain a refused source obligation.
    if !syntax.validity.is_empty() {
        return Err(CompileError::Missing(
            "enclosure source contains a validity predicate".into(),
        ));
    }
    for node in &syntax.nodes {
        let admitted = match node {
            Node::Number(number) => number.value.is_finite(),
            Node::Reference(_) | Node::Neg(_) | Node::Let { .. } | Node::Sequence { .. } => true,
            Node::Binary { op, rhs, .. } => match op {
                dsl::BinaryOp::Add
                | dsl::BinaryOp::Sub
                | dsl::BinaryOp::Mul
                | dsl::BinaryOp::Div => true,
                dsl::BinaryOp::Pow => integral(syntax, *rhs),
            },
            _ => false,
        };
        if !admitted {
            return Err(CompileError::Missing(
                "enclosure source is outside exact real basic arithmetic and integer powers".into(),
            ));
        }
    }
    Ok(())
}

fn diagnostic_scope(scope: &pse_kernels::ExecutionScope) -> Result<()> {
    scope.check().map_err(|error| CompileError::from(match error {
        pse_kernels::ProviderError::Cancelled => MathError::Cancelled,
        other => MathError::Scope(other),
    }))
}

impl PreparedModeling {
    /// Consume the admitted original source selection, retaining source-domain
    /// obligations and issuing enclosure eligibility independently of normalization.
    pub fn prepare_mathematical_diagnostic(
        &self,
        values: &CaseValues,
        request: DiagnosticRequest,
        scope: &pse_kernels::ExecutionScope,
    ) -> Result<PreparedDiagnostic> {
        diagnostic_scope(scope)?;
        let selected = request.rows.iter().copied().collect::<BTreeSet<_>>();
        let structure = self.observation_structure(&selected)?;
        let context = TypeContext {
            admissions: None, formula_authority: None,
            quantities: &self.context.quantities, preconditions: &self.context.preconditions,
            scope: &self.projection.physical_scope,
        };
        let mut bodies = BTreeMap::new();
        let mut instances = Vec::new();
        let mut reached = BTreeSet::new();
        for original in structure.instances() {
            diagnostic_scope(scope)?;
            let index = self.projection.outputs.iter().position(|output| output.row_id() == original.instance)
                .ok_or_else(|| CompileError::Missing("diagnostic original region correspondence".into()))?;
            let region = &self.projection.regions[index];
            if region.inputs.len() != original.slots.len() {
                return Err(CompileError::Missing("diagnostic original exact input layout".into()));
            }
            let actual = region.bindings.iter().map(|binding| (binding.local, binding.actual)).collect::<BTreeMap<_, _>>();
            if region.inputs.iter().zip(original.slots.iter()).any(|(local, slot)| actual.get(local) != Some(&slot.source())) {
                return Err(CompileError::Missing("diagnostic original exact source binding".into()));
            }
            if request.claim == DiagnosticClaim::ValidatedEnclosure { arithmetic_source(&region.syntax)?; }
            let mut refuse_provider = |_: &str| Err(MathError::Contract("selected diagnostic provider lacks exact source arithmetic".into()));
            let providers = crate::typed_math::prepare_selected_providers(&region.syntax, &region.support,
                &mut refuse_provider, &BTreeMap::new(), &context)?;
            let original_body = self.admitted.bodies.get(&original.body)
                .ok_or_else(|| CompileError::Missing("diagnostic original admitted body".into()))?;
            let exact = crate::typed_math::Request {
                definition: self.projection.declarations[index].as_id(),
                syntax: &region.syntax, inputs: &region.inputs, support: &region.support, providers: &providers,
                physical: original_body.spec.physical, structure: region.syntax.identity()?, limits: self.projection.body_limits,
            }.admit_exact_source(&self.context.quantities, self.context.preconditions.as_ref(),
                EvaluationLimits {derivative_components:request.limits.terms,
                    operations:request.limits.work,scratch_bytes:request.limits.bytes,..Default::default()},
                request.limits.coefficient_bits,scope)?;
            let mut binding = original.clone();
            binding.body = exact.spec.key();
            // Definition/policy identify exact source arithmetic separately from
            // the ordinary numerical cache and its portable reconstruction recipe.
            bodies.insert(binding.body, Arc::clone(&exact.math));
            reached.extend(binding.contributions.iter().filter_map(|contribution| match contribution.target {
                Target::Row(row) => Some(row), _ => None,
            }));
            instances.push(binding);
        }
        if reached != selected { return Err(CompileError::Missing("diagnostic original source correspondence incomplete".into())); }
        let structure = Arc::new(CaseStructure::new(structure.variables().iter().cloned().collect(),
            structure.parameters().to_vec(), instances, structure.rows().to_vec(), None, CaseLimits::default())?);
        let plan = CasePlan::prepare(
            structure,
            bodies,
            &self.context.quantities,
            DerivativeOrder::Value,
            AssemblyLimits::default(),
            scope.cancellation(),
        )?;
        let admission = if request.claim == DiagnosticClaim::ValidatedEnclosure {
            // SAFETY: every reached original source node was checked above, using
            // the exact compiler-owned selected rows; no decoded assertion issues it.
            #[allow(
                unsafe_code,
                reason = "ADR-0265: compiler owns complete original source arithmetic admission"
            )]
            Some(unsafe {
                ArithmeticSourceAdmission::issue(plan.structure().key(), request.rows.clone())
            })
        } else {
            None
        };
        Ok(plan.prepare_mathematical_diagnostic(values, request, admission.as_ref(), scope)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn compiler_diagnostic_executes_admitted_source_and_preserves_original_guard_refusals() {
        use pse_math::mathematical_diagnostics::{DiagnosticInput, DiagnosticLimits, ExactValue};
        for (body, allowed) in [("x*x+2*x", true), ("sin(x)-sin(x)+x", false)] {
            let text = format!("package p {{def Root {{var x:Scalar;let f:Scalar={body};}}}}");
            let (mut workspace, _, _, root) = modeling::tests::setup(&text);
            let flag = Arc::new(AtomicBool::new(false));
            let model = workspace
                .prepare_modeling_cancellable(
                    root,
                    pse_modeling::specialize::root_instance(root),
                    Bindings {
                        demand: vec!["f".into(), "x".into()],
                        ..Default::default()
                    },
                    Limits::default(),
                    flag,
                )
                .unwrap();
            let x = model.model.paths["x"];
            let row = ModelingOutput::Member(model.model.paths["f"]).row_id();
            let request = DiagnosticRequest {
                rows: vec![row],
                unknowns: vec![x],
                parameters: vec![],
                inputs: vec![DiagnosticInput {
                    coordinate: x,
                    lower: ExactValue::integer(2),
                    upper: None,
                }],
                derivatives: vec![],
                claim: DiagnosticClaim::ValidatedEnclosure,
                precision: 100,
                limits: DiagnosticLimits::default(),
            };
            let scope =
                pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::default(), None);
            let product =
                model.prepare_mathematical_diagnostic(&CaseValues::default(), request, &scope);
            if allowed {
                let result = product.unwrap().evaluate(&scope).unwrap();
                assert_eq!(result.numerical[0].lower.as_deref(), Some("8"));
                assert_eq!(result.numerical[0].upper.as_deref(), Some("8"));
                assert!(!result.original_sources.is_empty());
                assert!(!result.mathematical_views.is_empty());
            } else {
                assert!(product.is_err());
            }
        }
    }

    #[test]
    fn compiler_diagnostic_preserves_canonical_binary64_source_bits_before_algebra() {
        use pse_math::mathematical_diagnostics::{DiagnosticInput, DiagnosticLimits, ExactValue};
        let text = "package p {def Root {var x:Scalar;eq balance:0.1*x==0.3;let h:Scalar=0.1*x;}}";
        let (mut workspace, _, _, root) = modeling::tests::setup(text);
        let model = workspace.prepare_modeling_cancellable(root, pse_modeling::specialize::root_instance(root),
            Bindings {demand:vec!["x".into(),"h".into()],..Default::default()},Limits::default(),Arc::new(AtomicBool::new(false))).unwrap();
        let x = model.model.paths["x"];
        let row = model.model.equations[0].id;
        let scope = pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::default(),None);
        let mut request = DiagnosticRequest {rows:vec![row],unknowns:vec![x],parameters:vec![],inputs:vec![],derivatives:vec![],
            claim:DiagnosticClaim::ExactIdentity,precision:160,limits:DiagnosticLimits::default()};
        let original = model.observation_structure(&BTreeSet::from([row])).unwrap();
        let result = model.prepare_mathematical_diagnostic(&CaseValues::default(),request.clone(),&scope).unwrap().evaluate(&scope).unwrap();
        assert_ne!(result.structure,original.key());
        // Exact dyadic images of the original f64 literals; deliberately differs
        // from rationalizing decimal text to 3/10 and 1/10.
        assert_eq!(result.exact.unwrap().recovered,vec![(x,"10808639105689190/3602879701896397".into())]);
        request.rows=vec![ModelingOutput::Member(model.model.paths["h"]).row_id()];
        request.inputs=vec![DiagnosticInput{coordinate:x,lower:ExactValue::integer(1),upper:None}];
        request.claim=DiagnosticClaim::ValidatedEnclosure;
        let result=model.prepare_mathematical_diagnostic(&CaseValues::default(),request,&scope).unwrap().evaluate(&scope).unwrap();
        assert_eq!(result.numerical[0].lower.as_deref(),Some("3602879701896397/36028797018963968"));
        assert_eq!(result.numerical[0].lower,result.numerical[0].upper);
    }

    #[test]
    fn enclosure_source_refuses_transcendentals_even_when_their_values_cancel() {
        let context = crate::authored_transfer_tests::context();
        for text in [
            "package direct {def Root {var x:Scalar;let f:Scalar=sin(x)-sin(x)+x;}}",
            "package direct {def Root {var x:Scalar;let f:Scalar=x^0.5;}}",
        ] {
            let fixture =
                crate::typed_math::direct_tests::fixture(context.clone(), text, &["f"]).unwrap();
            assert!(arithmetic_source(&fixture.region.syntax).is_err());
        }
        let fixture = crate::typed_math::direct_tests::fixture(
            context,
            "package direct {def Root {var x:Scalar;let f:Scalar=x*x+2*x;}}",
            &["f"],
        )
        .unwrap();
        assert!(arithmetic_source(&fixture.region.syntax).is_ok());
    }
}
