use std::convert::Infallible;
use symbolica::{
    atom::{Atom, packing::*},
    coefficient::Coefficient,
    domains::{
        float::{Complex, Float},
        rational::Rational,
    },
    parse,
};

fn environment() -> FloatEnvironment {
    FloatEnvironment {
        min_precision: rug::float::prec_min(),
        max_precision: rug::float::prec_max(),
        min_exponent: i64::from(rug::float::exp_min()),
        max_exponent: i64::from(rug::float::exp_max()),
    }
}

fn inspect(bytes: &[u8]) -> Result<PackingSummary, PackingError<Infallible>> {
    inspect_packing(
        bytes,
        &mut [PackingFrame::default(); 128],
        environment(),
        |_| Ok(()),
    )
}

fn bytes(atom: &Atom) -> &[u8] {
    atom.as_view().get_data()
}

fn issue(bytes: &[u8], expected: PackingIssue) {
    assert!(
        matches!(inspect(bytes), Err(PackingError::Layout { issue, .. }) if issue == expected),
        "{bytes:?}: {:?}",
        inspect(bytes)
    );
}

// Malformed text is spliced only into bytes from a safe supplier-created Float;
// these bytes are never passed to an Atom constructor or coefficient parser.
fn finite_float(text: &[u8], precision: u32, env: FloatEnvironment) -> Result<(), PackingIssue> {
    let one = Atom::num(Coefficient::Float(Complex {
        re: Float::with_val(1, 1),
        im: Float::with_val(1, 0),
    }));
    let mut packed = bytes(&one).to_vec();
    assert_eq!(&packed[..2], &[1, 9]);
    let length = u64::from_le_bytes(packed[2..10].try_into().unwrap()) as usize;
    packed.splice(14..10 + length, text.iter().copied());
    packed[2..10].copy_from_slice(&(text.len() as u64 + 4).to_le_bytes());
    packed[10..14].copy_from_slice(&precision.to_le_bytes());
    match inspect_packing(&packed, &mut [PackingFrame::default(); 8], env, |_| {
        Ok::<_, Infallible>(())
    }) {
        Ok(_) => Ok(()),
        Err(PackingError::Layout { issue, .. }) => Err(issue),
        Err(PackingError::Symbol(never)) => match never {},
    }
}

#[test]
fn supplier_nodes_and_symbol_occurrences() {
    let atom = parse!("packing_f(packing_x, packing_y^2) + 3*packing_x*packing_y");
    let mut refs = Vec::new();
    let result = inspect_packing(
        bytes(&atom),
        &mut [PackingFrame::default(); 32],
        environment(),
        |s| {
            refs.push(s);
            Ok::<_, Infallible>(())
        },
    )
    .unwrap();
    assert_eq!(
        (
            result.sums,
            result.functions,
            result.powers,
            result.products
        ),
        (1, 1, 1, 1)
    );
    assert_eq!((result.numbers, result.variables), (2, 4));
    assert_eq!(result.nodes, 10);
    assert_eq!(result.bytes, bytes(&atom).len());
    assert_eq!(refs.iter().filter(|s| s.function).count(), 1);
    assert_eq!(refs.len(), 5);
    assert!(result.max_depth >= 3);
}

#[test]
fn actual_wide_headers_and_deep_caller_frames() {
    let names = (0..300)
        .map(|index| format!("packing_wide_{index}"))
        .collect::<Vec<_>>();
    for (separator, function) in [("+", false), ("*", false), (",", true)] {
        let arguments = names.join(separator);
        let source = if function {
            format!("packing_many({arguments})")
        } else {
            arguments
        };
        let atom = parse!(&source);
        let result = inspect(bytes(&atom)).unwrap();
        assert_eq!(result.nodes, 301);
        assert_eq!(result.variables, 300);
        if function {
            assert_eq!(bytes(&atom)[1], 2); // actual u16 byte length
        } else {
            assert_eq!(bytes(&atom)[1], 0x22); // actual u16 count/u16 length
        }
    }
    let source = format!(
        "{}packing_leaf{}",
        "packing_deep(".repeat(64),
        ")".repeat(64)
    );
    let atom = parse!(&source);
    assert_eq!(inspect(bytes(&atom)).unwrap().max_depth, 65);
    assert!(matches!(
        inspect_packing(
            bytes(&atom),
            &mut [PackingFrame::default(); 64],
            environment(),
            |_| Ok::<_, Infallible>(())
        ),
        Err(PackingError::Layout {
            issue: PackingIssue::InsufficientScratch,
            ..
        })
    ));
}

#[test]
fn actual_small_large_and_mixed_complex_rationals() {
    let real = Rational::from(rug::Rational::from((
        (rug::Integer::from(1) << 131_072u32) + 1,
        (rug::Integer::from(1) << 65_536u32) + 3,
    )));
    let small = Rational::from((3, 7));
    for (re, im, large_count) in [
        (small.clone(), Rational::from(0), 0),
        (Rational::from((257, 65537)), Rational::from(0), 0),
        (Rational::from((65537, 257)), Rational::from(0), 0),
        (Rational::from((1, i64::MAX)), Rational::from(0), 0),
        (Rational::from(i64::MIN), Rational::from(0), 0),
        (Rational::from((-i64::MAX, 1)), small.clone(), 0),
        (real.clone(), Rational::from(0), 1),
        (-real.clone(), Rational::from(0), 1),
        (Rational::from(0), -real.clone(), 1),
        (real.clone(), small.clone(), 1),
        (small.clone(), real.clone(), 1),
        (real.clone(), real.clone(), 2),
        (-real.clone(), real.clone(), 2),
        (real.clone(), -real.clone(), 2),
    ] {
        let is_complex = im != Rational::from(0);
        let atom = Atom::num(Coefficient::Complex(Complex { re, im }));
        let summary = inspect(bytes(&atom)).unwrap();
        assert_eq!(summary.rational_components, if is_complex { 2 } else { 1 });
        assert_eq!(summary.large_rational_bytes != 0, large_count != 0);
        if large_count != 0 {
            assert!(summary.large_rational_bytes >= 24_576 * large_count);
        }
    }
}

#[test]
fn actual_full_width_mpfr_significands_and_precision_refusal() {
    let full_width = |precision: u32, negative: bool| {
        let integer = (rug::Integer::from(1) << (precision - 1))
            + (rug::Integer::from(1) << (precision / 2))
            + 1;
        let mut raw = rug::Float::with_val(precision, &integer);
        raw <<= -(precision as i32);
        if negative {
            raw = -raw;
        }
        // This independently observes the actual producer value: an odd,
        // precision-bit significand needs every bit, including bits beyond f64.
        let (significand, _) = raw.to_integer_exp().unwrap();
        assert_eq!(significand.significant_bits(), precision);
        assert!(significand.is_odd());
        assert!(precision > 53);
        Float::from_raw(raw)
    };
    for precision in [55, 61, 64, 113, 257, 1025] {
        let re = full_width(precision, false);
        let im = full_width(precision + 2, true);
        let re_text = re.serialize();
        let im_text = im.serialize();
        let atom = Atom::num(Coefficient::Float(Complex { re, im }));
        let packed = bytes(&atom);
        let result = inspect(packed).unwrap();
        assert_eq!(result.float_components, 2);
        assert_eq!(result.float_text_bytes, re_text.len() + im_text.len());
        let re_length = u64::from_le_bytes(packed[2..10].try_into().unwrap()) as usize;
        assert_eq!(&packed[14..10 + re_length], re_text);
        let im_start = 10 + re_length + 8;
        assert_eq!(&packed[im_start + 4..], im_text);
        let mut corrupt = packed.to_vec();
        corrupt[10..14].copy_from_slice(&(precision - 1).to_le_bytes());
        // These precisions keep the same producer digit allowance at p-1;
        // refusal therefore distinguishes exact span from mere text length.
        issue(&corrupt, PackingIssue::InexactFloatText);
    }
}

#[test]
fn actual_mpfr_current_exponent_limits_without_range_mutation() {
    let env = environment();
    for exponent in [env.min_exponent, env.max_exponent] {
        let exponent_i32 = i32::try_from(exponent).unwrap();
        for negative in [false, true] {
            let mut raw = rug::Float::with_val(
                113,
                rug::Float::u_exp(1, exponent_i32.checked_sub(1).unwrap()),
            );
            if negative {
                raw = -raw;
            }
            assert!(raw.is_finite() && !raw.is_zero());
            assert_eq!(raw.get_exp(), Some(exponent_i32));
            let atom = Atom::num(Coefficient::Float(Complex {
                re: Float::from_raw(raw),
                im: Float::with_val(17, 0),
            }));
            assert_eq!(inspect(bytes(&atom)).unwrap().float_components, 2);
            let narrowed = if exponent == env.min_exponent {
                FloatEnvironment {
                    min_exponent: exponent + 1,
                    ..env
                }
            } else {
                FloatEnvironment {
                    max_exponent: exponent - 1,
                    ..env
                }
            };
            assert!(matches!(
                inspect_packing(
                    bytes(&atom),
                    &mut [PackingFrame::default(); 8],
                    narrowed,
                    |_| Ok::<_, Infallible>(())
                ),
                Err(PackingError::Layout {
                    issue: PackingIssue::FloatExponentRange,
                    ..
                })
            ));
        }
    }
}

#[test]
fn oversized_small_rational_integer_encodings_refuse() {
    let one = Atom::num(1);
    for (negative, magnitude) in [
        (false, 1u64 << 63),
        (false, u64::MAX),
        (true, (1u64 << 63) + 1),
        (true, u64::MAX),
    ] {
        let mut corrupt = bytes(&one).to_vec();
        corrupt[1] = 4 | if negative { 0x80 } else { 0 };
        corrupt.splice(2.., magnitude.to_le_bytes());
        issue(&corrupt, PackingIssue::InvalidRational);
    }
    for denominator in [1u64 << 63, u64::MAX] {
        let mut corrupt = bytes(&one).to_vec();
        corrupt[1] = 0x41; // u8 numerator, u64 denominator
        corrupt.extend_from_slice(&denominator.to_le_bytes());
        issue(&corrupt, PackingIssue::InvalidDenominator);
    }
}

#[test]
fn nested_power_respects_bounded_composites_and_child_counts() {
    let atom =
        parse!("packing_outer(packing_base(packing_leaf)^packing_exponent, packing_sibling)");
    let base = parse!("packing_base(packing_leaf)");
    let exponent = parse!("packing_exponent");
    let original = bytes(&atom);
    assert!(inspect(original).is_ok());
    assert_eq!(&original[..2], &[3, 1]);
    let base_bytes = bytes(&base);
    let exponent_bytes = bytes(&exponent);
    let positions = original
        .windows(base_bytes.len())
        .enumerate()
        .filter_map(|(i, bytes)| (bytes == base_bytes).then_some(i))
        .collect::<Vec<_>>();
    assert_eq!(positions.len(), 1);
    let base_start = positions[0];
    let base_end = base_start + base_bytes.len();
    assert_eq!(original[base_start - 1], 6); // actual enclosing Pow
    assert_eq!(
        &original[base_end..base_end + exponent_bytes.len()],
        exponent_bytes
    );
    assert_eq!(base_bytes[1], 1); // actual bounded Fun length is u8

    let mut corrupt = original.to_vec();
    corrupt[base_start + 2] -= 1; // nested Fun ends inside its own variable
    issue(&corrupt, PackingIssue::Truncated);
    corrupt.copy_from_slice(original);
    corrupt[base_start + 2] += 1; // nested Fun claims a byte of the Pow exponent
    issue(&corrupt, PackingIssue::TrailingBytes);

    corrupt.copy_from_slice(original);
    corrupt[2] = u8::try_from(base_end - 3).unwrap(); // parent ends before Pow exponent
    issue(&corrupt, PackingIssue::Truncated); // bytes exist physically, beyond parent

    for child_range in [
        base_start..base_end,
        base_end..base_end + exponent_bytes.len(),
    ] {
        let mut corrupt = original.to_vec();
        corrupt[2] -= u8::try_from(child_range.len()).unwrap();
        corrupt.drain(child_range);
        // Pow can consume the sibling as a missing child, but cannot then
        // satisfy the enclosing Fun's original second-child obligation.
        issue(&corrupt, PackingIssue::Truncated);
    }

    let name_width = 1usize << ((original[3] & 15) - 1);
    corrupt = original.to_vec();
    corrupt[4 + name_width] = 1; // complete Pow followed by an undeclared sibling
    issue(&corrupt, PackingIssue::TrailingBytes);
}

#[test]
fn actual_float_serializer_profile_and_exact_components() {
    for precision in [1, 2, 3, 7, 24, 53, 64, 113, 256, 1024] {
        for value in [
            0.0,
            -0.0,
            1.0,
            -2.0,
            0.1,
            f64::MIN_POSITIVE,
            f64::MAX,
            f64::from_bits(1),
        ] {
            let re = Float::with_val(precision, value);
            let im = Float::with_val(precision + 1, -0.125);
            let real_text = re.serialize();
            let imaginary_text = im.serialize();
            let atom = Atom::num(Coefficient::Float(Complex { re, im }));
            let packed = bytes(&atom);
            let before = packed.to_vec();
            let result = inspect(packed)
                .unwrap_or_else(|e| panic!("p={precision} v={value}: {real_text:?}: {e:?}"));
            assert_eq!(packed, before);
            assert_eq!(result.float_components, 2);
            assert_eq!(
                result.float_text_bytes,
                real_text.len() + imaginary_text.len()
            );
            let real_length = u64::from_le_bytes(packed[2..10].try_into().unwrap()) as usize;
            assert_eq!(&packed[10..14], &precision.to_le_bytes());
            assert_eq!(&packed[14..10 + real_length], real_text);
            let im_start = 10 + real_length + 8;
            assert_eq!(
                &packed[im_start..im_start + 4],
                &(precision + 1).to_le_bytes()
            );
            assert_eq!(&packed[im_start + 4..], imaginary_text);
        }
    }
}

#[test]
fn every_truncation_and_trailing_byte_refuses() {
    let atoms = [
        parse!("packing_g(packing_a, packing_b^3)+7*packing_a*packing_b"),
        parse!(
            "123456789012345678901234567890123456789012345678901234567890/99999999999999999999999999999999999999999999991"
        ),
        Atom::num(Coefficient::Float(Complex {
            re: Float::with_val(53, 0.1),
            im: Float::with_val(113, 1.5),
        })),
    ];
    for atom in atoms {
        let packed = bytes(&atom);
        assert!(inspect(packed).is_ok());
        for end in 0..packed.len() {
            assert!(
                inspect(&packed[..end]).is_err(),
                "accepted truncation {end} of {packed:?}"
            );
        }
        let mut trailing = packed.to_vec();
        trailing.push(0);
        issue(&trailing, PackingIssue::TrailingBytes);
    }
}

#[test]
fn byte_corruptions_never_enter_unchecked_supplier_views() {
    for atom in [
        parse!("packing_h(packing_c^2, packing_c+3)*packing_d"),
        Atom::num(Coefficient::Float(Complex {
            re: Float::with_val(17, 0.1),
            im: Float::with_val(29, 1.5),
        })),
    ] {
        let original = bytes(&atom);
        let mut corrupt = original.to_vec();
        for offset in 0..original.len() {
            for byte in 0..=u8::MAX {
                corrupt[offset] = byte;
                let _ = inspect(&corrupt); // accepted mutations remain passive bytes
            }
            corrupt[offset] = original[offset];
        }
    }
}

#[test]
fn scratch_capacity_and_original_callback_error() {
    let atom = parse!("packing_k(packing_q^2)");
    assert!(matches!(
        inspect_packing(
            bytes(&atom),
            &mut [],
            environment(),
            |_| Ok::<_, Infallible>(())
        ),
        Err(PackingError::Layout {
            issue: PackingIssue::InsufficientScratch,
            ..
        })
    ));
    assert!(matches!(
        inspect_packing(
            bytes(&atom),
            &mut [PackingFrame::default(); 2],
            environment(),
            |_| Ok::<_, Infallible>(())
        ),
        Err(PackingError::Layout {
            issue: PackingIssue::InsufficientScratch,
            ..
        })
    ));
    struct Original {
        payload: Box<u8>,
    }
    let error = Original {
        payload: Box::new(17),
    };
    let pointer = (&*error.payload) as *const u8;
    let mut error = Some(error);
    let result = inspect_packing(
        bytes(&atom),
        &mut [PackingFrame::default(); 8],
        environment(),
        |_| Err(error.take().unwrap()),
    );
    match result {
        Err(PackingError::Symbol(original)) => {
            assert_eq!((&*original.payload) as *const u8, pointer)
        }
        _ => panic!("callback error was lost"),
    }
}

#[test]
fn callback_error_source_preserves_nonclone_original_and_chain() {
    #[derive(Debug)]
    struct OriginalCause {
        payload: Box<u8>,
    }
    impl std::fmt::Display for OriginalCause {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("original callback cause")
        }
    }
    impl std::error::Error for OriginalCause {}
    #[derive(Debug)]
    struct OriginalCallbackError {
        cause: OriginalCause,
    }
    impl std::fmt::Display for OriginalCallbackError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("original callback refusal")
        }
    }
    impl std::error::Error for OriginalCallbackError {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            Some(&self.cause)
        }
    }

    let original = OriginalCallbackError {
        cause: OriginalCause {
            payload: Box::new(23),
        },
    };
    let pointer = (&*original.cause.payload) as *const u8;
    let mut original = Some(original);
    let atom = parse!("packing_callback_cause");
    let error = inspect_packing(
        bytes(&atom),
        &mut [PackingFrame::default(); 8],
        environment(),
        |_| Err(original.take().unwrap()),
    )
    .unwrap_err();
    let source = std::error::Error::source(&error).unwrap();
    let callback = source.downcast_ref::<OriginalCallbackError>().unwrap();
    assert_eq!((&*callback.cause.payload) as *const u8, pointer);
    let cause = source
        .source()
        .unwrap()
        .downcast_ref::<OriginalCause>()
        .unwrap();
    assert_eq!((&*cause.payload) as *const u8, pointer);
    assert!(source.source().unwrap().source().is_none());

    let layout = PackingError::<OriginalCallbackError>::Layout {
        offset: 0,
        issue: PackingIssue::Truncated,
    };
    assert!(std::error::Error::source(&layout).is_none());
}

#[test]
fn headers_counts_parent_exhaustion_and_denominators() {
    let atom = parse!("packing_j(packing_u, packing_v)");
    let original = bytes(&atom);
    // This actual small function has a u8 byte length and u8/u8 symbol/count pair.
    assert_eq!(&original[..2], &[3, 1]);
    let name_width_tag = original[3] & 15;
    assert!((1..=4).contains(&name_width_tag));
    assert_eq!(original[3] & 0x70, 0x10);
    let count_offset = 4 + (1usize << (name_width_tag - 1));
    let mut corrupt = original.to_vec();
    corrupt[2] -= 1;
    assert!(inspect(&corrupt).is_err());
    corrupt.copy_from_slice(original);
    corrupt[count_offset] = 1;
    issue(&corrupt, PackingIssue::TrailingBytes);
    corrupt[count_offset] = u8::MAX;
    issue(&corrupt, PackingIssue::InvalidExtent);
    corrupt.copy_from_slice(original);
    corrupt[1] = 0x81;
    issue(&corrupt, PackingIssue::InvalidPackedInteger);
    corrupt.copy_from_slice(original);
    corrupt[1] = 0x11;
    corrupt.insert(3, 1); // denominator value 1 still violates the length grammar
    issue(&corrupt, PackingIssue::InvalidPackedInteger);
    let atom = Atom::num(Rational::from((3, 7)));
    let mut corrupt = bytes(&atom).to_vec();
    assert_eq!(corrupt, [1, 0x11, 3, 7]);
    corrupt[3] = 0;
    issue(&corrupt, PackingIssue::InvalidDenominator);
    let atom = parse!("packing_u*packing_v");
    let mut corrupt = bytes(&atom).to_vec();
    corrupt[0] |= 0x40;
    issue(&corrupt, PackingIssue::MissingProductCoefficient);
}

#[test]
fn large_rational_counts_zero_denominator_and_partial_payload() {
    let atom = parse!(
        "123456789012345678901234567890123456789012345678901/99999999999999999999999999999999999999999999"
    );
    let original = bytes(&atom);
    assert_eq!(original[1], 0x77);
    assert_eq!(original[2] & 0x7f, 0x11);
    let n = usize::from(original[3]);
    let d = usize::from(original[4]);
    let mut corrupt = original.to_vec();
    corrupt[5 + n..5 + n + d].fill(0);
    issue(&corrupt, PackingIssue::InvalidDenominator);
    corrupt.copy_from_slice(original);
    corrupt[2] = 0x44; // complete u64/u64 lengths must exist before payload use
    corrupt[3..19].fill(0xff);
    assert!(inspect(&corrupt).is_err());
    corrupt.copy_from_slice(original);
    corrupt[2] = 0x51; // unsupported denominator-width tag, never a shift
    issue(&corrupt, PackingIssue::InvalidPackedInteger);
}

#[test]
fn unsupported_flags_attributes_and_coefficients() {
    let atom = parse!("packing_plain");
    for flag in [8, 16, 32, 64, 128] {
        let mut corrupt = bytes(&atom).to_vec();
        corrupt[0] |= flag;
        issue(&corrupt, PackingIssue::UnsupportedFlags);
    }
    let mut corrupt = bytes(&atom).to_vec();
    assert_eq!(corrupt[1] & 0x70, 0);
    corrupt[1] |= 0x10;
    corrupt.push(3); // actual attributed Var's extra*2+1 representation
    issue(&corrupt, PackingIssue::UnsupportedSymbolAttributes);
    let attributed = Atom::var(symbolica::symbol!("packing_real"; Real));
    issue(
        bytes(&attributed),
        PackingIssue::UnsupportedSymbolAttributes,
    );
    let atom = Atom::num(1);
    for tag in [0, 5, 6, 7, 8, 11, 12, 13, 14, 15, 0x87, 0xf7] {
        let mut corrupt = bytes(&atom).to_vec();
        corrupt[1] = tag;
        issue(&corrupt, PackingIssue::UnsupportedCoefficient);
    }
    for value in [f64::INFINITY, f64::NEG_INFINITY, f64::NAN] {
        let atom = Atom::num(Coefficient::Float(Complex {
            re: Float::with_val(53, value),
            im: Float::with_val(53, 0),
        }));
        issue(bytes(&atom), PackingIssue::UnsupportedFloatText);
    }
}

#[test]
fn strict_float_text_exact_span_exponents_and_environment() {
    let env = FloatEnvironment {
        min_precision: 1,
        max_precision: 1024,
        min_exponent: -100,
        max_exponent: 100,
    };
    for text in [
        b"".as_slice(),
        b"-0",
        b"0.0",
        b"-0@3",
        b"nan",
        b"@Inf@",
        b"+1",
        b" 1",
        b"1_0",
        b"1p0",
        b"a@+1",
        b"1@",
        b".1",
        b"1.",
        b"1..1",
        b"1@1@1",
        b"1\0",
        b"A",
        b"00a",
    ] {
        assert!(finite_float(text, 53, env).is_err(), "{text:?}");
    }
    for text in [
        b"1@99999999999999999999999999".as_slice(),
        b"1@-9999999999999999999999999",
        b"1@26",
        b"1@-26",
    ] {
        assert_eq!(
            finite_float(text, 53, env),
            Err(PackingIssue::FloatExponentRange),
            "{text:?}"
        );
    }
    assert_eq!(
        finite_float(b"1.1", 4, env),
        Err(PackingIssue::InexactFloatText)
    );
    assert!(finite_float(b"1.1", 5, env).is_ok());
    assert!(finite_float(b"8", 1, env).is_ok());
    assert_eq!(
        finite_float(b"8.000", 1, env),
        Err(PackingIssue::UnsupportedFloatText)
    );
    assert!(finite_float(b"1@24", 1, env).is_ok());
    assert!(finite_float(b"1@-25", 1, env).is_ok());
    assert_eq!(
        finite_float(
            b"1@24",
            1,
            FloatEnvironment {
                max_exponent: 96,
                ..env
            }
        ),
        Err(PackingIssue::FloatExponentRange)
    );
    assert_eq!(
        finite_float(
            b"1@-25",
            1,
            FloatEnvironment {
                min_exponent: -98,
                ..env
            }
        ),
        Err(PackingIssue::FloatExponentRange)
    );
    assert!(finite_float(b"0", 1, env).is_ok());
}

#[test]
fn float_component_extents_and_precision_refuse_before_parsing() {
    let atom = Atom::num(Coefficient::Float(Complex {
        re: Float::with_val(53, 0.1),
        im: Float::with_val(113, 1.5),
    }));
    let original = bytes(&atom);
    for length in [0, 1, 2, 3, u64::MAX] {
        let mut corrupt = original.to_vec();
        corrupt[2..10].copy_from_slice(&length.to_le_bytes());
        assert!(inspect(&corrupt).is_err());
    }
    let mut corrupt = original.to_vec();
    corrupt[10..14].copy_from_slice(&0u32.to_le_bytes());
    issue(&corrupt, PackingIssue::InvalidFloatPrecision);
    let real_length = u64::from_le_bytes(original[2..10].try_into().unwrap()) as usize;
    corrupt.copy_from_slice(original);
    let imaginary_start = 10 + real_length + 8;
    corrupt[imaginary_start..imaginary_start + 4].copy_from_slice(&0u32.to_le_bytes());
    issue(&corrupt, PackingIssue::InvalidFloatPrecision);
    let mut env = environment();
    env.max_precision = 52;
    assert!(matches!(
        inspect_packing(original, &mut [PackingFrame::default(); 8], env, |_| Ok::<
            _,
            Infallible,
        >(
            ()
        )),
        Err(PackingError::Layout {
            issue: PackingIssue::InvalidFloatPrecision,
            ..
        })
    ));
    env.min_precision = 54;
    assert!(matches!(
        inspect_packing(original, &mut [PackingFrame::default(); 8], env, |_| Ok::<
            _,
            Infallible,
        >(
            ()
        )),
        Err(PackingError::Layout {
            issue: PackingIssue::InvalidFloatEnvironment,
            ..
        })
    ));
}
