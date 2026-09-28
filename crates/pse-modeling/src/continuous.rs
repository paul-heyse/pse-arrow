// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Pure continuous-axis contracts. Numerical libraries supply collocation stencils.
use crate::{DeclarationId, ModelingError, Result, invalid};
use pse_ids::SemanticId;
use pse_model::generated::authored::modeling_declarations::{
    AuthoredModelingDeclarationsFieldValueCollocationScheme as Collocation,
    AuthoredModelingDeclarationsFieldValueDifferenceScheme as Difference,
};

/// Authored numerical data interpreted by a generic realization mechanism.
#[derive(Clone, Copy, Debug)]
pub enum Scheme<'a> {
    /// A first-derivative lattice stencil and one element's quadrature weights.
    Difference(&'a Difference),
    /// Shifted Jacobi roots and whether the right boundary is a collocation node.
    Collocation(&'a Collocation),
}
impl Scheme<'_> {
    /// Refuse invalid mathematical domains and malformed coefficient extents.
    pub fn validate(self, at: DeclarationId) -> Result<()> {
        match self {
            Self::Difference(v) => {
                if !(1..=64).contains(&v.order)
                    || v.offsets.len() < 2
                    || v.offsets.len() > 129
                    || v.offsets.len() != v.weights.len()
                    || v.quadrature.len() != v.order as usize + 1
                    || v.offsets.iter().any(|i| !(-64..=64).contains(i))
                    || v.offsets.windows(2).any(|v| v[0] >= v[1])
                    || v.weights
                        .iter()
                        .chain(&v.quadrature)
                        .any(|w| !w.is_finite())
                {
                    return Err(invalid(
                        at,
                        "bounded, ordered difference stencil and quadrature extents required",
                    ));
                }
                // Constants and the coordinate itself are invariants of every first derivative.
                let zeroth = v.weights.iter().sum::<f64>();
                let first = v
                    .weights
                    .iter()
                    .zip(&v.offsets)
                    .map(|(w, i)| w * *i as f64)
                    .sum::<f64>();
                let scale = v.weights.iter().map(|w| w.abs()).sum::<f64>().max(1.0);
                let tolerance = 256.0 * f64::EPSILON * scale;
                if !scale.is_finite()
                    || zeroth.abs() > tolerance
                    || (first - 1.0).abs() > tolerance
                    || (v.quadrature.iter().sum::<f64>() - 1.0).abs() > 256.0 * f64::EPSILON
                {
                    return Err(invalid(
                        at,
                        "difference weights must differentiate constants/coordinates and integrate unity",
                    ));
                }
            }
            Self::Collocation(v) => {
                if !v.alpha.is_finite() || !v.beta.is_finite() || v.alpha <= -1.0 || v.beta <= -1.0
                {
                    return Err(invalid(
                        at,
                        "Jacobi parameters must be finite and greater than -1",
                    ));
                }
            }
        }
        Ok(())
    }
}
/// Resolve a scheme through the same lexical/import visibility as all package declarations.
pub(crate) fn scheme<'a>(
    p: &'a crate::CheckedPackage,
    at: DeclarationId,
    name: &str,
) -> Result<Scheme<'a>> {
    let id = p
        .resolve(at, name)
        .ok_or_else(|| invalid(at, "discretization scheme is not visible"))?;
    let row = &p.declarations[&id];
    let scheme = if let Some(v) = &row.value.difference_scheme {
        Scheme::Difference(v)
    } else if let Some(v) = &row.value.collocation_scheme {
        Scheme::Collocation(v)
    } else {
        return Err(invalid(
            at,
            "discretization requires an authored numerical scheme",
        ));
    };
    scheme.validate(id)?;
    Ok(scheme)
}

/// One unit element's interpolation and quadrature data, ordered from left to right.
#[derive(Clone, Debug, PartialEq)]
pub struct ElementStencil {
    /// Normalized coordinates in [0, 1], including the left boundary.
    pub nodes: Vec<f64>,
    /// First derivatives in normalized coordinates. Empty rows are boundary exclusions.
    pub derivative: Vec<Vec<(usize, f64)>>,
    /// Integration weights over the unit interval.
    pub integral: Vec<f64>,
    /// Optional right-endpoint interpolation from left and interior nodes.
    pub endpoint: Option<Vec<f64>>,
    /// Optional derivative stencil on the complete uniform mesh, in units of mesh spacing.
    pub lattice: Option<Vec<(i64, f64)>>,
}
impl ElementStencil {
    /// Validate library output before allocating a realized mesh.
    pub fn validate(&self, at: DeclarationId) -> Result<()> {
        let n = self.nodes.len();
        if n < 2
            || self
                .endpoint
                .as_ref()
                .is_some_and(|v| v.len() != n - 1 || v.iter().any(|w| !w.is_finite()))
            || self.nodes[0] != 0.0
            || self.derivative.len() != n
            || self.integral.len() != n
            || self
                .nodes
                .iter()
                .any(|x| !x.is_finite() || !(0.0..=1.0).contains(x))
            || self.nodes.windows(2).any(|p| p[0] >= p[1])
            || self.integral.iter().any(|x| !x.is_finite())
            || self
                .derivative
                .iter()
                .flatten()
                .any(|(i, w)| *i >= n || !w.is_finite())
        {
            return Err(invalid(at, "invalid continuous element stencil"));
        }
        Ok(())
    }
}

/// A bounded numerical-library boundary. It is invoked after structural binding.
pub trait Discretizer {
    /// Build one dimensionless element without performing model evaluation or I/O.
    fn element(
        &self,
        scheme: Scheme<'_>,
        order: usize,
        at: DeclarationId,
    ) -> Result<ElementStencil>;
}

/// Realization of authored lattice data, requiring no numerical library startup.
#[derive(Debug)]
pub struct FiniteDifference;
impl Discretizer for FiniteDifference {
    fn element(
        &self,
        scheme: Scheme<'_>,
        order: usize,
        at: DeclarationId,
    ) -> Result<ElementStencil> {
        scheme.validate(at)?;
        let Scheme::Difference(v) = scheme else {
            return Err(ModelingError::Unsupported {
                declaration: at.into(),
                capability: "Jacobi collocation".into(),
            });
        };
        if v.order as usize != order {
            return Err(invalid(
                at,
                "selected order differs from the authored difference stencil",
            ));
        }
        Ok(ElementStencil {
            nodes: (0..=order).map(|i| i as f64 / order as f64).collect(),
            derivative: vec![vec![]; order + 1],
            integral: v.quadrature.clone(),
            endpoint: None,
            lattice: Some(
                v.offsets
                    .iter()
                    .copied()
                    .zip(v.weights.iter().copied())
                    .collect(),
            ),
        })
    }
}

/// A realized domain, retaining numerical data separately from coordinate identity.
#[derive(Clone, Debug, PartialEq)]
pub struct Mesh {
    /// Instantiated axis identity.
    pub id: SemanticId,
    /// Coordinate values with declaration/element/local-coordinate identities.
    pub points: Vec<crate::specialize::Value>,
    /// Derivative weights in inverse canonical axis units.
    pub derivative: Vec<Vec<(usize, f64)>>,
    /// Integration weights in canonical axis units.
    pub integral: Vec<f64>,
    /// Per-element endpoint interpolation in global coordinate indices.
    pub continuity: Vec<(usize, Vec<(usize, f64)>)>,
}

/// Runtime time coordinate generated from an authored continuous domain.
#[derive(Clone, Debug, PartialEq)]
pub struct IntegratedAxis {
    pub id: SemanticId,
    pub coordinate: SemanticId,
    pub time: SemanticId,
    pub quantity: pse_quantity::QuantityTypeId,
    pub lower: f64,
    pub upper: f64,
}
/// A derivative coordinate belongs to one original state and one integrated time axis.
#[derive(Clone, Debug, PartialEq)]
pub struct IntegratedDerivative {
    pub state: SemanticId,
    pub rate: SemanticId,
    pub axis: SemanticId,
    pub lineage: crate::specialize::Lineage,
}

/// Definite integral over one complete integrated axis. The result is terminal data,
/// not a time-varying prefix integral available to the differential equations.
#[derive(Clone, Debug, PartialEq)]
pub struct IntegratedIntegral {
    pub result: SemanticId,
    pub integrand: SemanticId,
    pub axis: SemanticId,
}
