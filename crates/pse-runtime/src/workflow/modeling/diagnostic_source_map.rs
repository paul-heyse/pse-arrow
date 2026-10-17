// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit static source inspection; no numerical evaluation or solver activation.
use super::*;
use pse_ids::ContentHash;
use pse_kernels::DerivativeOrder;
use pse_math::{assembly::EvaluationDemand, binding::Target, jets::ProductKind};

/// Static eligibility includes both branch alternatives and is never execution evidence.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SourceMapScope {
    /// Original admitted sources and all statically eligible compiled branch alternatives.
    StaticEligibleAlternatives,
}
/// One exact current source field, independent of shared mathematical identity.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceFieldDocument {
    /// Original authored declaration owning this field.
    pub declaration: SemanticId,
    /// Compiler-supplied declaration-local field role.
    pub role: String,
    /// Original position of this repeated field.
    pub position: usize,
    /// Actual authoring document identity; absent when unavailable.
    pub document: Option<SemanticId>,
    /// UTF-8 starting byte offset in the supplied document.
    pub start: Option<u32>,
    /// UTF-8 exclusive ending byte offset in the supplied document.
    pub end: Option<u32>,
}
/// Original instantiated member ownership; a display path never becomes executable identity.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceMemberDocument {
    /// Actual instantiated semantic member or alias.
    pub member: SemanticId,
    /// Original authored declaration.
    pub declaration: SemanticId,
    /// Actual owning instance, preserving repeated definition uses.
    pub instance: SemanticId,
    /// Current compiler-supplied display path, independent of identity.
    pub path: String,
}
/// One source check's original range within a potentially shared arithmetic region.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ArithmeticSourceDocument {
    /// Physical arithmetic-region ordinal within the stated check program.
    pub region: usize,
    /// Original body-local source token, joined through checked_members; absent
    /// when the compiled owner supplied only a generated arithmetic range.
    pub source: Option<SemanticId>,
    /// Why this range has no original source token, independently of its existence.
    pub source_unavailable: Option<String>,
    /// First original check-output ordinal within the shared region.
    pub start_output: usize,
    /// Exclusive original check-output endpoint.
    pub end_output: usize,
    /// Actual program order owning these check ranges.
    pub check_order: DerivativeOrder,
}
/// Which actual program owns the exposed source-check ranges.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ArithmeticMappingScope {
    /// Actual selected compiled product, including both static branch alternatives.
    SelectedCompiledProduct,
    /// Direct weighted differentiation retains original Value checks. Their ranges
    /// do not identify derivative-kernel instructions or assign shared runtime cost.
    OriginalValueChecksForWeightedProduct,
    /// Arithmetic exists but no original check ranges were supplied by its owner.
    Unavailable {
        /// Actual missing owner mapping, never a guessed source.
        reason: String,
    },
}
/// One compiler-issued selected product template. Numerical weights/directions are not static identities.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceTemplateDocument {
    /// Compiler-issued template ordinal within this assembly.
    pub template: usize,
    /// Admitted reusable semantic body identity.
    pub body: ContentHash,
    /// Requested derivative ceiling.
    pub order: DerivativeOrder,
    /// Canonical mathematical product actually prepared.
    pub product: ProductKind,
    /// Ordered original body output selectors.
    pub outputs: Vec<usize>,
    /// Ordered original formal derivative selectors; aliases remain distinct.
    pub coordinates: Vec<usize>,
    /// Limits of the available arithmetic-origin projection.
    pub arithmetic_mapping: ArithmeticMappingScope,
    /// All original check ranges in the stated program, including static branches.
    pub arithmetic: Vec<ArithmeticSourceDocument>,
}
/// An ordered input gather: aliases deliberately retain separate formal positions.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceSlotDocument {
    /// Original formal input ordinal.
    pub formal: usize,
    /// Actual source coordinate identity.
    pub member: SemanticId,
    /// Canonical linear gather coefficient.
    pub scale: f64,
    /// Canonical affine gather offset.
    pub offset: f64,
    /// The source is absent from this selected assembly’s free columns.
    pub fixed: bool,
}
/// Actual selected destination of an original ordered output contribution.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SourceContributionTarget {
    /// Original selected constraint row.
    Row {
        /// Original semantic row identity.
        row: SemanticId,
    },
    /// Original selected objective.
    Objective {
        /// Objective ordinal, interpreted with objective_signs.
        ordinal: usize,
    },
}
/// One ordered addend before duplicate accumulation, including orientation/weight.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceContributionDocument {
    /// Original body output ordinal.
    pub output: usize,
    /// Selected original row or objective destination.
    pub target: SourceContributionTarget,
    /// Actual signed contribution weight before duplicate accumulation.
    pub scale: f64,
}
/// The original occurrence's actual selected template and selectors.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceDemandDocument {
    /// Canonical callback demand, distinct from the product kind.
    pub demand: EvaluationDemand,
    /// Actual selected compiled template ordinal.
    pub template: usize,
    /// Original output selectors for this occurrence and demand.
    pub outputs: Vec<usize>,
    /// Original formal coordinate selectors for this occurrence and demand.
    pub coordinates: Vec<usize>,
}
/// Original semantic occurrence and its concrete coordinate/output gathers.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceInstanceDocument {
    /// Original instance-binding ordinal in the assembly.
    pub ordinal: usize,
    /// Actual semantic occurrence identity.
    pub instance: SemanticId,
    /// Reusable semantic body identity; never an occurrence substitute.
    pub body: ContentHash,
    /// Complete body-local source-token to original-member map.
    pub checked_members: Vec<(SemanticId, SemanticId)>,
    /// Original formal gathers, preserving aliases and affine scales.
    pub slots: Vec<SourceSlotDocument>,
    /// Original ordered output addends, preserving repeated destinations.
    pub contributions: Vec<SourceContributionDocument>,
    /// Actual demand-to-template selectors for this occurrence.
    pub demands: Vec<SourceDemandDocument>,
}
/// Explicit bounded static source map. It supplies no native dual or stationarity pullback.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ModelingSourceMapDocument {
    /// Static source-map interpretation version.
    pub version: pse_model::document::Version<1>,
    /// Actual admitted original case structure identity.
    pub source_identity: ContentHash,
    /// Actual frozen numerical-policy identity.
    pub numerical_identity: ContentHash,
    /// Static source eligibility; it does not establish execution.
    pub scope: SourceMapScope,
    /// Selected original rows in assembly order.
    pub rows: Vec<SemanticId>,
    /// Selected original free coordinates in assembly order.
    pub columns: Vec<SemanticId>,
    /// Actually consumed fixed or parameter input identities.
    pub fixed_inputs: Vec<SemanticId>,
    /// Frozen physical scales of the selected original rows and coordinates.
    pub row_nominals: Vec<(SemanticId, f64)>,
    /// Frozen physical coordinate magnitudes in selected column order.
    pub variable_nominals: Vec<(SemanticId, f64)>,
    /// Objective signs in original objective order; never a native stationarity norm.
    pub objective_signs: Vec<f64>,
    /// Exact current authored field/document occurrences.
    pub fields: Vec<SourceFieldDocument>,
    /// Original instantiated member lineages and supplied aliases.
    pub members: Vec<SourceMemberDocument>,
    /// Deduplicated selected mathematical products and their source origins.
    pub templates: Vec<SourceTemplateDocument>,
    /// Original occurrence bindings into those shared templates.
    pub instances: Vec<SourceInstanceDocument>,
    /// Absent mappings are explicit; this source view is not a transformed native attempt.
    pub native_pullback_unavailable: String,
}
/// The requested static document retains its existing runtime allocation owner.
#[derive(Debug)]
pub struct ModelingSourceMap {
    /// Complete selected static source document under the retained lease.
    document: ModelingSourceMapDocument,
    _lease: Arc<pse_columnar::AllocationLease>,
}

impl serde::Serialize for ModelingSourceMap {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serde::Serialize::serialize(&self.document, serializer)
    }
}
impl ModelingSourceMap {
    /// Borrow the static manifest while its complete construction lease remains alive.
    pub fn document(&self) -> &ModelingSourceMapDocument { &self.document }
}

fn cell_count(lengths: &[usize]) -> Result<usize, WorkflowError> {
    lengths.iter().try_fold(0usize, |count, length| count.checked_add(*length))
        .ok_or_else(|| contract("source map population extent"))
}
fn member_lineages(model: &pse_modeling::specialize::SpecializedModel)
    -> impl Iterator<Item = (SemanticId, &pse_modeling::specialize::Lineage)> {
    model.symbols.values().map(|symbol| (symbol.id, &symbol.lineage))
        .chain(model.equations.iter().map(|row| (row.id, &row.lineage)))
        .chain(model.ports.values().map(|port| (port.id, &port.lineage)))
        .chain(model.material_ports.values().map(|port| (port.id, &port.lineage)))
        .chain(model.inventory_balances.values().map(|balance| (balance.id, &balance.lineage)))
        .chain(model.inventory_initial_conditions.values().map(|condition| (condition.row, &condition.lineage)))
}
impl ModelingPackage {
    /// Inspect the retained package's prepared source/product map without running science.
    /// # Errors
    /// Invalid allowance/clock, cancellation or actual construction admission refuses.
    pub async fn modeling_diagnostic_source_map(
        &self, prepared: &ModelingDiagnosticPreparation, maximum_bytes: usize,
        time_limit: std::time::Duration, cancel: &crate::CancelSource,
    ) -> Result<ModelingSourceMap, WorkflowError> {
        self.runtime.modeling_diagnostic_source_map(prepared, maximum_bytes, time_limit, cancel).await
    }
}
impl Runtime {
    /// Inspect already admitted compiled artifacts under a separate explicit finite
    /// clock/allocation. This never evaluates equations or initiates a native solve.
    ///
    /// # Errors
    /// Invalid finite bounds, original cancellation/deadline, unavailable compiled
    /// artifacts or insufficient construction/retention admission refuse the operation.
    pub async fn modeling_diagnostic_source_map(
        &self,
        prepared: &ModelingDiagnosticPreparation,
        maximum_bytes: usize,
        time_limit: std::time::Duration,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingSourceMap, WorkflowError> {
        if maximum_bytes == 0 || time_limit.is_zero() { return Err(contract("source map requires a positive finite allocation and clock")); }
        let deadline = std::time::Instant::now().checked_add(time_limit)
            .ok_or_else(|| contract("source map deadline extent"))?;
        let scope = pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::default(), Some(deadline));
        let check = || -> Result<(), WorkflowError> {
            scope.check().map_err(pse_backend_native::ProblemError::Provider).map_err(crate::math::MathRuntimeError::from)?;
            if cancel.token().is_cancelled() { return Err(crate::math::MathRuntimeError::Cancelled.into()); }
            Ok(())
        };
        check()?;
        let activity = self.admit_activity()?;
        activity.scope(async {
            let service = self.shared.math();
            // The selected construction has its own declared finite permission;
            // loader artifacts retain their independent existing owners.
            let lease = service.reserve("modeling:source-map", maximum_bytes)?;
            let executable = crate::math::MathService::within_task(&scope, cancel,
                service.assemble(prepared.model.case.clone())).await?;
            let assembly = &executable.assembly;
            check()?;
            let model = prepared.model.model.compiled();
            let fields = model.occurrence_identity();
            // Four times the copied payload plus per-row overhead covers source
            // strings, Vec growth and overlap with the inspected temporary views.
            // Count exact source-check populations before copying any origin rows.
            let mut bound = 4096usize;
            let mut add = |count: usize, strings: usize| -> Result<(), WorkflowError> {
                bound = count.checked_mul(512).and_then(|bytes| bytes.checked_add(strings.checked_mul(4)?))
                    .and_then(|bytes| bound.checked_add(bytes))
                    .ok_or_else(|| contract("source map construction extent"))?;
                if bound > maximum_bytes { return Err(crate::math::MathRuntimeError::Limit("source map construction allowance").into()); }
                Ok(())
            };
            add(cell_count(&[assembly.structure().rows().len(), assembly.columns().len(), assembly.input_ids().len(),
                assembly.structure().objectives().len()])?, 0)?;
            for (_, role, _, _) in fields.iter() { check()?; add(1, role.len())?; }
            for (_, lineage) in member_lineages(&model.model) { check()?; add(1, lineage.path.len())?; }
            for template in assembly.demand_census() {
                check()?;
                add(cell_count(&[1, template.demand.outputs.len(), template.demand.coordinates.len(),
                    template.program.arithmetic_origin_count(template.demand.order)])?, 0)?;
            }
            for instance in assembly.structure().instances() {
                check()?;
                add(cell_count(&[1, instance.checked_members.len(), instance.slots.len(), instance.contributions.len()])?, 0)?;
            }
            for demand in assembly.occurrence_demands() { check()?; add(cell_count(&[1, demand.outputs.len(), demand.coordinates.len()])?, 0)?; }
            let templates = assembly.demand_census().map(|template| -> Result<_, WorkflowError> {
                check()?;
                let mut stopped = None;
                let mut arithmetic = Vec::with_capacity(template.program.arithmetic_origin_count(template.demand.order));
                template.program.visit_arithmetic_origins(template.demand.order, |origin| {
                    if stopped.is_some() { return; }
                    if let Err(error) = check() { stopped = Some(error); return; }
                    arithmetic.push(ArithmeticSourceDocument {
                        region: origin.region, source: (origin.source != SemanticId::NIL).then_some(origin.source),
                        source_unavailable: (origin.source == SemanticId::NIL).then(|| "compiled owner supplied no original source token for this arithmetic range".into()),
                        start_output: origin.start_output,
                        end_output: origin.end_output, check_order: origin.check_order,
                    });
                });
                if let Some(error) = stopped { return Err(error); }
                let census = template.program.census().find(|census| census.order == template.demand.order);
                let arithmetic_mapping = if census.is_some_and(|census| census.direct_weighted) {
                    ArithmeticMappingScope::OriginalValueChecksForWeightedProduct
                } else if arithmetic.is_empty() && census.is_some_and(|census| census.arithmetic_blocks > 0) {
                    ArithmeticMappingScope::Unavailable { reason: "compiled owner supplies arithmetic but no original source-check ranges".into() }
                } else { ArithmeticMappingScope::SelectedCompiledProduct };
                Ok(SourceTemplateDocument { template: template.template, body: template.demand.body,
                    order: template.demand.order, product: template.demand.product,
                    outputs: template.demand.outputs.clone(), coordinates: template.demand.coordinates.clone(), arithmetic_mapping, arithmetic })
            }).collect::<Result<Vec<_>, WorkflowError>>()?;
            let mut instances = assembly.structure().instances().iter().enumerate().map(|(ordinal, instance)| -> Result<_, WorkflowError> {
                check()?;
                Ok(SourceInstanceDocument {
                    ordinal, instance: instance.instance, body: instance.body,
                    checked_members: instance.checked_members.iter().map(|(local, original)| (*local, *original)).collect(),
                    slots: instance.slots.iter().enumerate().map(|(formal, slot)| SourceSlotDocument {
                        formal, member: slot.source(), scale: slot.scale(), offset: slot.offset(),
                        fixed: !assembly.columns().contains(&slot.source()),
                    }).collect(),
                    contributions: instance.contributions.iter().map(|contribution| SourceContributionDocument {
                        output: contribution.output, scale: contribution.scale,
                        target: match contribution.target { Target::Row(row) => SourceContributionTarget::Row { row },
                            Target::Objective(ordinal) => SourceContributionTarget::Objective { ordinal } },
                    }).collect(), demands: Vec::new(),
                })
            }).collect::<Result<Vec<_>, WorkflowError>>()?;
            for demand in assembly.occurrence_demands() {
                check()?;
                instances[demand.instance].demands.push(SourceDemandDocument { demand: demand.demand,
                    template: demand.template, outputs: demand.outputs.to_vec(), coordinates: demand.coordinates.to_vec() });
            }
            let row_ids = assembly.structure().rows().iter().map(|row| row.id).collect::<Vec<_>>();
            let normalization = pse_math::normalization::Normalization::from_policy(
                &prepared.numerics, assembly.columns(), &row_ids).map_err(crate::math::MathRuntimeError::from)?;
            let document = ModelingSourceMapDocument {
                version: Default::default(), source_identity: assembly.structure().key(), numerical_identity: prepared.numerics.key,
                scope: SourceMapScope::StaticEligibleAlternatives,
                row_nominals: row_ids.iter().copied().zip(normalization.rows.iter().copied()).collect(),
                variable_nominals: assembly.columns().iter().copied().zip(normalization.variables.iter().copied()).collect(),
                rows: row_ids, columns: assembly.columns().to_vec(),
                fixed_inputs: assembly.input_ids().to_vec(), objective_signs: assembly.structure().objectives().iter().map(|objective| objective.sense.sign()).collect(),
                fields: fields.iter().map(|(declaration, role, position, span)| {
                    let document = (span.document_id != SemanticId::NIL).then_some(span.document_id);
                    let ranged = document.is_some() && !span.is_empty();
                    SourceFieldDocument { declaration: declaration.as_id(), role: role.clone(), position: *position,
                        document, start: ranged.then_some(span.start), end: ranged.then_some(span.end) }
                }).collect(),
                members: member_lineages(&model.model).map(|(member, lineage)| SourceMemberDocument {
                    member, declaration: lineage.declaration.as_id(), instance: lineage.instance.as_id(), path: lineage.path.clone(),
                }).collect(), templates, instances,
                native_pullback_unavailable: "static original source view has no transformed native attempt or dual pullback; native norms are not physical stationarity".into(),
            };
            check()?;
            Ok(ModelingSourceMap { document, _lease: lease })
        }).await
    }
}

#[cfg(test)]
#[path = "diagnostic_source_map_tests.rs"]
mod tests;
