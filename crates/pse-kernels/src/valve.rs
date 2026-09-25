// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Declared C2 nonreversing valve law; num-dual owns its exact local derivatives.
use crate::{
    DerivativeOrder, EvaluationContext, Phase, Port, Provider, ProviderError, ProviderFactory,
    ProviderRequest, ProviderSpec, ProviderValues,
};
use num_dual::DualNum;
use pse_ids::{FramedHasher, SemanticId, named_id};

/// Dimensionless pressure-difference shape. The physical caller declares a positive width.
fn shape<D: DualNum<f64> + Copy>(s: D) -> D {
    if s.re() <= 0.0 {
        D::from_re(0.0.into())
    } else if s.re() >= 1.0 {
        s.sqrt()
    } else {
        s.powi(3) * (D::from_re(63.0.into()) - s * 90.0 + s.powi(2) * 35.0) / 8.0
    }
}

/// Immutable physical-law registration, evaluated on independent worker-local instances.
#[derive(Clone, Debug)]
pub struct DirectionalValve {
    spec: ProviderSpec,
}
impl DirectionalValve {
    /// Admit the normalized pressure and flow-shape ports in the same neutral unit.
    pub fn new(
        id: SemanticId,
        neutral: pse_quantity::QuantityTypeId,
        registry: &pse_quantity::QuantityRegistry,
    ) -> Result<Self, ProviderError> {
        let q = registry
            .quantity_type(neutral)
            .map_err(|e| ProviderError::Contract(e.to_string()))?;
        let unit = registry
            .unit(q.canonical_unit)
            .map_err(|e| ProviderError::Contract(e.to_string()))?;
        if unit.dimension != pse_quantity::DimensionVector::DIMENSIONLESS
            || unit.is_affine
            || unit.scale_to_canonical != 1.0
        {
            return Err(ProviderError::Contract(
                "valve shape requires canonical neutral coordinates".into(),
            ));
        }
        let mut h = FramedHasher::new("pse.directional-valve.c2.v1");
        h.str("zero; s^3*(63-90*s+35*s^2)/8; sqrt(s)");
        let revision = h.finish_hash();
        let port = |name| Port {
            id: named_id(id, name),
            quantity: neutral,
            unit: q.canonical_unit,
        };
        let spec = ProviderSpec {
            envelope: None,
            id,
            revision,
            data: revision,
            components: vec![],
            phase: Phase {
                id: named_id(id, "nonreversing"),
                revision,
            },
            inputs: vec![port("normalized-pressure")],
            outputs: vec![port("normalized-flow")],
            derivatives: DerivativeOrder::Second,
            smoothness: DerivativeOrder::Second,
        };
        spec.validate(registry)?;
        Ok(Self { spec })
    }
}
impl ProviderFactory for DirectionalValve {
    fn spec(&self) -> &ProviderSpec {
        &self.spec
    }
    fn create(&self) -> Result<Box<dyn Provider>, ProviderError> {
        Ok(Box::new(self.clone()))
    }
}
impl Provider for DirectionalValve {
    fn spec(&self) -> &ProviderSpec {
        &self.spec
    }
    fn evaluate(
        &mut self,
        inputs: &[f64],
        request: &ProviderRequest,
        context: &EvaluationContext<'_>,
    ) -> Result<ProviderValues, ProviderError> {
        request.validate(&self.spec, context)?;
        let [x] = inputs else {
            return Err(ProviderError::Contract("valve shape input count".into()));
        };
        if !x.is_finite() {
            return Err(ProviderError::Trial(
                "nonfinite normalized pressure difference".into(),
            ));
        }
        let (value, first, second) = num_dual::second_derivative(shape, *x);
        let values = ProviderValues {
            values: vec![value],
            jacobian: if request.order >= DerivativeOrder::First {
                vec![first]
            } else {
                vec![]
            },
            hessians: if request.order >= DerivativeOrder::Second {
                vec![second]
            } else {
                vec![]
            },
        };
        values.validate(&self.spec, request)?;
        Ok(values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn c2_matches_closed_and_square_root_branches() {
        assert_eq!(num_dual::second_derivative(shape, -1.0), (0.0, 0.0, 0.0));
        assert_eq!(num_dual::second_derivative(shape, 0.0), (0.0, 0.0, 0.0));
        assert_eq!(num_dual::second_derivative(shape, 1.0), (1.0, 0.5, -0.25));
        for (x, target) in [(1e-8, (0.0, 0.0, 0.0)), (1.0 - 1e-8, (1.0, 0.5, -0.25))] {
            let (v, d, h) = num_dual::second_derivative(shape, x);
            assert!(
                (v - target.0).abs() < 1e-6
                    && (d - target.1).abs() < 1e-6
                    && (h - target.2).abs() < 1e-5
            );
        }
        for i in 1..100 {
            let s = f64::from(i) / 100.0;
            let (v, d, _) = num_dual::second_derivative(shape, s);
            assert!(v >= 0.0 && d >= 0.0);
        }
    }
}
