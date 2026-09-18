// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Single operator contract table (blueprint §7.2–§7.3).
use super::Arity;
use pse_quantity::Opcode;

pse_quantity::closed_enum! {
    /// Operator families used by visitors; the table assigns each opcode once.
    pub enum OperatorFamily {
        /// Literal or symbol source.
        Leaf => "leaf",
        /// Ordered scalar arithmetic.
        Arithmetic => "arithmetic",
        /// Transcendental scalar function.
        Transcendental => "transcendental",
        /// Explicit smooth approximation.
        Smooth => "smooth",
        /// Guarded branch selection.
        Conditional => "conditional",
        /// Bound-domain reduction.
        Reduction => "reduction",
        /// Explicit index manipulation.
        Index => "index",
        /// Continuous-domain operation awaiting discretization.
        Calculus => "calculus",
        /// Declared external or implicit binding.
        Binding => "binding",
        /// Explicit physical conversion.
        Conversion => "conversion",
        /// Piecewise interpolation.
        Piecewise => "piecewise",
    }
}
pse_quantity::closed_enum! {
    /// Whether all operands or only the selected expression region are evaluated.
    pub enum ArgumentEvaluation {
        /// Ordered eager operands.
        Eager => "eager",
        /// Evaluate the guard and only the selected region.
        GuardedExpression => "guarded_expression",
    }
}
pse_quantity::closed_enum! {
    /// Observable evaluation failure classes of operator contracts.
    pub enum KernelFailure {
        /// An input is outside the admitted domain.
        DomainViolation => "domain_violation",
        /// An intermediate or output is not finite.
        NonFinite => "non_finite",
        /// A selected binding or policy is unsupported.
        Unsupported => "unsupported",
    }
}
pse_quantity::closed_enum! {
    /// Conditional smoothness guarantees; restrictions remain part of the contract.
    pub enum Differentiability {
        /// Smooth of all orders on the admitted domain.
        CInfinity => "c_infinity",
        /// Continuous but nonsmooth at declared breakpoints.
        C0 => "c0",
        /// First differentiable on the selected implicit branch.
        BranchwiseC1 => "branchwise_c1",
        /// Inherit child smoothness.
        AsChildren => "as_children",
        /// A branch switch can be discontinuous.
        Discontinuous => "discontinuous",
        /// Supplied by the binding or index model.
        Declared => "declared",
    }
}
pse_quantity::closed_enum! {
    /// Relation in a simple scalar domain restriction.
    pub enum RelationOp {
        /// Strictly greater than the bound.
        Gt => "gt",
        /// Greater than or equal to the bound.
        Ge => "ge",
        /// Less than or equal to the bound.
        Le => "le",
        /// Unequal to the bound.
        Ne => "ne",
    }
}
pse_quantity::closed_enum! {
    /// Declared lowering route; availability and policy still require P16 admission.
    pub enum BackendBinding {
        /// Native ordered evaluation.
        Native => "native",
        /// An NL expression, expanded where required.
        Nl => "nl",
        /// A Pyomo expression, expanded where required.
        Pyomo => "pyomo",
        /// A DataFusion expression or aggregate.
        DataFusion => "datafusion",
        /// Discretize through P11 before backend admission.
        Discretization => "discretization",
        /// A declared native kernel implementation.
        NativeKernel => "native_kernel",
        /// An explicitly bound NL external function.
        NlExternalFunction => "nl_external_function",
        /// A declared Pyomo external function or expansion.
        PyomoExternalFunction => "pyomo_external_function",
        /// A generated UDF under its kernel contract.
        DataFusionUdf => "datafusion_udf",
    }
}
/// A simple argument restriction; dependent/payload restrictions use the textual contract.
#[derive(Clone, Copy, Debug)]
pub struct DomainRestriction {
    /// Zero-based ordered argument.
    pub argument: u16,
    /// Relation to the bound.
    pub relation: RelationOp,
    /// Finite scalar bound.
    pub bound: f64,
}
/// One operator's contract, projected directly into `reference.operator_specs`.
#[derive(Clone, Copy, Debug)]
pub struct OperatorSpec {
    /// Operator identity.
    pub opcode: Opcode,
    /// Traversal family.
    pub family: OperatorFamily,
    /// Authoritative ordinary-child arity.
    pub arity: Arity,
    /// Complete quantity/index rule.
    pub shape_rule: &'static str,
    /// Differentiation contract.
    pub derivative_rule: &'static str,
    /// Ordered or guarded evaluation.
    pub argument_evaluation: ArgumentEvaluation,
    /// Observable runtime failures.
    pub failure_classes: &'static [KernelFailure],
    /// Simple scalar restrictions.
    pub domain_restrictions: &'static [DomainRestriction],
    /// Payload/dependent restrictions not representable as scalar bounds.
    pub domain_rule: &'static str,
    /// Smoothness on the admitted domain.
    pub smoothness: Differentiability,
    /// Convexity obligation; no unsupported inference is promised.
    pub convexity_rule: &'static str,
    /// Monotonicity obligation.
    pub monotonicity_rule: &'static str,
    /// Sparsity propagation.
    pub sparsity_rule: &'static str,
    /// Explicitly disabled or conditioned rewrites.
    pub rewrite_conditions: &'static [&'static str],
    /// Potential lowering routes, subject to backend admission.
    pub lowering: &'static [BackendBinding],
    /// Whether guarded literal folding may consider this operator.
    pub foldable: bool,
}
const ORDINARY: &[BackendBinding] = &[
    BackendBinding::Native,
    BackendBinding::Nl,
    BackendBinding::Pyomo,
    BackendBinding::DataFusion,
];
const CONDITIONAL: &[BackendBinding] = &[
    BackendBinding::Native,
    BackendBinding::Pyomo,
    BackendBinding::DataFusion,
];
const KERNEL: &[BackendBinding] = &[
    BackendBinding::NativeKernel,
    BackendBinding::NlExternalFunction,
    BackendBinding::PyomoExternalFunction,
    BackendBinding::DataFusionUdf,
];
const FINITE: &[KernelFailure] = &[KernelFailure::NonFinite];
const RESTRICTED: &[KernelFailure] = &[KernelFailure::DomainViolation, KernelFailure::NonFinite];
const NO_REASSOCIATION: &[&str] = &[
    "reassociation_disabled",
    "commutative_sorting_disabled",
    "fma_contraction_disabled",
    "evaluated_zero_terms_preserved",
];
const fn restriction(argument: u16, relation: RelationOp, bound: f64) -> DomainRestriction {
    DomainRestriction {
        argument,
        relation,
        bound,
    }
}
// The macro supplies only shared policy; arity and mathematical behavior are explicit rows.
macro_rules! spec {
    ($op:ident, $family:ident, $arity:expr, $shape:literal, $derivative:literal, $domain:literal, $restrictions:expr, $smooth:ident, $fold:expr, $lowering:expr) => {
        OperatorSpec {
            opcode: Opcode::$op,
            family: OperatorFamily::$family,
            arity: $arity,
            shape_rule: $shape,
            derivative_rule: $derivative,
            argument_evaluation: ArgumentEvaluation::Eager,
            failure_classes: if $domain.is_empty() {
                FINITE
            } else {
                RESTRICTED
            },
            domain_restrictions: $restrictions,
            domain_rule: $domain,
            smoothness: Differentiability::$smooth,
            convexity_rule: "derive only under the operator domain and child contracts",
            monotonicity_rule: "derive only under the operator domain and child contracts",
            sparsity_rule: "ordered union of referenced symbol dependencies",
            rewrite_conditions: NO_REASSOCIATION,
            lowering: $lowering,
            foldable: $fold,
        }
    };
}
const NONZERO_DIVISOR: &[DomainRestriction] = &[restriction(1, RelationOp::Ne, 0.0)];
const POSITIVE_ARGUMENT: &[DomainRestriction] = &[restriction(0, RelationOp::Gt, 0.0)];

/// All 43 operator declarations, in `Opcode::ALL` order.
pub static OPERATOR_TABLE: [OperatorSpec; 43] = [
    spec!(
        Const,
        Leaf,
        Arity::Payload,
        "explicit literal type",
        "zero",
        "",
        &[],
        CInfinity,
        false,
        ORDINARY
    ),
    spec!(
        SymbolRef,
        Leaf,
        Arity::Payload,
        "complete symbol type",
        "one with respect to the same symbol",
        "",
        &[],
        CInfinity,
        false,
        ORDINARY
    ),
    spec!(
        Add,
        Arithmetic,
        Arity::Fixed(2),
        "complete addition rule §8.3",
        "ordered sum",
        "quantity compatibility",
        &[],
        AsChildren,
        true,
        ORDINARY
    ),
    spec!(
        Sub,
        Arithmetic,
        Arity::Fixed(2),
        "complete subtraction rule §8.3",
        "ordered difference",
        "quantity compatibility",
        &[],
        AsChildren,
        true,
        ORDINARY
    ),
    spec!(
        Affine,
        Arithmetic,
        Arity::Variadic,
        "complete ordered affine rule §8.3",
        "chain rule through ordered children",
        "finite coefficients; no reassociation",
        &[],
        AsChildren,
        false,
        ORDINARY
    ),
    spec!(
        WeightedMean,
        Arithmetic,
        Arity::Payload,
        "identical complete value types; dimensionless weights",
        "ordered quotient and chain rule",
        "nonzero weight sum or established unit-sum invariant",
        &[],
        AsChildren,
        false,
        ORDINARY
    ),
    spec!(
        Mul,
        Arithmetic,
        Arity::Fixed(2),
        "registered product or neutral scaling §8.3",
        "ordered product rule",
        "",
        &[],
        CInfinity,
        true,
        ORDINARY
    ),
    OperatorSpec {
        rewrite_conditions: &[
            "x_div_x_requires_finite_nonzero_and_failure_equivalence",
            "reassociation_disabled",
            "commutative_sorting_disabled",
            "fma_contraction_disabled",
        ],
        ..spec!(
            Div,
            Arithmetic,
            Arity::Fixed(2),
            "registered quotient or neutral scaling §8.3",
            "ordered quotient rule",
            "nonzero divisor",
            NONZERO_DIVISOR,
            CInfinity,
            true,
            ORDINARY
        )
    },
    spec!(
        Pow,
        Arithmetic,
        Arity::Fixed(2),
        "registered rational power; variable exponent requires dimensionless base",
        "power rule",
        "fractional exponent requires positive base; negative exponent requires nonzero base",
        &[],
        CInfinity,
        false,
        ORDINARY
    ),
    spec!(
        Neg,
        Arithmetic,
        Arity::Fixed(1),
        "preserve complete type",
        "minus one",
        "",
        &[],
        CInfinity,
        true,
        ORDINARY
    ),
    spec!(
        Abs,
        Arithmetic,
        Arity::Fixed(1),
        "preserve complete type",
        "sign; nondifferentiable at zero",
        "",
        &[],
        C0,
        true,
        ORDINARY
    ),
    spec!(
        Exp,
        Transcendental,
        Arity::Fixed(1),
        "declared dimensionless kind and result §8.3",
        "exp(x)",
        "",
        &[],
        CInfinity,
        false,
        ORDINARY
    ),
    OperatorSpec {
        rewrite_conditions: &[
            "log_exp_elimination_disabled",
            "reassociation_disabled",
            "fma_contraction_disabled",
        ],
        ..spec!(
            Log,
            Transcendental,
            Arity::Fixed(1),
            "declared dimensionless kind and result §8.3",
            "1/x",
            "positive argument",
            POSITIVE_ARGUMENT,
            CInfinity,
            false,
            ORDINARY
        )
    },
    spec!(
        Log10,
        Transcendental,
        Arity::Fixed(1),
        "declared dimensionless kind and result §8.3",
        "1/(x*ln(10))",
        "positive argument",
        POSITIVE_ARGUMENT,
        CInfinity,
        false,
        ORDINARY
    ),
    spec!(
        Sqrt,
        Arithmetic,
        Arity::Fixed(1),
        "registered root §8.3",
        "1/(2*sqrt(x))",
        "nonnegative argument; nondifferentiable at zero",
        &[restriction(0, RelationOp::Ge, 0.0)],
        C0,
        true,
        ORDINARY
    ),
    spec!(
        Sin,
        Transcendental,
        Arity::Fixed(1),
        "declared dimensionless kind and result §8.3",
        "cos(x)",
        "",
        &[],
        CInfinity,
        false,
        ORDINARY
    ),
    spec!(
        Cos,
        Transcendental,
        Arity::Fixed(1),
        "declared dimensionless kind and result §8.3",
        "-sin(x)",
        "",
        &[],
        CInfinity,
        false,
        ORDINARY
    ),
    spec!(
        Tan,
        Transcendental,
        Arity::Fixed(1),
        "declared dimensionless kind and result §8.3",
        "1/cos(x)^2",
        "cos(x) nonzero",
        &[],
        CInfinity,
        false,
        ORDINARY
    ),
    spec!(
        Asin,
        Transcendental,
        Arity::Fixed(1),
        "declared dimensionless kind and result §8.3",
        "1/sqrt(1-x*x)",
        "argument within [-1,1]",
        &[
            restriction(0, RelationOp::Ge, -1.0),
            restriction(0, RelationOp::Le, 1.0)
        ],
        C0,
        false,
        ORDINARY
    ),
    spec!(
        Acos,
        Transcendental,
        Arity::Fixed(1),
        "declared dimensionless kind and result §8.3",
        "-1/sqrt(1-x*x)",
        "argument within [-1,1]",
        &[
            restriction(0, RelationOp::Ge, -1.0),
            restriction(0, RelationOp::Le, 1.0)
        ],
        C0,
        false,
        ORDINARY
    ),
    spec!(
        Atan,
        Transcendental,
        Arity::Fixed(1),
        "declared dimensionless kind and result §8.3",
        "1/(1+x*x)",
        "",
        &[],
        CInfinity,
        false,
        ORDINARY
    ),
    spec!(
        Sinh,
        Transcendental,
        Arity::Fixed(1),
        "declared dimensionless kind and result §8.3",
        "cosh(x)",
        "",
        &[],
        CInfinity,
        false,
        ORDINARY
    ),
    spec!(
        Cosh,
        Transcendental,
        Arity::Fixed(1),
        "declared dimensionless kind and result §8.3",
        "sinh(x)",
        "",
        &[],
        CInfinity,
        false,
        ORDINARY
    ),
    spec!(
        Tanh,
        Transcendental,
        Arity::Fixed(1),
        "declared dimensionless kind and result §8.3",
        "1-tanh(x)^2",
        "",
        &[],
        CInfinity,
        false,
        ORDINARY
    ),
    spec!(
        Erf,
        Transcendental,
        Arity::Fixed(1),
        "declared dimensionless kind and result §8.3",
        "2*exp(-x*x)/sqrt(pi)",
        "",
        &[],
        CInfinity,
        false,
        ORDINARY
    ),
    spec!(
        SmoothMax,
        Smooth,
        Arity::Fixed(2),
        "complete smooth-operator typing §8.3",
        "closed-form chain rule",
        "positive finite eps",
        &[],
        CInfinity,
        false,
        ORDINARY
    ),
    spec!(
        SmoothMin,
        Smooth,
        Arity::Fixed(2),
        "complete smooth-operator typing §8.3",
        "closed-form chain rule",
        "positive finite eps",
        &[],
        CInfinity,
        false,
        ORDINARY
    ),
    spec!(
        SmoothAbs,
        Smooth,
        Arity::Fixed(1),
        "complete smooth-operator typing §8.3",
        "closed-form chain rule",
        "positive finite eps",
        &[],
        CInfinity,
        false,
        ORDINARY
    ),
    spec!(
        SafeSqrt,
        Smooth,
        Arity::Fixed(1),
        "complete smooth-operator typing §8.3",
        "closed-form chain rule",
        "positive finite eps",
        &[],
        CInfinity,
        false,
        ORDINARY
    ),
    spec!(
        SafeLog,
        Smooth,
        Arity::Fixed(1),
        "complete smooth-operator typing §8.3",
        "closed-form chain rule",
        "positive finite eps",
        &[],
        CInfinity,
        false,
        ORDINARY
    ),
    OperatorSpec {
        argument_evaluation: ArgumentEvaluation::GuardedExpression,
        ..spec!(
            Conditional,
            Conditional,
            Arity::Fixed(2),
            "identical complete branch contracts; boolean guard",
            "selected branch only",
            "guard must be boolean; excluded branches remain unevaluated",
            &[],
            Discontinuous,
            false,
            CONDITIONAL
        )
    },
    spec!(
        SumOver,
        Reduction,
        Arity::Fixed(1),
        "remove bound index; additive or difference values",
        "ordered termwise sum",
        "no origin-sensitive point sum",
        &[],
        AsChildren,
        false,
        ORDINARY
    ),
    spec!(
        ProdOver,
        Reduction,
        Arity::Fixed(1),
        "remove bound index; registered finite product",
        "ordered product rule",
        "dimensionless body or declared finite-cardinality rule",
        &[],
        AsChildren,
        false,
        ORDINARY
    ),
    spec!(
        MinOver,
        Reduction,
        Arity::Fixed(1),
        "remove bound index; preserve value type",
        "selected minimum; ties nonsmooth",
        "nonempty admitted domain",
        &[],
        C0,
        false,
        ORDINARY
    ),
    spec!(
        MaxOver,
        Reduction,
        Arity::Fixed(1),
        "remove bound index; preserve value type",
        "selected maximum; ties nonsmooth",
        "nonempty admitted domain",
        &[],
        C0,
        false,
        ORDINARY
    ),
    spec!(
        Gather,
        Index,
        Arity::Payload,
        "group type at actual domain-member tuple",
        "one for selected member",
        "valid bound coordinate tuple",
        &[],
        Declared,
        false,
        ORDINARY
    ),
    spec!(
        Broadcast,
        Index,
        Arity::Fixed(1),
        "explicitly add bound index",
        "broadcast child derivative",
        "",
        &[],
        AsChildren,
        false,
        ORDINARY
    ),
    spec!(
        Derivative,
        Calculus,
        Arity::Fixed(1),
        "registered body/domain-unit power rule",
        "linear differential operator",
        "continuous domain and positive order",
        &[],
        Declared,
        false,
        &[BackendBinding::Discretization]
    ),
    spec!(
        Integral,
        Calculus,
        Arity::Fixed(1),
        "registered body times domain-unit rule",
        "linear integral operator",
        "continuous domain and declared quadrature",
        &[],
        Declared,
        false,
        &[BackendBinding::Discretization]
    ),
    spec!(
        KernelCall,
        Binding,
        Arity::Payload,
        "complete binding input/output contracts",
        "declared kernel derivative bindings",
        "declared kernel validity and supported binding",
        &[],
        Declared,
        false,
        KERNEL
    ),
    spec!(
        ImplicitRef,
        Binding,
        Arity::Payload,
        "declared unknown type",
        "-G_z inverse times G_u on selected branch",
        "nonsingular G_z on selected branch",
        &[],
        BranchwiseC1,
        false,
        &[BackendBinding::Native]
    ),
    spec!(
        UnitConvert,
        Conversion,
        Arity::Fixed(1),
        "exact declared point/difference conversion",
        "declared scale",
        "finite coefficients; offset respects point/difference contract",
        &[],
        CInfinity,
        true,
        ORDINARY
    ),
    spec!(
        PiecewiseLinear,
        Piecewise,
        Arity::Fixed(1),
        "declared input and output contracts",
        "segment slopes",
        "strictly increasing finite breakpoints",
        &[],
        C0,
        false,
        ORDINARY
    ),
];
/// The sole operator declaration for an opcode.
pub const fn operator_spec(opcode: Opcode) -> &'static OperatorSpec {
    &OPERATOR_TABLE[opcode as usize]
}

/// Payload tags admitted by an opcode across parsed and resolved expression families.
/// Family admission narrows unresolved alternatives at the native row boundary.
#[must_use]
pub const fn payload_kinds(opcode: Opcode) -> &'static [&'static str] {
    match opcode {
        Opcode::Const => &["float", "integer"],
        Opcode::SymbolRef => &["symbol", "pending_path"],
        Opcode::Affine => &["affine"],
        Opcode::WeightedMean => &["weighted_mean"],
        Opcode::SmoothMax
        | Opcode::SmoothMin
        | Opcode::SmoothAbs
        | Opcode::SafeSqrt
        | Opcode::SafeLog => &["smooth", "pending_smooth"],
        Opcode::Conditional => &["conditional"],
        Opcode::SumOver | Opcode::ProdOver | Opcode::MinOver | Opcode::MaxOver => &["reduction"],
        Opcode::Gather => &["gather", "pending_gather", "pending_path"],
        Opcode::Broadcast => &["broadcast"],
        Opcode::Derivative => &["derivative"],
        Opcode::Integral => &["integral"],
        Opcode::KernelCall => &["kernel_call"],
        Opcode::ImplicitRef => &["implicit_ref"],
        Opcode::UnitConvert => &["unit_convert", "pending_unit_convert"],
        Opcode::PiecewiseLinear => &["piecewise_linear"],
        Opcode::Add
        | Opcode::Sub
        | Opcode::Mul
        | Opcode::Div
        | Opcode::Pow
        | Opcode::Neg
        | Opcode::Abs
        | Opcode::Exp
        | Opcode::Log
        | Opcode::Log10
        | Opcode::Sqrt
        | Opcode::Sin
        | Opcode::Cos
        | Opcode::Tan
        | Opcode::Asin
        | Opcode::Acos
        | Opcode::Atan
        | Opcode::Sinh
        | Opcode::Cosh
        | Opcode::Tanh
        | Opcode::Erf => &["none"],
    }
}

/// A reduction opcode fixes the meaning of its domain payload.
#[must_use]
pub const fn reduction_kind(opcode: Opcode) -> Option<pse_quantity::ReductionKind> {
    match opcode {
        Opcode::SumOver => Some(pse_quantity::ReductionKind::Sum),
        Opcode::ProdOver => Some(pse_quantity::ReductionKind::Prod),
        Opcode::MinOver => Some(pse_quantity::ReductionKind::Min),
        Opcode::MaxOver => Some(pse_quantity::ReductionKind::Max),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn table_order_and_folding_policy_are_complete() {
        assert_eq!(
            OPERATOR_TABLE.iter().map(|x| x.opcode).collect::<Vec<_>>(),
            Opcode::ALL
        );
        assert_eq!(
            OPERATOR_TABLE
                .iter()
                .filter(|x| x.foldable)
                .map(|x| x.opcode)
                .collect::<Vec<_>>(),
            vec![
                Opcode::Add,
                Opcode::Sub,
                Opcode::Mul,
                Opcode::Div,
                Opcode::Neg,
                Opcode::Abs,
                Opcode::Sqrt,
                Opcode::UnitConvert
            ]
        );
        assert_eq!(
            operator_spec(Opcode::Conditional).argument_evaluation,
            ArgumentEvaluation::GuardedExpression
        );
        assert!(
            operator_spec(Opcode::Div)
                .rewrite_conditions
                .contains(&"x_div_x_requires_finite_nonzero_and_failure_equivalence")
        );
        assert!(
            operator_spec(Opcode::Log)
                .rewrite_conditions
                .contains(&"log_exp_elimination_disabled")
        );
    }
}
