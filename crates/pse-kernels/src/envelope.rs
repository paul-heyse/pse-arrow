// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Declared operating windows, separate from empirical accuracy and intrinsic domains.
use crate::ProviderError;
use pse_ids::FramedHasher;

/// A finite closed interval in the physical port's canonical units.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Interval {
    lower: f64,
    upper: f64,
}
impl Eq for Interval {}
impl Interval {
    /// Admit finite ordered bounds. NaN cannot enter equality or identity.
    /// # Errors
    /// Returns a contract error for nonfinite or nonincreasing bounds.
    pub fn new(lower: f64, upper: f64) -> Result<Self, ProviderError> {
        if !lower.is_finite() || !upper.is_finite() || lower >= upper {
            return Err(ProviderError::Contract(
                "finite increasing envelope bounds required".into(),
            ));
        }
        Ok(Self {
            lower: if lower == 0.0 { 0.0 } else { lower },
            upper: if upper == 0.0 { 0.0 } else { upper },
        })
    }
    /// Canonical interval endpoints.
    pub fn bounds(self) -> [f64; 2] {
        [self.lower, self.upper]
    }
    /// Reject a trial outside its explicitly declared operating window.
    /// # Errors
    /// Returns `OutsideEnvelope` for nonfinite or out-of-range values.
    pub fn check(self, axis: &str, value: f64) -> Result<(), ProviderError> {
        if !value.is_finite() || value < self.lower || value > self.upper {
            return Err(ProviderError::OutsideEnvelope {
                axis: axis.into(),
                value,
                lower: self.lower,
                upper: self.upper,
            });
        }
        Ok(())
    }
    fn frame(self, h: &mut FramedHasher) {
        h.u64(self.lower.to_bits()).u64(self.upper.to_bits());
    }
}

/// SI operating envelope for the admitted homogeneous ternary state.
/// These declarations are not a certificate of empirical accuracy or phase stability.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateEnvelope {
    /// Kelvin.
    pub temperature: Interval,
    /// Moles per cubic metre.
    pub density: Interval,
    /// Pascals, checked even when pressure is not an output demand.
    pub pressure: Interval,
    /// Ordered component mole fractions, including the dependent complement.
    pub composition: [Interval; 3],
    /// Authored basis for choosing this operating window.
    pub provenance: String,
}
impl StateEnvelope {
    /// Reject an invalid physical interpretation before registering a provider.
    /// # Errors
    /// Returns a contract error for missing provenance or nonphysical ranges.
    pub fn validate(&self) -> Result<(), ProviderError> {
        if self.provenance.trim().is_empty()
            || self.provenance.len() > 4096
            || self.temperature.lower <= 0.0
            || self.density.lower < 0.0
            || self.pressure.lower < 0.0
            || self
                .composition
                .iter()
                .any(|v| v.lower < 0.0 || v.upper > 1.0)
        {
            return Err(ProviderError::Contract(
                "invalid physical operating envelope".into(),
            ));
        }
        Ok(())
    }
    /// Check temperature and all fractions; intrinsic interior restrictions apply separately.
    /// # Errors
    /// Returns `OutsideEnvelope` when temperature or any fraction is outside its window.
    pub fn thermal_composition(
        &self,
        temperature: f64,
        fractions: [f64; 3],
    ) -> Result<(), ProviderError> {
        self.temperature.check("temperature", temperature)?;
        for (i, (range, value)) in self.composition.iter().zip(fractions).enumerate() {
            range.check(&format!("composition[{i}]"), value)?;
        }
        Ok(())
    }
    pub(crate) fn frame(&self, h: &mut FramedHasher) {
        h.str("declared-operating-window;empirical-validity-unestablished;reject-outside;v1")
            .str(&self.provenance);
        self.temperature.frame(h);
        self.density.frame(h);
        self.pressure.frame(h);
        for range in self.composition {
            range.frame(h);
        }
    }
}
