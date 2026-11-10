// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Fixed scientific heads. Symbolica owns symbolic and Taylor chain composition.
use crate::MathError;
use std::sync::OnceLock;
use symbolica::atom::{
    Atom, EvaluationInfo, FunctionBuilder, NamespacedSymbol, Symbol, SymbolAttribute, SymbolBuilder,
};

/// Versioned, closed operation identity; callbacks never come from authored models.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Operation {
    Log1p,
    Log1pFirst,
    Log1pSecond,
    Magnitude,
    MagnitudeX,
    MagnitudeY,
    MagnitudeXX,
    MagnitudeXY,
    MagnitudeYY,
    MagnitudeDomain,
    LogRatio,
    LogRatioX,
    LogRatioReference,
    LogRatioXX,
    LogRatioXReference,
    LogRatioReferenceReference,
}
const OPERATIONS: [Operation; 16] = [
    Operation::Log1p,
    Operation::Log1pFirst,
    Operation::Log1pSecond,
    Operation::Magnitude,
    Operation::MagnitudeX,
    Operation::MagnitudeY,
    Operation::MagnitudeXX,
    Operation::MagnitudeXY,
    Operation::MagnitudeYY,
    Operation::MagnitudeDomain,
    Operation::LogRatio,
    Operation::LogRatioX,
    Operation::LogRatioReference,
    Operation::LogRatioXX,
    Operation::LogRatioXReference,
    Operation::LogRatioReferenceReference,
];
impl Operation {
    pub(crate) const fn identity(self) -> &'static str {
        match self {
            Self::Log1p => "pse_math::stable_v1_log1p",
            Self::Log1pFirst => "pse_math::stable_v1_log1p_d1",
            Self::Log1pSecond => "pse_math::stable_v1_log1p_d2",
            Self::Magnitude => "pse_math::stable_v1_magnitude",
            Self::MagnitudeX => "pse_math::stable_v1_magnitude_dx",
            Self::MagnitudeY => "pse_math::stable_v1_magnitude_dy",
            Self::MagnitudeXX => "pse_math::stable_v1_magnitude_dxx",
            Self::MagnitudeXY => "pse_math::stable_v1_magnitude_dxy",
            Self::MagnitudeYY => "pse_math::stable_v1_magnitude_dyy",
            Self::MagnitudeDomain => "pse_math::stable_v1_magnitude_domain",
            Self::LogRatio => "pse_math::stable_v1_log_ratio",
            Self::LogRatioX => "pse_math::stable_v1_log_ratio_dx",
            Self::LogRatioReference => "pse_math::stable_v1_log_ratio_dref",
            Self::LogRatioXX => "pse_math::stable_v1_log_ratio_dxx",
            Self::LogRatioXReference => "pse_math::stable_v1_log_ratio_dxref",
            Self::LogRatioReferenceReference => "pse_math::stable_v1_log_ratio_drefref",
        }
    }
    const fn export_name(self) -> &'static str {
        match self {
            Self::Log1p => "pse_math_stable_v1_log1p",
            Self::Log1pFirst => "pse_math_stable_v1_log1p_d1",
            Self::Log1pSecond => "pse_math_stable_v1_log1p_d2",
            Self::Magnitude => "pse_math_stable_v1_magnitude",
            Self::MagnitudeX => "pse_math_stable_v1_magnitude_dx",
            Self::MagnitudeY => "pse_math_stable_v1_magnitude_dy",
            Self::MagnitudeXX => "pse_math_stable_v1_magnitude_dxx",
            Self::MagnitudeXY => "pse_math_stable_v1_magnitude_dxy",
            Self::MagnitudeYY => "pse_math_stable_v1_magnitude_dyy",
            Self::MagnitudeDomain => "pse_math_stable_v1_magnitude_domain",
            Self::LogRatio => "pse_math_stable_v1_log_ratio",
            Self::LogRatioX => "pse_math_stable_v1_log_ratio_dx",
            Self::LogRatioReference => "pse_math_stable_v1_log_ratio_dref",
            Self::LogRatioXX => "pse_math_stable_v1_log_ratio_dxx",
            Self::LogRatioXReference => "pse_math_stable_v1_log_ratio_dxref",
            Self::LogRatioReferenceReference => "pse_math_stable_v1_log_ratio_drefref",
        }
    }
    fn evaluator(self) -> fn(&[f64]) -> f64 {
        match self {
            Self::Log1p => |args| Self::Log1p.evaluate(args),
            Self::Log1pFirst => |args| Self::Log1pFirst.evaluate(args),
            Self::Log1pSecond => |args| Self::Log1pSecond.evaluate(args),
            Self::Magnitude => |args| Self::Magnitude.evaluate(args),
            Self::MagnitudeX => |args| Self::MagnitudeX.evaluate(args),
            Self::MagnitudeY => |args| Self::MagnitudeY.evaluate(args),
            Self::MagnitudeXX => |args| Self::MagnitudeXX.evaluate(args),
            Self::MagnitudeXY => |args| Self::MagnitudeXY.evaluate(args),
            Self::MagnitudeYY => |args| Self::MagnitudeYY.evaluate(args),
            Self::MagnitudeDomain => |args| Self::MagnitudeDomain.evaluate(args),
            Self::LogRatio => |args| Self::LogRatio.evaluate(args),
            Self::LogRatioX => |args| Self::LogRatioX.evaluate(args),
            Self::LogRatioReference => |args| Self::LogRatioReference.evaluate(args),
            Self::LogRatioXX => |args| Self::LogRatioXX.evaluate(args),
            Self::LogRatioXReference => |args| Self::LogRatioXReference.evaluate(args),
            Self::LogRatioReferenceReference => {
                |args| Self::LogRatioReferenceReference.evaluate(args)
            }
        }
    }
    pub(crate) const fn arity(self) -> usize {
        match self {
            Self::Log1p | Self::Log1pFirst | Self::Log1pSecond => 1,
            _ => 2,
        }
    }
    pub(crate) const fn derivative_order(self) -> usize {
        match self {
            Self::Log1p | Self::Magnitude | Self::MagnitudeDomain | Self::LogRatio => 0,
            Self::Log1pFirst
            | Self::MagnitudeX
            | Self::MagnitudeY
            | Self::LogRatioX
            | Self::LogRatioReference => 1,
            _ => 2,
        }
    }
    fn partial(self, argument: usize) -> Option<Self> {
        match (self, argument) {
            (Self::Log1p, 0) => Some(Self::Log1pFirst),
            (Self::Log1pFirst, 0) => Some(Self::Log1pSecond),
            (Self::Magnitude, 0) => Some(Self::MagnitudeX),
            (Self::Magnitude, 1) => Some(Self::MagnitudeY),
            (Self::MagnitudeX, 0) => Some(Self::MagnitudeXX),
            (Self::MagnitudeX, 1) | (Self::MagnitudeY, 0) => Some(Self::MagnitudeXY),
            (Self::MagnitudeY, 1) => Some(Self::MagnitudeYY),
            (Self::LogRatio, 0) => Some(Self::LogRatioX),
            (Self::LogRatio, 1) => Some(Self::LogRatioReference),
            (Self::LogRatioX, 0) => Some(Self::LogRatioXX),
            (Self::LogRatioX, 1) | (Self::LogRatioReference, 0) => Some(Self::LogRatioXReference),
            (Self::LogRatioReference, 1) => Some(Self::LogRatioReferenceReference),
            _ => None,
        }
    }
    pub(crate) fn cpp_body(self) -> &'static str {
        macro_rules! scalar_body {
            (unary, $body:literal) => { concat!("if (!std::isfinite(x) || x<=-1.0) return std::nan(\"\"); ", $body) };
            (binary, $body:literal) => { concat!("if (!std::isfinite(x) || !std::isfinite(y)) return std::nan(\"\"); ", $body) };
            (partial, $body:literal) => { concat!("if (!std::isfinite(x) || !std::isfinite(y) || (x==0.0 && y==0.0)) return std::nan(\"\"); ", $body) };
            (ratio, $body:literal) => { concat!("if (!std::isfinite(x) || !std::isfinite(y) || x<=0.0 || y<=0.0) return std::nan(\"\"); ", $body) };
        }
        match self {
            Self::Log1p => scalar_body!(unary, "return std::log1p(x);"),
            Self::Log1pFirst => scalar_body!(unary, "return 1.0/(1.0+x);"),
            Self::Log1pSecond => scalar_body!(
                unary,
                "int e; double m=std::frexp(1.0+x,&e); return std::scalbn(-1.0/(m*m),-2*e);"
            ),

            Self::LogRatio => scalar_body!(
                ratio,
                "if (std::fmax(x,y)*0.5<=std::fmin(x,y)) return std::log1p((x-y)/y); double q=x/y; if (std::isfinite(q) && q>0.0) return std::log(q); return std::log(x)-std::log(y);"
            ),
            Self::LogRatioX => scalar_body!(ratio, "return 1.0/x;"),
            Self::LogRatioReference => scalar_body!(ratio, "return -1.0/y;"),
            Self::LogRatioXX => scalar_body!(
                ratio,
                "int e; double m=std::frexp(x,&e); return std::scalbn(-1.0/(m*m),-2*e);"
            ),
            Self::LogRatioXReference => scalar_body!(ratio, "return 0.0;"),
            Self::LogRatioReferenceReference => scalar_body!(
                ratio,
                "int e; double m=std::frexp(y,&e); return std::scalbn(1.0/(m*m),-2*e);"
            ),
            Self::Magnitude => scalar_body!(binary, "return std::hypot(x,y);"),
            Self::MagnitudeDomain => scalar_body!(binary, "return (x==0.0 && y==0.0) ? 0.0 : 1.0;"),
            Self::MagnitudeX => scalar_body!(
                partial,
                "double m=std::fmax(std::fabs(x),std::fabs(y)); double a=x/m,b=y/m; return a/std::sqrt(a*a+b*b);"
            ),
            Self::MagnitudeY => scalar_body!(
                partial,
                "double m=std::fmax(std::fabs(x),std::fabs(y)); double a=x/m,b=y/m; return b/std::sqrt(a*a+b*b);"
            ),
            Self::MagnitudeXX => scalar_body!(
                partial,
                "double m=std::fmax(std::fabs(x),std::fabs(y)); double a=x/m,b=y/m,r=std::sqrt(a*a+b*b); if(y==0.0) return 0.0; int ey,em; double my=std::frexp(y,&ey),mm=std::frexp(m,&em); return std::scalbn((my*my)/(mm*mm*mm*r*r*r),2*ey-3*em);"
            ),
            Self::MagnitudeXY => scalar_body!(
                partial,
                "double m=std::fmax(std::fabs(x),std::fabs(y)); double a=x/m,b=y/m,r=std::sqrt(a*a+b*b); if(x==0.0 || y==0.0) return 0.0; int ex,ey,em; double mx=std::frexp(x,&ex),my=std::frexp(y,&ey),mm=std::frexp(m,&em); return std::scalbn(-(mx*my)/(mm*mm*mm*r*r*r),ex+ey-3*em);"
            ),
            Self::MagnitudeYY => scalar_body!(
                partial,
                "double m=std::fmax(std::fabs(x),std::fabs(y)); double a=x/m,b=y/m,r=std::sqrt(a*a+b*b); if(x==0.0) return 0.0; int ex,em; double mx=std::frexp(x,&ex),mm=std::frexp(m,&em); return std::scalbn((mx*mx)/(mm*mm*mm*r*r*r),2*ex-3*em);"
            ),
        }
    }
    fn cpp(self, name: &str) -> String {
        let arguments = if self.arity() == 1 {
            "double x"
        } else {
            "double x, double y"
        };
        let body = self.cpp_body();
        format!("inline double {name}({arguments}) {{ {body} }}")
    }
    fn evaluate(self, args: &[f64]) -> f64 {
        if args.len() != self.arity() || args.iter().any(|v| !v.is_finite()) {
            return f64::NAN;
        }
        if self.arity() == 1 {
            let x = args[0];
            if x <= -1.0 {
                return f64::NAN;
            }
            return match self {
                Self::Log1p => x.ln_1p(),
                Self::Log1pFirst => 1.0 / (1.0 + x),
                Self::Log1pSecond => -ratio(&[1.0], &[1.0 + x, 1.0 + x]),
                _ => f64::NAN,
            };
        }
        let (x, y) = (args[0], args[1]);
        if matches!(
            self,
            Self::LogRatio
                | Self::LogRatioX
                | Self::LogRatioReference
                | Self::LogRatioXX
                | Self::LogRatioXReference
                | Self::LogRatioReferenceReference
        ) {
            if x <= 0.0 || y <= 0.0 {
                return f64::NAN;
            }
            return match self {
                Self::LogRatio => {
                    if x.max(y) * 0.5 <= x.min(y) {
                        ((x - y) / y).ln_1p()
                    } else {
                        let q = x / y;
                        if q.is_finite() && q > 0.0 {
                            q.ln()
                        } else {
                            x.ln() - y.ln()
                        }
                    }
                }
                Self::LogRatioX => 1.0 / x,
                Self::LogRatioReference => -1.0 / y,
                Self::LogRatioXX => -ratio(&[1.0], &[x, x]),
                Self::LogRatioXReference => 0.0,
                Self::LogRatioReferenceReference => ratio(&[1.0], &[y, y]),
                _ => f64::NAN,
            };
        }
        if self == Self::Magnitude {
            return x.hypot(y);
        }
        let m = x.abs().max(y.abs());
        if self == Self::MagnitudeDomain {
            return if m == 0.0 { 0.0 } else { 1.0 };
        }
        if m == 0.0 {
            return f64::NAN;
        }
        let a = x / m;
        let b = y / m;
        let r = (a * a + b * b).sqrt();
        match self {
            Self::MagnitudeX => a / r,
            Self::MagnitudeY => b / r,
            // Retain raw components until exponent decomposition. Squaring the
            // normalized component first can lose a representable final curvature.
            Self::MagnitudeXX => ratio(&[y, y], &[m, m, m, r, r, r]),
            Self::MagnitudeXY => -ratio(&[x, y], &[m, m, m, r, r, r]),
            Self::MagnitudeYY => ratio(&[x, x], &[m, m, m, r, r, r]),
            _ => f64::NAN,
        }
    }
}

// Exact binary decomposition, including subnormals. Only the bounded mantissas
// participate in intermediate multiplication; exponent scaling happens once.
fn decompose(value: f64) -> (f64, i32) {
    let bits = value.to_bits();
    let exponent = ((bits >> 52) & 0x7ff) as i32;
    if exponent == 0 {
        let (mantissa, exponent) = decompose(value * 18_014_398_509_481_984.0);
        return (mantissa, exponent - 54);
    }
    (
        f64::from_bits((bits & 0x800f_ffff_ffff_ffff) | (1023_u64 << 52)),
        exponent - 1023,
    )
}
fn scale(value: f64, exponent: i32) -> f64 {
    if value == 0.0 {
        return value;
    }
    let (mantissa, shift) = decompose(value);
    let exponent = exponent + shift;
    if exponent > 1023 {
        return f64::INFINITY.copysign(value);
    }
    if exponent < -1075 {
        return 0.0_f64.copysign(value);
    }
    if exponent < -1022 {
        return mantissa * 2.0_f64.powi(exponent + 1074) * f64::from_bits(1);
    }
    mantissa * 2.0_f64.powi(exponent)
}
fn ratio(numerators: &[f64], denominators: &[f64]) -> f64 {
    let mut mantissa = 1.0;
    let mut exponent = 0;
    for &value in numerators {
        if value == 0.0 {
            return value;
        }
        let (m, e) = decompose(value);
        mantissa *= m;
        exponent += e;
    }
    for &value in denominators {
        let (m, e) = decompose(value);
        mantissa /= m;
        exponent -= e;
    }
    scale(mantissa, exponent)
}

static SYMBOLS: OnceLock<Result<Vec<Symbol>, String>> = OnceLock::new();
pub(crate) fn initialize() -> Result<(), String> {
    SYMBOLS
        .get_or_init(|| {
            let builders = OPERATIONS
                .iter()
                .copied()
                .map(|operation| {
                    let name = NamespacedSymbol::try_from(operation.identity())?;
                    Ok(SymbolBuilder::new(name)
                        .with_attributes(Vec::<SymbolAttribute>::new())
                        .with_generator(move |symbols, builder| {
                            let symbols = symbols.to_vec();
                            let builder = builder.with_evaluation_info(
                                EvaluationInfo::new()
                                    .register(move |args: &[f64]| operation.evaluate(args))
                                    .with_cpp_generator(move |name, _tags| operation.cpp(name)),
                            );
                            if operation.derivative_order() == 2
                                || operation == Operation::MagnitudeDomain
                            {
                                return builder;
                            }
                            builder.with_derivative_function(move |view, argument, out| {
                                if let Some(partial) = operation.partial(argument) {
                                    if partial == Operation::LogRatioXReference {
                                        **out = Atom::num(0);
                                        return;
                                    }
                                    if let Some(index) =
                                        OPERATIONS.iter().position(|value| *value == partial)
                                    {
                                        if let symbolica::atom::AtomView::Fun(function) = view {
                                            let mut call = FunctionBuilder::new(symbols[index]);
                                            for arg in function {
                                                call = call.add_arg(arg);
                                            }
                                            **out = call.finish();
                                        }
                                    }
                                }
                            })
                        }))
                })
                .collect::<Result<Vec<_>, String>>()?;
            SymbolBuilder::build_group(builders).map_err(|error| error.to_string())
        })
        .as_ref()
        .map(|_| ())
        .map_err(Clone::clone)
}
static TRANSFER: OnceLock<[symbolica::evaluate::F64KnownHead; OPERATIONS.len()]> = OnceLock::new();
/// Fixed operation/profile descriptors; no callback is received from wire data.
pub(crate) fn transfer_heads() -> Result<&'static [symbolica::evaluate::F64KnownHead], MathError> {
    let symbols = SYMBOLS
        .get()
        .ok_or_else(|| MathError::Contract("stable heads require explicit initialization".into()))?
        .as_ref()
        .map_err(|error| MathError::Library(error.clone()))?;
    Ok(TRANSFER
        .get_or_init(|| {
            std::array::from_fn(|index| {
                let operation = OPERATIONS[index];
                symbolica::evaluate::F64KnownHead {
                    tag: (index + 1) as u32,
                    version: 1,
                    symbol: symbols[index],
                    arity: operation.arity(),
                    export_name: operation.export_name(),
                    evaluate: operation.evaluator(),
                }
            })
        })
        .as_slice())
}

static KNOWN: OnceLock<[symbolica::evaluate::KnownHead; OPERATIONS.len()]> = OnceLock::new();
/// Closed partial-symbol table consumed by checked supplier Taylor composition.
pub(crate) fn known_heads() -> Result<&'static [symbolica::evaluate::KnownHead], MathError> {
    let symbols = SYMBOLS
        .get()
        .ok_or_else(|| MathError::Contract("stable heads require explicit initialization".into()))?
        .as_ref()
        .map_err(|error| MathError::Library(error.clone()))?;
    let symbol_for = |operation: Operation| {
        OPERATIONS
            .iter()
            .position(|candidate| *candidate == operation)
            .map(|index| symbols[index])
    };
    Ok(KNOWN
        .get_or_init(|| {
            std::array::from_fn(|index| {
                let operation = OPERATIONS[index];
                let partial = |argument| operation.partial(argument);
                let second = |left, right| {
                    partial(left)
                        .and_then(|p| p.partial(right))
                        .and_then(symbol_for)
                };
                symbolica::evaluate::KnownHead {
                    symbol: symbols[index],
                    arity: operation.arity(),
                    first: [
                        partial(0).and_then(symbol_for),
                        partial(1).and_then(symbol_for),
                    ],
                    second: [second(0, 0), second(0, 1), second(1, 1)],
                }
            })
        })
        .as_slice())
}
#[cfg(feature = "compiled-evaluator")]
static CPP_HEADS: OnceLock<[symbolica::evaluate::CppKnownHead; OPERATIONS.len()]> = OnceLock::new();
/// Closed source descriptors; packed scientific kernels are qualified separately.
#[cfg(feature = "compiled-evaluator")]
pub(crate) fn cpp_heads() -> Result<&'static [symbolica::evaluate::CppKnownHead], MathError> {
    let symbols = SYMBOLS
        .get()
        .ok_or_else(|| MathError::Contract("stable heads require explicit initialization".into()))?
        .as_ref()
        .map_err(|error| MathError::Library(error.clone()))?;
    Ok(CPP_HEADS
        .get_or_init(|| {
            std::array::from_fn(|index| {
                let operation = OPERATIONS[index];
                symbolica::evaluate::CppKnownHead {
                    symbol: symbols[index],
                    arity: operation.arity(),
                    body: operation.cpp_body(),
                    packed_body: None,
                }
            })
        })
        .as_slice())
}
pub(crate) fn identify(symbol: Symbol) -> Option<Operation> {
    SYMBOLS
        .get()?
        .as_ref()
        .ok()?
        .iter()
        .position(|candidate| *candidate == symbol)
        .map(|index| OPERATIONS[index])
}
pub(crate) fn call(operation: Operation, arguments: &[&Atom]) -> Result<Atom, MathError> {
    if arguments.len() != operation.arity() {
        return Err(MathError::Contract("stable head arity".into()));
    }
    let symbols = SYMBOLS
        .get()
        .ok_or_else(|| MathError::Contract("stable heads require explicit initialization".into()))?
        .as_ref()
        .map_err(|error| MathError::Library(error.clone()))?;
    let index = OPERATIONS
        .iter()
        .position(|candidate| *candidate == operation)
        .ok_or_else(|| MathError::Contract("stable operation identity".into()))?;
    let mut call = FunctionBuilder::new(symbols[index]);
    for argument in arguments {
        call = call.add_arg(*argument);
    }
    Ok(call.finish())
}

#[cfg(test)]
mod tests {
    use super::*;
    use symbolica::atom::{AtomCore, Indeterminate};
    #[test]
    fn stable_checked_taylor_products_preserve_mixed_axes_and_second_factorials() {
        use crate::{
            guarded::{PreparedBody, Stage},
            jets::EvaluationLimits,
            library::Optimization,
        };
        use pse_kernels::DerivativeOrder;
        crate::initialize().unwrap();
        let x = crate::library::formal(0).unwrap();
        let y = crate::library::formal(1).unwrap();
        let xy = &x * &y;
        let expressions = vec![
            call(Operation::Log1p, &[&xy]).unwrap(),
            call(Operation::Magnitude, &[&x, &y]).unwrap(),
        ];
        let body = PreparedBody::new(
            2,
            4,
            vec![2, 3],
            vec![Stage::Block {
                expressions,
                outputs: vec![2, 3],
                source: pse_ids::SemanticId::NIL,
            }],
            DerivativeOrder::Second,
        )
        .unwrap();
        let cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let scope =
            pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::new(false), None);
        let compiled = body
            .compile(
                &[1, 0],
                &[1, 0],
                DerivativeOrder::Second,
                Optimization::default(),
                EvaluationLimits::default(),
                &scope,
            )
            .unwrap();
        let mut worker = compiled.worker();
        for order in [
            DerivativeOrder::Value,
            DerivativeOrder::First,
            DerivativeOrder::Second,
        ] {
            let r = worker
                .evaluate(
                    &[0.2, 0.3],
                    order,
                    &mut pse_kernels::ProviderBindings::new(),
                    &cancel,
                )
                .unwrap();
            let h = 0.2_f64.hypot(0.3);
            assert!((r.values[0] - h).abs() < 1e-16);
            assert!((r.values[1] - 0.06_f64.ln_1p()).abs() < 1e-16);
            if order >= DerivativeOrder::First {
                for (actual, expected) in
                    r.jacobian
                        .into_iter()
                        .zip([0.3 / h, 0.2 / h, 0.2 / 1.06, 0.3 / 1.06])
                {
                    assert!((actual - expected).abs() < 1e-15);
                }
            }
            if order >= DerivativeOrder::Second {
                let c = h * h * h;
                for (actual, expected) in r.hessians.into_iter().zip([
                    0.04 / c,
                    -0.06 / c,
                    -0.06 / c,
                    0.09 / c,
                    -0.04 / (1.06 * 1.06),
                    1.0 / (1.06 * 1.06),
                    1.0 / (1.06 * 1.06),
                    -0.09 / (1.06 * 1.06),
                ]) {
                    assert!((actual - expected).abs() < 3e-15, "{actual} != {expected}");
                }
            }
        }
    }
    #[test]
    fn stable_log_ratio_retains_positive_extremes_and_symbolica_taylor_partials() {
        use crate::{
            guarded::{PreparedBody, Stage},
            jets::EvaluationLimits,
            library::Optimization,
        };
        use pse_kernels::DerivativeOrder;
        let tiny = f64::from_bits(1);
        for (x, y) in [(f64::MAX, tiny), (tiny, f64::MAX), (f64::MAX, f64::MAX)] {
            let actual = Operation::LogRatio.evaluate(&[x, y]);
            let expected = x.ln() - y.ln();
            assert!(actual.is_finite());
            assert!((actual - expected).abs() < 3e-13);
        }
        // Squaring x first overflows, while the final curvature is representable.
        let curvature = Operation::LogRatioXX.evaluate(&[1e160, 1.0]);
        assert!(curvature < 0.0 && (curvature / -1e-320 - 1.0).abs() < 5e-4);
        for x in [0.0, -1.0, f64::INFINITY, f64::NAN] {
            assert!(Operation::LogRatio.evaluate(&[x, 1.0]).is_nan());
            assert!(Operation::LogRatio.evaluate(&[1.0, x]).is_nan());
        }
        crate::initialize().unwrap();
        let x = crate::library::formal(0).unwrap();
        let y = crate::library::formal(1).unwrap();
        let dx = Indeterminate::try_from(x.clone()).unwrap();
        let dy = Indeterminate::try_from(y.clone()).unwrap();
        let expression = call(Operation::LogRatio, &[&x, &y]).unwrap();
        assert_eq!(
            expression.derivative(&dx),
            call(Operation::LogRatioX, &[&x, &y]).unwrap()
        );
        assert_eq!(
            expression.derivative(&dy).derivative(&dy),
            call(Operation::LogRatioReferenceReference, &[&x, &y]).unwrap()
        );
        assert_eq!(expression.derivative(&dx).derivative(&dy), Atom::num(0));
        let body = PreparedBody::new(
            2,
            3,
            vec![2],
            vec![Stage::Block {
                expressions: vec![expression],
                outputs: vec![2],
                source: pse_ids::SemanticId::NIL,
            }],
            DerivativeOrder::Second,
        )
        .unwrap();
        let cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let scope =
            pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::new(false), None);
        // Reversed derivative axes exercise both closed first heads and the zero mixed head.
        let compiled = body
            .compile(
                &[0],
                &[1, 0],
                DerivativeOrder::Second,
                Optimization::default(),
                EvaluationLimits::default(),
                &scope,
            )
            .unwrap();
        let mut worker = compiled.worker();
        for order in [
            DerivativeOrder::Value,
            DerivativeOrder::First,
            DerivativeOrder::Second,
        ] {
            let jet = worker
                .evaluate(
                    &[3.0, 4.0],
                    order,
                    &mut pse_kernels::ProviderBindings::new(),
                    &cancel,
                )
                .unwrap();
            assert!((jet.values[0] - (0.75_f64).ln()).abs() < 1e-16);
            if order >= DerivativeOrder::First {
                assert_eq!(jet.jacobian, vec![-0.25, 1.0 / 3.0]);
            }
            if order >= DerivativeOrder::Second {
                assert_eq!(jet.hessians, vec![1.0 / 16.0, 0.0, 0.0, -1.0 / 9.0]);
            }
        }
    }
    #[test]
    fn stable_selected_taylor_channels_drop_unrequested_singular_local_partials() {
        use crate::{
            guarded::{PreparedBody, Stage},
            jets::EvaluationLimits,
            library::Optimization,
        };
        use pse_kernels::DerivativeOrder;
        crate::initialize().unwrap();
        let x = crate::library::formal(0).unwrap();
        let y = crate::library::formal(1).unwrap();
        let cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let scope =
            pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::new(false), None);
        for (operation, order, inputs, value, first, second) in [
            (
                Operation::Magnitude,
                DerivativeOrder::Second,
                [f64::from_bits(1), 0.0],
                f64::from_bits(1),
                1.0,
                0.0,
            ),
            (
                Operation::LogRatio,
                DerivativeOrder::First,
                [1.0, f64::from_bits(1)],
                744.440_071_921_381_2,
                1.0,
                0.0,
            ),
        ] {
            let body = PreparedBody::new(
                2,
                3,
                vec![2],
                vec![Stage::Block {
                    expressions: vec![call(operation, &[&x, &y]).unwrap()],
                    outputs: vec![2],
                    source: pse_ids::SemanticId::NIL,
                }],
                order,
            )
            .unwrap();
            let compiled = body
                .compile(
                    &[0],
                    &[0],
                    order,
                    Optimization::default(),
                    EvaluationLimits::default(),
                    &scope,
                )
                .unwrap();
            let jet = compiled
                .worker()
                .evaluate(
                    &inputs,
                    order,
                    &mut pse_kernels::ProviderBindings::new(),
                    &cancel,
                )
                .unwrap();
            assert_eq!(jet.values, [value]);
            assert_eq!(jet.jacobian, [first]);
            if order == DerivativeOrder::Second {
                assert_eq!(jet.hessians, [second]);
            }
        }
    }
    #[test]
    fn stable_registered_partials_retain_symbolica_chain_composition() {
        crate::initialize().unwrap();
        let x = crate::library::formal(0).unwrap();
        let y = crate::library::formal(1).unwrap();
        let dx = Indeterminate::try_from(x.clone()).unwrap();
        let dy = Indeterminate::try_from(y.clone()).unwrap();
        let log = call(Operation::Log1p, &[&x]).unwrap();
        assert_eq!(
            log.derivative(&dx),
            call(Operation::Log1pFirst, &[&x]).unwrap()
        );
        assert_eq!(
            log.derivative(&dx).derivative(&dx),
            call(Operation::Log1pSecond, &[&x]).unwrap()
        );
        let h = call(Operation::Magnitude, &[&x, &y]).unwrap();
        assert_eq!(
            h.derivative(&dx).derivative(&dy),
            call(Operation::MagnitudeXY, &[&x, &y]).unwrap()
        );
        let products = [
            log,
            h.derivative(&dx),
            h.derivative(&dy),
            h.derivative(&dx).derivative(&dy),
        ];
        let cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let mut scalars = crate::library::ScalarCache::default();
        let mut evaluator = crate::library::bounded_scalar_evaluator(
            &products,
            &[x, y],
            crate::library::Optimization::default(),
            &cancel,
            crate::jets::EvaluationLimits::default(),
            &mut scalars,
        )
        .unwrap();
        let mut result = [0.0; 4];
        evaluator
            .evaluator
            .try_evaluate(&[3.0, 4.0], &mut result)
            .unwrap();
        for (actual, expected) in result
            .into_iter()
            .zip([4.0_f64.ln(), 0.6, 0.8, -12.0 / 125.0])
        {
            assert!((actual - expected).abs() < 1e-15);
        }
        let symbols = SYMBOLS.get().unwrap().as_ref().unwrap();
        for (&symbol, operation) in symbols.iter().zip(OPERATIONS) {
            assert_eq!(identify(symbol), Some(operation));
            assert!(
                symbol
                    .get_evaluation_info()
                    .unwrap()
                    .generate_cpp("fixed", &[])
                    .unwrap()
                    .contains("inline double fixed")
            );
        }
    }
    #[test]
    fn stable_local_partials_preserve_tiny_increments_and_extreme_scales() {
        for x in [1e-30, -1e-30, 0.0, -0.0] {
            assert_eq!(Operation::Log1p.evaluate(&[x]).to_bits(), x.to_bits());
        }
        assert_eq!(Operation::Log1pFirst.evaluate(&[0.0]), 1.0);
        assert_eq!(Operation::Log1pSecond.evaluate(&[0.0]), -1.0);
        assert!(Operation::Log1pSecond.evaluate(&[1e160]) < 0.0);
        for x in [-1.0, -2.0, f64::INFINITY, f64::NAN] {
            assert!(Operation::Log1p.evaluate(&[x]).is_nan());
        }
        let d = Operation::MagnitudeX.evaluate(&[f64::MAX, f64::MAX]);
        assert!((d - std::f64::consts::FRAC_1_SQRT_2).abs() < 2e-16);
        assert!(
            Operation::Magnitude
                .evaluate(&[f64::MAX, f64::MAX])
                .is_infinite()
        );
        let xx = Operation::MagnitudeXX.evaluate(&[3.0, 4.0]);
        let xy = Operation::MagnitudeXY.evaluate(&[3.0, 4.0]);
        let yy = Operation::MagnitudeYY.evaluate(&[3.0, 4.0]);
        assert!((xx - 16.0 / 125.0).abs() < 1e-16);
        assert!((xy + 12.0 / 125.0).abs() < 1e-16);
        assert!((yy - 9.0 / 125.0).abs() < 1e-16);
        // (x/y)^2 is subnormal, but x^2/y^3 is an ordinary finite number.
        let curvature = Operation::MagnitudeYY.evaluate(&[1e-320, 1e-160]);
        // Decimal(100 digits), using exact Decimal.from_float input values.
        let expected = 9.999_777_344_893_057e-161;
        assert!(curvature > 0.0);
        assert!((curvature / expected - 1.0).abs() < 5e-16);
        assert_eq!(
            Operation::Magnitude.evaluate(&[f64::from_bits(3), f64::from_bits(4)]),
            f64::from_bits(5)
        );
        assert_eq!(Operation::Magnitude.evaluate(&[0.0, 0.0]), 0.0);
        assert!(Operation::MagnitudeX.evaluate(&[0.0, 0.0]).is_nan());
    }
}
