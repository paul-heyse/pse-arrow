// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Generated designs are ordinary physical overlays; study admission owns their identities.
use super::{
    BindingAssignment, BindingQuantity, BindingTarget, ModelingPackage, PointOverlay,
    PreparationServices, WorkflowError, contract,
};
use pse_ids::SemanticId;
use pse_model::{generated::enums::ModelingVariableDomain, scalars::FiniteBound};
use pse_quantity::{CanonicalConversionPlan, QuantityTypeId, UnitId};
use std::{collections::BTreeSet, sync::Arc};

/// Explicit library selection. Classic, rather than the supplier's ESE default, is local default.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum DesignKind {
    /// Random point within each Latin hypercube stratum.
    #[default]
    Classic,
    /// Shuffled stratum centers.
    Centered,
    /// Best minimum separation of five classic candidates.
    Maximin,
    /// Best minimum separation of five centered candidates.
    CenteredMaximin,
    /// Independent uniform unit-cube points.
    Random,
    /// Exact Cartesian product of the requested coordinate levels.
    Grid,
}
impl DesignKind {
    pub(crate) fn quadratic(self) -> bool {
        matches!(self, Self::Maximin | Self::CenteredMaximin)
    }
}
/// Preserve integral authoring before conversion to the finite-f64 assignment boundary.
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum DesignLevel {
    /// An explicitly binary64 physical value.
    Real(FiniteBound),
    /// An exact authored integer, refused if finite-f64 assignments cannot represent it.
    Integer(i64),
}
impl DesignLevel {
    pub(crate) fn value(self) -> Option<f64> {
        match self {
            Self::Real(value) => Some(value.into_inner()),
            Self::Integer(value) => {
                let represented = value as f64;
                ((represented as i128) == i128::from(value)).then_some(represented)
            }
        }
    }
}
/// One numeric, quantity-typed authored coordinate, in its supplied representation.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DesignCoordinate {
    /// Stable member or authored path, resolved against this package's selected root.
    pub target: BindingTarget,
    /// Complete quantity meaning.
    pub quantity: QuantityTypeId,
    /// Representation for every emitted physical assignment.
    pub unit: UnitId,
    /// Closed finite interval; omission selects finite authored bounds.
    pub range: Option<[FiniteBound; 2]>,
    /// Ordered admissible values, required for noncontinuous domains.
    #[serde(default)]
    pub levels: Vec<DesignLevel>,
}
/// Explicit finite producer request; does not change StudyRequest's wire interpretation.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DesignRequest {
    /// Original root whose numeric inputs are varied.
    pub root: pse_modeling::DeclarationId,
    /// Original root specialization; callers may retain structural arguments/demands.
    #[serde(
        default,
        serialize_with = "super::replay::serialize_bindings",
        deserialize_with = "super::replay::deserialize_bindings"
    )]
    #[schemars(with = "Vec<u8>")]
    pub bindings: pse_modeling::Bindings,
    /// Coordinate order is output order.
    pub coordinates: Vec<DesignCoordinate>,
    /// Explicit producer selection.
    #[serde(default)]
    pub kind: DesignKind,
    /// Positive complete point count; grid product must equal it exactly.
    pub count: usize,
    /// One fresh Xoshiro256Plus stream per generation; unused for Grid.
    pub seed: u64,
    /// For Grid only: positive counts per coordinate, last coordinate varies fastest.
    #[serde(default)]
    pub grid_levels: Vec<usize>,
    /// Finite generation-work allowance, counting normalized coordinate and distance components.
    pub maximum_work: usize,
}
/// Attribution travels with the retained design; realized assignments remain identity authority.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DesignAttribution {
    /// Producer kind.
    pub kind: DesignKind,
    /// Submitted seed; Grid consumes no random stream.
    pub seed: u64,
    /// Complete occurrence count, including duplicate realized inputs.
    pub count: usize,
    /// Exact installed library and local mapping realization.
    pub algorithm: String,
    /// Original ordered physical coordinate description.
    pub coordinates: Vec<DesignCoordinate>,
}
/// Owning result; cloning retains the same output reservation rather than cloning point buffers.
#[derive(Clone, Debug)]
pub struct GeneratedDesign {
    inner: Arc<DesignData>,
    _owner: Arc<pse_columnar::AllocationLease>,
}
#[derive(Debug, serde::Serialize)]
struct DesignData {
    points: Vec<PointOverlay>,
    attribution: DesignAttribution,
}
impl GeneratedDesign {
    /// Borrow complete raw overlays while this result retains their prepaid storage.
    pub fn points(&self) -> &[PointOverlay] {
        &self.inner.points
    }
    /// Construction attribution; not a regeneration promise or scientific distribution claim.
    pub fn attribution(&self) -> &DesignAttribution {
        &self.inner.attribution
    }
    /// Export an independent bounded document copy, retaining both source and conversion storage.
    /// # Errors
    /// Finite conversion extent or shared allocation admission fails before copying.
    pub fn export(&self, services: &PreparationServices) -> Result<DesignExport, WorkflowError> {
        let json = super::study_variants::document_extent(
            &*self.inner,
            services.math().policy().worker_bytes,
        )?;
        let extent = json
            .checked_mul(8)
            .and_then(|n| n.checked_add(self._owner.size()))
            .ok_or_else(|| contract("design export extent"))?;
        let owner = services
            .math()
            .reserve("design:export", extent)
            .map_err(WorkflowError::Math)?;
        Ok(DesignExport {
            points: self.inner.points.clone(),
            attribution: self.inner.attribution.clone(),
            _source: self.clone(),
            _owner: owner,
        })
    }
    pub(crate) fn from_parts(
        points: Vec<PointOverlay>,
        attribution: DesignAttribution,
        owner: Arc<pse_columnar::AllocationLease>,
    ) -> Self {
        Self {
            inner: Arc::new(DesignData {
                points,
                attribution,
            }),
            _owner: owner,
        }
    }
}
/// Explicit independent document copy. Its native owners live through document conversion.
#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DesignExport {
    /// Complete ordered raw overlays for StudyPoint authoring.
    pub points: Vec<PointOverlay>,
    /// Generator attribution, excluded from binding identity.
    pub attribution: DesignAttribution,
    #[serde(skip)]
    #[schemars(skip)]
    _source: GeneratedDesign,
    #[serde(skip)]
    #[schemars(skip)]
    _owner: Arc<pse_columnar::AllocationLease>,
}
#[derive(Clone, Debug)]
pub(crate) struct Coordinate {
    pub(crate) description: DesignCoordinate,
    pub(crate) lower: f64,
    pub(crate) upper: f64,
}
/// No clamp, no fused operation, and no overflowing subtraction of opposite-sign endpoints.
pub(crate) fn interpolate(lower: f64, upper: f64, u: f64) -> f64 {
    if u == 0.0 {
        return lower;
    }
    if u == 1.0 {
        return upper;
    }
    if lower == upper {
        return lower;
    }
    if lower < 0.0 && upper > 0.0 {
        let left = (1.0 - u) * lower;
        let right = u * upper;
        left + right
    } else {
        let distance = upper - lower;
        if u <= 0.5 {
            let offset = u * distance;
            lower + offset
        } else {
            let offset = (1.0 - u) * distance;
            upper - offset
        }
    }
}
// Units admit positive finite scales. Applying that existing conversion is monotone under
// the declared binary64 arithmetic, including plateaus. Search its representable preimage;
// inverting coefficients and rounding once does not establish containment near affine offsets.
fn default_range(
    plan: &CanonicalConversionPlan,
    lower: f64,
    upper: f64,
) -> Result<[f64; 2], WorkflowError> {
    fn key(value: f64) -> u64 {
        let bits = value.to_bits();
        if value.is_sign_negative() {
            !bits
        } else {
            bits ^ (1_u64 << 63)
        }
    }
    fn value(key: u64) -> f64 {
        f64::from_bits(if key & (1_u64 << 63) != 0 {
            key ^ (1_u64 << 63)
        } else {
            !key
        })
    }
    let compare = |candidate: f64, bound: f64| -> Result<std::cmp::Ordering, WorkflowError> {
        match plan.apply(candidate) {
            Ok(actual) => actual
                .value()
                .partial_cmp(&bound)
                .ok_or_else(|| contract("finite design bound comparison")),
            // The admitted plan applies multiply then add; finite offset cannot change
            // the sign of an overflowing product. No other application failure is hidden.
            Err(pse_quantity::QuantityError::NonfiniteConversion { .. }) => {
                Ok(if candidate.is_sign_negative() {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Greater
                })
            }
            Err(error) => Err(super::math(pse_math::MathError::Quantity(error))),
        }
    };
    let (first, last) = (key(-f64::MAX), key(f64::MAX));
    let (mut low, mut high) = (first, last);
    while low < high {
        let middle = low + (high - low) / 2;
        if compare(value(middle), lower)?.is_lt() {
            low = middle + 1;
        } else {
            high = middle;
        }
    }
    let a = value(low);
    let (mut low, mut high) = (first, last);
    while low < high {
        let distance = high - low;
        let middle = low + distance / 2 + distance % 2;
        if compare(value(middle), upper)?.is_gt() {
            high = middle - 1;
        } else {
            low = middle;
        }
    }
    let b = value(low);
    let inside = |candidate: f64| {
        plan.apply(candidate)
            .is_ok_and(|actual| actual.value() >= lower && actual.value() <= upper)
    };
    if a > b || !inside(a) || !inside(b) {
        return Err(contract(
            "authored bounds have no finite representable values in selected unit",
        ));
    }
    // A fixed canonical coordinate does not acquire artificial spread from a rounded plateau.
    Ok(if lower == upper { [a, a] } else { [a, b] })
}
impl ModelingPackage {
    /// Produce one complete bounded design without connecting a store or starting a solver.
    /// # Errors
    /// Invalid targets/physical meaning/domains, finite work/storage refusal, cancellation or expiry.
    pub async fn generate_design(
        &self,
        services: &PreparationServices,
        request: &DesignRequest,
        limits: pse_modeling::Limits,
        cancel: &crate::CancelSource,
    ) -> Result<GeneratedDesign, WorkflowError> {
        self.check_service_bindings(services)?;
        // Request serialization is a bounded counting traversal, before any protected clone.
        let input = super::study_variants::document_extent(
            request,
            services.math().policy().workspace_bytes / 32,
        )?;
        let extent = input
            .checked_mul(32)
            .and_then(|n| n.checked_add(4096))
            .ok_or_else(|| contract("design preparation extent"))?;
        let _preparation = services
            .math()
            .reserve("design:preparation", extent)
            .map_err(WorkflowError::Math)?;
        if request.coordinates.len() > limits.members {
            return Err(contract("design coordinate extent"));
        }
        let driver = services.math().preparation_driver(cancel)?;
        services
            .scope_activity(async {
                let instance = pse_modeling::specialize::root_instance(request.root);
                let mut bindings = request.bindings.clone();
                for coordinate in &request.coordinates {
                    if let BindingTarget::Path(path) = &coordinate.target {
                        bindings.demand.push(path.clone());
                    }
                }
                let model = self
                    .prepare(services, request.root, instance, bindings, limits, &driver)
                    .await?;
                let targets = request
                    .coordinates
                    .iter()
                    .map(|coordinate| match &coordinate.target {
                        BindingTarget::Member(id) => Ok(*id),
                        BindingTarget::Path(path) => model
                            .compiled()
                            .model
                            .paths
                            .get(path)
                            .copied()
                            .ok_or_else(|| contract("unknown design target")),
                    })
                    .collect::<Result<BTreeSet<_>, _>>()?;
                let (states, _states_owner) = self
                    .design_variable_states(services, &model, &targets, &driver)
                    .await?;
                let coordinates = self.design_coordinates(model.compiled(), &states, request)?;
                services
                    .math()
                    .generate_design_job(request, coordinates, &driver)
                    .await
                    .map_err(WorkflowError::Math)
            })
            .await
    }
    fn design_coordinates(
        &self,
        model: &pse_compiler::workspace::PreparedModeling,
        states: &std::collections::BTreeMap<
            SemanticId,
            pse_compiler::workspace::ModelingVariableState,
        >,
        request: &DesignRequest,
    ) -> Result<Vec<Coordinate>, WorkflowError> {
        if request.coordinates.is_empty()
            || request.count == 0
            || request.count > super::MAXIMUM_STUDY_POINTS
            || request.maximum_work == 0
        {
            return Err(contract(
                "positive bounded design dimensions, count and work",
            ));
        }
        if request.kind.quadratic() && request.count < 2 {
            return Err(contract("maximin requires at least two points"));
        }
        let mut seen = BTreeSet::new();
        let mut output = Vec::with_capacity(request.coordinates.len());
        for description in &request.coordinates {
            let probe = PointOverlay {
                assignments: vec![BindingAssignment {
                    target: description.target.clone(),
                    value: BindingQuantity {
                        magnitude: FiniteBound::try_new(0.0)
                            .map_err(|_| contract("finite probe"))?,
                        quantity: description.quantity,
                        unit: description.unit,
                    },
                }],
            };
            let admitted = self.admit_overlay(model, &probe)?;
            let entry = admitted
                .entries
                .values()
                .next()
                .ok_or_else(|| contract("missing admitted design coordinate"))?;
            if !seen.insert(entry.member) {
                return Err(contract("duplicate resolved design coordinate"));
            }
            let variable = model
                .admitted
                .case()
                .variables()
                .iter()
                .find(|v| v.port.id == entry.member);
            let (domain, mut lower, mut upper) = variable
                .map_or((ModelingVariableDomain::Continuous, None, None), |v| {
                    (v.domain, v.lower, v.upper)
                });
            if let Some(state) = states.get(&entry.member) {
                lower = state.lower.unwrap_or(lower);
                upper = state.upper.unwrap_or(upper);
            }
            let canonical = self
                .quantities
                .quantity_type(description.quantity)
                .map_err(|e| super::math(pse_math::MathError::Quantity(e)))?
                .canonical_unit;
            let conversion = if description.unit == canonical {
                CanonicalConversionPlan::canonical(&self.quantities, description.quantity)
            } else {
                CanonicalConversionPlan::registered(
                    &self.quantities,
                    description.quantity,
                    description.unit,
                )
            }
            .map_err(|e| super::math(pse_math::MathError::Quantity(e)))?;
            let [a, b] = match description.range {
                Some(range) => range.map(FiniteBound::into_inner),
                None => {
                    let [a, b] = [lower, upper].map(|v| {
                        v.ok_or_else(|| {
                            contract("design needs explicit finite range for unbounded coordinate")
                        })
                    });
                    let (a, b) = (a?, b?);
                    if description.unit == canonical {
                        [a, b]
                    } else {
                        default_range(&conversion, a, b)?
                    }
                }
            };
            if !a.is_finite() || !b.is_finite() || a > b {
                return Err(contract("ordered finite design range"));
            }
            let contains = |value: f64| -> Result<bool, WorkflowError> {
                let value = conversion
                    .apply(value)
                    .map_err(|e| super::math(pse_math::MathError::Quantity(e)))?
                    .value();
                Ok(domain.contains(
                    value,
                    lower.unwrap_or(f64::NEG_INFINITY),
                    upper.unwrap_or(f64::INFINITY),
                ))
            };
            let canonical_a = conversion
                .apply(a)
                .map_err(|e| super::math(pse_math::MathError::Quantity(e)))?
                .value();
            let canonical_b = conversion
                .apply(b)
                .map_err(|e| super::math(pse_math::MathError::Quantity(e)))?
                .value();
            // Explicit levels may select the existing semi-domain's separate canonical
            // zero branch. Their finite envelope is bounded by that union's hull;
            // individual values still pass domain.contains below, excluding its gap.
            let range_lower = lower.unwrap_or(f64::NEG_INFINITY);
            let range_upper = upper.unwrap_or(f64::INFINITY);
            let (range_lower, range_upper) = if domain.is_semi() && !description.levels.is_empty() {
                (range_lower.min(0.0), range_upper.max(0.0))
            } else {
                (range_lower, range_upper)
            };
            if canonical_a.min(canonical_b) < range_lower
                || canonical_a.max(canonical_b) > range_upper
            {
                return Err(contract("design range exceeds authored bounds"));
            }
            if description.levels.is_empty() {
                if domain != ModelingVariableDomain::Continuous {
                    return Err(contract(
                        "discrete or semi domain requires explicit admissible levels",
                    ));
                }
                if !contains(a)? || !contains(b)? {
                    return Err(contract("design range exceeds authored domain"));
                }
            } else {
                for (i, level) in description.levels.iter().enumerate() {
                    let level = level.value().ok_or_else(|| {
                        contract("integer design level not representable by physical assignment")
                    })?;
                    let previous = if i > 0 {
                        description.levels[i - 1].value()
                    } else {
                        None
                    };
                    if level < a
                        || level > b
                        || !contains(level)?
                        || previous.is_some_and(|previous| previous >= level)
                    {
                        return Err(contract("ordered distinct admissible design levels"));
                    }
                }
            }
            output.push(Coordinate {
                description: description.clone(),
                lower: a,
                upper: b,
            });
        }
        if request.kind == DesignKind::Grid {
            if request.grid_levels.len() != output.len() {
                return Err(contract("one grid count per coordinate"));
            }
            let mut product = 1usize;
            for (coordinate, &count) in output.iter().zip(&request.grid_levels) {
                if count == 0
                    || (count == 1
                        && coordinate.lower != coordinate.upper
                        && coordinate.description.levels.len() != 1)
                    || (!coordinate.description.levels.is_empty()
                        && coordinate.description.levels.len() != count)
                {
                    return Err(contract(
                        "explicit positive grid levels including singleton value",
                    ));
                }
                product = product
                    .checked_mul(count)
                    .ok_or_else(|| contract("grid product overflow"))?;
            }
            if product != request.count {
                return Err(contract("grid product differs from requested count"));
            }
        } else if !request.grid_levels.is_empty() {
            return Err(contract("grid counts supplied to non-grid design"));
        }
        Ok(output)
    }
}

#[cfg(test)]
#[path = "sampling_tests.rs"]
mod tests;
