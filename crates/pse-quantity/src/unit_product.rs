// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! A unit literal as written: a canonical product of unit symbols with rational
//! exponents (ADR-0124, blueprint §8.2).
//!
//! The expression parser flattens `J/(K*mol)`, `m^3`, `kg/mol` and rational exponents
//! into one [`UnitProduct`]: each symbol once, nonzero exponents, symbol order. Two
//! spellings of one product are therefore equal values, and
//! [`crate::QuantityRegistry::compose`] resolves each symbol to an atomic unit, so no
//! composite spelling is ever looked up whole. The numeral `1` is the empty product.

use crate::{DimensionError, Ratio};

/// A canonical product of unit symbols with nonzero rational exponents, in symbol order.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UnitProduct {
    factors: Vec<(String, Ratio)>,
}

impl UnitProduct {
    /// The empty product, spelled `1`.
    pub const fn one() -> Self {
        Self {
            factors: Vec::new(),
        }
    }
    /// One symbol with exponent one.
    pub fn symbol(symbol: impl Into<String>) -> Self {
        Self {
            factors: vec![(symbol.into(), Ratio::ONE)],
        }
    }
    /// Merge repeated symbols, drop zero exponents and order by symbol.
    ///
    /// # Errors
    /// An exponent sum that does not fit the canonical rational pair.
    pub fn from_factors(
        factors: impl IntoIterator<Item = (String, Ratio)>,
    ) -> Result<Self, DimensionError> {
        let mut merged = std::collections::BTreeMap::<String, Ratio>::new();
        for (symbol, exponent) in factors {
            let slot = merged.entry(symbol).or_insert(Ratio::ZERO);
            *slot = slot.checked_add(exponent)?;
        }
        Ok(Self {
            factors: merged
                .into_iter()
                .filter(|(_, exponent)| !exponent.is_zero())
                .collect(),
        })
    }
    /// Canonical factors in symbol order.
    pub fn factors(&self) -> &[(String, Ratio)] {
        &self.factors
    }
    /// Whether this is the empty product `1`.
    pub fn is_one(&self) -> bool {
        self.factors.is_empty()
    }
    /// The product of two products.
    ///
    /// # Errors
    /// An exponent sum that does not fit the canonical rational pair.
    pub fn mul(&self, other: &Self) -> Result<Self, DimensionError> {
        Self::from_factors(self.factors.iter().chain(&other.factors).cloned())
    }
    /// Every exponent multiplied by `exponent`.
    ///
    /// # Errors
    /// A product exponent that does not fit the canonical rational pair.
    pub fn pow(&self, exponent: Ratio) -> Result<Self, DimensionError> {
        Self::from_factors(
            self.factors
                .iter()
                .map(|(symbol, power)| Ok((symbol.clone(), power.checked_mul(exponent)?)))
                .collect::<Result<Vec<_>, DimensionError>>()?,
        )
    }
    /// The quotient of two products.
    ///
    /// # Errors
    /// An exponent that does not fit the canonical rational pair.
    pub fn div(&self, other: &Self) -> Result<Self, DimensionError> {
        self.mul(&other.pow(Ratio::new(-1, 1)?)?)
    }
    /// Owned heap extent of the symbols, for admission budgets.
    pub fn retained_bytes(&self) -> usize {
        self.factors.iter().fold(
            self.factors
                .capacity()
                .saturating_mul(size_of::<(String, Ratio)>()),
            |bytes, (symbol, _)| bytes.saturating_add(symbol.capacity()),
        )
    }
}

fn render_factor(f: &mut std::fmt::Formatter<'_>, symbol: &str, power: Ratio) -> std::fmt::Result {
    f.write_str(symbol)?;
    if power == Ratio::ONE {
        Ok(())
    } else if power.is_integer() {
        write!(f, "^{}", power.num())
    } else {
        write!(f, "^({}/{})", power.num(), power.den())
    }
}

/// The canonical spelling: numerator factors, then `/` and the denominator, which is
/// parenthesized when it has more than one factor. The expression parser reads it back
/// to the same product.
impl std::fmt::Display for UnitProduct {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let numerator: Vec<_> = self.factors.iter().filter(|(_, e)| e.num() > 0).collect();
        let denominator: Vec<_> = self.factors.iter().filter(|(_, e)| e.num() < 0).collect();
        if numerator.is_empty() {
            f.write_str("1")?;
        }
        for (i, (symbol, power)) in numerator.iter().enumerate() {
            if i > 0 {
                f.write_str("*")?;
            }
            render_factor(f, symbol, *power)?;
        }
        if denominator.is_empty() {
            return Ok(());
        }
        f.write_str("/")?;
        let grouped = denominator.len() > 1;
        if grouped {
            f.write_str("(")?;
        }
        for (i, (symbol, power)) in denominator.iter().enumerate() {
            if i > 0 {
                f.write_str("*")?;
            }
            let magnitude = Ratio::new(-i32::from(power.num()), i32::from(power.den()))
                .map_err(|_| std::fmt::Error)?;
            render_factor(f, symbol, magnitude)?;
        }
        if grouped {
            f.write_str(")")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(num: i32, den: i32) -> Ratio {
        Ratio::new(num, den).unwrap()
    }

    #[test]
    fn products_merge_order_and_render_canonically() {
        let j = UnitProduct::symbol("J");
        let per_mol_k = UnitProduct::symbol("mol")
            .mul(&UnitProduct::symbol("K"))
            .unwrap();
        let a = j.div(&per_mol_k).unwrap();
        let b = j
            .div(&UnitProduct::symbol("K"))
            .unwrap()
            .div(&UnitProduct::symbol("mol"))
            .unwrap();
        assert_eq!(a, b);
        assert_eq!(a.to_string(), "J/(K*mol)");
        assert_eq!(UnitProduct::one().to_string(), "1");
        assert_eq!(
            UnitProduct::symbol("s").pow(r(-1, 1)).unwrap().to_string(),
            "1/s"
        );
        assert_eq!(
            UnitProduct::symbol("m").pow(r(3, 2)).unwrap().to_string(),
            "m^(3/2)"
        );
        assert_eq!(
            UnitProduct::symbol("m").pow(r(-1, 2)).unwrap().to_string(),
            "1/m^(1/2)"
        );
        assert!(j.div(&j).unwrap().is_one());
    }
}
