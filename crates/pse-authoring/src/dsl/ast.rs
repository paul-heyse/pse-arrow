// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Source-bearing syntax trees. These describe authored syntax, not relation authority.

/// An exact UTF-8 byte range in an expression document.
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Span {
    /// Inclusive starting byte.
    pub start: u32,
    /// Exclusive ending byte.
    pub end: u32,
}

/// A numeric token and its optional authored unit.
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Number {
    /// Finite numeric value. A syntactic minus is represented by `ExprKind::Neg`.
    #[serde(with = "finite_float_bits")]
    pub value: f64,
    /// Exact integral token outside the consecutive IEEE-754 integer range.
    pub exact_integer: Option<i128>,
    /// The unit as a canonical product of unit symbols with rational exponents, before
    /// the physical registry composes it (ADR-0124). Spellings of one product are equal.
    pub unit: Option<pse_quantity::UnitProduct>,
}

// Scientific AST literals use exact IEEE-754 bits in portable descriptions.
// Diagnostic nonfinite values belong to the separate raw diagnostic codec.
mod finite_float_bits {
    use serde::{Deserialize, Deserializer, Serializer};
    pub(super) fn serialize<S: Serializer>(value: &f64, serializer: S) -> Result<S::Ok, S::Error> {
        if !value.is_finite() {
            return Err(serde::ser::Error::custom(
                "scientific literal must be finite",
            ));
        }
        serializer.serialize_u64(value.to_bits())
    }
    pub(super) fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<f64, D::Error> {
        let value = f64::from_bits(u64::deserialize(deserializer)?);
        if !value.is_finite() {
            return Err(serde::de::Error::custom(
                "scientific literal must be finite",
            ));
        }
        Ok(value)
    }
}

impl PartialEq for Number {
    fn eq(&self, other: &Self) -> bool {
        self.value.to_bits() == other.value.to_bits()
            && self.unit == other.unit
            && self.integer() == other.integer()
    }
}

impl Number {
    /// Exact authored integer, or a losslessly represented small integral literal.
    pub fn integer(&self) -> Option<i128> {
        self.exact_integer.or_else(|| {
            (self.value.fract() == 0.0 && self.value.abs() <= 9_007_199_254_740_992.0)
                .then_some(self.value as i128)
        })
    }
}
impl std::fmt::Debug for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut d = f.debug_struct("Number");
        d.field("value", &self.value);
        if let Some(exact) = self.exact_integer {
            d.field("exact_integer", &exact);
        }
        // The canonical spelling reads more plainly than the factor list.
        d.field("unit", &self.unit.as_ref().map(ToString::to_string))
            .finish()
    }
}

/// One dotted member and any subscripts attached to it.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct PathSegment {
    /// The authored identifier.
    pub name: String,
    /// Ordered index expressions.
    pub indices: Vec<Expr>,
}

/// A member path, retaining indices at their exact member position.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct Path {
    /// Dotted segments in source order.
    pub segments: Vec<PathSegment>,
}

impl Path {
    /// A generated or parser-owned scalar identifier; dots remain part of that identifier.
    pub fn single(name: impl Into<String>) -> Self {
        Self {
            segments: vec![PathSegment {
                name: name.into(),
                indices: Vec::new(),
            }],
        }
    }
    /// The unindexed scalar identifier, if this path is exactly one segment.
    pub fn ident(&self) -> Option<&str> {
        match self.segments.as_slice() {
            [segment] if segment.indices.is_empty() => Some(&segment.name),
            _ => None,
        }
    }
    /// Whether this path names exactly the given scalar identifier.
    pub fn is_ident(&self, name: &str) -> bool {
        self.ident() == Some(name)
    }
}
impl std::fmt::Display for Path {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&super::render_path(self))
    }
}

/// An arithmetic binary operator.
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
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

// The same declaration drives parsing and typed library admission.
pub use pse_quantity::functions::Function;

/// A domain reduction.
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
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
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct Binder {
    /// The local index name.
    pub var: String,
    /// The declared domain path.
    pub domain: Path,
    /// An optional membership restriction.
    pub filter: Option<Box<Predicate>>,
}

/// One source-bearing expression.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct Expr {
    /// The expression form.
    pub kind: ExprKind,
    /// The exact authored byte range.
    pub span: Span,
}

/// Authored expression forms (blueprint §7.7).
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
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
    },
    /// A package-defined function, resolved before mathematical admission.
    NamedCall {
        /// Lexically resolved function path.
        name: Path,
        /// Positional arguments.
        args: Vec<Expr>,
    },
    /// Partial derivatives of an explicit function argument, applied to arguments.
    Partial {
        /// Function path.
        function: Path,
        /// Ordered differentiated formal arguments (repetition denotes higher order).
        wrt: Vec<Path>,
        /// Applied arguments.
        args: Vec<Expr>,
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
    /// A nonempty finite left fold in the admitted set's semantic order.
    Fold {
        /// Lexical accumulator name, visible only in the step.
        accumulator: String,
        /// Lexical current-value name, visible only in the step.
        item: String,
        /// Bound domain and optional membership filter.
        binder: Box<Binder>,
        /// Value at each selected coordinate; the first seeds the accumulator.
        value: Box<Expr>,
        /// Type-preserving combination of accumulator and current value.
        step: Box<Expr>,
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
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
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
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct Predicate {
    /// Predicate syntax.
    pub kind: PredicateKind,
    /// The exact authored byte range.
    pub span: Span,
}

/// Predicate syntax, before name and domain resolution.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
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

/// Logic syntax sharing the expression grammar and source-bearing atom/count nodes.
#[derive(Clone, Debug, PartialEq)]
pub enum Proposition {
    /// A binary-variable reference admitted by the path grammar.
    Atom(Expr),
    /// Logical complement.
    Not(Box<Proposition>),
    /// Ordered conjunction.
    And(Vec<Proposition>),
    /// Ordered disjunction.
    Or(Vec<Proposition>),
    /// Exclusive disjunction.
    Xor(Box<Proposition>, Box<Proposition>),
    /// Right-associative material implication.
    Implies(Box<Proposition>, Box<Proposition>),
    /// Exact count of true operands, with an admitted arithmetic count expression.
    Exactly(Expr, Vec<Proposition>),
}
impl Proposition {
    /// Visit every retained expression, including cardinality counts.
    pub fn expressions(&self, visit: &mut impl FnMut(&Expr)) {
        match self {
            Self::Atom(expression) => visit(expression),
            Self::Not(value) => value.expressions(visit),
            Self::And(values) | Self::Or(values) => {
                for value in values {
                    value.expressions(visit);
                }
            }
            Self::Xor(a, b) | Self::Implies(a, b) => {
                a.expressions(visit);
                b.expressions(visit);
            }
            Self::Exactly(count, values) => {
                visit(count);
                for value in values {
                    value.expressions(visit);
                }
            }
        }
    }
}
