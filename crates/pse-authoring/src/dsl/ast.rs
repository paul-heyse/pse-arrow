// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Source-bearing syntax trees. These describe authored syntax, not relation authority.

/// An exact UTF-8 byte range in an expression document.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Span {
    /// Inclusive starting byte.
    pub start: u32,
    /// Exclusive ending byte.
    pub end: u32,
}

/// A numeric token and its optional authored unit expression.
#[derive(Clone, Debug)]
pub struct Number {
    /// Finite numeric value. A syntactic minus is represented by `ExprKind::Neg`.
    pub value: f64,
    /// The unit syntax, before unit resolution.
    pub unit: Option<String>,
}

impl PartialEq for Number {
    fn eq(&self, other: &Self) -> bool {
        self.value.to_bits() == other.value.to_bits() && self.unit == other.unit
    }
}

/// One dotted member and any subscripts attached to it.
#[derive(Clone, Debug, PartialEq)]
pub struct PathSegment {
    /// The authored identifier.
    pub name: String,
    /// Ordered index expressions.
    pub indices: Vec<Expr>,
}

/// A member path, retaining indices at their exact member position.
#[derive(Clone, Debug, PartialEq)]
pub struct Path {
    /// Dotted segments in source order.
    pub segments: Vec<PathSegment>,
}

/// An arithmetic binary operator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinaryOp {
    /// Ordered addition.
    Add,
    /// Ordered subtraction.
    Sub,
    /// Ordered multiplication.
    Mul,
    /// Ordered division.
    Div,
    /// Right-associative power.
    Pow,
}

impl BinaryOp {
    /// The authored token.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Add => "+",
            Self::Sub => "-",
            Self::Mul => "*",
            Self::Div => "/",
            Self::Pow => "^",
        }
    }
}

/// The bounded expression-language function vocabulary (blueprint §7.7).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Function {
    /// Exponential.
    Exp,
    /// Natural logarithm.
    Log,
    /// Base-ten logarithm.
    Log10,
    /// Square root.
    Sqrt,
    /// Absolute value.
    Abs,
    /// Smooth maximum.
    SmoothMax,
    /// Smooth minimum.
    SmoothMin,
    /// Smooth absolute value.
    SmoothAbs,
    /// Domain-safe square root.
    SafeSqrt,
    /// Domain-safe logarithm.
    SafeLog,
    /// Sine.
    Sin,
    /// Cosine.
    Cos,
    /// Tangent.
    Tan,
    /// Hyperbolic tangent.
    Tanh,
    /// Error function.
    Erf,
    /// Minimum; P3 refuses it until an opcode is admitted.
    Min,
    /// Maximum; P3 refuses it until an opcode is admitted.
    Max,
    /// Explicit unit conversion.
    Convert,
    /// Alternating ordered weight/value pairs.
    WeightedMean,
}

impl Function {
    /// All function spellings paired with their syntax variant.
    pub const SPELLINGS: [(&'static str, Self); 19] = [
        ("exp", Self::Exp),
        ("log", Self::Log),
        ("log10", Self::Log10),
        ("sqrt", Self::Sqrt),
        ("abs", Self::Abs),
        ("smooth_max", Self::SmoothMax),
        ("smooth_min", Self::SmoothMin),
        ("smooth_abs", Self::SmoothAbs),
        ("safe_sqrt", Self::SafeSqrt),
        ("safe_log", Self::SafeLog),
        ("sin", Self::Sin),
        ("cos", Self::Cos),
        ("tan", Self::Tan),
        ("tanh", Self::Tanh),
        ("erf", Self::Erf),
        ("min", Self::Min),
        ("max", Self::Max),
        ("convert", Self::Convert),
        ("weighted_mean", Self::WeightedMean),
    ];
    /// Resolve one exact authored spelling.
    pub fn parse(name: &str) -> Option<Self> {
        Self::SPELLINGS
            .iter()
            .find(|(candidate, _)| *candidate == name)
            .map(|(_, value)| *value)
    }
    /// The authored spelling.
    pub fn as_str(self) -> &'static str {
        Self::SPELLINGS
            .iter()
            .find(|(_, value)| *value == self)
            .map_or("", |(name, _)| *name)
    }
    /// Whether the grammar admits a final `eps=` argument.
    pub const fn has_epsilon(self) -> bool {
        matches!(
            self,
            Self::SmoothMax | Self::SmoothMin | Self::SmoothAbs | Self::SafeSqrt | Self::SafeLog
        )
    }
}

/// A final named function argument.
#[derive(Clone, Debug, PartialEq)]
pub struct NamedArg {
    /// Its declared name (`eps` in this language version).
    pub name: String,
    /// The supplied expression.
    pub value: Expr,
}

/// A domain reduction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReduceKind {
    /// Summation.
    Sum,
    /// Product.
    Prod,
    /// Continuous integral.
    Integral,
}

impl ReduceKind {
    /// The authored keyword.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Sum => "sum",
            Self::Prod => "prod",
            Self::Integral => "integral",
        }
    }
}

/// A lexical reduction binder.
#[derive(Clone, Debug, PartialEq)]
pub struct Binder {
    /// The local index name.
    pub var: String,
    /// The declared domain path.
    pub domain: Path,
    /// An optional membership restriction.
    pub filter: Option<Box<Predicate>>,
}

/// One source-bearing expression.
#[derive(Clone, Debug, PartialEq)]
pub struct Expr {
    /// The expression form.
    pub kind: ExprKind,
    /// The exact authored byte range.
    pub span: Span,
}

/// Authored expression forms (blueprint §7.7).
#[derive(Clone, Debug, PartialEq)]
pub enum ExprKind {
    /// A finite numeric token.
    Number(Number),
    /// A possibly indexed member reference.
    Path(Path),
    /// Unary negation.
    Neg(Box<Expr>),
    /// Ordered binary arithmetic.
    Binary {
        /// Arithmetic operator.
        op: BinaryOp,
        /// Left operand.
        lhs: Box<Expr>,
        /// Right operand.
        rhs: Box<Expr>,
    },
    /// A built-in authored function.
    Call {
        /// The closed function name.
        function: Function,
        /// Ordered positional arguments.
        args: Vec<Expr>,
        /// Final named arguments.
        named: Vec<NamedArg>,
    },
    /// An explicitly qualified kernel call.
    Kernel {
        /// Kernel path after `kernel.`.
        name: String,
        /// Ordered arguments.
        args: Vec<Expr>,
    },
    /// A lexical domain reduction.
    Reduce {
        /// Reduction kind.
        kind: ReduceKind,
        /// Bound domain and optional filter.
        binder: Box<Binder>,
        /// Reduced expression.
        body: Box<Expr>,
    },
    /// A derivative in a declared continuous domain.
    Derivative {
        /// Differentiated expression.
        body: Box<Expr>,
        /// Differentiation domain.
        wrt: Path,
    },
    /// A conditional expression.
    Conditional {
        /// Branch selector.
        guard: Box<Predicate>,
        /// Selected when the guard is true.
        then: Box<Expr>,
        /// Selected when the guard is false.
        otherwise: Box<Expr>,
    },
    /// The trailing `where` local bindings.
    Let {
        /// Named bindings in declaration order.
        bindings: Vec<(String, Expr)>,
        /// Expression using the bindings.
        body: Box<Expr>,
    },
}

/// A comparison token.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompareOp {
    /// Equality.
    Eq,
    /// Inequality.
    NotEq,
    /// Strict less-than.
    Lt,
    /// Less-than or equal.
    Le,
    /// Strict greater-than.
    Gt,
    /// Greater-than or equal.
    Ge,
}

impl CompareOp {
    /// The authored token.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Eq => "==",
            Self::NotEq => "!=",
            Self::Lt => "<",
            Self::Le => "<=",
            Self::Gt => ">",
            Self::Ge => ">=",
        }
    }
}

/// A source-bearing predicate.
#[derive(Clone, Debug, PartialEq)]
pub struct Predicate {
    /// Predicate syntax.
    pub kind: PredicateKind,
    /// The exact authored byte range.
    pub span: Span,
}

/// Predicate syntax, before name and domain resolution.
#[derive(Clone, Debug, PartialEq)]
pub enum PredicateKind {
    /// Scalar comparison.
    Compare {
        /// Comparison operator.
        op: CompareOp,
        /// Left operand.
        lhs: Box<Expr>,
        /// Right operand.
        rhs: Box<Expr>,
    },
    /// Domain membership.
    In {
        /// Tested expression.
        expr: Box<Expr>,
        /// Named domain.
        domain: Path,
    },
    /// Ordered conjunction.
    And(Box<Predicate>, Box<Predicate>),
    /// Ordered disjunction.
    Or(Box<Predicate>, Box<Predicate>),
    /// Logical negation.
    Not(Box<Predicate>),
    /// A feature or other boolean expression.
    Atom(Box<Expr>),
    /// A boolean literal.
    Bool(bool),
    /// An explicitly unknown/null predicate.
    Null,
}

/// The three admitted equation senses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EquationSense {
    /// Equality.
    Eq,
    /// Less-than or equal.
    Le,
    /// Greater-than or equal.
    Ge,
}

impl EquationSense {
    /// The authored token.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Eq => "==",
            Self::Le => "<=",
            Self::Ge => ">=",
        }
    }
}

/// A source-bearing equation, including the conditional equation in blueprint §11.3.
#[derive(Clone, Debug, PartialEq)]
pub struct Equation {
    /// Equation syntax.
    pub kind: EquationKind,
    /// Exact source range.
    pub span: Span,
}

/// Equation syntax preserves conditional branches with their individual senses.
#[derive(Clone, Debug, PartialEq)]
pub enum EquationKind {
    /// A scalar equation.
    Relation {
        /// Left side.
        lhs: Expr,
        /// Equation sense.
        sense: EquationSense,
        /// Right side.
        rhs: Expr,
    },
    /// A compile-time conditional equation.
    Conditional {
        /// Branch selector.
        guard: Predicate,
        /// True branch.
        then: Box<Equation>,
        /// False branch.
        otherwise: Box<Equation>,
    },
}
