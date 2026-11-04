// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Selected original scalar reports for independently constructed scientific comparisons.
use super::*;
use pse_model::reference_exchange::ReferenceObservable;

/// An explicit correspondence to one original report, including indexed target identity.
#[derive(Clone, Copy, Debug)]
pub struct ReferenceReportSelection {
    /// Independently agreed output identity in the complete comparison population.
    pub output: SemanticId,
    /// Original authored report annotation, never its rendered label.
    pub source: DeclarationId,
    /// Original specialized target, distinguishing indexed instances of an annotation.
    pub target: SemanticId,
}

#[cfg(all(test, feature = "solver-kinsol"))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn actual_native_reports_preserve_physical_meaning_and_refuse_incomplete_selection() {
        let runtime = crate::workflow::tests::runtime();
        let rows = pse_authoring::language::parse(
            "package p {def Root {var x:Temperature;eq root:x==300{K};annotation start x(300{K});annotation report x(\"original root\");}}",
            SemanticId::NIL, pse_authoring::language::IdentityPolicy::Named, Default::default()).unwrap();
        let root = rows
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        let package = runtime
            .modeling_package(rows, crate::workflow::tests::physical())
            .await
            .unwrap();
        let cancel = crate::CancelSource::new();
        let prepared = package
            .prepare_solve(
                &runtime.preparation_services(),
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                pse_compiler::workspace::ModelingCaseBindings::default(),
                pse_kernels::DerivativeOrder::First,
                crate::workflow::tests::compiler_profile(),
                crate::workflow::tests::profile(),
                crate::math::solves::NumericalInputs::default(),
                &cancel,
            )
            .await
            .unwrap();
        let result = package
            .solve_case(
                &runtime.execution_services(),
                prepared,
                crate::workflow::tests::compiler_profile(),
                &cancel,
            )
            .await
            .unwrap();
        assert!(result.accepted);
        let row = result
            .reports
            .iter()
            .find(|row| row.label == "original root")
            .unwrap();
        let selection = ReferenceReportSelection {
            output: SemanticId::from_bytes([77; 16]),
            source: row.source_id,
            target: row.target_id,
        };
        let before = runtime.preparation_services().pool().reserved();
        let actual = result.reference_observations(&[selection]).unwrap();
        assert_eq!(actual.len(), 1);
        assert_eq!(actual[0].value.to_bits(), row.value.to_bits());
        assert!(
            (actual[0].value - 300.0).abs() < 1e-10,
            "independently specified temperature root"
        );
        let physical = package
            .physical_context()
            .quantities()
            .quantity_type(row.quantity_id.into())
            .unwrap();
        assert_eq!(actual[0].basis, physical.key.basis.map(|id| id.as_id()));
        assert_eq!(
            actual[0].datum,
            physical.key.reference_state.map(|id| id.as_id())
        );
        assert!(runtime.preparation_services().pool().reserved() > before);
        drop(actual);
        assert_eq!(runtime.preparation_services().pool().reserved(), before);
        assert!(result.reference_observations(&[]).is_err());
        assert!(
            result
                .reference_observations(&[selection, selection])
                .is_err()
        );
        let absent = ReferenceReportSelection {
            target: SemanticId::NIL,
            ..selection
        };
        assert!(result.reference_observations(&[absent]).is_err());
        assert_eq!(runtime.preparation_services().pool().reserved(), before);
        runtime.shared.math().shutdown_native_lanes().await.unwrap();
    }
}

impl ModelingResult {
    /// Select original accepted reports without decimal conversion or physical reinterpretation.
    /// The actual admitted quantity registry supplies basis and datum. The returned allocation
    /// remains charged while a caller builds, encodes or compares the envelope.
    /// # Errors
    /// Unaccepted results, empty/duplicate selections, ambiguous/missing original reports,
    /// unknown physical types/units, nonfinite values and allocation refusal fail as a whole.
    pub fn reference_observations(
        &self,
        selection: &[ReferenceReportSelection],
    ) -> Result<pse_columnar::Leased<Vec<ReferenceObservable>>, WorkflowError> {
        if !self.accepted || selection.is_empty() || selection.len() > 65_536 {
            return Err(contract(
                "reference observations require an accepted original result and 1..=65536 selected reports",
            ));
        }
        let bytes = selection
            .len()
            .checked_mul(size_of::<ReferenceObservable>() + 256)
            .ok_or_else(|| contract("reference report selection extent overflow"))?;
        let owner = self
            .services
            .math()
            .reserve("modeling:reference-observations", bytes)?;
        let mut ids = BTreeSet::new();
        let registry = self.prepared.source.physical_context().quantities();
        let mut outputs = Vec::with_capacity(selection.len());
        for selected in selection {
            if !ids.insert(selected.output) {
                return Err(contract(
                    "reference output correspondence contains duplicate identities",
                ));
            }
            let mut original = self
                .reports
                .iter()
                .filter(|row| row.source_id == selected.source && row.target_id == selected.target);
            let row = original
                .next()
                .ok_or_else(|| contract("required original reference report is absent"))?;
            if original.next().is_some() || !row.value.is_finite() {
                return Err(contract(
                    "required original reference report is ambiguous or nonfinite",
                ));
            }
            let quantity = registry
                .quantity_type(row.quantity_id.into())
                .map_err(|error| contract(error.to_string()))?;
            registry
                .unit(row.unit_id.into())
                .map_err(|error| contract(error.to_string()))?;
            outputs.push(ReferenceObservable {
                id: selected.output,
                quantity: row.quantity_id,
                unit: row.unit_id,
                basis: quantity.key.basis.map(|id| id.as_id()),
                datum: quantity.key.reference_state.map(|id| id.as_id()),
                value: row.value,
            });
        }
        Ok(pse_columnar::Leased::new(Arc::new(outputs), owner))
    }
}
