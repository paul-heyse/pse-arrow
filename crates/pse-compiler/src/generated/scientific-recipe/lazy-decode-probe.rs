trait ProbeDeferred<'a>: buffa::LazyMessageView<'a> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError>;
}
fn probe_singular<'a, L: ProbeDeferred<'a>>(
    field: &buffa::LazyMessageFieldView<'a, L>,
    ctx: buffa::DecodeContext<'_>,
) -> Result<(), buffa::DecodeError> {
    if let Some((first, rest)) = field.fragments().split_first() {
        let ctx = ctx.descend()?;
        ctx.require_allocation_accounted(L::allocation_accounted())?;
        let mut child = L::decode_lazy_with_ctx(first, ctx)?;
        for fragment in rest {
            ctx.require_allocation_accounted(L::allocation_accounted())?;
            child.merge_lazy(fragment, ctx)?;
        }
        child.probe_deferred(ctx)?;
    }
    Ok(())
}
fn probe_repeated<'a, L: ProbeDeferred<'a>>(
    field: &buffa::LazyRepeatedView<'a, L>,
    ctx: buffa::DecodeContext<'_>,
) -> Result<(), buffa::DecodeError> {
    for bytes in field.raw_elements() {
        let ctx = ctx.descend()?;
        ctx.require_allocation_accounted(L::allocation_accounted())?;
        let child = L::decode_lazy_with_ctx(bytes, ctx)?;
        child.probe_deferred(ctx)?;
    }
    Ok(())
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::ScientificRecipeLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_singular(&self.semantic_identity, ctx)?;
        probe_singular(&self.syntax, ctx)?;
        probe_singular(&self.support, ctx)?;
        probe_singular(&self.providers, ctx)?;
        probe_repeated(&self.inputs, ctx)?;
        probe_repeated(&self.outputs, ctx)?;
        probe_singular(&self.receipts, ctx)?;
        probe_singular(&self.proofs, ctx)?;
        probe_singular(&self.spec, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::MathBodySpecLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::MathematicalProofStreamLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_repeated(&self.entries, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::MathematicalProofReceiptLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::PhysicalReceiptStreamLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_repeated(&self.entries, ctx)?;
        probe_repeated(&self.boundaries, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::PhysicalOutputBoundaryReceiptLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_singular(&self.result, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::PhysicalOutputBoundaryFactsLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_singular(&self.source, ctx)?;
        probe_singular(&self.target, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::PhysicalContractFactsLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_singular(&self.named, ctx)?;
        probe_repeated(&self.kinds, ctx)?;
        probe_repeated(&self.qualified, ctx)?;
        probe_repeated(&self.axes, ctx)?;
        probe_repeated(&self.indices, ctx)?;
        probe_repeated(&self.factors, ctx)?;
        probe_singular(&self.dimension, ctx)?;
        probe_singular(&self.formula, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::PhysicalFormulaFactsLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_repeated(&self.operands, ctx)?;
        probe_repeated(&self.powers, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::PhysicalRatioLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::PhysicalDimensionVectorLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_singular(&self.exponents, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::Array8PhysicalRatioLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_repeated(&self.values, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::UnitFactorLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_singular(&self.unit, ctx)?;
        probe_singular(&self.exponent, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::UnitIdLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::BoundIndexReferenceLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_singular(&self.bound_index, ctx)?;
        probe_singular(&self.domain, ctx)?;
        probe_singular(&self.kind, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::EntityKindIdLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::DomainIdLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::BoundIndexIdLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::PhysicalQualifiedFactorLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_singular(&self.key, ctx)?;
        probe_singular(&self.exponent, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::QuantityTypeKeyLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_singular(&self.kind, ctx)?;
        probe_singular(&self.basis, ctx)?;
        probe_singular(&self.reference_state, ctx)?;
        probe_singular(&self.scale_kind, ctx)?;
        probe_repeated(&self.shape, ctx)?;
        probe_singular(&self.subject_kind, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::ScaleKindLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::ReferenceStateIdLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::BasisIdLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::QuantityKindIdLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::KindFactorLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_singular(&self.kind, ctx)?;
        probe_singular(&self.exponent, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::QuantityTypeIdLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::PhysicalInferenceReceiptLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_singular(&self.result, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::PhysicalInferenceFactsLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_repeated(&self.operands, ctx)?;
        probe_singular(&self.result, ctx)?;
        probe_singular(&self.selected, ctx)?;
        probe_repeated(&self.conversions, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::PhysicalOperandConversionLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_singular(&self.conversion, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::ConversionIdLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::PhysicalOperationSelectionLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::ModelingResolvedSymbolIxLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::CompilerSelectedProvidersRecordLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_repeated(&self.descriptors, ctx)?;
        probe_repeated(&self.calls, ctx)?;
        probe_repeated(&self.direct_calls, ctx)?;
        probe_singular(&self.premises, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::ModelingPhysicalEvidenceRecordLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::CompilerDirectProviderCallRecordLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_singular(&self.prepared, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::MathematicalPreparedProviderCallRecordLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_singular(&self.registration, ctx)?;
        probe_repeated(&self.inputs, ctx)?;
        probe_repeated(&self.input_conversions, ctx)?;
        probe_repeated(&self.outputs, ctx)?;
        probe_repeated(&self.output_conversions, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::PhysicalRepresentationConversionRecordLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_singular(&self.quantity, ctx)?;
        probe_singular(&self.context, ctx)?;
        probe_singular(&self.source, ctx)?;
        probe_singular(&self.target, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::PhysicalContractRecordLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_singular(&self.value, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::KernelProviderRecordLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_singular(&self.spec, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::KernelProviderSpecLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_singular(&self.shapes, ctx)?;
        probe_singular(&self.derivative_source, ctx)?;
        probe_repeated(&self.inputs, ctx)?;
        probe_repeated(&self.outputs, ctx)?;
        probe_singular(&self.derivatives, ctx)?;
        probe_singular(&self.smoothness, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::KernelDerivativeOrderLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::KernelProviderPortLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_singular(&self.quantity, ctx)?;
        probe_singular(&self.unit, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::ExternalDerivativeSourceLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::KernelProviderShapesLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_repeated(&self.inputs, ctx)?;
        probe_repeated(&self.outputs, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::KernelProviderShapeLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_repeated(&self.coordinates, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::kernel_provider_shape::Field3ValueLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::CompilerSelectedProviderCallRecordLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_repeated(&self.inputs, ctx)?;
        probe_singular(&self.result, ctx)?;
        probe_singular(&self.prepared, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::PhysicalContractBoundaryRecordLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_singular(&self.source, ctx)?;
        probe_singular(&self.target, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::Tuple2OfContentHashAndKernelProviderRecordLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_singular(&self.second, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::ModelingSupportRecordLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_repeated(&self.nodes, ctx)?;
        probe_repeated(&self.outputs, ctx)?;
        probe_repeated(&self.inputs, ctx)?;
        probe_singular(&self.premises, ctx)?;
        probe_singular(&self.boolean, ctx)?;
        probe_singular(&self.reduction, ctx)?;
        probe_singular(&self.reduction_output, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::PhysicalInferenceRecordLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_singular(&self.value, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::ModelingSupportBooleanRecordLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_singular(&self.contract, ctx)?;
        probe_singular(&self.conversion, ctx)?;
        probe_singular(&self.literal, ctx)?;
        probe_singular(&self.conditional, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::PhysicalCanonicalConversionRecordLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_singular(&self.quantity, ctx)?;
        probe_singular(&self.context, ctx)?;
        probe_singular(&self.source, ctx)?;
        probe_singular(&self.target, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::Tuple2OfUsizeAndPhysicalContractRecordLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_singular(&self.second, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::ModelingSupportNodeRecordLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_singular(&self.ty, ctx)?;
        probe_singular(&self.contract, ctx)?;
        probe_singular(&self.operation, ctx)?;
        probe_singular(&self.literal, ctx)?;
        probe_singular(&self.boundary, ctx)?;
        probe_singular(&self.call, ctx)?;
        probe_singular(&self.range, ctx)?;
        probe_singular(&self.exponent, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::ModelingSupportRangeRecordLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_singular(&self.lower, ctx)?;
        probe_singular(&self.upper, ctx)?;
        probe_singular(&self.lower_difference, ctx)?;
        probe_singular(&self.upper_difference, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::ModelingSupportCallRecordLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_singular(&self.nominal_result, ctx)?;
        probe_repeated(&self.inputs, ctx)?;
        probe_singular(&self.body_result, ctx)?;
        probe_singular(&self.partial, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::ModelingBodyInputRecordLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_singular(&self.relationship, ctx)?;
        probe_singular(&self.boundary, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::PhysicalOutputBoundaryRecordLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_singular(&self.source, ctx)?;
        probe_singular(&self.target, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::ModelingDerivedRelationshipLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::ModelingTypeRecordLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_singular(&self.value, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::ModelingTypeFactsLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::ModelingResolvedRecordLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_singular(&self.syntax, ctx)?;
        probe_repeated(&self.functions, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::ModelingResolvedFunctionRecordLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_singular(&self.declaration, ctx)?;
        probe_singular(&self.syntax, ctx)?;
        probe_singular(&self.physical, ctx)?;
        probe_repeated(&self.formals, ctx)?;
        probe_repeated(&self.signature, ctx)?;
        probe_repeated(&self.arguments, ctx)?;
        probe_repeated(&self.selections, ctx)?;
        probe_repeated(&self.literals, ctx)?;
        probe_singular(&self.result, ctx)?;
        probe_singular(&self.body, ctx)?;
        probe_singular(&self.validity, ctx)?;
        probe_repeated(&self.envelopes, ctx)?;
        probe_repeated(&self.applicability, ctx)?;
        probe_repeated(&self.admissions, ctx)?;
        probe_singular(&self.reduction_admission, ctx)?;
        probe_singular(&self.reduction, ctx)?;
        probe_singular(&self.reduction_selection, ctx)?;
        probe_singular(&self.physical_operation, ctx)?;
        probe_singular(&self.output_boundary, ctx)?;
        probe_repeated(&self.body_inputs, ctx)?;
        probe_singular(&self.validity_reads, ctx)?;
        probe_singular(&self.external, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::ModelingResolvedExternalRecordLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_singular(&self.derivative_source, ctx)?;
        probe_repeated(&self.shapes, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::ModelingResolvedShapeRecordLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_repeated(&self.coordinates, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::modeling_resolved_shape_record::Field3ValueLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::ModelingEnvelopeReadFactsLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::modeling_resolved_function_record::Field22ValueLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_singular(&self.value, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::ModelingPhysicalOperationRecordLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::ModelingResolvedReductionSelectionRecordLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_singular(&self.carrier, ctx)?;
        probe_repeated(&self.selected, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::ModelingValueRecordLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_singular(&self.value, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::ModelingStaticValueFactsLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::ModelingResolvedCarrierRecordLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_singular(&self.kind, ctx)?;
        probe_singular(&self.element, ctx)?;
        probe_repeated(&self.members, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::ModelingCarrierKindLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::ModelingResolvedReductionRecordLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_singular(&self.kind, ctx)?;
        probe_singular(&self.domain, ctx)?;
        probe_singular(&self.prototype, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::ReductionKindLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::ModelingPhysicalAdmissionRecordLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_singular(&self.request, ctx)?;
        probe_repeated(&self.operands, ctx)?;
        probe_singular(&self.resolved, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::PhysicalSchemeRecordLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_singular(&self.value, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::PhysicalSchemeShapeLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::ModelingPhysicalRequestFactsLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::Tuple2OfModelingResolvedNodeIxAndModelingPhysicalAdmissionRecordLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_singular(&self.first, ctx)?;
        probe_singular(&self.second, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::ModelingResolvedNodeIxLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::ModelingResolvedEvidenceRecordLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_singular(&self.node, ctx)?;
        probe_repeated(&self.predicates, ctx)?;
        probe_repeated(&self.inputs, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::ModelingResolvedPredicateIxLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::ApplicabilityNodeLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_singular(&self.claim, ctx)?;
        probe_singular(&self.region, ctx)?;
        probe_repeated(&self.dependencies, ctx)?;
        probe_repeated(&self.inputs, ctx)?;
        probe_repeated(&self.permissions, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::ApplicabilityPermissionLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_singular(&self.target_kind, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::ModelingPermissionTargetLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::Tuple3OfStringAndUsizeAndSemanticIdLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::ApplicabilityRegionLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::ApplicabilityClaimLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_singular(&self.layer, ctx)?;
        probe_singular(&self.basis, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::ModelingApplicabilityBasisLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::ModelingValidityLayerLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::ModelingResolvedEnvelopeRecordLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_singular(&self.owner, ctx)?;
        probe_singular(&self.ty, ctx)?;
        probe_singular(&self.predicate, ctx)?;
        probe_singular(&self.reads, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::DeclarationIdLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::Tuple2OfModelingResolvedNodeIxAndModelingTypeRecordLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_singular(&self.first, ctx)?;
        probe_singular(&self.second, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::ModelingResolvedSelectionRecordLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_singular(&self.dependencies, ctx)?;
        probe_repeated(&self.roots, ctx)?;
        probe_singular(&self.context, ctx)?;
        probe_repeated(&self.records, ctx)?;
        probe_repeated(&self.edges, ctx)?;
        probe_repeated(&self.selected, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::ModelingResolvedSelectedRecordLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_singular(&self.kind, ctx)?;
        probe_singular(&self.origin, ctx)?;
        probe_repeated(&self.values, ctx)?;
        probe_repeated(&self.uncertainties, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::Tuple3OfStringAndModelingUncertaintyKindAndU64LazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_singular(&self.second, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::ModelingUncertaintyKindLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::Tuple2OfStringAndModelingValueRecordLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_singular(&self.second, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::Tuple2OfModelingValueRecordAndListModelingValueRecordLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_singular(&self.first, ctx)?;
        probe_repeated(&self.second, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::ModelingResolvedArgumentRecordLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_singular(&self.ty, ctx)?;
        probe_repeated(&self.tuples, ctx)?;
        probe_repeated(&self.domains, ctx)?;
        probe_singular(&self.static_value, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::Tuple3OfUsizeAndListModelingValueRecordAndModelingResolvedCarrierRecordLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_repeated(&self.second, ctx)?;
        probe_singular(&self.third, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::modeling_resolved_argument_record::Field4ValueLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_repeated(&self.value, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::ModelingResolvedSyntaxRecordLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_repeated(&self.symbols, ctx)?;
        probe_repeated(&self.nodes, ctx)?;
        probe_repeated(&self.predicates, ctx)?;
        probe_repeated(&self.binders, ctx)?;
        probe_repeated(&self.functions, ctx)?;
        probe_repeated(&self.evidence, ctx)?;
        probe_repeated(&self.carriers, ctx)?;
        probe_repeated(&self.groups, ctx)?;
        probe_repeated(&self.member_ranges, ctx)?;
        probe_repeated(&self.finite_folds, ctx)?;
        probe_repeated(&self.outputs, ctx)?;
        probe_repeated(&self.validity, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::ModelingResolvedFoldRecordLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_singular(&self.root, ctx)?;
        probe_singular(&self.carrier, ctx)?;
        probe_repeated(&self.selected, ctx)?;
        probe_singular(&self.prototype, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::ModelingResolvedCarrierIxLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::Tuple2OfModelingResolvedSymbolIxAndModelingMemberRangeLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_singular(&self.first, ctx)?;
        probe_singular(&self.second, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::ModelingMemberRangeLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_singular(&self.lower, ctx)?;
        probe_singular(&self.upper, ctx)?;
        probe_singular(&self.layer, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::ModelingResolvedGroupRecordLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_repeated(&self.axes, ctx)?;
        probe_repeated(&self.members, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::Tuple2OfListModelingValueRecordAndOptionalModelingResolvedSymbolIxLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_repeated(&self.first, ctx)?;
        probe_singular(&self.second, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::ModelingResolvedFunctionSlotLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_singular(&self.declaration, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::ModelingResolvedBindingLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::ModelingResolvedPredicateLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::ModelingResolvedNodeLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::ModelingResolvedScalarRecordLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        probe_singular(&self.ty, ctx)?;
        probe_singular(&self.role, ctx)?;
        probe_singular(&self.domain, ctx)?;
        probe_singular(&self.definition, ctx)?;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::ModelingVariableDomainLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::ModelingDeclarationKindLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        Ok(())
    }
}
impl<'a> ProbeDeferred<'a>
for crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::SemanticBodyHashLazyView<
    'a,
> {
    fn probe_deferred(
        &self,
        ctx: buffa::DecodeContext<'_>,
    ) -> Result<(), buffa::DecodeError> {
        let _ = ctx;
        Ok(())
    }
}
pub(super) fn probe_scientific_recipe<'a>(
    bytes: &'a [u8],
    ctx: buffa::DecodeContext<'_>,
) -> Result<
    crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::ScientificRecipeLazyView<
        'a,
    >,
    buffa::DecodeError,
> {
    ctx.require_allocation_accounted(
        <crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::ScientificRecipeLazyView<
            'a,
        > as buffa::LazyMessageView>::allocation_accounted(),
    )?;
    let view = <crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::lazy_view::ScientificRecipeLazyView<
        'a,
    > as buffa::LazyMessageView>::decode_lazy_with_ctx(bytes, ctx)?;
    view.probe_deferred(ctx)?;
    Ok(view)
}
