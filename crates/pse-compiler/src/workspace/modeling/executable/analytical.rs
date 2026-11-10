// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Original selected analytical operations over producer-issued typed regions.
use super::*;
use pse_math::{
    analytical::{AnalyticalObservation, AnalyticalResult, GuardObservation},
    jets::EvaluationLimits,
};
use pse_modeling::resolved::{RegionOutput, RegionRequest, SelectedRegion};
use pse_quantity::resolved::receipts::ContractRecord;
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;

pse_contract::projection_enum! {
/// Scientific requests select original members and physical axes, never formula strings.
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize,schemars::JsonSchema)]
#[serde(tag="kind",rename_all="snake_case",deny_unknown_fields)]
pub enum AnalyticalOperation {
    projection: { name: "AnalyticalOperation", rust_path: crate::workspace::AnalyticalOperation, version: 1, direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Exact repeated derivative in original physical coordinate order.
    Derivative {
        /// Ordered actual original independent members.
        coordinates:Vec<SemanticId> => {tag:1,direction:pse_contract::boundary_projection::Direction::Both,rules:[]};
    } => 1;
    /// Native formal series around another original selected member.
    Series {
        /// Actual original independent member.
        coordinate:SemanticId => {tag:1,direction:pse_contract::boundary_projection::Direction::Both,rules:[]};
        /// Original exact source member supplying the expansion point.
        point_member:SemanticId => {tag:2,direction:pse_contract::boundary_projection::Direction::Both,rules:[]};
        /// Explicit bounded native depth.
        order:u32 => {tag:3,direction:pse_contract::boundary_projection::Direction::Both,rules:[]};
        /// Select relative rather than absolute native depth.
        relative:bool => {tag:4,direction:pse_contract::boundary_projection::Direction::Both,rules:[]};
    } => 2;
    /// Compare an original derivative or value with another original output.
    Identity {
        /// Original comparison output.
        other:SemanticId => {tag:1,direction:pse_contract::boundary_projection::Direction::Both,rules:[]};
        /// Ordered original derivative axes; empty compares values.
        coordinates:Vec<SemanticId> => {tag:2,direction:pse_contract::boundary_projection::Direction::Both,rules:[]};
        /// Optional original physical multiplier, applied after differentiation.
        #[serde(default)]
        multiplier:Option<SemanticId> => {tag:3,direction:pse_contract::boundary_projection::Direction::Both,rules:[]};
    } => 3;
    /// Canonicalize the original supported whole finite contraction.
    Indexed => 4;
    /// Formal native Padé product; never executable replacement or interval certificate.
    RationalApproximation {
        /// Actual original independent member.
        coordinate:SemanticId => {tag:1,direction:pse_contract::boundary_projection::Direction::Both,rules:[]};
        /// Original exact source expansion point.
        point_member:SemanticId => {tag:2,direction:pse_contract::boundary_projection::Direction::Both,rules:[]};
        /// Maximum numerator degree.
        numerator_degree:u32 => {tag:3,direction:pse_contract::boundary_projection::Direction::Both,rules:[]};
        /// Maximum denominator degree.
        denominator_degree:u32 => {tag:4,direction:pse_contract::boundary_projection::Direction::Both,rules:[]};
    } => 5;
    /// Opt-in bounded native relation research with independent exact verification.
    IntegerRelation {
        /// Other original closed selected outputs, in integer coefficient order.
        others:Vec<SemanticId> => {tag:1,direction:pse_contract::boundary_projection::Direction::Both,rules:[]};
        /// Actual finite native search controls and numerical tolerance.
        request:pse_math::analytical::RelationRequest => {tag:2,direction:pse_contract::boundary_projection::Direction::Both,rules:[]};
    } => 6;
}
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
/// An original independent coordinate and its complete admitted physical meaning.
pub struct AnalyticalCoordinate {
    /// Actual original selected scalar member identity.
    pub member: SemanticId,
    /// Original specialized source path labeling this actual physical axis.
    pub path: String,
    /// Independent input ordinal in this exact selected region.
    pub formal: usize,
    /// Complete actual admitted physical coordinate contract.
    pub contract: ContractRecord,
}
/// An additional original selected output, frozen beside its complete physical meaning.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AnalyticalOutput {
    /// Actual original selected member.
    pub member: SemanticId,
    /// Original specialized source path.
    pub path: String,
    /// Complete original output contract.
    pub contract: ContractRecord,
    /// Native original expression presentation, never execution input.
    pub expression: pse_math::analytical::MathematicalView,
}
/// Complete owned physical/source observation, sufficient for original protected readback.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AnalyticalSource {
    /// Actual original selected scalar member identity.
    pub member: SemanticId,
    /// Original authored declaration identity.
    pub declaration: SemanticId,
    /// Original specialized source path.
    pub path: String,
    /// Exact physical context consumed by source admission.
    pub consumed_physical: ContentHash,
    /// Actual native supplier environment used by the producer.
    pub environment: ContentHash,
    /// Actual admitted arithmetic policy: exact canonical binary64 bit authority,
    /// distinct from the ordinary numerical construction and portable cache.
    pub arithmetic_policy: ContentHash,
    /// Complete original output physical contract.
    pub original_contract: ContractRecord,
    /// Original selected expression, before differentiation or normalization.
    pub original_expression: pse_math::analytical::MathematicalView,
    /// Original point, comparator, multiplier or research terms in requested order.
    pub additional_outputs: Vec<AnalyticalOutput>,
    /// Complete physical contract of the requested mathematical output.
    pub result_contract: ContractRecord,
    /// Exact binary bits of the admitted derivative representation conversion.
    pub result_scale_bits: u64,
    /// Exact binary bits of the comparison's conversion to that same contract.
    pub comparison_scale_bits: Option<u64>,
    /// Original physical differentiation coordinates in requested order.
    pub coordinates: Vec<AnalyticalCoordinate>,
    /// Original reached source occurrences, including function selection.
    pub occurrences: Vec<pse_model::diagnostic::SourceLocation>,
    /// Original numeric domain and empirical applicability obligations.
    pub guards: Vec<GuardObservation>,
    /// Original finite carrier, selected membership and family correspondence.
    pub indexed_context: Option<serde_json::Value>,
}
#[derive(Clone, Debug)]
/// Source-bound native mathematical product retaining its exact source region and body.
pub struct CompiledAnalyticalResult {
    /// Immutable original admitted source or its complete owned projection.
    pub source: AnalyticalSource,
    /// Actual original mathematical demand.
    pub operation: AnalyticalOperation,
    /// Frozen mathematical observation produced once.
    pub observation: AnalyticalObservation,
    region: Arc<SelectedRegion>,
    body: Option<Arc<AdmittedBody>>,
    native: AnalyticalResult,
}
impl CompiledAnalyticalResult {
    /// Borrow the original exact source selection.
    pub fn region(&self) -> &SelectedRegion {
        &self.region
    }
    /// Borrow the native mathematical product and its retained original allocation owner.
    pub fn native(&self) -> &AnalyticalResult {
        &self.native
    }
    /// Actual retained native product and original source metadata extent.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + self.region.retained_bytes()
            + self.body.as_ref().map_or(0, |body| {
                body.descriptor_bytes() + body.math().retained_bytes()
            })
            + self.native.retained_bytes()
            + pse_math::analytical::owned_projection_extent(&self.source)
            + pse_math::analytical::owned_projection_extent(&self.operation)
    }
}
impl CompilerWorkspace {
    /// Admit exactly the selected original output and operation; none of this runs in
    /// a protected reader. The caller supplies the full original execution clock.
    pub fn prepare_analytical_member(
        &mut self,
        frontier: &ModelingPreparationFrontier,
        member: SemanticId,
        operation: AnalyticalOperation,
        limits: EvaluationLimits,
        scope: &pse_kernels::ExecutionScope,
    ) -> Result<CompiledAnalyticalResult> {
        scope.check().map_err(|error| match error {
            pse_kernels::ProviderError::Cancelled => MathError::Cancelled,
            other => MathError::Scope(other),
        })?;
        let compatible = frontier.physical_premises.as_ref().is_some_and(|premises| {
            premises.observe(&self.inputs.quantities, &self.inputs.preconditions)
        });
        if !compatible || frontier.environment != *self.inventory.environment(&self.db) {
            return Err(MathError::Contract(
                "analytical frontier physical or supplier premises changed".into(),
            )
            .into());
        }
        let model = frontier.model();
        let symbol = model
            .symbols
            .get(&member)
            .ok_or_else(|| CompileError::Missing("selected analytical member".into()))?;
        let root = model.symbol_root(member)?;
        let mut roots = vec![RegionOutput::Value(root)];
        match &operation {
            AnalyticalOperation::Series { point_member, .. }
            | AnalyticalOperation::RationalApproximation { point_member, .. } => {
                roots.push(RegionOutput::Value(model.symbol_root(*point_member)?))
            }
            AnalyticalOperation::Identity {
                other, multiplier, ..
            } => {
                roots.push(RegionOutput::Value(model.symbol_root(*other)?));
                if let Some(multiplier) = multiplier {
                    roots.push(RegionOutput::Value(model.symbol_root(*multiplier)?));
                }
            }
            AnalyticalOperation::IntegerRelation { others, .. } => {
                for other in others {
                    roots.push(RegionOutput::Value(model.symbol_root(*other)?));
                }
            }
            _ => {}
        }
        let context = TypeContext {
            admissions: None,
            formula_authority: None,
            quantities: &self.inputs.quantities,
            preconditions: &self.inputs.preconditions,
            scope: &frontier.projection.physical_scope,
        };
        let region = Arc::new(model.prepare_region(
            RegionRequest {
                outputs: &roots,
                validity: &[],
            },
            &context,
        )?);
        let selected_root = region.syntax.outputs[0];
        let original_contract = region
            .support
            .output(0)?
            .apply_contract(&region.support.node(selected_root)?.contract)
            .map_err(MathError::from)?
            .0;
        let mut refuse_provider = |_: &str| {
            Err(MathError::Contract(
                "selected analytical provider has no exact symbolic contract".into(),
            ))
        };
        let providers = crate::typed_math::prepare_selected_providers(
            &region.syntax,
            &region.support,
            &mut refuse_provider,
            &BTreeMap::new(),
            &context,
        )?;
        let body = Arc::new(
            crate::typed_math::Request {
                definition: symbol.lineage.declaration.as_id(),
                syntax: &region.syntax,
                inputs: &region.inputs,
                support: &region.support,
                providers: &providers,
                physical: frontier.physical,
                structure: region.syntax.identity()?,
                limits: frontier.projection.body_limits,
            }
            .admit_exact_source(
                &self.inputs.quantities,
                self.inputs.preconditions.as_ref(),
                limits,
                limits.scratch_bytes.saturating_mul(8),
                scope,
            )?,
        );
        let source = body.math().analytical_expression(0, limits, scope)?;
        if matches!(operation, AnalyticalOperation::Indexed) {
            return self.analytical_contraction(
                frontier,
                member,
                operation,
                region,
                original_contract,
                body.spec.policy,
                source.guard_context().to_vec(),
                limits,
                scope,
            );
        }
        let original_expression =
            pse_math::analytical::mathematical_view(source.expression(), limits, scope)?;
        let additional_members = match &operation {
            AnalyticalOperation::Series { point_member, .. }
            | AnalyticalOperation::RationalApproximation { point_member, .. } => {
                vec![*point_member]
            }
            AnalyticalOperation::Identity {
                other, multiplier, ..
            } => std::iter::once(*other)
                .chain(multiplier.iter().copied())
                .collect(),
            AnalyticalOperation::IntegerRelation { others, .. } => others.clone(),
            _ => vec![],
        };
        let mut additional_outputs = Vec::new();
        for (index, member) in additional_members.into_iter().enumerate() {
            let output = index + 1;
            let selected = body.math().analytical_expression(output, limits, scope)?;
            let contract = region
                .support
                .output(output)?
                .apply_contract(&region.support.node(region.syntax.outputs[output])?.contract)
                .map_err(MathError::from)?
                .0;
            additional_outputs.push(AnalyticalOutput {
                member,
                path: model
                    .symbols
                    .get(&member)
                    .ok_or_else(|| {
                        CompileError::Missing("original additional analytical source".into())
                    })?
                    .lineage
                    .path
                    .clone(),
                contract: ContractRecord::capture(&contract),
                expression: pse_math::analytical::mathematical_view(
                    selected.expression(),
                    limits,
                    scope,
                )?,
            });
        }
        let physical_axes = match &operation {
            AnalyticalOperation::Derivative { coordinates }
            | AnalyticalOperation::Identity { coordinates, .. } => coordinates.clone(),
            AnalyticalOperation::Series { coordinate, .. }
            | AnalyticalOperation::RationalApproximation { coordinate, .. } => vec![*coordinate],
            AnalyticalOperation::Indexed | AnalyticalOperation::IntegerRelation { .. } => vec![],
        };
        let mut axes = Vec::new();
        let mut axis_contracts = Vec::new();
        for member in &physical_axes {
            let binding = region
                .bindings
                .iter()
                .find(|binding| binding.actual == *member)
                .ok_or_else(|| {
                    CompileError::Missing(
                        "analytical axis is not an original selected input".into(),
                    )
                })?;
            let formal = region
                .inputs
                .iter()
                .position(|slot| *slot == binding.local)
                .ok_or_else(|| {
                    CompileError::Missing(
                        "analytical axis is a definition rather than an independent coordinate"
                            .into(),
                    )
                })?;
            let contract = region.support.input(binding.local)?.clone();
            axes.push(AnalyticalCoordinate {
                member: *member,
                path: model
                    .symbols
                    .get(member)
                    .ok_or_else(|| CompileError::Missing("original analytical axis source".into()))?
                    .lineage
                    .path
                    .clone(),
                formal,
                contract: ContractRecord::capture(&contract),
            });
            axis_contracts.push(contract);
        }
        let mut result_contract = original_contract.clone();
        let mut result_scale = 1.0_f64;
        let mut comparison_scale = None;
        if matches!(
            operation,
            AnalyticalOperation::Derivative { .. } | AnalyticalOperation::Identity { .. }
        ) && !axis_contracts.is_empty()
        {
            let inference = pse_quantity::resolved::infer_partial(
                &original_contract,
                &axis_contracts,
                &self.inputs.quantities,
                self.inputs.preconditions.as_ref(),
            )
            .map_err(MathError::from)?;
            result_scale = inference.result_scale;
            result_contract = inference.result;
        }
        let coordinate_ordinals = axes.iter().map(|axis| axis.formal).collect::<Vec<_>>();
        let mut guards = source.guard_context().to_vec();
        let native = match &operation {
            AnalyticalOperation::IntegerRelation { others, request } => {
                let mut selected = Vec::new();
                for output in 1..=others.len() {
                    let contract = region
                        .support
                        .output(output)?
                        .apply_contract(
                            &region.support.node(region.syntax.outputs[output])?.contract,
                        )
                        .map_err(MathError::from)?
                        .0;
                    if contract != original_contract {
                        return Err(MathError::Contract(
                            "integer relation compares different complete physical contracts"
                                .into(),
                        )
                        .into());
                    }
                    let other = body.math().analytical_expression(output, limits, scope)?;
                    guards.extend_from_slice(other.guard_context());
                    selected.push(other);
                }
                source.integer_relation(
                    &selected.iter().collect::<Vec<_>>(),
                    request,
                    limits,
                    scope,
                )?
            }
            AnalyticalOperation::Derivative { .. } => {
                source.derivative_with_scale(&coordinate_ordinals, result_scale, limits, scope)?
            }
            AnalyticalOperation::Series {
                order, relative, ..
            } => {
                let point = body.math().analytical_expression(1, limits, scope)?;
                let point_contract = region
                    .support
                    .output(1)?
                    .apply_contract(&region.support.node(region.syntax.outputs[1])?.contract)
                    .map_err(MathError::from)?
                    .0;
                if point_contract != axis_contracts[0] {
                    return Err(MathError::Contract(
                        "analytical expansion point changes the physical coordinate contract"
                            .into(),
                    )
                    .into());
                }
                guards.extend_from_slice(point.guard_context());
                let depth = if *relative {
                    pse_math::analytical::SeriesDepth::relative(*order)
                } else {
                    pse_math::analytical::SeriesDepth::absolute(*order)
                };
                source.series(
                    coordinate_ordinals[0],
                    point.expression(),
                    depth,
                    limits,
                    scope,
                )?
            }
            AnalyticalOperation::RationalApproximation {
                numerator_degree,
                denominator_degree,
                ..
            } => {
                let point = body.math().analytical_expression(1, limits, scope)?;
                let point_contract = region
                    .support
                    .output(1)?
                    .apply_contract(&region.support.node(region.syntax.outputs[1])?.contract)
                    .map_err(MathError::from)?
                    .0;
                if point_contract != axis_contracts[0] {
                    return Err(MathError::Contract(
                        "Padé expansion point changes physical coordinate contract".into(),
                    )
                    .into());
                }
                guards.extend_from_slice(point.guard_context());
                source.rational_approximation(
                    coordinate_ordinals[0],
                    point.expression(),
                    *numerator_degree,
                    *denominator_degree,
                    limits,
                    scope,
                )?
            }
            AnalyticalOperation::Identity { multiplier, .. } => {
                let other = body.math().analytical_expression(1, limits, scope)?;
                let factor = if multiplier.is_some() {
                    let factor = body.math().analytical_expression(2, limits, scope)?;
                    let factor_contract = region
                        .support
                        .output(2)?
                        .apply_contract(&region.support.node(region.syntax.outputs[2])?.contract)
                        .map_err(MathError::from)?
                        .0;
                    let inferred = pse_quantity::resolved::infer_operation(
                        &pse_quantity::infer::OpRequest::Mul,
                        &[result_contract.clone(), factor_contract],
                        None,
                        &self.inputs.quantities,
                        self.inputs.preconditions.as_ref(),
                    )
                    .map_err(MathError::from)?;
                    result_contract = inferred.result;
                    result_scale *= inferred.result_scale;
                    guards.extend_from_slice(factor.guard_context());
                    Some(factor)
                } else {
                    None
                };
                let other_contract = region
                    .support
                    .output(1)?
                    .apply_contract(&region.support.node(region.syntax.outputs[1])?.contract)
                    .map_err(MathError::from)?
                    .0;
                let (_, scale) = other_contract
                    .at_contract_boundary(&result_contract, &self.inputs.quantities)
                    .map_err(MathError::from)?;
                comparison_scale = Some(scale);
                guards.extend_from_slice(other.guard_context());
                source.identity_derivative_product_with_scales(
                    &other,
                    &coordinate_ordinals,
                    factor.as_ref(),
                    pse_math::analytical::ComparisonScales {
                        left: result_scale,
                        right: scale,
                    },
                    limits,
                    scope,
                )?
            }
            AnalyticalOperation::Indexed => {
                return Err(CompileError::Missing(
                    "indexed operation escaped its source route".into(),
                ));
            }
        };
        let source = AnalyticalSource {
            member,
            declaration: symbol.lineage.declaration.as_id(),
            path: symbol.lineage.path.clone(),
            consumed_physical: frontier.physical,
            environment: frontier.environment,
            arithmetic_policy: body.spec.policy,
            original_contract: ContractRecord::capture(&original_contract),
            original_expression,
            additional_outputs,
            result_contract: ContractRecord::capture(&result_contract),
            result_scale_bits: result_scale.to_bits(),
            comparison_scale_bits: comparison_scale.map(f64::to_bits),
            coordinates: axes,
            occurrences: region.occurrences.values().cloned().collect(),
            guards,
            indexed_context: None,
        };
        scope.check().map_err(|error| match error {
            pse_kernels::ProviderError::Cancelled => MathError::Cancelled,
            other => MathError::Scope(other),
        })?;
        Ok(CompiledAnalyticalResult {
            source,
            operation,
            observation: native.observation.clone(),
            region,
            body: Some(body),
            native,
        })
    }
    fn analytical_contraction(
        &self,
        frontier: &ModelingPreparationFrontier,
        member: SemanticId,
        operation: AnalyticalOperation,
        region: Arc<SelectedRegion>,
        contract: pse_quantity::ResolvedPhysicalContract,
        arithmetic_policy: ContentHash,
        guards: Vec<GuardObservation>,
        limits: EvaluationLimits,
        scope: &pse_kernels::ExecutionScope,
    ) -> Result<CompiledAnalyticalResult> {
        use pse_math::analytical::{
            IndexAlphabet, TensorAxis, TensorOccurrence, canonicalize_contraction,
        };
        // Follow only actual source aliases/control wrappers. Arithmetic outside
        // the reduction must never be mistaken for the whole contraction.
        let mut root = region.syntax.outputs[0];
        let mut visited = BTreeSet::new();
        while !region.indexed.contains_key(&root) && visited.insert(root) {
            use pse_modeling::resolved::{Node, Reference};
            root = match &region.syntax.nodes[root] {
                Node::Reference(Reference::Symbol(slot)) => match region
                    .syntax
                    .symbols
                    .at(*slot)
                    .and_then(|symbol| symbol.definition)
                {
                    Some(root) => root,
                    None => break,
                },
                Node::Sequence { value, .. } | Node::Evidence { value, .. } => *value,
                Node::Let { body, .. } => *body,
                _ => break,
            };
        }
        let contraction = region.indexed.get(&root).ok_or_else(|| {
            MathError::Contract(
                "selected output is not an original supported whole contraction".into(),
            )
        })?;
        let mut group = FramedHasher::new(pse_ids::Frame::ModelingResolvedBodyV1);
        group.str("analytical-index-group-v1");
        match contraction.carrier.kind {
            pse_modeling::resolved::CarrierKind::OrderedSet => {
                group.str("ordered-set");
            }
            pse_modeling::resolved::CarrierKind::Continuous { domain } => {
                group.str("continuous").id(&domain.as_id());
            }
        }
        pse_modeling::resolved::frame_type(&contraction.carrier.element, &mut group);
        group.u64(contraction.carrier.members.len() as u64);
        for value in contraction.carrier.members.iter() {
            value.frame(&mut group);
        }
        group.u64(contraction.selected.len() as u64);
        for value in contraction.selected.iter() {
            value.frame(&mut group);
        }
        let group = group.finish_id();
        let binder = pse_ids::named_id(
            contraction.source.field.declaration.as_id(),
            &format!(
                "{}:{}:{}",
                contraction.source.field.role,
                contraction.source.field.position,
                contraction.binder_spelling
            ),
        );
        let factors = contraction
            .factors
            .iter()
            .map(|factor| {
                let family =
                    pse_ids::named_id(factor.instance.as_id(), &factor.declaration.to_string());
                let axes = factor
                    .axes
                    .iter()
                    .map(|axis| match axis {
                        pse_modeling::resolved::ContractionAxis::Dummy => {
                            TensorAxis::Dummy { binder, group }
                        }
                        pse_modeling::resolved::ContractionAxis::Fixed(value) => {
                            TensorAxis::External {
                                identity: value.identity(),
                                group,
                            }
                        }
                    })
                    .collect();
                TensorOccurrence { family, axes }
            })
            .collect::<Vec<_>>();
        let native = canonicalize_contraction(
            &factors,
            &[IndexAlphabet { group, labels: 1 }],
            limits,
            scope,
        )?;
        let symbol = frontier
            .model()
            .symbols
            .get(&member)
            .ok_or_else(|| CompileError::Missing("indexed member".into()))?;
        let indexed_context = Some(
            serde_json::json!({"binder":contraction.binder_spelling,"carrier":contraction.carrier,"selected":contraction.selected,"filtered":contraction.filtered,"factors":contraction.factors.iter().zip(&factors).map(|(factor,native)|serde_json::json!({"family":native.family,"instance":factor.instance,"declaration":factor.declaration,"source":{"declaration":factor.source.field.declaration,"role":factor.source.field.role,"position":factor.source.field.position,"start":factor.source.span.start,"end":factor.source.span.end},"components":factor.components.iter().map(|member|serde_json::json!({"member":member,"path":frontier.model().symbols.get(member).map(|symbol|symbol.lineage.path.as_str())})).collect::<Vec<_>>(),"axes":factor.axes.iter().map(|axis|match axis{pse_modeling::resolved::ContractionAxis::Dummy=>serde_json::json!({"kind":"dummy","binder":binder,"group":group}),pse_modeling::resolved::ContractionAxis::Fixed(value)=>serde_json::json!({"kind":"fixed","identity":value.identity(),"value":value})}).collect::<Vec<_>>()})).collect::<Vec<_>>()}),
        );
        let original_expression = match &native.observation {
            AnalyticalObservation::Indexed(observation) => observation.original.clone(),
            _ => {
                return Err(CompileError::Missing(
                    "original indexed native observation".into(),
                ));
            }
        };
        let source = AnalyticalSource {
            member,
            declaration: symbol.lineage.declaration.as_id(),
            path: symbol.lineage.path.clone(),
            consumed_physical: frontier.physical,
            environment: frontier.environment,
            arithmetic_policy,
            original_contract: ContractRecord::capture(&contract),
            original_expression,
            additional_outputs: vec![],
            result_contract: ContractRecord::capture(&contract),
            result_scale_bits: 1.0_f64.to_bits(),
            comparison_scale_bits: None,
            coordinates: vec![],
            occurrences: region.occurrences.values().cloned().collect(),
            guards,
            indexed_context,
        };
        Ok(CompiledAnalyticalResult {
            source,
            operation,
            observation: native.observation.clone(),
            region,
            body: None,
            native,
        })
    }
}
