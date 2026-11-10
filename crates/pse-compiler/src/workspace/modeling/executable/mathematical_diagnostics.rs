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

impl PreparedModeling {
    /// Consume the admitted original source selection, retaining source-domain
    /// obligations and issuing enclosure eligibility independently of normalization.
    pub fn prepare_mathematical_diagnostic(
        &self,
        values: &CaseValues,
        request: DiagnosticRequest,
        scope: &pse_kernels::ExecutionScope,
    ) -> Result<PreparedDiagnostic> {
        scope.check().map_err(|error| {
            CompileError::from(match error {
                pse_kernels::ProviderError::Cancelled => MathError::Cancelled,
                other => MathError::Scope(other),
            })
        })?;
        let selected = request.rows.iter().copied().collect::<BTreeSet<_>>();
        let structure = self.observation_structure(&selected)?;
        let mut reached = BTreeSet::new();
        if request.claim == DiagnosticClaim::ValidatedEnclosure {
            for (output, region) in self.projection.outputs.iter().zip(&self.projection.regions) {
                if selected.contains(&output.row_id()) {
                    arithmetic_source(&region.syntax)?;
                    reached.insert(output.row_id());
                }
            }
            if reached != selected {
                return Err(CompileError::Missing(
                    "enclosure original source correspondence incomplete".into(),
                ));
            }
        }
        let bodies = structure
            .instances()
            .iter()
            .map(|binding| binding.body)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .map(|identity| {
                self.admitted
                    .bodies
                    .get(&identity)
                    .map(|body| (identity, Arc::clone(&body.math)))
                    .ok_or_else(|| {
                        CompileError::Missing("diagnostic original admitted body".into())
                    })
            })
            .collect::<Result<BTreeMap<_, _>>>()?;
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
