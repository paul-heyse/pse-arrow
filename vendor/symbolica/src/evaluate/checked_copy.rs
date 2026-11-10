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
pub(super) fn extent<T, E>(
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

    #[test]
    fn checked_mapping_admits_post_horner_payload_without_cached_capacity() {
        let x = parse!("checked_mapping_x");
        let expression = parse!("(2*checked_mapping_x+2)^512 + checked_mapping_x");
        let source = expression
            .evaluator(&[x])
            .direct_translation(true)
            .horner_iterations(1)
            .build()
            .unwrap();
        let original_payload = source
            .stack
            .iter()
            .map(|value| control::payload::<Infallible>(value).unwrap())
            .sum::<usize>();
        assert!(original_payload > 64);
        let warmed = (0..32)
            .map(|_| {
                MultiPrecisionInteger::from_raw(RawMultiPrecisionInteger::from(1) << 1_000_000_u32)
            })
            .collect::<Vec<_>>();
        drop(warmed);
        let mut peak = 0;
        let mut final_bytes = 0;
        let mut mapped = source
            .try_map_f64_with(&mut |extent: OperationExtent| {
                peak = peak.max(extent.peak_bytes().unwrap());
                if extent.stage == OperationStage::Finalization {
                    final_bytes = extent.output_bytes;
                }
                Ok::<_, Infallible>(())
            })
            .unwrap();
        assert!(peak < 2_000_000);
        assert_eq!(
            final_bytes,
            extent::<_, Infallible>(&mapped, |_| Ok(0))
                .unwrap()
                .input_bytes
        );
        assert_eq!(mapped.evaluate_single(&[0.]), 2_f64.powi(512));
        assert_eq!(
            source
                .stack
                .iter()
                .map(|value| control::payload::<Infallible>(value).unwrap())
                .sum::<usize>(),
            original_payload
        );
        #[derive(Debug, PartialEq, Eq)]
        struct Refused;
        assert_eq!(
            source
                .try_map_f64_with(&mut |extent: OperationExtent| {
                    if extent.peak_bytes().unwrap() >= peak {
                        Err(Refused)
                    } else {
                        Ok(())
                    }
                })
                .unwrap_err(),
            OperationError::Control(Refused)
        );
    }

    #[test]
    fn checked_mapping_uses_fixed_implementations_and_native_constant_rounding() {
        use std::sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        };
        struct Capture(Arc<AtomicUsize>);
        impl Capture {
            fn value(&self) -> f64 {
                1.
            }
        }
        impl Clone for Capture {
            fn clone(&self) -> Self {
                self.0.fetch_add(1, Ordering::SeqCst);
                Self(self.0.clone())
            }
        }
        let clones = Arc::new(AtomicUsize::new(0));
        let capture = Capture(clones.clone());
        let symbol = crate::symbol!(
            "checked_copy::fixed_mapping_head",
            eval = EvaluationInfo::new().register(move |args: &[f64]| args[0] + capture.value())
        );
        let value = parse!("4611686018427387903/4611686018427387907");
        let source = symbol
            .call(&[value][..])
            .evaluator::<Atom>(&[])
            .direct_translation(true)
            .build()
            .unwrap();
        let head = F64KnownHead {
            tag: 1,
            version: 1,
            symbol,
            arity: 1,
            export_name: "checked_copy_fixed_mapping_head",
            evaluate: |args| args[0] + 1.,
        };
        let before = clones.load(Ordering::SeqCst);
        let mut final_bytes = 0;
        let mut mapped = source
            .try_map_f64_known_with(&[head], &mut |extent: OperationExtent| {
                if extent.stage == OperationStage::Finalization {
                    final_bytes = extent.output_bytes;
                }
                Ok::<_, Infallible>(())
            })
            .unwrap();
        assert_eq!(
            clones.load(Ordering::SeqCst),
            before,
            "registered callback is never cloned"
        );
        assert_eq!(
            final_bytes,
            extent::<_, Infallible>(&mapped, |_| Ok(0))
                .unwrap()
                .input_bytes
        );
        let mut ordinary = source.clone().map_coeff(&|value| value.re.to_f64());
        assert!(clones.load(Ordering::SeqCst) > before);
        assert_eq!(
            mapped.evaluate_single(&[]).to_bits(),
            ordinary.evaluate_single(&[]).to_bits()
        );
        let stopped = source.try_map_f64_known_with(&[head], &mut |extent: OperationExtent| {
            if extent.stage == OperationStage::Finalization {
                Err("current stop")
            } else {
                Ok(())
            }
        });
        assert_eq!(
            stopped.unwrap_err(),
            OperationError::Control("current stop")
        );
    }

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

impl ExpressionEvaluator<Complex<Rational>> {
    /// Copy and map the native programme to binary64 after admitting the actual
    /// post-normalization exact payload, target backing and simultaneous copies.
    /// Only fixed implementations from the trusted table are materialized;
    /// arbitrary registered/tagged generators are never cloned or invoked.
    pub fn try_map_f64_known_with<C: OperationControl>(
        &self,
        heads: &[F64KnownHead],
        control: &mut C,
    ) -> Result<ExpressionEvaluator<f64>, OperationError<C::Error>> {
        control
            .check(OperationExtent {
                stage: OperationStage::Construction,
                ..OperationExtent::default()
            })
            .map_err(OperationError::Control)?;
        super::f64_transfer::check_heads(heads)
            .map_err(|_| OperationError::Unsupported("invalid fixed binary64 head table"))?;
        let source = extent(self, control::payload)?;
        if self.settings.abort_check.is_some() {
            return Err(OperationError::Unsupported(
                "retained producer abort callback",
            ));
        }
        let mut fixed_values = 0usize;
        for external in &self.external_fns {
            if external.imp.is_some() || !external.stack.is_empty() {
                return Err(OperationError::Unsupported(
                    "materialized exact external callback",
                ));
            }
            let builtin_constant = external.constant_index.is_some()
                && external.fixed_args.is_empty()
                && [Symbol::PI, Symbol::E].contains(&external.symbol);
            let head = heads.iter().find(|head| head.symbol == external.symbol);
            if !builtin_constant && head.is_none() {
                return Err(OperationError::Unsupported(
                    "unqualified binary64 numerical head",
                ));
            }
            if let Some(index) = external.constant_index {
                if index >= self.reserved_indices - self.param_count {
                    return Err(OperationError::Unsupported("invalid constant-head slot"));
                }
                if !builtin_constant
                    && head.is_some_and(|head| external.fixed_args.len() != head.arity)
                {
                    return Err(OperationError::Unsupported("fixed constant-head arity"));
                }
                if external.fixed_args.iter().any(|value| !value.im.is_zero()) {
                    return Err(OperationError::Unsupported(
                        "nonreal fixed numerical argument",
                    ));
                }
            } else if !external.fixed_args.is_empty() {
                return Err(OperationError::Unsupported("nonconstant fixed arguments"));
            }
            fixed_values = add(fixed_values, external.fixed_args.len())?;
        }
        for (instruction, _) in &self.instructions {
            if let Instr::ExternalFun(_, index, args) = instruction {
                let external = self
                    .external_fns
                    .get(*index)
                    .ok_or(OperationError::Unsupported("external index"))?;
                let head = heads
                    .iter()
                    .find(|head| head.symbol == external.symbol)
                    .ok_or(OperationError::Unsupported(
                        "unqualified binary64 numerical head",
                    ))?;
                if args.len() != head.arity || external.constant_index.is_some() {
                    return Err(OperationError::Unsupported("fixed numerical call arity"));
                }
            }
        }
        let values = add(self.stack.len(), fixed_values)?;
        // Fresh Clone does not adopt cleared backing. Two carry limbs per copied
        // integer cover native destination rounding independently of source cache.
        let clone_bytes = add(
            source.input_bytes,
            values
                .checked_mul(4 * 2 * size_of::<u64>())
                .ok_or(OperationError::Overflow)?,
        )?;
        let target = add(
            clone_bytes,
            add(
                bytes::<C::Error, f64>(self.stack.len())?,
                self.external_fns
                    .len()
                    .checked_mul(size_of::<fn(&[f64]) -> f64>())
                    .ok_or(OperationError::Overflow)?,
            )?,
        )?;
        // Native mapping holds the source clone, target stack, copied external
        // fixed args, and at most one two-argument MPFR53 conversion at once.
        // Internal GMP/MPFR algorithm scratch stays in the caller's foreign allowance.
        let temporary = add(
            clone_bytes.checked_mul(2).ok_or(OperationError::Overflow)?,
            add(
                source.output_bytes,
                4 * (size_of::<Float>() + 2 * size_of::<u64>()),
            )?,
        )?;
        let admitted = OperationExtent {
            stage: OperationStage::Construction,
            work: self.stack.len(),
            input_bytes: source.input_bytes,
            temporary_bytes: temporary,
            output_bytes: target,
            output_entries: self.stack.len(),
        };
        admitted.peak_bytes().ok_or(OperationError::Overflow)?;
        control.check(admitted).map_err(OperationError::Control)?;
        IntegerAllocationContext::fresh().run(|| {
            let product = self.clone().map_coeff_impl(
                &|value| value.re.to_f64(),
                |external| {
                    let builtin = external.constant_index.is_some()
                        && external.fixed_args.is_empty()
                        && [Symbol::PI, Symbol::E].contains(&external.symbol);
                    let implementation = if builtin {
                        None
                    } else {
                        heads
                            .iter()
                            .find(|head| head.symbol == external.symbol)
                            .map(|head| Box::new(head.evaluate) as Box<dyn ExternalFunction<f64>>)
                    };
                    external.map_with_implementation(implementation)
                },
                |external| {
                    let builtin = external.fixed_args.is_empty()
                        && [Symbol::PI, Symbol::E].contains(&external.symbol);
                    if !builtin
                        && let Some(head) = heads.iter().find(|head| head.symbol == external.symbol)
                    {
                        let arguments = external.fixed_argument_values::<f64>(53).unwrap();
                        (head.evaluate)(&arguments)
                    } else {
                        // Fixed supplier pi/e registrations, never arbitrary
                        // user constant evaluators or tag generators.
                        external.evaluate_constant::<f64>(53).unwrap()
                    }
                },
            );
            let resident = extent(&product, |_| Ok(0))?;
            control
                .check(OperationExtent {
                    stage: OperationStage::Finalization,
                    work: 0,
                    input_bytes: source.input_bytes,
                    temporary_bytes: 0,
                    output_bytes: resident.input_bytes,
                    output_entries: product.stack.len(),
                })
                .map_err(OperationError::Control)?;
            Ok(product)
        })
    }

    /// Copy/map a builtin-only programme to binary64 with current admission.
    pub fn try_map_f64_with<C: OperationControl>(
        &self,
        control: &mut C,
    ) -> Result<ExpressionEvaluator<f64>, OperationError<C::Error>> {
        self.try_map_f64_known_with(&[], control)
    }
}

impl ExpressionEvaluator<Complex<Rational>> {
    /// Fallible precision mapping for the native real diagnostic profile. Arbitrary
    /// callbacks, tagged generators, branches and nested bodies are not admitted.
    pub fn try_map_coeff_with_prec<T: EvaluationDomain + Real, C: OperationControl>(
        &self,
        map: &impl Fn(&Complex<Rational>) -> Result<T, String>,
        precision: u32,
        control: &mut C,
    ) -> Result<ExpressionEvaluator<T>, OperationError<C::Error>> {
        if precision == 0 {
            return Err(OperationError::Unsupported("zero precision"));
        }
        let source = extent(self, control::payload)?;
        if self.settings.abort_check.is_some() {
            return Err(OperationError::Unsupported(
                "retained producer abort callback",
            ));
        }
        for external in &self.external_fns {
            let literal_constant = external.fixed_args.is_empty()
                && [Symbol::PI, Symbol::E].contains(&external.symbol);
            let fixed_builtin = external.fixed_args.len() == 1
                && [
                    Symbol::EXP,
                    Symbol::LOG,
                    Symbol::SIN,
                    Symbol::COS,
                    Symbol::SQRT,
                    Symbol::ABS,
                    Symbol::CONJ,
                ]
                .contains(&external.symbol)
                && external.fixed_args[0].im.is_zero();
            if external.imp.is_some()
                || external.body.is_some()
                || !external.stack.is_empty()
                || external.constant_index.is_none()
                || !external.tags.is_empty()
                || !(literal_constant || fixed_builtin)
            {
                return Err(OperationError::Unsupported(
                    "unqualified precision provider",
                ));
            }
        }
        // Phase metadata may be `Any` for an unconstrained symbolic formal. Real
        // inputs, real constants and the checked instruction profile establish this
        // realization's real arithmetic; that metadata is not a complex input proof.
        for (instruction, _) in &self.instructions {
            if matches!(
                instruction,
                Instr::ExternalFun(..) | Instr::IfElse(..) | Instr::Goto(..) | Instr::Join(..)
            ) {
                return Err(OperationError::Unsupported(
                    "nonreal or conditional diagnostic programme",
                ));
            }
        }
        if self.stack.iter().any(|value| !value.im.is_zero()) {
            return Err(OperationError::Unsupported(
                "nonreal diagnostic coefficient",
            ));
        }
        let limbs = (precision as usize)
            .checked_add(63)
            .ok_or(OperationError::Overflow)?
            / 64;
        let target = bytes::<C::Error, T>(self.stack.len())?
            .checked_add(
                self.stack
                    .len()
                    .checked_mul(limbs.checked_mul(32).ok_or(OperationError::Overflow)?)
                    .ok_or(OperationError::Overflow)?,
            )
            .ok_or(OperationError::Overflow)?;
        control
            .check(OperationExtent {
                stage: OperationStage::Construction,
                work: self.stack.len(),
                input_bytes: source.input_bytes,
                temporary_bytes: source
                    .input_bytes
                    .checked_mul(8)
                    .ok_or(OperationError::Overflow)?,
                output_bytes: target,
                output_entries: self.stack.len(),
            })
            .map_err(OperationError::Control)?;
        let mut stack = Vec::new();
        stack
            .try_reserve_exact(self.stack.len())
            .map_err(|_| OperationError::Allocation)?;
        for value in &self.stack {
            control
                .check(OperationExtent {
                    work: 1,
                    ..OperationExtent::default()
                })
                .map_err(OperationError::Control)?;
            stack.push(map(value).map_err(OperationError::InvalidProgram)?);
        }
        for external in &self.external_fns {
            let index = external
                .constant_index
                .ok_or(OperationError::Unsupported("constant slot"))?;
            if self.param_count + index >= self.reserved_indices {
                return Err(OperationError::Unsupported("invalid constant slot"));
            }
            stack[self.param_count + index] = if external.fixed_args.len() == 1 {
                let argument =
                    map(&external.fixed_args[0]).map_err(OperationError::InvalidProgram)?;
                match external.symbol.get_id() {
                    Symbol::EXP_ID => argument.exp(),
                    Symbol::LOG_ID => argument.log(),
                    Symbol::SIN_ID => argument.sin(),
                    Symbol::COS_ID => argument.cos(),
                    Symbol::SQRT_ID => argument.sqrt(),
                    Symbol::ABS_ID => argument.norm(),
                    Symbol::CONJ_ID => argument.conj(),
                    _ => {
                        return Err(OperationError::Unsupported(
                            "fixed precision constant profile",
                        ));
                    }
                }
            } else {
                external
                    .evaluate_constant::<T>(precision)
                    .map_err(OperationError::InvalidProgram)?
            };
        }
        let product = ExpressionEvaluator {
            stack,
            param_count: self.param_count,
            reserved_indices: self.reserved_indices,
            instructions: self.instructions.clone(),
            result_indices: self.result_indices.clone(),
            external_fns: vec![],
            settings: self.settings.clone(),
        };
        control
            .check(OperationExtent {
                stage: OperationStage::Finalization,
                work: 0,
                input_bytes: source.input_bytes,
                output_bytes: target,
                output_entries: product.stack.len(),
                ..OperationExtent::default()
            })
            .map_err(OperationError::Control)?;
        Ok(product)
    }

    /// Map only certifying real arithmetic to balls. Source eligibility remains a
    /// separate caller obligation; this validates the actual final native programme.
    pub fn try_map_real_ball_with<C: OperationControl>(
        &self,
        precision: u32,
        control: &mut C,
    ) -> Result<ExpressionEvaluator<crate::domains::float::RealBall>, OperationError<C::Error>>
    {
        use crate::domains::float::RealBall;
        if !self.external_fns.is_empty()
            || self.instructions.iter().any(|(instruction, _)| {
                !matches!(
                    instruction,
                    Instr::Add(..) | Instr::Mul(..) | Instr::Pow(..)
                )
            })
        {
            return Err(OperationError::Unsupported(
                "noncertifying real-ball instruction",
            ));
        }
        self.try_map_coeff_with_prec(
            &|value| {
                if !value.im.is_zero() {
                    return Err("nonreal ball coefficient".into());
                }
                Ok(RealBall::from_rational_bounds(
                    &value.re, &value.re, precision,
                ))
            },
            precision,
            control,
        )
    }
}
