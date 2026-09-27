// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Semantics of the registry-generated variable domain (ADR-0103). The generated enum is
//! the single authority from authoring to the native boundary; no second domain type exists.
use crate::generated::enums::ModelingVariableDomain;

impl ModelingVariableDomain {
    /// Every domain except continuous is a discrete decision and changes the problem class.
    pub const fn is_discrete(self) -> bool {
        !matches!(self, Self::Continuous)
    }
    /// Integrality applies only to these declared domains.
    pub const fn is_integer(self) -> bool {
        matches!(self, Self::Integer | Self::Binary | Self::Semiinteger)
    }
    /// A semi-variable has a separate zero branch, outside its positive interval.
    pub const fn is_semi(self) -> bool {
        matches!(self, Self::Semicontinuous | Self::Semiinteger)
    }
    /// Integer and binary decisions are dimensionless counts or indicators (PS-01); a
    /// semi domain keeps the physical quantity of its active branch.
    pub const fn requires_pure_number(self) -> bool {
        matches!(self, Self::Integer | Self::Binary)
    }
    /// Exact domain membership for fixed values; no solver tolerance is involved.
    pub fn contains(self, value: f64, lower: f64, upper: f64) -> bool {
        value.is_finite()
            && (self.is_semi() && value == 0.0
                || value >= lower
                    && value <= upper
                    && (!self.is_integer() || value.fract() == 0.0)
                    && (self != Self::Binary || value == 0.0 || value == 1.0))
    }
}

#[cfg(test)]
mod tests {
    use super::ModelingVariableDomain as D;
    #[test]
    fn domain_classes_follow_the_declared_members() {
        let discrete = D::ALL.iter().filter(|d| d.is_discrete()).count();
        assert_eq!(discrete, 4);
        assert!(D::Semiinteger.is_integer() && D::Semiinteger.is_semi());
        assert!(!D::Semicontinuous.is_integer() && D::Semicontinuous.is_semi());
        assert!(D::Binary.requires_pure_number() && D::Integer.requires_pure_number());
        assert!(!D::Semiinteger.requires_pure_number() && !D::Continuous.requires_pure_number());
        assert!(D::Semicontinuous.contains(0.0, 2.0, 5.0));
        assert!(!D::Semicontinuous.contains(1.0, 2.0, 5.0));
        assert!(!D::Integer.contains(1.5, 0.0, 5.0));
        assert!(!D::Binary.contains(2.0, 0.0, 5.0));
    }
}
