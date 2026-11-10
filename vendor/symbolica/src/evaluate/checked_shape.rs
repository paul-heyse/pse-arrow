//! Admit native dynamic Taylor metadata before constructing the dualizer.
use super::*;
use numerica::domains::{backend::integer::IntegerAllocationContext, dual::HyperDual};
use std::mem::align_of;

fn add<E>(a: usize, b: usize) -> Result<usize, OperationError<E>> {
    a.checked_add(b).ok_or(OperationError::Overflow)
}
fn mul<E>(a: usize, b: usize) -> Result<usize, OperationError<E>> {
    a.checked_mul(b).ok_or(OperationError::Overflow)
}
fn bytes<E, T>(n: usize) -> Result<usize, OperationError<E>> {
    mul(n, size_of::<T>())
}

// The selected native HashMap/HashSet use power-of-two SwissTable buckets,
// maximum 7/8 occupancy and at least four buckets. Include control bytes, the
// trailing control group and alignment, plus old/new table growth overlap.
fn table_bytes<E, T>(n: usize) -> Result<usize, OperationError<E>> {
    if n == 0 {
        return Ok(0);
    }
    let buckets = add(mul(n, 8)?, 6)? / 7;
    let buckets = buckets
        .max(4)
        .checked_next_power_of_two()
        .ok_or(OperationError::Overflow)?;
    mul(
        2,
        add(add(bytes::<E, T>(buckets)?, buckets)?, 16 + align_of::<T>())?,
    )
}

struct Metadata {
    input: usize,
    peak: usize,
    resident: usize,
    work: usize,
}

fn metadata<E>(
    shape: &[Vec<usize>],
    zeros: &[(usize, usize)],
) -> Result<Metadata, OperationError<E>> {
    let Some(unit) = shape.first() else {
        return Err(OperationError::Unsupported("empty Taylor shape"));
    };
    let width = shape.len();
    let axes = unit.len();
    let mut input = add(
        bytes::<E, Vec<usize>>(width)?,
        bytes::<E, (usize, usize)>(zeros.len())?,
    )?;
    let mut squares = 0;
    let mut mixed = 0;
    for component in shape {
        if component.len() != axes {
            return Err(OperationError::Unsupported(
                "inconsistent Taylor shape axes",
            ));
        }
        input = add(input, bytes::<E, usize>(component.capacity())?)?;
        let depth = component
            .iter()
            .try_fold(0usize, |depth, &power| add::<E>(depth, power))?;
        if depth > 2 {
            return Err(OperationError::Unsupported("Taylor depth greater than two"));
        }
        if depth == 2 {
            if component.contains(&2) {
                squares = add(squares, 1)?;
            } else {
                mixed = add(mixed, 1)?;
            }
        }
    }
    if unit.iter().any(|&power| power != 0) {
        return Err(OperationError::Unsupported("Taylor unit must be first"));
    }
    // Unit times every non-unit, one square pair and two ordered mixed pairs.
    // This exact output cardinality avoids reserving a dense width² table.
    let entries = add(add(width - 1, squares)?, mul(2, mixed)?)?;
    let table_capacity = if entries == 0 {
        0
    } else {
        entries
            .max(4)
            .checked_next_power_of_two()
            .ok_or(OperationError::Overflow)?
    };
    let shape_rows = bytes::<E, Vec<usize>>(width)?;
    let shape_data = bytes::<E, usize>(mul(width, axes)?)?;
    let value_data = bytes::<E, Complex<Rational>>(width)?;
    let table = bytes::<E, (usize, usize, usize)>(table_capacity)?;
    // Native DualShape is two Vecs, max_pow and the shared allocation counters.
    let shared_shape = size_of::<Vec<Vec<usize>>>()
        + size_of::<usize>()
        + size_of::<Vec<(usize, usize, usize)>>()
        + 2 * size_of::<usize>();
    let zero_table = table_bytes::<E, (usize, usize)>(zeros.len())?;
    let resident = add(
        size_of::<Dualizer<HyperDual<Complex<Rational>>>>(),
        add(
            add(add(shape_rows, shape_data)?, value_data)?,
            add(add(table, shared_shape)?, zero_table)?,
        )?,
    )?;
    // Native construction owns a second clone of every shape as the map key,
    // max-per-axis/sum/ancestor scratch, old/new multiplication-table capacity,
    // and the copied structural-zero input while collecting its hash set.
    let native_temporary = add(
        add(shape_data, table_bytes::<E, (Vec<usize>, usize)>(width)?)?,
        add(
            add(bytes::<E, usize>(mul(3, axes)?)?, table)?,
            bytes::<E, (usize, usize)>(zeros.len())?,
        )?,
    )?;
    let validation = add(bytes::<E, usize>(axes)?, bytes::<E, (usize, usize)>(width)?)?;
    let peak = add(resident, native_temporary)?.max(validation);
    // Report the native pair/coordinate scan work separately from stored table
    // cardinality. The current native constructor performs that scan even when
    // only a sparse set of products survives.
    let work = mul(mul(width, width)?, add(mul(3, axes)?, 1)?)?;
    Ok(Metadata {
        input,
        peak,
        resident,
        work,
    })
}

fn validate<E>(
    shape: &[Vec<usize>],
    zeros: &[(usize, usize)],
    parameters: usize,
) -> Result<(), OperationError<E>> {
    let axes = shape[0].len();
    let mut first = vec![false; axes];
    let mut seconds = Vec::with_capacity(shape.len());
    let mut units = 0;
    for component in shape {
        let mut indices = component
            .iter()
            .enumerate()
            .flat_map(|(axis, &power)| std::iter::repeat_n(axis, power));
        match (indices.next(), indices.next()) {
            (None, None) => units += 1,
            (Some(a), None) => {
                if first[a] {
                    return Err(OperationError::Unsupported("duplicate Taylor component"));
                }
                first[a] = true;
            }
            (Some(a), Some(b)) => seconds.push((a, b)),
            _ => unreachable!("a component has at most two coordinates"),
        }
    }
    seconds.sort_unstable();
    if units != 1 || seconds.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(OperationError::Unsupported("duplicate Taylor component"));
    }
    if seconds.iter().any(|&(a, b)| !first[a] || !first[b]) {
        return Err(OperationError::Unsupported(
            "Taylor shape is not ancestor-closed",
        ));
    }
    if zeros.iter().any(|&(parameter, component)| {
        parameter >= parameters || component == 0 || component >= shape.len()
    }) {
        return Err(OperationError::Unsupported(
            "invalid structural zero component",
        ));
    }
    Ok(())
}

fn flatten<E>(error: OperationError<OperationError<E>>) -> OperationError<E> {
    match error {
        OperationError::Control(error) => error,
        OperationError::Overflow => OperationError::Overflow,
        OperationError::Allocation => OperationError::Allocation,
        OperationError::Unsupported(reason) => OperationError::Unsupported(reason),
        OperationError::InvalidProgram(reason) => OperationError::InvalidProgram(reason),
        OperationError::Interrupted => OperationError::Interrupted,
    }
}

impl ExpressionEvaluator<Complex<Rational>> {
    /// Construct and vectorize native Taylor metadata under the same borrowed
    /// control, including metadata still live during coefficient expansion.
    pub fn try_vectorize_shape_with<C: OperationControl>(
        &self,
        shape: &[Vec<usize>],
        zero_components: &[(usize, usize)],
        control: &mut C,
    ) -> Result<Self, OperationError<C::Error>> {
        self.try_vectorize_shape_known_with(shape, zero_components, &[], control)
    }

    /// As `try_vectorize_known_with`, also admitting and validating dynamic
    /// HyperDual/Dualizer construction before their native buffers can grow.
    /// Shapes are unique, unit-first, ancestor-closed and of depth at most two.
    pub fn try_vectorize_shape_known_with<C: OperationControl>(
        &self,
        shape: &[Vec<usize>],
        zero_components: &[(usize, usize)],
        heads: &[KnownHead],
        control: &mut C,
    ) -> Result<Self, OperationError<C::Error>> {
        control
            .check(OperationExtent {
                stage: OperationStage::Construction,
                ..Default::default()
            })
            .map_err(OperationError::Control)?;
        let source = super::checked_copy::extent(self, super::control::payload::<C::Error>)?;
        let meta = metadata::<C::Error>(shape, zero_components)?;
        let input = add(source.input_bytes, meta.input)?;
        let extent = OperationExtent {
            stage: OperationStage::Construction,
            work: meta.work,
            input_bytes: input,
            temporary_bytes: meta.peak,
            output_entries: shape.len(),
            ..Default::default()
        };
        extent.peak_bytes().ok_or(OperationError::Overflow)?;
        control.check(extent).map_err(OperationError::Control)?;
        validate::<C::Error>(shape, zero_components, self.param_count)?;
        control
            .check(OperationExtent { work: 0, ..extent })
            .map_err(OperationError::Control)?;
        IntegerAllocationContext::fresh().run(|| {
            let dual = Dualizer::new(
                HyperDual::<Complex<Rational>>::new(shape.to_vec()),
                zero_components.to_vec(),
            );
            let mut wrapped =
                |mut extent: OperationExtent| -> Result<(), OperationError<C::Error>> {
                    extent.input_bytes = add(extent.input_bytes, meta.input)?;
                    extent.temporary_bytes = add(extent.temporary_bytes, meta.resident)?;
                    extent.peak_bytes().ok_or(OperationError::Overflow)?;
                    control.check(extent).map_err(OperationError::Control)
                };
            self.try_vectorize_known_with(&dual, heads, &mut wrapped)
                .map_err(flatten)
        })
    }
}

#[cfg(test)]
mod checked_shape_tests {
    use super::*;
    use std::convert::Infallible;

    fn source() -> ExpressionEvaluator<Complex<Rational>> {
        let x = crate::parse!("checked_shape::x");
        let y = crate::parse!("checked_shape::y");
        (x.clone() * &x * &y + y.clone() * 2)
            .evaluator(&[x, y])
            .build()
            .unwrap()
    }
    fn shape() -> Vec<Vec<usize>> {
        vec![
            vec![0, 0],
            vec![0, 1],
            vec![1, 0],
            vec![1, 1],
            vec![0, 2],
            vec![2, 0],
        ]
    }
    #[test]
    fn checked_shape_precedes_native_growth_and_preserves_typed_refusal() {
        let source = source();
        let shape = shape();
        let mut metadata_peak = 0;
        source
            .try_vectorize_shape_with(&shape, &[], &mut |extent: OperationExtent| {
                if extent.stage == OperationStage::Construction {
                    metadata_peak = metadata_peak.max(extent.peak_bytes().unwrap());
                }
                Ok::<_, Infallible>(())
            })
            .unwrap();
        assert!(metadata_peak > 0);
        let mut calls = 0;
        let error = source
            .try_vectorize_shape_with(&shape, &[], &mut |extent: OperationExtent| {
                calls += 1;
                if extent.peak_bytes().unwrap() > metadata_peak - 1 {
                    Err("one byte short")
                } else {
                    Ok(())
                }
            })
            .unwrap_err();
        assert_eq!(error, OperationError::Control("one byte short"));
        assert_eq!(
            calls, 2,
            "refusal occurs at metadata admission, before native construction"
        );
        let error = source
            .try_vectorize_shape_with(&shape, &[], &mut |extent: OperationExtent| {
                if extent.stage == OperationStage::Vectorization {
                    assert!(
                        extent.temporary_bytes > 0,
                        "native metadata is live in subsequent admission"
                    );
                    Err("original operation stopped")
                } else {
                    Ok(())
                }
            })
            .unwrap_err();
        assert_eq!(error, OperationError::Control("original operation stopped"));
    }
    #[test]
    fn checked_shape_mixed_reordered_components_match_native_and_analytic_coefficients() {
        let source = source();
        let shape = shape();
        let zeros = vec![
            (0, 1),
            (0, 3),
            (0, 4),
            (0, 5),
            (1, 2),
            (1, 3),
            (1, 4),
            (1, 5),
        ];
        let checked = source
            .try_vectorize_shape_with(&shape, &zeros, &mut |_| Ok::<_, Infallible>(()))
            .unwrap();
        let native = source
            .clone()
            .vectorize(&Dualizer::new(
                HyperDual::<Complex<Rational>>::new(shape),
                zeros,
            ))
            .unwrap();
        let mut checked = checked.map_coeff(&|value| value.re.to_f64());
        let mut native = native.map_coeff(&|value| value.re.to_f64());
        let inputs = [2., 0., 1., 0., 0., 0., 3., 1., 0., 0., 0., 0.];
        let mut actual = vec![0.; 6];
        let mut expected = vec![0.; 6];
        checked.evaluate(&inputs, &mut actual);
        native.evaluate(&inputs, &mut expected);
        assert_eq!(actual, expected);
        assert_eq!(actual, [18., 6., 12., 4., 0., 3.]);
    }
    #[test]
    fn checked_shape_rejects_invalid_metadata_and_accepts_zero_axis_unit() {
        let source = source();
        for shape in [
            vec![],
            vec![vec![1]],
            vec![vec![0], vec![0]],
            vec![vec![0], vec![2]],
            vec![vec![0, 0], vec![1]],
            vec![vec![0], vec![3]],
            vec![vec![0, 0], vec![1, 0], vec![1, 1]],
        ] {
            let result =
                source.try_vectorize_shape_with(&shape, &[], &mut |_| Ok::<_, Infallible>(()));
            assert!(
                matches!(result, Err(OperationError::Unsupported(_))),
                "invalid shape {shape:?}"
            );
        }
        for zero in [(0, 0), (0, 6), (2, 1)] {
            assert!(matches!(
                source.try_vectorize_shape_with(
                    &shape(),
                    &[zero],
                    &mut |_| Ok::<_, Infallible>(())
                ),
                Err(OperationError::Unsupported(_))
            ));
        }
        let constant = crate::parse!("5").evaluator::<Atom>(&[]).build().unwrap();
        let product = constant
            .try_vectorize_shape_with(&[vec![]], &[], &mut |_| Ok::<_, Infallible>(()))
            .unwrap();
        let mut product = product.map_coeff(&|value| value.re.to_f64());
        let mut values = vec![0.];
        product.evaluate(&[], &mut values);
        assert_eq!(values, [5.]);
        assert!(matches!(metadata::<Infallible>(&[vec![0; 2]], &[]), Ok(_)));
        assert!(matches!(
            table_bytes::<Infallible, (usize, usize)>(usize::MAX),
            Err(OperationError::Overflow)
        ));
    }
}
