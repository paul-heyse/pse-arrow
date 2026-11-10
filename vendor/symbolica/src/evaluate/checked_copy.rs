//! Passive admission of the public instruction representation's actual widening.
use super::*;
use numerica::domains::backend::integer::IntegerAllocationContext;

fn add<E>(a: usize, b: usize) -> Result<usize, OperationError<E>> {
    a.checked_add(b).ok_or(OperationError::Overflow)
}
fn bytes<E, T>(n: usize) -> Result<usize, OperationError<E>> {
    n.checked_mul(size_of::<T>())
        .ok_or(OperationError::Overflow)
}

// No owned Atom is formatted here: a canonical tag can invoke a custom printer.
// Retained bodies likewise need a separate recursive sharing/backing contract.
fn extent<T, E>(
    evaluator: &ExpressionEvaluator<T>,
    payload: impl Fn(&T) -> Result<usize, OperationError<E>>,
) -> Result<OperationExtent, OperationError<E>> {
    let e = evaluator;
    if e.param_count > e.reserved_indices || e.reserved_indices > e.stack.len() {
        return Err(OperationError::Unsupported("invalid evaluator stack"));
    }
    if e.settings.hot_start.is_some() {
        return Err(OperationError::Unsupported("retained Horner hot start"));
    }
    let mut input = bytes::<E, T>(e.stack.capacity())?;
    for value in &e.stack {
        input = add(input, payload(value)?)?;
    }
    input = add(
        input,
        bytes::<E, (Instr, ComplexPhase)>(e.instructions.capacity())?,
    )?;
    input = add(input, bytes::<E, usize>(e.result_indices.capacity())?)?;
    input = add(
        input,
        bytes::<E, ExternalFunctionContainer<T>>(e.external_fns.capacity())?,
    )?;
    let mut output = bytes::<E, Instruction>(add(e.instructions.len(), e.result_indices.len())?)?;
    output = add(output, bytes::<E, T>(e.reserved_indices - e.param_count)?)?;
    for value in &e.stack[e.param_count..e.reserved_indices] {
        output = add(output, payload(value)?)?;
    }
    output = add(
        output,
        bytes::<E, ExportedConstantFunction>(e.external_fns.len())?,
    )?;
    for external in &e.external_fns {
        if external.body.is_some() || !external.tags.is_empty() {
            return Err(OperationError::Unsupported(
                "tagged or retained-body instruction export",
            ));
        }
        input = add(input, external.export_name.capacity())?;
        input = add(input, bytes::<E, T>(external.stack.capacity())?)?;
        for value in &external.stack {
            input = add(input, payload(value)?)?;
        }
        input = add(
            input,
            bytes::<E, Complex<Rational>>(external.fixed_args.capacity())?,
        )?;
        for value in &external.fixed_args {
            input = add(input, control::payload(value)?)?;
        }
        if let Some(implementation) = &external.imp {
            input = add(input, size_of_val(implementation.as_ref()))?;
        }
        if external.constant_index.is_some() {
            output = add(
                output,
                bytes::<E, Complex<Rational>>(external.fixed_args.len())?,
            )?;
            for value in &external.fixed_args {
                output = add(output, control::payload(value)?)?;
            }
        }
    }
    for (instruction, _) in &e.instructions {
        let arguments = match instruction {
            Instr::Add(_, arguments) | Instr::Mul(_, arguments) => {
                input = add(input, bytes::<E, usize>(arguments.capacity())?)?;
                arguments.len()
            }
            Instr::ExternalFun(_, index, arguments) => {
                if *index >= e.external_fns.len() {
                    return Err(OperationError::Unsupported("external function index"));
                }
                input = add(input, bytes::<E, usize>(arguments.capacity())?)?;
                output = add(output, size_of::<(Symbol, Vec<String>, Vec<Slot>)>())?;
                arguments.len()
            }
            Instr::BuiltinFun(..) => {
                output = add(output, size_of::<(Symbol, Vec<String>, Vec<Slot>)>())?;
                1
            }
            _ => 0,
        };
        output = add(output, bytes::<E, Slot>(arguments)?)?;
    }
    // The shared exporter preallocates its public instruction list and constant
    // metadata. Calls and the outer body table coexist with the final product.
    // Exact Clone uses fresh GMP allocations; one limb of backend rounding per
    // copied integer plus one complete payload overlap bounds the fresh copy.
    let copies = add(
        e.reserved_indices - e.param_count,
        e.external_fns
            .iter()
            .try_fold(0, |n, f| add(n, f.fixed_args.len()))?,
    )?;
    let rounding = copies
        .checked_mul(4 * size_of::<u64>())
        .ok_or(OperationError::Overflow)?;
    output = add(output, rounding)?;
    let temporary = add(
        output,
        add(
            bytes::<E, Option<ExportedSubEvaluator<T>>>(e.external_fns.len())?,
            bytes::<E, usize>(e.instructions.len())?,
        )?,
    )?;
    let extent = OperationExtent {
        stage: OperationStage::Export,
        work: e.instructions.len(),
        input_bytes: input,
        temporary_bytes: temporary,
        output_bytes: output,
        output_entries: e.stack.len(),
    };
    extent.peak_bytes().ok_or(OperationError::Overflow)?;
    Ok(extent)
}

fn export<T: Clone, C: OperationControl>(
    evaluator: &ExpressionEvaluator<T>,
    control: &mut C,
    payload: impl Fn(&T) -> Result<usize, OperationError<C::Error>>,
) -> Result<ExportedInstructions<T>, OperationError<C::Error>> {
    control
        .check(OperationExtent {
            stage: OperationStage::Export,
            ..OperationExtent::default()
        })
        .map_err(OperationError::Control)?;
    let extent = extent(evaluator, payload)?;
    control.check(extent).map_err(OperationError::Control)?;
    let product = IntegerAllocationContext::fresh().run(|| evaluator.export_instructions());
    control
        .check(OperationExtent {
            stage: OperationStage::Finalization,
            work: 0,
            ..extent
        })
        .map_err(OperationError::Control)?;
    Ok(product)
}

impl ExpressionEvaluator<Complex<Rational>> {
    /// Export the widened public instruction/slot representation after passive
    /// backing admission. Tagged calls and retained bodies refuse before copying.
    /// Exact copies use fresh allocation policy; no caller control is retained.
    pub fn try_export_instructions_with<C: OperationControl>(
        &self,
        control: &mut C,
    ) -> Result<ExportedInstructions<Complex<Rational>>, OperationError<C::Error>> {
        export(self, control, control::payload)
    }
}

impl ExpressionEvaluator<f64> {
    /// Export a native numerical program with borrowed admission before widening.
    /// Registered numerical callbacks are never invoked or cloned by the export.
    pub fn try_export_instructions_with<C: OperationControl>(
        &self,
        control: &mut C,
    ) -> Result<ExportedInstructions<f64>, OperationError<C::Error>> {
        export(self, control, |_| Ok(0))
    }
}

#[cfg(all(test, feature = "integer-gmp"))]
mod tests {
    use super::*;
    use crate::{
        domains::integer::{MultiPrecisionInteger, RawMultiPrecisionInteger},
        parse,
    };
    use std::convert::Infallible;

    fn exported_backing(e: &ExportedInstructions<Complex<Rational>>) -> usize {
        let mut bytes = e.instructions.capacity() * size_of::<Instruction>()
            + e.constants.capacity() * size_of::<Complex<Rational>>()
            + e.constant_functions.capacity() * size_of::<ExportedConstantFunction>()
            + e.sub_evaluators.capacity() * size_of::<ExportedSubEvaluator<Complex<Rational>>>();
        for value in &e.constants {
            bytes += control::payload::<Infallible>(value).unwrap();
        }
        for instruction in &e.instructions {
            bytes += match instruction {
                Instruction::Add(_, args, _) | Instruction::Mul(_, args, _) => {
                    args.capacity() * size_of::<Slot>()
                }
                Instruction::Fun(_, body, _) => {
                    size_of_val(body.as_ref()) + body.2.capacity() * size_of::<Slot>()
                }
                _ => 0,
            };
        }
        bytes
    }

    #[test]
    fn checked_public_export_admits_widening_and_fresh_exact_copies() {
        let x = parse!("checked_copy_x");
        let source = parse!("checked_copy_x + 1361129467683753853853498429727072845824");
        let evaluator = source.evaluator(&[x]).horner_iterations(0).build().unwrap();
        let warmed = (0..32)
            .map(|_| {
                MultiPrecisionInteger::from_raw(RawMultiPrecisionInteger::from(1) << 1_000_000_u32)
            })
            .collect::<Vec<_>>();
        drop(warmed);
        let mut admitted = 0;
        let exported = evaluator
            .try_export_instructions_with(&mut |extent: OperationExtent| {
                admitted = admitted.max(extent.output_bytes);
                Ok::<_, Infallible>(())
            })
            .unwrap();
        assert!(exported_backing(&exported) <= admitted);
        assert!(
            admitted < 32_768,
            "public widening does not adopt the warmed limb pool"
        );
        assert_eq!(
            exported.instructions.len(),
            evaluator.export_instructions().instructions.len()
        );
        let numeric = evaluator.map_coeff(&|c| c.re.to_f64());
        assert_eq!(
            numeric
                .try_export_instructions_with(&mut |_: OperationExtent| Ok::<_, Infallible>(()))
                .unwrap()
                .input_count,
            1
        );
    }

    #[test]
    fn checked_public_export_preserves_original_refusal_and_final_stop() {
        #[derive(Debug, PartialEq, Eq)]
        struct Refused(u64);
        let evaluator = parse!("1").evaluator(&[] as &[Atom]).build().unwrap();
        let refusal = evaluator.try_export_instructions_with(&mut |extent: OperationExtent| {
            if extent.output_bytes > 0 {
                Err(Refused(73))
            } else {
                Ok(())
            }
        });
        assert_eq!(refusal.unwrap_err(), OperationError::Control(Refused(73)));
        let stopped = evaluator.try_export_instructions_with(&mut |extent: OperationExtent| {
            if extent.stage == OperationStage::Finalization {
                Err(Refused(91))
            } else {
                Ok(())
            }
        });
        assert_eq!(stopped.unwrap_err(), OperationError::Control(Refused(91)));
    }
}
