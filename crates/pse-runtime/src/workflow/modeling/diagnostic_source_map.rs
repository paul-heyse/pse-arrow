// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit static source inspection; no numerical evaluation or solver activation.
use super::*;
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
    pub declaration: SemanticId,
    pub role: String,
    pub position: usize,
    pub document: Option<SemanticId>,
    pub start: Option<u32>,
    pub end: Option<u32>,
}
/// Original instantiated member ownership; a display path never becomes executable identity.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceMemberDocument {
    pub member: SemanticId,
    pub declaration: SemanticId,
    pub instance: SemanticId,
    pub path: String,
}
/// One source check's original range within a potentially shared arithmetic region.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ArithmeticSourceDocument {
    pub region: usize,
    pub source: SemanticId,
    pub start_output: usize,
    pub end_output: usize,
    pub check_order: DerivativeOrder,
}
/// Which actual program owns the exposed source-check ranges.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ArithmeticMappingScope {
    /// Actual selected compiled product, including both static branch alternatives.
    SelectedCompiledProduct,
    /// Direct weighted differentiation retains original Value checks. Their ranges
    /// do not identify derivative-kernel instructions or assign shared runtime cost.
    OriginalValueChecksForWeightedProduct,
    /// Arithmetic exists but no original check ranges were supplied by its owner.
    Unavailable { reason: String },
}
/// One compiler-issued selected product template. Numerical weights/directions are not static identities.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceTemplateDocument {
    pub template: usize,
    pub body: ContentHash,
    pub order: DerivativeOrder,
    pub product: ProductKind,
    pub outputs: Vec<usize>,
    pub coordinates: Vec<usize>,
    pub arithmetic_mapping: ArithmeticMappingScope,
    pub arithmetic: Vec<ArithmeticSourceDocument>,
}
/// An ordered input gather: aliases deliberately retain separate formal positions.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceSlotDocument {
    pub formal: usize,
    pub member: SemanticId,
    pub scale: f64,
    pub offset: f64,
    pub fixed: bool,
}
/// Actual selected destination of an original ordered output contribution.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SourceContributionTarget {
    Row { row: SemanticId },
    Objective { ordinal: usize },
}
/// One ordered addend before duplicate accumulation, including orientation/weight.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceContributionDocument {
    pub output: usize,
    pub target: SourceContributionTarget,
    pub scale: f64,
}
/// The original occurrence's actual selected template and selectors.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceDemandDocument {
    pub demand: EvaluationDemand,
    pub template: usize,
    pub outputs: Vec<usize>,
    pub coordinates: Vec<usize>,
}
/// Original semantic occurrence and its concrete coordinate/output gathers.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceInstanceDocument {
    pub ordinal: usize,
    pub instance: SemanticId,
    pub body: ContentHash,
    pub checked_members: Vec<(SemanticId, SemanticId)>,
    pub slots: Vec<SourceSlotDocument>,
    pub contributions: Vec<SourceContributionDocument>,
    pub demands: Vec<SourceDemandDocument>,
}
/// Explicit bounded static source map. It supplies no native dual or stationarity pullback.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ModelingSourceMapDocument {
    pub version: pse_model::document::Version<1>,
    pub source_identity: ContentHash,
    pub numerical_identity: ContentHash,
    pub scope: SourceMapScope,
    pub rows: Vec<SemanticId>,
    pub columns: Vec<SemanticId>,
    pub fixed_inputs: Vec<SemanticId>,
    /// Frozen physical scales of the selected original rows and coordinates.
    pub row_nominals: Vec<(SemanticId, f64)>,
    pub variable_nominals: Vec<(SemanticId, f64)>,
    /// Objective signs in original objective order; never a native stationarity norm.
    pub objective_signs: Vec<f64>,
    pub fields: Vec<SourceFieldDocument>,
    pub members: Vec<SourceMemberDocument>,
    pub templates: Vec<SourceTemplateDocument>,
    pub instances: Vec<SourceInstanceDocument>,
    /// Absent mappings are explicit; this source view is not a transformed native attempt.
    pub native_pullback_unavailable: String,
}
/// The requested static document retains its existing runtime allocation owner.
#[derive(Debug)]
pub struct ModelingSourceMap {
    pub document: ModelingSourceMapDocument,
    _lease: Arc<pse_columnar::AllocationLease>,
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
impl Runtime {
    /// Inspect already admitted compiled artifacts under a separate explicit finite
    /// clock/allocation. This never evaluates equations or initiates a native solve.
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
            let assembly = crate::math::MathService::within_task(&scope, cancel,
                service.assemble(prepared.model.case.clone())).await?;
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
                        region: origin.region, source: origin.source, start_output: origin.start_output,
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
