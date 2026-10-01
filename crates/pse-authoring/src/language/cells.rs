// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Data cells as typed tagged values (ADR-0123 Outcome 1).
//!
//! A cell is parsed once, where it is written, into one of nine variants: a Boolean, an
//! integer, a quantity (a magnitude and a canonical unit product), text, an identifier of a
//! declared scheme, a reference or a set of references by path, a keyed-row reference, or
//! explicit absence. Each may carry an uncertainty. A cell is never an expression: nothing
//! here evaluates, and names are path segments the checker resolves once. [`render_cell`]
//! prints the canonical spelling that [`parse_cell`] reads back to the same cell.
//!
//! ```text
//! cell  := value ['±' ('standard' | 'relative' | 'bound') '(' number ')']
//! value := 'missing' | 'true' | 'false' | quoted
//!        | 'Id' '<' path '>' '(' quoted ')'
//!        | ['-'] number ['{' unit '}']
//!        | '{' [path {',' path}] '}' | path ['[' key {',' key} ']']
//! key   := a cell without an uncertainty that is a Boolean, an integer, a quantity, text,
//!          an identifier or a reference
//! ```
//!
//! A number without a unit, decimal point or exponent is an integer cell; any other number
//! is a quantity cell, dimensionless when it has no unit. A reference path starts with a
//! plain identifier; later segments may be quoted. `path[keys]` names a row of a keyed kind
//! or of a table by its key cells (Plan 23 KR5); admission resolves it.
use crate::AuthoringError;
pub use pse_model::generated::enums::{
    ModelingCellKind as CellKind, ModelingKeyCellKind as KeyCellKind, ModelingUncertaintyKind,
};
pub use pse_model::generated::structures::{
    ModelingCell as Cell, ModelingCellUncertainty as CellUncertainty,
    ModelingCellValue as CellValue, ModelingCellValueBoolean as CellBoolean,
    ModelingCellValueIdentifier as CellIdentifier, ModelingCellValueInteger as CellInteger,
    ModelingCellValueQuantity as CellQuantity, ModelingCellValueReference as CellReference,
    ModelingCellValueReferences as CellReferences,
    ModelingCellValueReferencesPathsItem as CellPath, ModelingCellValueRow as CellRow,
    ModelingCellValueSelected as CellSelected, ModelingCellValueText as CellText,
    ModelingKeyCell as KeyCell, ModelingKeyCellSelected as KeyCellSelected,
    ModelingUnitFactor as CellUnitFactor,
};

fn bad(reason: impl Into<String>) -> AuthoringError {
    AuthoringError::Contract {
        at: None,
        reason: reason.into(),
    }
}

/// A cell of `value` without an uncertainty.
pub fn cell(value: CellValue) -> Cell {
    Cell {
        value,
        uncertainty: None,
    }
}

/// The key cell a key of a keyed-row reference is written as: a scalar cell without an
/// uncertainty.
///
/// # Errors
/// Absence, a set, a further row reference or an uncertainty.
pub fn key_cell(cell: &Cell) -> Result<KeyCell, AuthoringError> {
    if cell.uncertainty.is_some() {
        return Err(bad("a key cell carries no uncertainty"));
    }
    Ok(
        match cell.value.selected().map_err(|e| bad(e.to_string()))? {
            CellSelected::Boolean(v) => KeyCell::from_boolean(v.clone()),
            CellSelected::Integer(v) => KeyCell::from_integer(v.clone()),
            CellSelected::Quantity(v) => KeyCell::from_quantity(v.clone()),
            CellSelected::Text(v) => KeyCell::from_text(v.clone()),
            CellSelected::Identifier(v) => KeyCell::from_identifier(v.clone()),
            CellSelected::Reference(v) => KeyCell::from_reference(v.clone()),
            CellSelected::Missing | CellSelected::References(_) | CellSelected::Row(_) => {
                return Err(bad(
                    "a key cell is a Boolean, an integer, a quantity, text, an identifier or a reference",
                ));
            }
        },
    )
}

/// The cell a key cell stands for, typed as any cell is.
///
/// # Errors
/// A malformed tagged key cell.
pub fn key_cell_value(key: &KeyCell) -> Result<Cell, AuthoringError> {
    Ok(cell(
        match key.selected().map_err(|e| bad(e.to_string()))? {
            KeyCellSelected::Boolean(v) => CellValue::from_boolean(v.clone()),
            KeyCellSelected::Integer(v) => CellValue::from_integer(v.clone()),
            KeyCellSelected::Quantity(v) => CellValue::from_quantity(v.clone()),
            KeyCellSelected::Text(v) => CellValue::from_text(v.clone()),
            KeyCellSelected::Identifier(v) => CellValue::from_identifier(v.clone()),
            KeyCellSelected::Reference(v) => CellValue::from_reference(v.clone()),
        },
    ))
}

/// The canonical unit product of a quantity cell; `None` for a bare number, which is the
/// neutral scalar, while a written unit (`{1}` included) resolves against the expected type.
///
/// # Errors
/// A factor whose exponent is not a reduced rational with a positive denominator.
pub fn unit_product(
    quantity: &CellQuantity,
) -> Result<Option<pse_quantity::UnitProduct>, AuthoringError> {
    let Some(unit) = &quantity.unit else {
        return Ok(None);
    };
    pse_quantity::UnitProduct::from_factors(
        unit.iter()
            .map(|factor| {
                pse_quantity::Ratio::from_parts(factor.num, factor.den)
                    .map(|exponent| (factor.symbol.clone(), exponent))
                    .map_err(|e| bad(format!("unit factor {}: {e}", factor.symbol)))
            })
            .collect::<Result<Vec<_>, _>>()?,
    )
    .map(Some)
    .map_err(|e| bad(e.to_string()))
}

/// The stored factors of a canonical unit product.
pub fn unit_factors(unit: &pse_quantity::UnitProduct) -> Vec<CellUnitFactor> {
    unit.factors()
        .iter()
        .map(|(symbol, exponent)| CellUnitFactor {
            symbol: symbol.clone(),
            num: exponent.num(),
            den: exponent.den(),
        })
        .collect()
}

/// Quoted text with the escapes the lexer reads back.
pub(crate) fn quote_text(value: &str) -> String {
    let escaped = value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t");
    format!("\"{escaped}\"")
}

fn path(segments: &[String]) -> Result<String, AuthoringError> {
    let Some((first, _)) = segments.split_first() else {
        return Err(bad("a cell path has at least one segment"));
    };
    let plain = crate::grammar::render_name(first) == *first;
    if !plain
        || segments.iter().any(String::is_empty)
        || segments.len() == 1 && matches!(first.as_str(), "true" | "false" | "missing")
        || first == "Id"
    {
        return Err(bad(format!(
            "a cell path starts with a plain identifier that is not a literal: {segments:?}"
        )));
    }
    Ok(segments
        .iter()
        .map(|s| crate::grammar::render_name(s))
        .collect::<Vec<_>>()
        .join("."))
}

/// A finite magnitude in a spelling that parses back to the same bits as a quantity: `{:?}`
/// always prints a decimal point or an exponent.
fn magnitude(value: f64) -> Result<String, AuthoringError> {
    if value.is_finite() {
        Ok(format!("{value:?}"))
    } else {
        Err(bad("a cell magnitude is finite"))
    }
}

/// The canonical spelling of a cell.
///
/// # Errors
/// A malformed tagged value, a nonfinite magnitude, a malformed unit or a path that has no
/// cell spelling.
pub fn render_cell(cell: &Cell) -> Result<String, AuthoringError> {
    let value = match cell.value.selected().map_err(|e| bad(e.to_string()))? {
        CellSelected::Missing => "missing".to_owned(),
        CellSelected::Boolean(v) => v.value.to_string(),
        CellSelected::Integer(v) => v.value.to_string(),
        CellSelected::Quantity(v) => match unit_product(v)? {
            None => magnitude(v.magnitude)?,
            Some(unit) => format!("{}{{{unit}}}", magnitude(v.magnitude)?),
        },
        CellSelected::Text(v) => quote_text(&v.value),
        CellSelected::Identifier(v) => {
            format!("Id<{}>({})", path(&v.scheme)?, quote_text(&v.value))
        }
        CellSelected::Reference(v) => path(&v.path)?,
        CellSelected::References(v) => format!(
            "{{{}}}",
            v.paths
                .iter()
                .map(|p| match &p.keys {
                    None => path(&p.path),
                    Some(keys) => render_cell(&self::cell(CellValue::from_row(CellRow {
                        target: p.path.clone(),
                        keys: keys.clone()
                    }))),
                })
                .collect::<Result<Vec<_>, _>>()?
                .join(", ")
        ),
        CellSelected::Row(v) => {
            if v.keys.is_empty() {
                return Err(bad("a keyed-row reference names at least one key"));
            }
            format!(
                "{}[{}]",
                path(&v.target)?,
                v.keys
                    .iter()
                    .map(|key| render_cell(&key_cell_value(key)?))
                    .collect::<Result<Vec<_>, _>>()?
                    .join(", ")
            )
        }
    };
    Ok(match &cell.uncertainty {
        None => value,
        Some(u) => format!("{value} ± {}({})", u.kind.as_str(), magnitude(u.magnitude)?),
    })
}

/// Parse one standalone cell.
///
/// # Errors
/// The text is not exactly one cell.
pub fn parse_cell(text: &str) -> Result<Cell, AuthoringError> {
    super::parser::parse_cell(text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use proptest::test_runner::RngSeed;

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 1024,
            rng_seed: RngSeed::Fixed(0x5053_452d_4345_4c4c),
            failure_persistence: None,
            ..ProptestConfig::default()
        })]
        /// Every cell over every variant, with and without an uncertainty, prints to a
        /// spelling that parses back to the same cell (ADR-0123 Outcome 1).
        #[test]
        fn cell_render_parse_roundtrip(cell in crate::dsl::cells()) {
            let rendered = render_cell(&cell)
                .map_err(|e| TestCaseError::fail(format!("{cell:?}: {e}")))?;
            let parsed = parse_cell(&rendered)
                .map_err(|e| TestCaseError::fail(format!("{rendered}: {e}")))?;
            prop_assert_eq!(&parsed, &cell, "{}", rendered);
        }
    }

    /// Spellings of one cell are one value; integers and quantities are told apart by
    /// their spelling, never by a column type.
    #[test]
    fn cell_spellings_parse_to_one_value() {
        let canonical = |text: &str| render_cell(&parse_cell(text).unwrap()).unwrap();
        for (text, expected) in [
            ("6", "6"),
            ("-6", "-6"),
            ("6.0", "6.0"),
            ("6{1}", "6.0{1}"),
            ("1e-08{mol/m^3}", "1e-8{mol/m^3}"),
            ("129440{J/(kmol*K)}", "129440.0{J/(K*kmol)}"),
            ("-169.5{J/(kmol*K^2)}", "-169.5{J/(K^2*kmol)}"),
            ("\"a b\"", "\"a b\""),
            ("Id<cas>(\"71-43-2\")", "Id<cas>(\"71-43-2\")"),
            ("chem.benzene", "chem.benzene"),
            ("{ a , b.c }", "{a, b.c}"),
            ("{}", "{}"),
            (
                "caloric_set[chem.benzene, 1, \"a\"]",
                "caloric_set[chem.benzene, 1, \"a\"]",
            ),
            ("element[ \"C\" ]", "element[\"C\"]"),
            ("{a, bank[fit, a, b, 2]}", "{a, bank[fit, a, b, 2]}"),
            ("missing", "missing"),
            ("true", "true"),
            (
                "12.011{g/mol} ± standard(0.0008)",
                "12.011{g/mol} ± standard(0.0008)",
            ),
        ] {
            assert_eq!(canonical(text), expected, "{text}");
        }
        assert_eq!(parse_cell("6").unwrap().value.kind, CellKind::Integer);
        assert_eq!(parse_cell("6.0").unwrap().value.kind, CellKind::Quantity);
        assert_eq!(parse_cell("6{kg}").unwrap().value.kind, CellKind::Quantity);
        // A written dimensionless unit is kept apart from a bare number.
        assert_ne!(parse_cell("0.5{1}").unwrap(), parse_cell("0.5").unwrap());
        assert_eq!(parse_cell("kind[a, 2]").unwrap().value.kind, CellKind::Row);
        for invalid in [
            "",
            "1+2",
            "x*2",
            "Id<cas>(71)",
            "{a",
            "a ± sigma(1)",
            "f(x)",
            "[a]",
            "kind[]",
            "kind[missing]",
            "kind[{a}]",
            "kind[a[1]]",
            "kind[1 ± standard(1.0)]",
            "{bank[fit] ± standard(1.0)}",
            "{1}",
        ] {
            assert!(parse_cell(invalid).is_err(), "{invalid}");
        }
    }
}
