// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One physical admission shared by fixed and adaptive estimators.
use super::super::{
    BindingAssignment, BindingQuantity, BindingTarget, ModelingPackage, PointOverlay,
    PreparationServices, WorkflowError, contract,
};
use super::*;
use pse_compiler::workspace::PreparedModeling;
use pse_ids::SemanticId;
use pse_quantity::{CanonicalConversionPlan, QuantityAdditionKind, QuantityRegistry, ScaleKind};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(super) enum LawCoordinate {
    Uniform {
        member: SemanticId,
        quantity: pse_quantity::QuantityTypeId,
        unit: pse_quantity::UnitId,
        lower: f64,
        upper: f64,
    },
    Categorical {
        member: SemanticId,
        quantity: pse_quantity::QuantityTypeId,
        unit: pse_quantity::UnitId,
        categories: Vec<(SemanticId, f64, f64)>,
    },
}

#[derive(Clone, Debug)]
pub(super) struct CoordinateDomain {
    domain: pse_model::generated::enums::ModelingVariableDomain,
    lower: f64,
    upper: f64,
}
impl CoordinateDomain {
    fn contains(&self, value: f64) -> bool {
        self.domain.contains(value, self.lower, self.upper)
    }
    fn continuous_interval(&self, lower: f64, upper: f64) -> bool {
        use pse_model::generated::enums::ModelingVariableDomain as Domain;
        self.contains(lower)
            && self.contains(upper)
            && matches!(self.domain, Domain::Continuous | Domain::Semicontinuous)
            && lower >= self.lower
            && upper <= self.upper
    }
}
impl LawCoordinate {
    pub(super) fn member(&self) -> SemanticId {
        match self {
            Self::Uniform { member, .. } | Self::Categorical { member, .. } => *member,
        }
    }
}

/// Admitted output-coordinate conversion and interval controls.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AdmittedOutput {
    /// Original physically checked output request.
    pub request: ExpectationOutput,
    /// Reference point expressed in the requested result unit.
    pub reference: f64,
    /// Characteristic interval expressed in the requested result unit.
    pub scale: f64,
    /// Absolute interval precision target in the requested result unit.
    pub absolute_error: Option<f64>,
    pub(super) conversion_scale: f64,
    pub(super) conversion_offset: f64,
}
impl AdmittedOutput {
    /// Convert an original canonical physical output into the selected result coordinate.
    pub fn represented(&self, canonical: f64) -> Result<f64, WorkflowError> {
        let value = (canonical - self.conversion_offset) / self.conversion_scale;
        if !value.is_finite() {
            return Err(contract(
                "expectation output has no finite selected representation",
            ));
        }
        Ok(value)
    }
}

/// Source-bound immutable admitted scientific meaning, never constructed by wire decoding.
#[derive(Clone, Debug)]
pub struct AdmittedExpectation {
    pub(super) request: std::sync::Arc<ExpectationRequest>,
    pub(super) coordinates: std::sync::Arc<[LawCoordinate]>,
    pub(super) outputs: std::sync::Arc<[AdmittedOutput]>,
    pub(super) original: ModelingPackage,
    pub(super) member_model: crate::math::modeling::ModelingPreparation,
    pub(super) transform_model: Option<crate::math::modeling::ModelingPreparation>,
    pub(super) member_domains: std::sync::Arc<BTreeMap<SemanticId, CoordinateDomain>>,
    pub(super) services: PreparationServices,
    pub(super) _owner: std::sync::Arc<pse_columnar::AllocationLease>,
}
impl AdmittedExpectation {
    /// Original request retained with every estimate and checkpoint.
    pub fn request(&self) -> &ExpectationRequest {
        &self.request
    }
    /// Ordered admitted physical outputs and their representation controls.
    pub fn outputs(&self) -> &[AdmittedOutput] {
        &self.outputs
    }
    /// Original member modeling preparation retained through actual populations.
    pub fn member_model(&self) -> &crate::math::modeling::ModelingPreparation {
        &self.member_model
    }
    /// Admitted normalized categorical masses, retaining original category identity and zeros.
    pub fn category_masses(&self) -> Vec<(SemanticId, Vec<(SemanticId, f64)>)> {
        self.coordinates
            .iter()
            .filter_map(|coordinate| match coordinate {
                LawCoordinate::Categorical {
                    member, categories, ..
                } => Some((
                    *member,
                    categories
                        .iter()
                        .map(|(category, _, mass)| (*category, *mass))
                        .collect(),
                )),
                _ => None,
            })
            .collect()
    }
    pub(super) fn validate_realized(&self, overlay: &PointOverlay) -> Result<(), WorkflowError> {
        let binding = self
            .original
            .admit_overlay(self.member_model.compiled(), overlay)?;
        for (member, entry) in &binding.entries {
            if !self
                .member_domains
                .get(member)
                .is_some_and(|domain| domain.contains(entry.canonical.into_inner()))
            {
                return Err(contract(
                    "expectation transformed assignment exceeds its authored physical domain",
                ));
            }
        }
        Ok(())
    }
}

fn quantity_error(error: pse_quantity::QuantityError) -> WorkflowError {
    super::super::math(pse_math::MathError::Quantity(error))
}
fn conversion(
    registry: &QuantityRegistry,
    quantity: pse_quantity::QuantityTypeId,
    unit: pse_quantity::UnitId,
) -> Result<CanonicalConversionPlan, WorkflowError> {
    CanonicalConversionPlan::registered(registry, quantity, unit).map_err(quantity_error)
}
fn canonical(registry: &QuantityRegistry, value: &BindingQuantity) -> Result<f64, WorkflowError> {
    conversion(registry, value.quantity, value.unit)?
        .apply(value.magnitude.into_inner())
        .map(|v| v.value())
        .map_err(quantity_error)
}

fn admit_output(
    registry: &QuantityRegistry,
    output: &ExpectationOutput,
) -> Result<AdmittedOutput, WorkflowError> {
    let quantity = registry
        .quantity_type(output.quantity)
        .map_err(quantity_error)?;
    if !quantity.key.shape.is_empty() {
        return Err(contract("expectation output must be a physical scalar"));
    }
    let origin_sensitive = registry
        .kind(quantity.key.kind)
        .map_err(quantity_error)?
        .addition_kind
        == QuantityAdditionKind::OriginSensitive;
    let point = origin_sensitive && quantity.key.scale_kind == ScaleKind::Point;
    let plan = conversion(registry, output.quantity, output.unit)?;
    let reference = match &output.reference {
        Some(reference) if reference.quantity == output.quantity => {
            (canonical(registry, reference)? - plan.offset()) / plan.scale()
        }
        Some(_) => {
            return Err(contract(
                "expectation reference has a different physical quantity",
            ));
        }
        None if point => {
            return Err(contract(
                "expectation affine point output requires an admitted reference",
            ));
        }
        None => 0.,
    };
    let mut interval_key = quantity.key.clone();
    if point {
        interval_key.scale_kind = ScaleKind::Difference;
    }
    let interval = |value: &BindingQuantity| -> Result<f64, WorkflowError> {
        let selected = registry
            .quantity_type(value.quantity)
            .map_err(quantity_error)?;
        if selected.key != interval_key {
            return Err(contract(
                "expectation scale/error must be a compatible physical interval",
            ));
        }
        let value = canonical(registry, value)? / plan.scale();
        if !value.is_finite() || value <= 0. {
            return Err(contract(
                "expectation scale/error must be finite and positive",
            ));
        }
        Ok(value)
    };
    let scale = interval(&output.scale)?;
    let absolute_error = output.absolute_error.as_ref().map(interval).transpose()?;
    if output
        .relative_error
        .is_some_and(|value| !value.is_finite() || value <= 0.)
    {
        return Err(contract(
            "expectation relative precision must be finite and positive",
        ));
    }
    if !reference.is_finite() {
        return Err(contract(
            "expectation reference representation is nonfinite",
        ));
    }
    Ok(AdmittedOutput {
        request: output.clone(),
        reference,
        scale,
        absolute_error,
        conversion_scale: plan.scale(),
        conversion_offset: plan.offset(),
    })
}

pub(super) fn validate_controls(request: &ExpectationRequest) -> Result<(), WorkflowError> {
    let controls = &request.controls;
    if controls.lanes == 0
        || controls.lanes > controls.samples_per_iteration
        || controls.samples_per_iteration < 2
        || controls.samples_per_iteration > super::super::MAXIMUM_STUDY_POINTS
        || controls.maximum_iterations == 0
        || controls.maximum_samples < 2
        || !controls.normalization_tolerance.is_finite()
        || controls.normalization_tolerance < 0.
        || controls.normalization_tolerance >= 1.
    {
        return Err(contract(
            "expectation finite population or normalization controls",
        ));
    }
    if request.coordinates.is_empty()
        || request.outputs.is_empty()
        || request.coordinates.len() > 64
        || request.outputs.len() > 1024
    {
        return Err(contract(
            "expectation requires bounded nonempty coordinates and physical outputs",
        ));
    }
    if let ExpectationEstimator::Adaptive(settings) = &request.estimator {
        // Maintained Numerica 3.0.1 serde replay uses nested Grid/Bin values and
        // may retain a full nested Sample inside deep accumulator history.
        // At most 4 grid containers + 2 sample containers per categorical level,
        // plus 16 envelope/continuous/history containers, stays at 112 for 16
        // levels, below serde_json's default 128. Continuous dimensions and
        // fixed Monte Carlo attribution are flat and keep the 64-coordinate cap.
        if request.coordinates.iter().filter(|coordinate|matches!(coordinate,ExpectationCoordinate::Categorical{..})).count()>16 {
            return Err(contract("expectation maintained Numerica 3.0.1 serde replay profile supports at most 16 adaptive categorical levels; deeper laws require a qualified deeper or flat supplier-state interface"));
        }
        if settings.bins == 0
            || settings.bins > super::super::MAXIMUM_STUDY_POINTS
            || settings.minimum_training_samples == 0
            || !settings.uniform_mixture.is_finite()
            || settings.uniform_mixture <= 0.
            || settings.uniform_mixture > 1.
            || !settings.maximum_probability_ratio.is_finite()
            || settings.maximum_probability_ratio < 1.
            || settings.bin_evolution.as_ref().is_some_and(|bins| {
                bins.is_empty()
                    || bins
                        .iter()
                        .any(|n| *n == 0 || *n > super::super::MAXIMUM_STUDY_POINTS)
            })
            || [
                settings.discrete_learning_rate,
                settings.continuous_learning_rate,
            ]
            .iter()
            .any(|value| !value.is_finite() || !(0. ..=1.).contains(value))
            || !request.outputs.iter().any(|output| output.train)
        {
            return Err(contract(
                "expectation adaptive controls or training-output selection",
            ));
        }
    }
    Ok(())
}

fn bind_value(
    package: &ModelingPackage,
    model: &PreparedModeling,
    target: &BindingTarget,
    value: &BindingQuantity,
) -> Result<super::super::AdmittedBindingEntry, WorkflowError> {
    package
        .admit_overlay(
            model,
            &PointOverlay {
                assignments: vec![BindingAssignment {
                    target: target.clone(),
                    value: value.clone(),
                }],
            },
        )?
        .entries
        .into_values()
        .next()
        .ok_or_else(|| contract("expectation coordinate binding absent"))
}

fn admit_coordinates(
    package: &ModelingPackage,
    model: &PreparedModeling,
    request: &ExpectationRequest,
) -> Result<Vec<LawCoordinate>, WorkflowError> {
    let mut members = BTreeSet::new();
    let mut result = Vec::with_capacity(request.coordinates.len());
    for coordinate in &request.coordinates {
        let admitted = match coordinate {
            ExpectationCoordinate::Uniform {
                target,
                lower,
                upper,
            } => {
                let a = bind_value(package, model, target, lower)?;
                let b = bind_value(package, model, target, upper)?;
                if a.member != b.member
                    || a.quantity != b.quantity
                    || a.canonical.into_inner() >= b.canonical.into_inner()
                {
                    return Err(contract(
                        "expectation uniform endpoints must define one finite increasing physical interval",
                    ));
                }
                LawCoordinate::Uniform {
                    member: a.member,
                    quantity: a.quantity,
                    unit: package
                        .quantities
                        .quantity_type(a.quantity)
                        .map_err(quantity_error)?
                        .canonical_unit,
                    lower: a.canonical.into_inner(),
                    upper: b.canonical.into_inner(),
                }
            }
            ExpectationCoordinate::Categorical { target, categories } => {
                if categories.is_empty() || categories.len() > super::super::MAXIMUM_STUDY_POINTS {
                    return Err(contract("expectation categorical population extent"));
                }
                let mut identities = BTreeSet::new();
                let mut sum = 0.;
                let mut selected = Vec::with_capacity(categories.len());
                let mut first = None;
                for category in categories {
                    if !identities.insert(category.category)
                        || !category.probability.is_finite()
                        || category.probability < 0.
                    {
                        return Err(contract("expectation category identity or probability"));
                    }
                    let value = bind_value(package, model, target, &category.value)?;
                    if first.as_ref().is_some_and(
                        |original: &super::super::AdmittedBindingEntry| {
                            original.member != value.member || original.quantity != value.quantity
                        },
                    ) {
                        return Err(contract(
                            "expectation categories have different physical meanings",
                        ));
                    }
                    if first.is_none() {
                        first = Some(value.clone());
                    }
                    sum += category.probability;
                    selected.push((
                        category.category,
                        value.canonical.into_inner(),
                        category.probability,
                    ));
                }
                if !sum.is_finite()
                    || sum <= 0.
                    || (sum - 1.).abs() > request.controls.normalization_tolerance
                {
                    return Err(contract(
                        "expectation categorical probabilities are not explicitly normalized",
                    ));
                }
                // Record the submitted masses in the original request; admission applies
                // exactly its declared normalization interpretation once.
                for (_, _, probability) in &mut selected {
                    *probability /= sum;
                }
                let first =
                    first.ok_or_else(|| contract("expectation category population empty"))?;
                LawCoordinate::Categorical {
                    member: first.member,
                    quantity: first.quantity,
                    unit: package
                        .quantities
                        .quantity_type(first.quantity)
                        .map_err(quantity_error)?
                        .canonical_unit,
                    categories: selected,
                }
            }
        };
        if !members.insert(admitted.member()) {
            return Err(contract(
                "expectation coordinates assign the same resolved member",
            ));
        }
        result.push(admitted);
    }
    let mut minimum_mass = 1.;
    for coordinate in &result {
        if let LawCoordinate::Categorical { categories, .. } = coordinate {
            minimum_mass *= categories
                .iter()
                .filter(|(_, _, p)| *p > 0.)
                .map(|(_, _, p)| *p)
                .fold(1_f64, f64::min);
        }
    }
    if !minimum_mass.is_finite() || minimum_mass <= 0. {
        return Err(contract(
            "expectation positive target mass is not representable",
        ));
    }
    Ok(result)
}

fn output_quantity(
    package: &ModelingPackage,
    model: &PreparedModeling,
    member: SemanticId,
) -> Result<pse_quantity::QuantityTypeId, WorkflowError> {
    let symbol = model.model.symbols.get(&member).ok_or_else(|| {
        contract("expectation output is not an original selected physical member")
    })?;
    let physical = symbol
        .ty
        .quantity_scheme()
        .ok_or_else(|| contract("expectation output is not physically typed"))?
        .resolve_contract_with_evidence(
            &package.quantities,
            &BTreeMap::new(),
            package.physical.preconditions.as_ref(),
        )
        .map_err(|error| {
            WorkflowError::from(pse_model::diagnostic::project_typed(
                &error,
                pse_diagnostics::DiagnosticStage::StudyAdmission,
            ))
        })?;
    if !physical.axes().is_empty() {
        return Err(contract("expectation output is not scalar"));
    }
    physical
        .named_id()
        .ok_or_else(|| contract("expectation output has no registered physical quantity"))
}

async fn domains(
    package: &ModelingPackage,
    services: &PreparationServices,
    model: &crate::math::modeling::ModelingPreparation,
    targets: &BTreeSet<SemanticId>,
    cancel: &crate::CancelSource,
) -> Result<BTreeMap<SemanticId, CoordinateDomain>, WorkflowError> {
    let (states, _owner) = package
        .design_variable_states(services, model, targets, cancel)
        .await?;
    let mut result = BTreeMap::new();
    for target in targets {
        let variable = model
            .compiled()
            .admitted
            .case()
            .variables()
            .iter()
            .find(|variable| variable.port.id == *target);
        let (domain, lower, upper) = variable.map_or(
            (
                pse_model::generated::enums::ModelingVariableDomain::Continuous,
                None,
                None,
            ),
            |variable| (variable.domain, variable.lower, variable.upper),
        );
        let lower = states
            .get(target)
            .and_then(|state| state.lower)
            .unwrap_or(lower)
            .unwrap_or(f64::NEG_INFINITY);
        let upper = states
            .get(target)
            .and_then(|state| state.upper)
            .unwrap_or(upper)
            .unwrap_or(f64::INFINITY);
        if lower > upper {
            return Err(contract(
                "expectation authored coordinate bounds are inconsistent",
            ));
        }
        result.insert(
            *target,
            CoordinateDomain {
                domain,
                lower,
                upper,
            },
        );
    }
    Ok(result)
}

impl ModelingPackage {
    /// Admit the original law through ordinary case/observation preparation and the common binder.
    pub async fn admit_expectation(
        &self,
        services: &PreparationServices,
        request: ExpectationRequest,
        cancel: &crate::CancelSource,
    ) -> Result<AdmittedExpectation, WorkflowError> {
        self.check_service_bindings(services)?;
        request.source.check(self)?;
        validate_controls(&request)?;
        request
            .preparation
            .limits
            .validate()
            .map_err(super::super::modeling_error)?;
        let bytes = super::super::study_variants::document_extent(&request, 16 << 20)?;
        let owner = services.math().reserve(
            "expectation:admitted-science",
            bytes
                .checked_mul(8)
                .ok_or_else(|| contract("expectation admission extent"))?,
        )?;
        let member_model = match &request.member {
            ExpectationMember::Case(case) => {
                let execution = self
                    .declared_execution_with_demand(
                        services,
                        case.case,
                        request.preparation.compiler,
                        case.settings.clone().profile()?,
                        Default::default(),
                        request.preparation.limits,
                        Vec::new(),
                        cancel,
                    )
                    .await?;
                super::super::study_operations::validate_declared(
                    case.case, case.route, &execution,
                )?;
                execution.model
            }
            ExpectationMember::Observation(operation) => {
                self.prepare_selected_observation(
                    services,
                    operation,
                    &request.preparation,
                    None,
                    cancel,
                )
                .await?
                .model
            }
        };
        let transform_model = match &request.transform {
            Some(transform) => {
                if transform.outputs.is_empty() {
                    return Err(contract(
                        "expectation transform requires physical output assignments",
                    ));
                }
                Some(
                    self.prepare_selected_observation(
                        services,
                        &transform.operation,
                        &request.preparation,
                        None,
                        cancel,
                    )
                    .await?
                    .model,
                )
            }
            None => None,
        };
        let law_model = transform_model.as_ref().unwrap_or(&member_model);
        let coordinates = admit_coordinates(self, law_model.compiled(), &request)?;
        let targets = coordinates
            .iter()
            .map(LawCoordinate::member)
            .collect::<BTreeSet<_>>();
        let law_domains = domains(self, services, law_model, &targets, cancel).await?;
        for coordinate in &coordinates {
            let domain = law_domains
                .get(&coordinate.member())
                .ok_or_else(|| contract("expectation original coordinate domain absent"))?;
            let valid = match coordinate {
                LawCoordinate::Uniform { lower, upper, .. } => {
                    domain.continuous_interval(*lower, *upper)
                }
                LawCoordinate::Categorical { categories, .. } => categories
                    .iter()
                    .all(|(_, value, _)| domain.contains(*value)),
            };
            if !valid {
                return Err(contract(
                    "expectation law exceeds its authored coordinate domain",
                ));
            }
        }
        let mut output_ids = BTreeSet::new();
        let mut outputs = Vec::with_capacity(request.outputs.len());
        for output in &request.outputs {
            if !output_ids.insert(output.output)
                || output_quantity(self, member_model.compiled(), output.output)? != output.quantity
            {
                return Err(contract(
                    "expectation output identity or original physical quantity differs",
                ));
            }
            if let ExpectationMember::Observation(operation) = &request.member {
                let selected = operation
                    .outputs
                    .iter()
                    .any(|selected| match &selected.member {
                        BindingTarget::Member(id) => *id == output.output,
                        BindingTarget::Path(path) => {
                            member_model.compiled().model.paths.get(path) == Some(&output.output)
                        }
                    });
                if !selected {
                    return Err(contract(
                        "expectation output was not demanded from selected observation",
                    ));
                }
            }
            outputs.push(admit_output(&self.quantities, output)?);
        }
        let member_domains = if let Some(transform) = &request.transform {
            let model = transform_model
                .as_ref()
                .ok_or_else(|| contract("expectation transform model absent"))?;
            let mut targets = BTreeSet::new();
            for output in &transform.outputs {
                if !transform
                    .operation
                    .outputs
                    .iter()
                    .any(|selected| match &selected.member {
                        BindingTarget::Member(member) => *member == output.output,
                        BindingTarget::Path(path) => {
                            model.compiled().model.paths.get(path) == Some(&output.output)
                        }
                    })
                {
                    return Err(contract(
                        "expectation transform output was not demanded from selected observation",
                    ));
                }
                let quantity = output_quantity(self, model.compiled(), output.output)?;
                // A physically typed reference point is used only to resolve the target;
                // every realized value is admitted again before child creation.
                let symbol = match &output.target {
                    BindingTarget::Member(id) => *id,
                    BindingTarget::Path(path) => *member_model
                        .compiled()
                        .model
                        .paths
                        .get(path)
                        .ok_or_else(|| contract("expectation transform target absent"))?,
                };
                if output_quantity(self, member_model.compiled(), symbol)? != quantity
                    || !member_model.compiled().admitted.inputs.contains(&symbol)
                    || !targets.insert(symbol)
                {
                    return Err(contract(
                        "expectation transform output target or quantity conflicts",
                    ));
                }
            }
            domains(self, services, &member_model, &targets, cancel).await?
        } else {
            law_domains
        };
        Ok(AdmittedExpectation {
            request: std::sync::Arc::new(request),
            coordinates: coordinates.into(),
            outputs: outputs.into(),
            original: self.clone(),
            member_model,
            transform_model,
            member_domains: std::sync::Arc::new(member_domains),
            services: services.clone(),
            _owner: owner,
        })
    }
}
