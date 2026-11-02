// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Task-owned structural diagnosis without numerical state or solver admission.
use super::*;

/// Structural evidence at the exact selected semantic root and case specification.
#[derive(Clone, Debug)]
pub struct ModelingStructuralAnalysis {
    /// Exact immutable source revision inspected by this task.
    pub source_revision: pse_ids::roles::SourceRevisionHash,
    /// Selected semantic declaration.
    pub root: DeclarationId,
    /// Selected instantiated scope.
    pub instance: InstanceId,
    /// Consumed semantic fixed/free and bound overrides; no point or starts are implied.
    pub variables: Arc<
        pse_columnar::Leased<BTreeMap<SemanticId, pse_compiler::workspace::ModelingVariableState>>,
    >,
    /// Library-owned matching, DM and BTF evidence; not numerical rank.
    pub analysis: Arc<pse_columnar::Leased<pse_structural::incidence::StructuralAnalysis>>,
    _model: ModelingPreparation,
}
impl ModelingPackage {
    /// Inspect static selected case incidence without starts or a physical candidate.
    /// # Errors
    /// Invalid source/case scopes and unsupported integrated derivatives are refused.
    pub async fn analyze_structure(
        &self,
        services: &crate::workflow::PreparationServices,
        analysis: &ModelingAnalysis,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingStructuralAnalysis, WorkflowError> {
        self.check_service_bindings(services)?;
        let activity = services.admit_activity()?;
        activity
            .scope(async {
                let deadline = services.math().preparation_deadline(cancel)?;
                let scoped = cancel.with_deadline(Some(deadline));
                // Structural overrides use the same caller-demand binder as ordinary
                // cases. The compiler resolves materialized observation paths and
                // expands any remaining finite member demand without numerical values.
                let mut bindings = analysis.bindings.clone();
                bindings
                    .demand
                    .extend(analysis.case.variables.keys().cloned());
                bindings.demand.sort();
                bindings.demand.dedup();
                let model = self
                    .prepare(
                        services,
                        analysis.root,
                        analysis.instance,
                        bindings,
                        analysis.limits,
                        &scoped,
                    )
                    .await?;
                let scope_bytes = analysis
                    .case
                    .variables
                    .len()
                    .checked_mul(
                        size_of::<(SemanticId, pse_compiler::workspace::ModelingVariableState)>()
                            + 128,
                    )
                    .and_then(|bytes| {
                        bytes.checked_add(size_of::<ModelingStructuralAnalysis>() + 128)
                    })
                    .ok_or_else(|| contract("structural scope metadata extent"))?;
                let scope_owner = services
                    .math()
                    .reserve("modeling:structural_scope", scope_bytes)?;
                let mut states = BTreeMap::new();
                for (path, state) in &analysis.case.variables {
                    let id = model.compiled().model.paths.get(path).ok_or_else(|| {
                        contract(format!("structural specification target absent: {path}"))
                    })?;
                    states.insert(*id, state.clone());
                }
                // Bound expressions are numerical observations on the point route. Do
                // not silently substitute their unevaluated defaults in this task.
                let annotations = model
                    .compiled()
                    .numerical_annotations(pse_compiler::workspace::NumericalAnnotationMode::Steady)
                    .map_err(crate::math::MathRuntimeError::from)?;
                for hint in annotations.outer_hints() {
                    use pse_compiler::workspace::ModelingHint;
                    let supplied = match hint.kind {
                        ModelingHint::Lower => states
                            .get(&hint.target)
                            .is_some_and(|state| state.lower.is_some()),
                        ModelingHint::Upper => states
                            .get(&hint.target)
                            .is_some_and(|state| state.upper.is_some()),
                        _ => true,
                    };
                    if !supplied {
                        return Err(contract(format!(
                            "structural analysis requires an explicit bound override for {}",
                            hint.target
                        )));
                    }
                }
                let states = Arc::new(pse_columnar::Leased::new(Arc::new(states), scope_owner));
                let result = services
                    .math()
                    .analyze_modeling_structure(
                        model.clone(),
                        analysis.instance.as_id(),
                        states.clone(),
                        analysis.compiler,
                        self.quantities.clone(),
                        &scoped,
                    )
                    .await?;
                Ok(ModelingStructuralAnalysis {
                    source_revision: self.revision.identity(),
                    root: analysis.root,
                    instance: analysis.instance,
                    variables: states,
                    analysis: result,
                    _model: model,
                })
            })
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn structural_analysis_succeeds_for_an_underdetermined_model_without_starts() {
        let rows = pse_authoring::language::parse(
            "package p {def D {var x:Scalar; var y:Scalar; eq balance:x+y==2;}}",
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            Default::default(),
        )
        .unwrap();
        let root = rows
            .iter()
            .find(|row| row.name == "D")
            .unwrap()
            .declaration_id;
        let runtime = crate::workflow::tests::runtime();
        let package = runtime
            .modeling_package(rows, crate::workflow::tests::physical())
            .await
            .unwrap();
        let request = ModelingAnalysis {
            root,
            instance: pse_modeling::specialize::root_instance(root),
            bindings: Default::default(),
            limits: Limits::default(),
            case: Default::default(),
            order: pse_kernels::DerivativeOrder::Value,
            compiler: crate::workflow::tests::compiler_profile(),
            solver: Default::default(),
            numerical: Default::default(),
        };
        let evidence = package
            .analyze_structure(
                &runtime.preparation_services(),
                &request,
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        assert_eq!(evidence.analysis.under.columns.len(), 2);
        assert_eq!(evidence.analysis.under.rows.len(), 1);
        assert_eq!(evidence.instance, request.instance);
        assert!(evidence.variables.is_empty());
        assert!(
            !evidence._model.compiled().model.paths.contains_key("x"),
            "the scalar is materialized without a caller path observation"
        );
        let mut specified = request.clone();
        let x = evidence
            ._model
            .compiled()
            .model
            .symbols
            .values()
            .find(|symbol| symbol.lineage.path.ends_with(".x"))
            .unwrap()
            .id;
        specified.case.variables.insert(
            "x".into(),
            pse_compiler::workspace::ModelingVariableState {
                fixed: Some(true),
                ..Default::default()
            },
        );
        let fixed = package
            .analyze_structure(
                &runtime.preparation_services(),
                &specified,
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        assert_eq!(fixed.source_revision, evidence.source_revision);
        assert_eq!(fixed.variables.len(), 1);
        assert_eq!(fixed.variables.get(&x).unwrap().fixed, Some(true));
        assert_eq!(fixed._model.compiled().model.paths.get("x"), Some(&x));
        assert_eq!(fixed.analysis.square.columns.len(), 1);
        assert!(fixed.analysis.under.columns.is_empty());
        assert!(
            specified.case.values.is_empty() && specified.case.members.is_empty(),
            "the fixed structural state has no numerical point"
        );
    }

    #[tokio::test]
    async fn structural_case_variables_use_the_existing_nested_indexed_path_binder() {
        let rows = pse_authoring::language::parse(
            "package p {entity kind item {} entity item a {} entity item b {} set items:Set<item>={a,b}; def Leaf {param p:Scalar=2; var x[j in items]:Scalar; eq balance:sum(j in items | x[j])==p;} def Root {child leaf:Leaf=Leaf();}}",
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            Default::default(),
        ).unwrap();
        let root = rows
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        let runtime = crate::workflow::tests::runtime();
        let services = runtime.preparation_services();
        let package = runtime
            .modeling_package(rows, crate::workflow::tests::physical())
            .await
            .unwrap();
        let mut request = ModelingAnalysis {
            root,
            instance: pse_modeling::specialize::root_instance(root),
            bindings: Default::default(),
            limits: Limits::default(),
            case: Default::default(),
            order: pse_kernels::DerivativeOrder::Value,
            compiler: crate::workflow::tests::compiler_profile(),
            solver: Default::default(),
            numerical: Default::default(),
        };
        request.case.variables.insert(
            "leaf.x[a]".into(),
            pse_compiler::workspace::ModelingVariableState {
                fixed: Some(true),
                ..Default::default()
            },
        );
        let evidence = package
            .analyze_structure(&services, &request, &crate::CancelSource::new())
            .await
            .unwrap();
        let id = evidence._model.compiled().model.paths["leaf.x[a]"];
        assert_eq!(evidence.variables.get(&id).unwrap().fixed, Some(true));
        assert_eq!(evidence.analysis.square.columns.len(), 1);
        assert!(evidence.analysis.under.columns.is_empty());
        assert!(request.case.values.is_empty() && request.case.members.is_empty());
        request.case.variables.clear();
        request.case.variables.insert(
            "leaf.p".into(),
            pse_compiler::workspace::ModelingVariableState {
                fixed: Some(true),
                ..Default::default()
            },
        );
        let error = package
            .analyze_structure(&services, &request, &crate::CancelSource::new())
            .await
            .unwrap_err();
        assert!(
            error.to_string().contains("requires an outer variable"),
            "{error}"
        );
    }
}
