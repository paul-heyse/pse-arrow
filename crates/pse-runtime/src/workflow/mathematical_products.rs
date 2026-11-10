// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One concrete original-result family for selected analytical and exact mathematics.
use super::*;
use pse_compiler::workspace::AnalyticalOperation;
use pse_ids::{ContentHash, SemanticId};
use pse_modeling::{Bindings, DeclarationId, InstanceId};
use serde::{Deserialize, Serialize};

pse_contract::projection_struct! {
/// An original authored analytical selection; mathematical requests contain no parser input.
#[derive(Clone,Debug,Serialize,Deserialize,schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AnalyticalOperationRequest {
    projection: { name: "AnalyticalOperationRequest", rust_path: crate::workflow::AnalyticalOperationRequest, version:1,direction:pse_contract::boundary_projection::Direction::Both,reserved_tags:[],reserved_names:[] };
    /// Original authored definition.
    pub root:DeclarationId => {tag:1,direction:pse_contract::boundary_projection::Direction::Both,rules:[]};
    /// Original specialized occurrence.
    pub instance:InstanceId => {tag:2,direction:pse_contract::boundary_projection::Direction::Both,rules:[]};
    /// Original structural demand.
    pub bindings:Bindings => {tag:3,direction:pse_contract::boundary_projection::Direction::Both,rules:[]};
    /// Actual original mathematical output.
    pub member:SemanticId => {tag:4,direction:pse_contract::boundary_projection::Direction::Both,rules:[]};
    /// Actual bounded native mathematical operation.
    pub operation:AnalyticalOperation => {tag:5,direction:pse_contract::boundary_projection::Direction::Both,rules:[]};
    /// Explicit mathematical construction bounds.
    pub limits:pse_math::jets::EvaluationLimits => {tag:6,direction:pse_contract::boundary_projection::Direction::Both,rules:[]};
}
}
pse_contract::projection_enum! {
/// Source-bound mathematical operations share custody, not mathematical claim strength.
#[derive(Clone,Debug,Serialize,Deserialize,schemars::JsonSchema)]
#[serde(tag="kind",content="request",rename_all="snake_case",deny_unknown_fields)]
pub enum MathematicalRequest {
    projection: {name:"MathematicalRequest",rust_path:crate::workflow::MathematicalRequest,version:1,direction:pse_contract::boundary_projection::Direction::Both,reserved_tags:[],reserved_names:[]};
    /// Original authored exact analytical selection.
    Analytical(AnalyticalOperationRequest) => 1;
    /// Original authored exact/numerical assurance selection.
    Diagnostic(MathematicalDiagnosticOperation) => 2;
}
}
impl MathematicalRequest {
    /// Original source declaration selected by this operation.
    pub fn root(&self) -> DeclarationId {
        match self {
            Self::Analytical(request) => request.root,
            Self::Diagnostic(request) => request.root,
        }
    }
    /// Original specialized occurrence selected by this operation.
    pub fn instance(&self) -> InstanceId {
        match self {
            Self::Analytical(request) => request.instance,
            Self::Diagnostic(request) => request.instance,
        }
    }
}
/// Prepared immutable source and exact original request, independently of an attempt.
#[derive(Clone, Debug)]
pub struct PreparedMathematicalOperation {
    /// Immutable original admitted source or its complete owned projection.
    pub source: ModelingPackage,
    /// Exact original source-bound operation descriptor.
    pub request: MathematicalRequest,
    /// Original expansion and compiler admission settings.
    pub preparation: PreparationSettings,
}
/// One frozen native printer output, addressed from the contextual observation document.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StoredMathematicalView {
    /// Original specialized source path.
    pub path: String,
    /// Namespace-preserving native Symbolica file presentation.
    pub symbolica: String,
    /// Namespace-preserving native LaTeX presentation.
    pub latex: String,
    /// Namespace-preserving native Typst presentation.
    pub typst: String,
}
/// A completed mathematical observation. It does not grant physical use permission.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MathematicalProduct {
    #[serde(skip)]
    #[schemars(skip)]
    _owner: Option<Arc<dyn pse_columnar::PayloadOwner>>,
    /// Original selected mathematical owner.
    pub source_id: SemanticId,
    /// Exact immutable producing source revision.
    pub source_revision: ContentHash,
    /// Original mathematical producer family.
    pub product_kind: String,
    /// Actual original mathematical demand.
    pub operation: String,
    /// Actual mathematical assurance or hypothesis label.
    pub claim: String,
    /// Producer execution completed, including a fully observed unsupported outcome.
    pub succeeded: bool,
    /// The requested mathematical product/claim is available under original premises.
    pub supported: bool,
    /// Original reason the requested claim was unavailable.
    pub support_reason: Option<String>,
    /// Frozen original physical, source and guard context document.
    pub source_context: String,
    /// Frozen operation-specific result document; never executable input.
    pub result_context: String,
    /// Native printer outputs referenced by the frozen context documents.
    pub native_views: Vec<StoredMathematicalView>,
}
fn projection_owner(
    services: &PreparationServices,
    source: &impl Serialize,
    result: &impl Serialize,
) -> Result<Arc<dyn pse_columnar::PayloadOwner>, WorkflowError> {
    let bytes = document_extent(source, usize::MAX)?
        .checked_add(document_extent(result, usize::MAX)?)
        .and_then(|bytes| bytes.checked_mul(128))
        .and_then(|bytes| bytes.checked_add(size_of::<MathematicalProduct>()))
        .ok_or_else(|| {
            WorkflowError::Resource("mathematical owned projection extent overflow".into())
        })?;
    // Covers simultaneous serde Value/control nodes, geometric array/string
    // backing, view extraction and final encoded documents. This is a charge
    // against the same live deployment pool, not a second memory allowance.
    Ok(services
        .math()
        .reserve("mathematical:owned-publication", bytes)?)
}
fn frozen(
    value: &impl Serialize,
    path: &str,
    views: &mut Vec<StoredMathematicalView>,
) -> Result<String, WorkflowError> {
    fn extract(value: &mut serde_json::Value, path: &str, views: &mut Vec<StoredMathematicalView>) {
        match value {
            serde_json::Value::Object(fields)
                if fields.len() == 3
                    && ["symbolica", "latex", "typst"].iter().all(|name| {
                        fields.get(*name).is_some_and(serde_json::Value::is_string)
                    }) =>
            {
                let index = views.len();
                let string = |name: &str| {
                    fields
                        .get(name)
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or_default()
                        .to_owned()
                };
                views.push(StoredMathematicalView {
                    path: path.into(),
                    symbolica: string("symbolica"),
                    latex: string("latex"),
                    typst: string("typst"),
                });
                *value = serde_json::json!({"native_view":index});
            }
            serde_json::Value::Object(fields) => {
                for (name, value) in fields {
                    extract(value, &format!("{path}.{name}"), views)
                }
            }
            serde_json::Value::Array(values) => {
                for (index, value) in values.iter_mut().enumerate() {
                    extract(value, &format!("{path}[{index}]"), views)
                }
            }
            _ => {}
        }
    }
    let mut value = serde_json::to_value(value).map_err(|error| contract(error.to_string()))?;
    extract(&mut value, path, views);
    serde_json::to_string(&value).map_err(|error| contract(error.to_string()))
}
impl PreparedMathematicalOperation {
    /// Exact original operation identity, including source, physical context and suppliers.
    pub fn identity(&self) -> Result<ContentHash, WorkflowError> {
        let providers = self
            .source
            .registrations()
            .into_iter()
            .map(|(key, provider)| (key.0, provider.configuration_key()))
            .collect::<Vec<_>>();
        pse_ids::document::of(
            pse_ids::Frame::DurableMathematicalRequestV1,
            &(
                self.source.revision.identity(),
                self.source.physical.identity(),
                &self.request,
                &self.preparation,
                providers,
            ),
        )
        .map_err(|error| contract(error.to_string()))
    }
    /// Execute once while source mathematics and its original guards are available.
    pub async fn evaluate(
        &self,
        services: &PreparationServices,
        cancel: &crate::CancelSource,
    ) -> Result<MathematicalProduct, WorkflowError> {
        let mut views = Vec::new();
        match &self.request {
            MathematicalRequest::Analytical(request) => {
                let result = self
                    .source
                    .analyze_authored(
                        services,
                        request.root,
                        request.instance,
                        request.bindings.clone(),
                        self.preparation.limits,
                        request.member,
                        request.operation.clone(),
                        request.limits,
                        cancel,
                    )
                    .await?;
                let (claim, supported, reason) = match &result.observation {
                    pse_math::analytical::AnalyticalObservation::Derivative { .. } => {
                        ("exact_derivative", true, None)
                    }
                    pse_math::analytical::AnalyticalObservation::Series(_) => {
                        ("formal_series", true, None)
                    }
                    pse_math::analytical::AnalyticalObservation::Identity {
                        meaning: pse_math::analytical::IdentityMeaning::Inconclusive,
                        ..
                    } => (
                        "inconclusive_identity",
                        false,
                        Some(
                            "native exact normalization did not establish the requested identity"
                                .into(),
                        ),
                    ),
                    pse_math::analytical::AnalyticalObservation::Identity { .. } => {
                        ("exact_identity_comparison", true, None)
                    }
                    pse_math::analytical::AnalyticalObservation::Indexed(_) => {
                        ("canonical_indexed_mathematics", true, None)
                    }
                    pse_math::analytical::AnalyticalObservation::RationalApproximation {
                        ..
                    } => ("analytical_approximation", true, None),
                    pse_math::analytical::AnalyticalObservation::IntegerRelation(relation) => {
                        match relation.meaning {
                            pse_math::analytical::RelationMeaning::ExactAlgebraicZero => {
                                ("exact_algebraic_relation", true, None)
                            }
                            pse_math::analytical::RelationMeaning::NumericalHypothesis => {
                                ("numerical_relation_hypothesis", true, None)
                            }
                            pse_math::analytical::RelationMeaning::Unavailable => (
                                "relation_unavailable",
                                false,
                                Some("bounded native relation search returned no candidate".into()),
                            ),
                        }
                    }
                };
                let owner = projection_owner(
                    services,
                    &(
                        result.source_revision,
                        &result.source_versions,
                        &result.source,
                    ),
                    &result.observation,
                )?;
                let source_context = frozen(
                    &(
                        result.source_revision,
                        &result.source_versions,
                        &result.source,
                    ),
                    "source",
                    &mut views,
                )?;
                let result_context = frozen(&result.observation, "result", &mut views)?;
                Ok(MathematicalProduct {
                    _owner: Some(owner),
                    source_id: request.member,
                    source_revision: result.source_revision,
                    product_kind: "analytical".into(),
                    operation: serde_json::to_string(&request.operation)
                        .map_err(|error| contract(error.to_string()))?,
                    claim: claim.into(),
                    succeeded: true,
                    supported,
                    support_reason: reason,
                    source_context,
                    result_context,
                    native_views: views,
                })
            }
            MathematicalRequest::Diagnostic(request) => {
                let result = self
                    .source
                    .diagnose_selected_mathematics(services, request, &self.preparation, cancel)
                    .await?;
                let source_revision = self.source.revision.identity().as_id();
                let owner = projection_owner(
                    services,
                    &(source_revision, self.source.physical.identity(), request),
                    &**result,
                )?;
                let source_context = frozen(
                    &(source_revision, self.source.physical.identity(), request),
                    "source",
                    &mut views,
                )?;
                let result_context = frozen(&**result, "result", &mut views)?;
                let claim = serde_json::to_value(result.claim)
                    .map_err(|error| contract(error.to_string()))?
                    .as_str()
                    .unwrap_or("unknown")
                    .to_owned();
                Ok(MathematicalProduct {
                    _owner: Some(owner),
                    source_id: request.root.as_id(),
                    source_revision,
                    product_kind: "diagnostic".into(),
                    operation: "exact_assurance".into(),
                    claim,
                    succeeded: true,
                    supported: result.unsupported.is_none(),
                    support_reason: result.unsupported.clone(),
                    source_context,
                    result_context,
                    native_views: views,
                })
            }
        }
    }
}
impl MathematicalProduct {
    /// Original registry relations actually produced by this observation.
    pub fn table_ids(&self) -> Vec<SemanticId> {
        vec![pse_relations::generated::runtime::mathematical_products::RELATION_ID]
    }
    /// Project only frozen data. This method neither holds nor reopens scientific source.
    pub(super) async fn encode(
        &self,
        run_id: pse_model::generated::identities::RunId,
        services: &ExecutionServices,
        projection: &result_export::Projection,
        validation: &pse_relations::validate::ValidationContext,
    ) -> Result<(), WorkflowError> {
        use pse_relations::generated::runtime::mathematical_products as table;
        let registry = services.preparation().registry();
        let pool = services.preparation().pool();
        let mut rows = result_export::Rows::<table::Row>::new(
            projection,
            registry,
            &pool,
            &projection.cancel,
            validation,
        )
        .map_err(relation)?;
        let row = table::Row {
            run_id,
            ordinal: 0,
            source_id: self.source_id,
            source_revision: self.source_revision,
            product_kind: self.product_kind.clone(),
            operation: self.operation.clone(),
            claim: self.claim.clone(),
            supported: self.supported,
            support_reason: self.support_reason.clone(),
            source_context: self.source_context.clone(),
            result_context: self.result_context.clone(),
            native_views: self
                .native_views
                .iter()
                .map(
                    |view| pse_model::generated::structures::MathematicalNativeView {
                        path: view.path.clone(),
                        symbolica: view.symbolica.clone(),
                        latex: view.latex.clone(),
                        typst: view.typst.clone(),
                    },
                )
                .collect(),
        };
        rows.push(row).await.map_err(relation)?;
        rows.finish().await.map_err(relation)
    }
}
