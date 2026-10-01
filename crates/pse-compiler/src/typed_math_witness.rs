// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Reuse physical lowering with abstract scientific calls before implementation substitution.
use super::*;
use pse_math::typed::ScientificWitness;

#[derive(Clone, Debug)]
pub(super) enum WitnessPass {
    Potential {
        response: DeclarationId,
        selected: DeclarationId,
        witness: ScientificWitness,
    },
    Reconstruction {
        root: DeclarationId,
        witness: ScientificWitness,
    },
}

impl WitnessPass {
    fn witness(&self) -> &ScientificWitness {
        match self {
            Self::Potential { witness, .. } | Self::Reconstruction { witness, .. } => witness,
        }
    }
}

impl<'a, 'b> Lower<'a, 'b> {
    fn witness_lower(&self, witness_pass: WitnessPass) -> Self {
        Self {
            checker: self.checker,
            request: self.request,
            registry: self.registry,
            paths: self.paths.clone(),
            locals: self.locals.clone(),
            occurrences: Vec::new(),
            hash: FramedHasher::new(pse_ids::Frame::MathTypedDefinitionV7),
            cancelled: self.cancelled,
            coordinates: self.coordinates.clone(),
            physical_only: self.physical_only,
            functions: self.functions,
            local_quantities: self.local_quantities,
            calls: self.calls.clone(),
            validity: self.validity,
            validating: self.validating.clone(),
            witness_pass: Some(witness_pass),
            operation_scope: self.operation_scope.clone(),
        }
    }

    pub(super) fn verify_scientific_response(
        &self,
        function: &pse_modeling::Function,
        name: &str,
        args: &[Expr],
        builder: &BodyBuilder<'_>,
        depth: usize,
        source: SemanticId,
    ) -> Result<(), MathError> {
        if self.witness_pass.is_some() || self.physical_only {
            return Ok(());
        }
        let Some(pse_modeling::PhysicalOperation::Response {
            potential: Some(selected),
            ..
        }) = function.physical_operation
        else {
            return Ok(());
        };
        let mut builder = builder.scientific_pass()?;
        let mut lower = self.witness_lower(WitnessPass::Potential {
            response: function.id,
            selected,
            witness: ScientificWitness::default(),
        });
        let value = lower.function(name, args, &[], &mut builder, depth, source)?;
        let pass = lower
            .witness_pass
            .as_ref()
            .ok_or_else(|| MathError::Contract("scientific witness pass absent".into()))?;
        if !builder.retains_scientific_witness(&value, pass.witness())? {
            return Err(MathError::Contract(format!(
                "response {} must retain its selected potential or partials after normalization",
                function.id
            )));
        }
        Ok(())
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "the intercepted call retains its source arguments, checked physical substitution and mathematical context"
    )]
    pub(super) fn abstract_scientific_call(
        &mut self,
        function: &pse_modeling::Function,
        name: &str,
        args: &[Expr],
        arguments: &mut [TypedValue],
        substitutions: &pse_quantity::scheme::Substitution,
        wrt: &[dsl::Path],
        builder: &mut BodyBuilder<'_>,
        depth: usize,
        source: SemanticId,
    ) -> Result<Option<TypedValue>, MathError> {
        let abstract_arguments = match self.witness_pass.as_ref() {
            Some(WitnessPass::Potential { response, .. }) => *response == function.id,
            Some(WitnessPass::Reconstruction { root, .. }) => *root == function.id,
            None => false,
        };
        if abstract_arguments {
            for value in arguments.iter_mut() {
                *value = builder.scientific_argument(value.clone())?;
            }
        }
        let intercept = match self.witness_pass.as_ref() {
            Some(WitnessPass::Potential { selected, .. }) => *selected == function.id,
            Some(WitnessPass::Reconstruction { .. }) => matches!(
                function.physical_operation,
                Some(pse_modeling::PhysicalOperation::Reconstruction { .. })
            ),
            None => false,
        };
        if !intercept {
            return Ok(None);
        }
        if matches!(self.witness_pass, Some(WitnessPass::Potential { .. })) {
            let mut shadow_builder = builder.scientific_pass()?;
            let mut lower = self.witness_lower(WitnessPass::Reconstruction {
                root: function.id,
                witness: ScientificWitness::default(),
            });
            // Check the selected potential's abstract reconstruction dependence, before
            // differentiating or substituting any reduced-law implementation (including 0).
            let value = lower.function(name, args, &[], &mut shadow_builder, depth, source)?;
            let pass = lower
                .witness_pass
                .as_ref()
                .ok_or_else(|| MathError::Contract("reconstruction witness pass absent".into()))?;
            if !shadow_builder.retains_scientific_witness(&value, pass.witness())? {
                return Err(MathError::Contract(format!(
                    "selected potential {} must retain its physical reconstruction after normalization",
                    function.id
                )));
            }
        }
        let result = function
            .result
            .quantity_scheme()
            .ok_or_else(|| {
                MathError::Contract("abstract scientific result must be physical".into())
            })?
            .resolve_contract_with_evidence(self.registry, substitutions, self.checker)
            .map_err(|error| MathError::Contract(error.to_string()))?;
        let partial = wrt
            .iter()
            .map(|path| {
                if path.segments.len() != 1 || !path.segments[0].indices.is_empty() {
                    return Err(MathError::Contract(
                        "abstract partial must select a specialized scalar argument".into(),
                    ));
                }
                function
                    .arguments
                    .iter()
                    .position(|(name, _)| name == &path.segments[0].name)
                    .ok_or_else(|| MathError::Contract("abstract partial argument absent".into()))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let witness = match self.witness_pass.as_mut() {
            Some(
                WitnessPass::Potential { witness, .. }
                | WitnessPass::Reconstruction { witness, .. },
            ) => witness,
            None => {
                return Err(MathError::Contract(
                    "abstract scientific pass absent".into(),
                ));
            }
        };
        Ok(Some(builder.scientific_placeholder(
            witness,
            function.id.as_id(),
            result,
            arguments,
            &partial,
            source,
        )?))
    }
}
