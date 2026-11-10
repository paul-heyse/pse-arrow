//! Allocation-free counting and admitted streaming for the maintained C++ profile.
use super::*;
use std::fmt::Write;

/// Explicit numerical/ISA source profile. Compiler flags remain the caller's policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CppProfile {
    /// Binary64 scalar instructions, without inline assembly or contraction.
    ScalarF64,
    /// Four binary64 lanes using an explicitly selected AVX2 xsimd batch.
    X4F64Avx2,
}

/// Source-owned finite definition of a qualified mathematical head.
#[derive(Clone, Copy, Debug)]
pub struct CppKnownHead {
    /// Fixed registered head identity.
    pub symbol: Symbol,
    /// One or two ordinary arguments, named x and y in the fixed body.
    pub arity: usize,
    /// Complete scalar function body, with no caller-supplied snippets/generators.
    pub body: &'static str,
    /// Qualified packed function body. Absence refuses the packed profile.
    pub packed_body: Option<&'static str>,
}

/// A complete source product and the extent accepted before its allocation.
#[derive(Debug)]
pub struct CheckedCpp {
    /// Generated source, with no retained producer control.
    pub source: String,
    /// Admitted input/source/temporary overlap.
    pub extent: OperationExtent,
}

struct Count {
    bytes: usize,
    overflow: bool,
}
impl Write for Count {
    fn write_str(&mut self, text: &str) -> std::fmt::Result {
        match self.bytes.checked_add(text.len()) {
            Some(bytes) => {
                self.bytes = bytes;
                Ok(())
            }
            None => {
                self.overflow = true;
                Err(std::fmt::Error)
            }
        }
    }
}
struct BoundedSource {
    source: String,
    limit: usize,
}
impl Write for BoundedSource {
    fn write_str(&mut self, text: &str) -> std::fmt::Result {
        if self
            .source
            .len()
            .checked_add(text.len())
            .is_none_or(|bytes| bytes > self.limit)
        {
            return Err(std::fmt::Error);
        }
        self.source.push_str(text);
        Ok(())
    }
}
struct Name<'a> {
    base: &'a str,
    profile: CppProfile,
}
impl std::fmt::Display for Name<'_> {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            out,
            "{}_{}",
            self.base,
            match self.profile {
                CppProfile::ScalarF64 => "realf64",
                CppProfile::X4F64Avx2 => "simd_realf64",
            }
        )
    }
}
fn identifier(name: &str) -> bool {
    !name.is_empty()
        && name.bytes().enumerate().all(|(i, byte)| {
            byte.is_ascii_alphabetic() || byte == b'_' || (i > 0 && byte.is_ascii_digit())
        })
}
fn head<'a, E>(
    heads: &'a [CppKnownHead],
    symbol: Symbol,
) -> Result<&'a CppKnownHead, OperationError<E>> {
    let mut matches = heads.iter().filter(|head| head.symbol == symbol);
    let head = matches
        .next()
        .ok_or(OperationError::Unsupported("unqualified C++ head"))?;
    if matches.next().is_some() || !(1..=2).contains(&head.arity) {
        return Err(OperationError::Unsupported("ambiguous C++ head or arity"));
    }
    Ok(head)
}

impl ExpressionEvaluator<f64> {
    /// Count without allocating, admit exact source backing/overlap, then stream
    /// the same emitter into an exactly reserved destination. No temporary operand
    /// strings, arbitrary source generators or hidden external bodies are accepted.
    pub fn try_export_cpp_with<C: OperationControl>(
        &self,
        function_name: &str,
        settings: ExportSettings,
        profile: CppProfile,
        heads: &[CppKnownHead],
        control: &mut C,
    ) -> Result<CheckedCpp, OperationError<C::Error>> {
        control
            .check(OperationExtent {
                stage: OperationStage::Export,
                ..Default::default()
            })
            .map_err(OperationError::Control)?;
        self.qualify_cpp(function_name, &settings, profile, heads)?;
        let input_bytes = self.cpp_input_bytes()?;
        let mut extent = OperationExtent {
            stage: OperationStage::Export,
            input_bytes,
            temporary_bytes: 4096,
            ..Default::default()
        };
        let name = Name {
            base: function_name,
            profile,
        };
        let mut count = Count {
            bytes: 0,
            overflow: false,
        };
        let mut error = None;
        let counted = self.write_checked_cpp(&name, &settings, heads, &mut count, &mut || {
            control.check(extent).map_err(|refusal| {
                error = Some(refusal);
                std::fmt::Error
            })
        });
        if let Some(error) = error {
            return Err(OperationError::Control(error));
        }
        if count.overflow {
            return Err(OperationError::Overflow);
        }
        counted.map_err(|_| OperationError::Interrupted)?;
        extent.output_bytes = count.bytes;
        extent.work = self
            .instructions
            .len()
            .checked_add(self.result_indices.len())
            .and_then(|n| n.checked_add(self.external_fns.len()))
            .ok_or(OperationError::Overflow)?;
        extent.peak_bytes().ok_or(OperationError::Overflow)?;
        control.check(extent).map_err(OperationError::Control)?;
        let mut source = String::new();
        source
            .try_reserve_exact(count.bytes)
            .map_err(|_| OperationError::Allocation)?;
        let mut source = BoundedSource {
            source,
            limit: count.bytes,
        };
        let safe_extent = OperationExtent { work: 0, ..extent };
        let emitted = self.write_checked_cpp(&name, &settings, heads, &mut source, &mut || {
            control.check(safe_extent).map_err(|refusal| {
                error = Some(refusal);
                std::fmt::Error
            })
        });
        if let Some(error) = error {
            return Err(OperationError::Control(error));
        }
        emitted
            .map_err(|_| OperationError::InvalidProgram("C++ count/emission mismatch".into()))?;
        if source.source.len() != count.bytes {
            return Err(OperationError::InvalidProgram(
                "C++ count/emission mismatch".into(),
            ));
        }
        control
            .check(OperationExtent {
                stage: OperationStage::Finalization,
                ..safe_extent
            })
            .map_err(OperationError::Control)?;
        Ok(CheckedCpp {
            source: source.source,
            extent,
        })
    }

    fn qualify_cpp<E>(
        &self,
        name: &str,
        settings: &ExportSettings,
        profile: CppProfile,
        heads: &[CppKnownHead],
    ) -> Result<(), OperationError<E>> {
        if !identifier(name)
            || settings.inline_asm != InlineASM::None
            || settings.custom_header.is_some()
        {
            return Err(OperationError::Unsupported(
                "C++ name, assembly or custom header",
            ));
        }
        if self.param_count > self.reserved_indices
            || self.reserved_indices > self.stack.len()
            || self.stack[self.param_count..self.reserved_indices]
                .iter()
                .any(|value| !value.is_finite())
        {
            return Err(OperationError::Unsupported(
                "nonfinite constants or invalid stack",
            ));
        }
        for external in &self.external_fns {
            let descriptor = head::<E>(heads, external.symbol)?;
            if external.body.is_some()
                || !external.tags.is_empty()
                || !identifier(external.export_name())
                || (profile == CppProfile::X4F64Avx2 && descriptor.packed_body.is_none())
            {
                return Err(OperationError::Unsupported("C++ body, tags or packed head"));
            }
            if external.constant_index.is_some() && external.fixed_args.len() != descriptor.arity {
                return Err(OperationError::Unsupported("C++ constant head arity"));
            }
        }
        for (instruction, _) in &self.instructions {
            match instruction {
                Instr::IfElse(..) | Instr::Goto(..) | Instr::Label(..) | Instr::Join(..)
                    if profile == CppProfile::X4F64Avx2 =>
                {
                    return Err(OperationError::Unsupported("packed control flow"));
                }
                Instr::Powf(..) if profile == CppProfile::X4F64Avx2 => {
                    return Err(OperationError::Unsupported("packed noninteger power"));
                }
                Instr::Pow(_, _, exponent) if exponent.unsigned_abs() > i32::MAX as u64 => {
                    return Err(OperationError::Unsupported(
                        "integer power exceeds Numerica f64 range",
                    ));
                }
                Instr::ExternalFun(_, index, args) => {
                    let external = self
                        .external_fns
                        .get(*index)
                        .ok_or(OperationError::Unsupported("C++ external index"))?;
                    if args.len() != head::<E>(heads, external.symbol)?.arity {
                        return Err(OperationError::Unsupported("C++ head arity"));
                    }
                }
                Instr::BuiltinFun(_, symbol, _)
                    if ![
                        Symbol::EXP_ID,
                        Symbol::LOG_ID,
                        Symbol::SIN_ID,
                        Symbol::COS_ID,
                        Symbol::SQRT_ID,
                        Symbol::ABS_ID,
                        Symbol::CONJ_ID,
                    ]
                    .contains(&symbol.get_id()) =>
                {
                    return Err(OperationError::Unsupported("C++ builtin"));
                }
                Instr::Add(_, args) | Instr::Mul(_, args) if args.is_empty() => {
                    return Err(OperationError::Unsupported("C++ empty arithmetic"));
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn cpp_input_bytes<E>(&self) -> Result<usize, OperationError<E>> {
        let mut bytes = self
            .stack
            .capacity()
            .checked_mul(size_of::<f64>())
            .and_then(|n| {
                n.checked_add(
                    self.instructions
                        .capacity()
                        .checked_mul(size_of::<(Instr, ComplexPhase)>())?,
                )
            })
            .and_then(|n| {
                n.checked_add(
                    self.result_indices
                        .capacity()
                        .checked_mul(size_of::<usize>())?,
                )
            })
            .and_then(|n| {
                n.checked_add(
                    self.external_fns
                        .capacity()
                        .checked_mul(size_of::<ExternalFunctionContainer<f64>>())?,
                )
            })
            .ok_or(OperationError::Overflow)?;
        for (instruction, _) in &self.instructions {
            if let Instr::Add(_, args) | Instr::Mul(_, args) | Instr::ExternalFun(_, _, args) =
                instruction
            {
                bytes = bytes
                    .checked_add(
                        args.capacity()
                            .checked_mul(size_of::<usize>())
                            .ok_or(OperationError::Overflow)?,
                    )
                    .ok_or(OperationError::Overflow)?;
            }
        }
        for external in &self.external_fns {
            bytes = bytes
                .checked_add(external.export_name.capacity())
                .and_then(|n| {
                    n.checked_add(external.stack.capacity().checked_mul(size_of::<f64>())?)
                })
                .and_then(|n| {
                    n.checked_add(
                        external
                            .fixed_args
                            .capacity()
                            .checked_mul(size_of::<Complex<Rational>>())?,
                    )
                })
                .ok_or(OperationError::Overflow)?;
            for value in &external.fixed_args {
                bytes = bytes
                    .checked_add(super::super::control::payload::<E>(value)?)
                    .ok_or(OperationError::Overflow)?;
            }
        }
        Ok(bytes)
    }

    fn write_checked_cpp(
        &self,
        name: &Name<'_>,
        settings: &ExportSettings,
        heads: &[CppKnownHead],
        out: &mut dyn Write,
        safe_point: &mut impl FnMut() -> std::fmt::Result,
    ) -> std::fmt::Result {
        safe_point()?;
        if settings.include_header {
            out.write_str("#include <cmath>\n#include <cstddef>\n")?;
            out.write_str("template<typename T> T conj(T value) { return value; }\n")?;
        }
        if name.profile == CppProfile::X4F64Avx2 {
            out.write_str("#include \"xsimd/xsimd.hpp\"\n#include <limits>\nusing simd = xsimd::batch<double, xsimd::avx2>;\nstatic_assert(simd::size == 4);\n")?;
        }
        writeln!(out, "namespace {name}_heads {{")?;
        if name.profile == CppProfile::X4F64Avx2 {
            // xsimd 14.3.0's direct logarithm does not normalize binary64
            // subnormals. Keep inactive normal lanes out of the scaling
            // multiplication and recover the exponent with split ln(2).
            out.write_str("inline simd symbolica_packed_log(simd x) { auto sub=(x>simd(0.0)) && (x<simd(std::numeric_limits<double>::min())); simd scaled=xsimd::select(sub,x,simd(0.0))*simd(18014398509481984.0); simd normalized=xsimd::select(sub,scaled,x); simd correction=xsimd::select(sub,simd(54.0),simd(0.0)); return (xsimd::log(normalized)-correction*xsimd::constants::log_2hi<simd>())-correction*xsimd::constants::log_2lo<simd>(); }\ninline simd log(simd x) { return symbolica_packed_log(x); }\n")?;
        }
        out.write_str("template<typename T> T pse_integer_power(T base, unsigned int exponent) { T result=T(1); while(exponent) { if(exponent&1u) result=result*base; exponent>>=1; if(exponent) base=base*base; } return result; }\n")?;
        for (index, external) in self.external_fns.iter().enumerate() {
            safe_point()?;
            if external.constant_index.is_some()
                || self.external_fns[..index].iter().any(|previous| {
                    previous.constant_index.is_none() && previous.symbol == external.symbol
                })
            {
                continue;
            }
            let descriptor = heads
                .iter()
                .find(|head| head.symbol == external.symbol)
                .unwrap();
            write!(out, "inline double {}(double x", external.export_name())?;
            if descriptor.arity == 2 {
                out.write_str(", double y")?;
            }
            writeln!(out, ") {{ {} }}", descriptor.body)?;
            if name.profile == CppProfile::X4F64Avx2 {
                write!(out, "inline simd {}(simd x", external.export_name())?;
                if descriptor.arity == 2 {
                    out.write_str(", simd y")?;
                }
                writeln!(out, ") {{ {} }}", descriptor.packed_body.unwrap())?;
            }
        }
        out.write_str("}\n")?;
        writeln!(
            out,
            "extern \"C\" unsigned long {name}_get_buffer_len() {{ return {}; }}",
            self.stack.len()
        )?;
        writeln!(
            out,
            "extern \"C\" std::size_t {name}_get_input_len() {{ return {}; }}",
            self.param_count
        )?;
        writeln!(
            out,
            "extern \"C\" std::size_t {name}_get_output_len() {{ return {}; }}",
            self.result_indices.len()
        )?;
        writeln!(
            out,
            "template<typename T> void {name}_gen(T* params, T* Z, T* out) {{\nusing std::abs;\nusing namespace {name}_heads;"
        )?;
        self.program()
            .write_cpp_impl(&self.external_fns, "", "T", true, true, out, safe_point)?;
        for (index, slot) in self.result_indices.iter().enumerate() {
            safe_point()?;
            write!(out, "out[{index}] = ")?;
            self.program().write_cpp_input(*slot, "", "T", true, out)?;
            out.write_str(";\n")?;
        }
        out.write_str("}\n")?;
        let number = if name.profile == CppProfile::ScalarF64 {
            "double"
        } else {
            "simd"
        };
        writeln!(
            out,
            "extern \"C\" void {name}({number}* params, {number}* Z, {number}* out) {{ {name}_gen(params, Z, out); }}"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn checked_cpp_counter_overflow_has_no_partial_extent() {
        let mut count = Count {
            bytes: usize::MAX,
            overflow: false,
        };
        assert!(count.write_str("x").is_err());
        assert!(count.overflow);
        assert_eq!(count.bytes, usize::MAX);
    }
    #[test]
    fn checked_cpp_packed_control_flow_and_oversized_integer_power_refuse() {
        let mut evaluator = ExpressionEvaluator {
            stack: vec![0.; 2],
            param_count: 1,
            reserved_indices: 1,
            instructions: vec![(Instr::IfElse(0, Label(0)), ComplexPhase::Any)],
            result_indices: vec![0],
            external_fns: vec![],
            settings: OptimizationSettings::default(),
        };
        let settings = || ExportSettings::new().inline_asm(InlineASM::None);
        assert!(matches!(
            evaluator.try_export_cpp_with(
                "checked",
                settings(),
                CppProfile::X4F64Avx2,
                &[],
                &mut |_| Ok::<_, &'static str>(())
            ),
            Err(OperationError::Unsupported("packed control flow"))
        ));
        evaluator.instructions = vec![(Instr::Powf(1, 0, 0), ComplexPhase::Any)];
        assert!(matches!(
            evaluator.try_export_cpp_with(
                "checked",
                settings(),
                CppProfile::X4F64Avx2,
                &[],
                &mut |_| Ok::<_, &'static str>(())
            ),
            Err(OperationError::Unsupported("packed noninteger power"))
        ));
        evaluator.instructions = vec![(Instr::Pow(1, 0, i64::MAX), ComplexPhase::Any)];
        assert!(matches!(
            evaluator.try_export_cpp_with(
                "checked",
                settings(),
                CppProfile::ScalarF64,
                &[],
                &mut |_| Ok::<_, &'static str>(())
            ),
            Err(OperationError::Unsupported(
                "integer power exceeds Numerica f64 range"
            ))
        ));
    }
}
