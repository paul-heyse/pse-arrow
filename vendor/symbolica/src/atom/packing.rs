//! Passive, allocation-free inspection of a restricted packed-atom profile.
//!
//! This module reads bytes directly. It never constructs an atom/view, materializes a
//! coefficient, normalizes mathematics or registers symbols. Success establishes only
//! the documented layout and finite Float profile, not canonical producer provenance,
//! destination-state compatibility, remapping correctness or authority to publish an Atom.

use super::{
    coefficient::{ARB_DEN, ARB_NUM, COMPLEX, DEN_MASK, FLOAT, NUM_MASK, SIGN},
    representation::{
        ADD_ID, FUN_ID, MUL_HAS_COEFF_FLAG, MUL_ID, NUM_ID, POW_ID, TYPE_MASK, VAR_ID,
    },
};

/// The actual receiving backend's supported precision and *current* exponent range.
/// The caller must supply these observations; there is deliberately no default.
#[derive(Clone, Copy, Debug)]
pub struct FloatEnvironment {
    pub min_precision: u32,
    pub max_precision: u32,
    pub min_exponent: i64,
    pub max_exponent: i64,
}

/// Caller-owned traversal storage. One frame is needed for the root, plus one for
/// each simultaneously open composite. An undersized slice refuses without growth.
#[derive(Clone, Copy, Debug, Default)]
pub struct PackingFrame {
    end: usize,
    remaining: usize,
    exact: bool,
    first_coefficient: bool,
}

/// A plain symbol occurrence. Eligibility belongs to the callback's state owner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PackedSymbol {
    pub id: u32,
    pub function: bool,
}

/// Counts describe the inspected bytes, with no retained numeric/symbolic objects.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PackingSummary {
    pub bytes: usize,
    pub nodes: usize,
    pub numbers: usize,
    pub variables: usize,
    pub functions: usize,
    pub powers: usize,
    pub products: usize,
    pub sums: usize,
    pub rational_components: usize,
    pub large_rational_bytes: usize,
    pub float_components: usize,
    pub float_text_bytes: usize,
    pub max_depth: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackingIssue {
    Truncated,
    ArithmeticOverflow,
    InvalidTag,
    UnsupportedFlags,
    UnsupportedSymbolAttributes,
    UnsupportedCoefficient,
    InvalidPackedInteger,
    InvalidDenominator,
    InvalidRational,
    InvalidExtent,
    TrailingBytes,
    InsufficientScratch,
    InvalidFloatEnvironment,
    InvalidFloatPrecision,
    UnsupportedFloatText,
    FloatExponentRange,
    InexactFloatText,
    MissingProductCoefficient,
}

/// Callback errors are retained unchanged, including non-cloneable state.
#[derive(Debug, PartialEq, Eq)]
pub enum PackingError<E> {
    Layout { offset: usize, issue: PackingIssue },
    Symbol(E),
}

impl<E: std::fmt::Display> std::fmt::Display for PackingError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Layout { offset, issue } => write!(f, "packed atom at byte {offset}: {issue:?}"),
            Self::Symbol(error) => error.fmt(f),
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for PackingError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Symbol(error) => Some(error),
            Self::Layout { .. } => None,
        }
    }
}

type LayoutResult<T> = Result<T, (usize, PackingIssue)>;

struct Cursor<'a> {
    data: &'a [u8],
    pos: usize,
    end: usize,
}

impl<'a> Cursor<'a> {
    fn take(&mut self, size: usize) -> LayoutResult<&'a [u8]> {
        let end = self
            .pos
            .checked_add(size)
            .ok_or((self.pos, PackingIssue::ArithmeticOverflow))?;
        if end > self.end {
            return Err((self.pos, PackingIssue::Truncated));
        }
        let bytes = &self.data[self.pos..end];
        self.pos = end;
        Ok(bytes)
    }

    fn byte(&mut self) -> LayoutResult<u8> {
        Ok(self.take(1)?[0])
    }

    fn integer(&mut self, width: u8) -> LayoutResult<u64> {
        let size = match width {
            1 => 1,
            2 => 2,
            3 => 4,
            4 => 8,
            _ => return Err((self.pos, PackingIssue::InvalidPackedInteger)),
        };
        let mut value = 0;
        for (i, byte) in self.take(size)?.iter().enumerate() {
            value |= u64::from(*byte) << (i * 8);
        }
        Ok(value)
    }

    // Sign is meaningful only for small rationals or large-rational byte counts.
    fn pair(&mut self, signed: bool) -> LayoutResult<(u64, u64, bool)> {
        let start = self.pos;
        let tag = self.byte()?;
        if !signed && tag & SIGN != 0 {
            return Err((start, PackingIssue::InvalidPackedInteger));
        }
        let num = self.integer(tag & NUM_MASK)?;
        let den = match (tag & DEN_MASK) >> 4 {
            0 => 1,
            width => self.integer(width)?,
        };
        Ok((num, den, tag & SIGN != 0))
    }

    fn usize(&self, value: u64) -> LayoutResult<usize> {
        usize::try_from(value).map_err(|_| (self.pos, PackingIssue::ArithmeticOverflow))
    }

    fn extent(&self, length: u64) -> LayoutResult<usize> {
        let end = self
            .pos
            .checked_add(self.usize(length)?)
            .ok_or((self.pos, PackingIssue::ArithmeticOverflow))?;
        if end > self.end {
            return Err((self.pos, PackingIssue::InvalidExtent));
        }
        Ok(end)
    }
}

/// Inspect one raw packed atom (without the export-version/u64 stream framing).
///
/// `symbol` is called once per Var/Fun occurrence in traversal order, after its
/// local layout is checked. It must check eligibility against caller-owned state.
/// Calls are provisional until this entire function succeeds: a later malformed
/// descendant can still refuse. The scanner grants no capability and performs no
/// allocation; callback effects/allocations and provisioning scratch belong to the
/// caller. NOT_NORMALIZED, attributed symbols, nonordinary flags and special,
/// field and polynomial coefficients explicitly refuse. Reduced fractions,
/// mathematical simplification and canonical ordering remain producer obligations.
pub fn inspect_packing<E>(
    data: &[u8],
    scratch: &mut [PackingFrame],
    floats: FloatEnvironment,
    mut symbol: impl FnMut(PackedSymbol) -> Result<(), E>,
) -> Result<PackingSummary, PackingError<E>> {
    inspect(data, scratch, floats, &mut symbol)
}

fn inspect<E>(
    data: &[u8],
    scratch: &mut [PackingFrame],
    floats: FloatEnvironment,
    symbol: &mut impl FnMut(PackedSymbol) -> Result<(), E>,
) -> Result<PackingSummary, PackingError<E>> {
    let layout = |(offset, issue)| PackingError::Layout { offset, issue };
    if floats.min_precision == 0
        || floats.min_precision > floats.max_precision
        || floats.min_exponent > floats.max_exponent
    {
        return Err(layout((0, PackingIssue::InvalidFloatEnvironment)));
    }
    let Some(root) = scratch.first_mut() else {
        return Err(layout((0, PackingIssue::InsufficientScratch)));
    };
    *root = PackingFrame {
        end: data.len(),
        remaining: 1,
        exact: true,
        first_coefficient: false,
    };
    let mut used = 1;
    let mut cursor = Cursor {
        data,
        pos: 0,
        end: data.len(),
    };
    let mut summary = PackingSummary {
        bytes: data.len(),
        ..PackingSummary::default()
    };
    while used != 0 {
        let frame = &mut scratch[used - 1];
        if frame.remaining == 0 {
            if frame.exact && cursor.pos != frame.end {
                return Err(layout((cursor.pos, PackingIssue::TrailingBytes)));
            }
            used -= 1;
            continue;
        }
        cursor.end = frame.end;
        frame.remaining -= 1;
        let start = cursor.pos;
        let tag = cursor.byte().map_err(layout)?;
        let kind = tag & TYPE_MASK;
        let allowed = if kind == MUL_ID {
            MUL_HAS_COEFF_FLAG
        } else {
            0
        };
        if tag & !TYPE_MASK & !allowed != 0 {
            return Err(layout((start, PackingIssue::UnsupportedFlags)));
        }
        if frame.first_coefficient {
            if kind != NUM_ID {
                return Err(layout((start, PackingIssue::MissingProductCoefficient)));
            }
            frame.first_coefficient = false;
        }
        summary.nodes = summary
            .nodes
            .checked_add(1)
            .ok_or_else(|| layout((start, PackingIssue::ArithmeticOverflow)))?;
        summary.max_depth = summary.max_depth.max(used);
        let children = match kind {
            NUM_ID => {
                summary.numbers += 1;
                coefficient(&mut cursor, floats, &mut summary).map_err(layout)?;
                None
            }
            VAR_ID => {
                summary.variables += 1;
                let (id, extra, _) = cursor.pair(false).map_err(layout)?;
                if extra != 1 {
                    return Err(layout((start, PackingIssue::UnsupportedSymbolAttributes)));
                }
                let id = u32::try_from(id)
                    .map_err(|_| layout((start, PackingIssue::InvalidPackedInteger)))?;
                symbol(PackedSymbol {
                    id,
                    function: false,
                })
                .map_err(PackingError::Symbol)?;
                None
            }
            FUN_ID => {
                summary.functions += 1;
                // function_metadata/skip_packed_function require an unsigned,
                // denominator-free length tag, not merely a pair whose value is 1.
                let width = cursor.byte().map_err(layout)?;
                let length = cursor.integer(width).map_err(layout)?;
                let end = cursor.extent(length).map_err(layout)?;
                cursor.end = end;
                let (id, count, _) = cursor.pair(false).map_err(layout)?;
                let id = u32::try_from(id)
                    .map_err(|_| layout((start, PackingIssue::UnsupportedSymbolAttributes)))?;
                symbol(PackedSymbol { id, function: true }).map_err(PackingError::Symbol)?;
                Some(PackingFrame {
                    end,
                    remaining: cursor.usize(count).map_err(layout)?,
                    exact: true,
                    first_coefficient: false,
                })
            }
            MUL_ID | ADD_ID => {
                if kind == MUL_ID {
                    summary.products += 1;
                } else {
                    summary.sums += 1;
                }
                let (count, length, _) = cursor.pair(false).map_err(layout)?;
                let end = cursor.extent(length).map_err(layout)?;
                if tag & MUL_HAS_COEFF_FLAG != 0 && count == 0 {
                    return Err(layout((start, PackingIssue::MissingProductCoefficient)));
                }
                Some(PackingFrame {
                    end,
                    remaining: cursor.usize(count).map_err(layout)?,
                    exact: true,
                    first_coefficient: tag & MUL_HAS_COEFF_FLAG != 0,
                })
            }
            POW_ID => {
                summary.powers += 1;
                Some(PackingFrame {
                    end: cursor.end,
                    remaining: 2,
                    exact: false,
                    first_coefficient: false,
                })
            }
            _ => return Err(layout((start, PackingIssue::InvalidTag))),
        };
        if let Some(child) = children {
            if child.remaining > child.end - cursor.pos {
                return Err(layout((start, PackingIssue::InvalidExtent)));
            }
            let Some(slot) = scratch.get_mut(used) else {
                return Err(layout((start, PackingIssue::InsufficientScratch)));
            };
            *slot = child;
            used += 1;
        }
    }
    Ok(summary)
}

fn rational(cursor: &mut Cursor<'_>, summary: &mut PackingSummary) -> LayoutResult<()> {
    let start = cursor.pos;
    let tag = cursor.byte()?;
    if tag == ARB_NUM | ARB_DEN {
        let (num, den, _) = cursor.pair(true)?;
        let n = cursor.usize(num)?;
        let d = cursor.usize(den)?;
        // Positive magnitude byte arrays, not GMP values. A zero numerator may
        // have zero bytes, but a denominator must contain a nonzero magnitude.
        cursor.take(n)?;
        let denominator = cursor.take(d)?;
        if !denominator.iter().any(|byte| *byte != 0) {
            return Err((start, PackingIssue::InvalidDenominator));
        }
        let total = n
            .checked_add(d)
            .ok_or((start, PackingIssue::ArithmeticOverflow))?;
        summary.large_rational_bytes = summary
            .large_rational_bytes
            .checked_add(total)
            .ok_or((start, PackingIssue::ArithmeticOverflow))?;
    } else if (1..=4).contains(&(tag & NUM_MASK)) {
        cursor.pos = start;
        let (num, den, negative) = cursor.pair(true)?;
        // The ordinary reader returns i64 numerator AND i64 denominator.
        if den == 0 || den > i64::MAX as u64 {
            return Err((start, PackingIssue::InvalidDenominator));
        }
        if num > i64::MAX as u64 + u64::from(negative) {
            return Err((start, PackingIssue::InvalidRational));
        }
    } else {
        return Err((start, PackingIssue::UnsupportedCoefficient));
    }
    summary.rational_components += 1;
    Ok(())
}

fn coefficient(
    cursor: &mut Cursor<'_>,
    floats: FloatEnvironment,
    summary: &mut PackingSummary,
) -> LayoutResult<()> {
    let start = cursor.pos;
    match cursor.byte()? {
        COMPLEX => {
            rational(cursor, summary)?;
            rational(cursor, summary)?;
        }
        FLOAT => {
            for _ in 0..2 {
                let length = cursor.integer(4)?;
                let length = cursor.usize(length)?;
                let component_start = cursor.pos;
                let bytes = cursor.take(length)?;
                let Some(precision_bytes) = bytes.get(..4) else {
                    return Err((component_start, PackingIssue::Truncated));
                };
                let mut precision = [0; 4];
                precision.copy_from_slice(precision_bytes);
                let precision = u32::from_le_bytes(precision);
                if !(floats.min_precision..=floats.max_precision).contains(&precision) {
                    return Err((component_start, PackingIssue::InvalidFloatPrecision));
                }
                finite_float(&bytes[4..], precision, floats)
                    .map_err(|issue| (component_start + 4, issue))?;
                summary.float_components += 1;
                summary.float_text_bytes = summary
                    .float_text_bytes
                    .checked_add(length - 4)
                    .ok_or((component_start, PackingIssue::ArithmeticOverflow))?;
            }
        }
        _ => {
            cursor.pos = start;
            rational(cursor, summary)?;
        }
    }
    Ok(())
}

// Numerica 3.0.1 serializes zero as exactly `0`, otherwise Rug's lowercase
// radix-16, None-digit output. This deliberately strict finite subset excludes
// Rug's permissive whitespace, separators, specials, + signs and p exponents.
// No Rug/GMP/MPFR parse, numeric allocation or coefficient materialization occurs.
fn finite_float(text: &[u8], precision: u32, env: FloatEnvironment) -> Result<(), PackingIssue> {
    text.len()
        .checked_add(2)
        .ok_or(PackingIssue::ArithmeticOverflow)?; // eventual Rug parser reserve
    if text == b"0" {
        return Ok(());
    }
    let text = text.strip_prefix(b"-").unwrap_or(text);
    let mut before = 0usize;
    let mut digits = 0usize;
    let mut point = false;
    let mut first = None;
    let mut last = (0usize, 0u8);
    let mut index = 0;
    while index < text.len() && text[index] != b'@' {
        let byte = text[index];
        if byte == b'.' {
            if point || digits == 0 {
                return Err(PackingIssue::UnsupportedFloatText);
            }
            point = true;
            before = digits;
        } else {
            let digit = match byte {
                b'0'..=b'9' => byte - b'0',
                b'a'..=b'f' => byte - b'a' + 10,
                _ => return Err(PackingIssue::UnsupportedFloatText),
            };
            if digit != 0 {
                first.get_or_insert((digits, digit));
                last = (digits, digit);
            }
            digits = digits
                .checked_add(1)
                .ok_or(PackingIssue::ArithmeticOverflow)?;
        }
        index += 1;
    }
    if digits == 0 || (point && before == digits) {
        return Err(PackingIssue::UnsupportedFloatText);
    }
    if !point {
        before = digits;
    }
    let mut exponent = 0i128;
    if index < text.len() {
        let exp = &text[index + 1..];
        let (negative, exp) = match exp.strip_prefix(b"-") {
            Some(d) => (true, d),
            None => (false, exp),
        };
        if exp.is_empty() {
            return Err(PackingIssue::UnsupportedFloatText);
        }
        for byte in exp {
            if !byte.is_ascii_digit() {
                return Err(PackingIssue::UnsupportedFloatText);
            }
            exponent = exponent
                .checked_mul(10)
                .and_then(|v| v.checked_add(i128::from(*byte - b'0')))
                .ok_or(PackingIssue::FloatExponentRange)?;
        }
        if negative {
            exponent = -exponent;
        }
        if exponent < std::os::raw::c_long::MIN as i128
            || exponent > std::os::raw::c_long::MAX as i128
        {
            return Err(PackingIssue::FloatExponentRange);
        }
    }
    let Some((first_index, first_digit)) = first else {
        return Err(PackingIssue::UnsupportedFloatText);
    };
    // MPFR 4.2.2 get_str_ndigits(16, p) = 1 + (p + 2) / 4.
    // The producer starts with a nonzero digit. Excluding leading-zero and
    // excessive padding also keeps strtofr's signed mantissa counters faithful
    // before it removes zeros; a small exact span alone does not bound them.
    let producer_digits = 1 + (u64::from(precision) + 2) / 4;
    if first_index != 0
        || digits as u128 > u128::from(producer_digits)
        || digits as u128 > std::os::raw::c_long::MAX as u128
    {
        return Err(PackingIssue::UnsupportedFloatText);
    }
    let first_bits = 8 - first_digit.leading_zeros();
    let span = (last.0 - first_index) as i128 * 4 + i128::from(first_bits)
        - i128::from(last.1.trailing_zeros());
    if span > i128::from(precision) {
        return Err(PackingIssue::InexactFloatText);
    }
    let e = (before as i128 - first_index as i128 - 1 + exponent)
        .checked_mul(4)
        .and_then(|v| v.checked_add(i128::from(first_bits)))
        .ok_or(PackingIssue::FloatExponentRange)?;
    if e < i128::from(env.min_exponent) || e > i128::from(env.max_exponent) {
        return Err(PackingIssue::FloatExponentRange);
    }
    Ok(())
}
