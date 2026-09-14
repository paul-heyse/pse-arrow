// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Bounded recursive descent with explicit precedence and committed keyword branches.

use super::DslError;
use super::ast::{
    BinaryOp, Binder, CompareOp, Equation, EquationKind, EquationSense, Expr, ExprKind, Function,
    NamedArg, Number, Path, PathSegment, Predicate, PredicateKind, ReduceKind, Span,
};
use super::lexer::{Kind, Token, syntax, tokenize};

struct Cursor<'a> {
    tokens: Vec<Token<'a>>,
    position: usize,
    end: u32,
    depth: u32,
}

impl<'a> Cursor<'a> {
    fn new(text: &'a str) -> Result<Self, DslError> {
        Ok(Self {
            tokens: tokenize(text)?,
            position: 0,
            end: u32::try_from(text.len()).unwrap_or(u32::MAX),
            depth: 0,
        })
    }
    fn token(&self) -> Option<Token<'a>> {
        self.tokens.get(self.position).copied()
    }
    fn at(&self) -> u32 {
        self.token().map_or(self.end, |token| token.span.start)
    }
    fn end(&self) -> u32 {
        self.position
            .checked_sub(1)
            .and_then(|index| self.tokens.get(index))
            .map_or(0, |token| token.span.end)
    }
    fn is(&self, text: &str) -> bool {
        self.token().is_some_and(|token| token.text == text)
    }
    fn eat(&mut self, text: &str) -> bool {
        if self.is(text) {
            self.position += 1;
            true
        } else {
            false
        }
    }
    fn expect(&mut self, text: &str) -> Result<(), DslError> {
        if self.eat(text) {
            Ok(())
        } else {
            Err(self.error(text))
        }
    }
    fn error(&self, expected: &str) -> DslError {
        syntax(
            self.at(),
            expected,
            self.token().map_or("end of input", |token| token.text),
        )
    }
    fn ident(&mut self) -> Result<String, DslError> {
        let token = self.token().ok_or_else(|| self.error("identifier"))?;
        if token.kind != Kind::Identifier {
            return Err(self.error("identifier"));
        }
        self.position += 1;
        Ok(token.text.to_owned())
    }
    fn enter(&mut self) -> Result<(), DslError> {
        self.depth += 1;
        if self.depth > 64 {
            Err(DslError::Budget {
                limit: "depth",
                allowed: 64,
                needed: u64::from(self.depth),
            })
        } else {
            Ok(())
        }
    }
    fn finish(&self) -> Result<(), DslError> {
        if self.position == self.tokens.len() {
            Ok(())
        } else {
            Err(self.error("end of input"))
        }
    }
    fn expr(&mut self) -> Result<Expr, DslError> {
        self.enter()?;
        let mut result = self.binary(0)?;
        if self.eat("where") {
            let mut bindings = Vec::new();
            let mut names = std::collections::BTreeSet::new();
            loop {
                let name = self.ident()?;
                if !names.insert(name.clone()) {
                    return Err(self.error("distinct local binding"));
                }
                self.expect("=")?;
                let value = self.binary(0)?;
                bindings.push((name, value));
                if !self.eat(",") {
                    break;
                }
            }
            let span = Span {
                start: result.span.start,
                end: self.end(),
            };
            result = Expr {
                kind: ExprKind::Let {
                    bindings,
                    body: Box::new(result),
                },
                span,
            };
        }
        self.depth -= 1;
        Ok(result)
    }
    fn binary(&mut self, min: u8) -> Result<Expr, DslError> {
        let mut lhs = self.unary()?;
        while let Some((op, precedence)) = self.token().and_then(|token| match token.text {
            "+" => Some((BinaryOp::Add, 1)),
            "-" => Some((BinaryOp::Sub, 1)),
            "*" => Some((BinaryOp::Mul, 2)),
            "/" => Some((BinaryOp::Div, 2)),
            "^" => Some((BinaryOp::Pow, 3)),
            _ => None,
        }) {
            if precedence < min {
                break;
            }
            self.position += 1;
            self.enter()?;
            let rhs = self.binary(if op == BinaryOp::Pow {
                precedence
            } else {
                precedence + 1
            })?;
            self.depth -= 1;
            let span = Span {
                start: lhs.span.start,
                end: rhs.span.end,
            };
            lhs = Expr {
                kind: ExprKind::Binary {
                    op,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                },
                span,
            };
        }
        Ok(lhs)
    }
    fn unary(&mut self) -> Result<Expr, DslError> {
        let start = self.at();
        if self.eat("-") {
            self.enter()?;
            let body = self.unary()?;
            self.depth -= 1;
            if self.is("^") {
                return Err(DslError::AmbiguousUnaryPower { offset: start });
            }
            let end = body.span.end;
            return Ok(Expr {
                kind: ExprKind::Neg(Box::new(body)),
                span: Span { start, end },
            });
        }
        self.primary()
    }
    fn primary(&mut self) -> Result<Expr, DslError> {
        let start = self.at();
        let kind = if self.eat("(") {
            let mut inner = self.expr()?;
            self.expect(")")?;
            inner.span = Span {
                start,
                end: self.end(),
            };
            return Ok(inner);
        } else if self.eat("if") {
            let guard = self.predicate(0)?;
            self.expect("then")?;
            let then = self.expr()?;
            self.expect("else")?;
            let otherwise = self.expr()?;
            ExprKind::Conditional {
                guard: Box::new(guard),
                then: Box::new(then),
                otherwise: Box::new(otherwise),
            }
        } else if self.token().is_some_and(|token| token.kind == Kind::Number) {
            let token = self.token().ok_or_else(|| self.error("number"))?;
            self.position += 1;
            let value = token
                .text
                .parse::<f64>()
                .map_err(|_| self.error("number"))?;
            let unit = if self.eat("{") {
                Some(self.unit()?)
            } else {
                None
            };
            ExprKind::Number(Number { value, unit })
        } else {
            self.named_primary(start)?
        };
        Ok(Expr {
            kind,
            span: Span {
                start,
                end: self.end(),
            },
        })
    }
    fn named_primary(&mut self, start: u32) -> Result<ExprKind, DslError> {
        let name = self.ident()?;
        if matches!(
            name.as_str(),
            "then" | "else" | "where" | "in" | "and" | "or" | "not" | "for"
        ) {
            return Err(syntax(start, "expression", &name));
        }
        if name == "kernel" && self.eat(".") {
            let mut name = self.ident()?;
            while self.eat(".") {
                name.push('.');
                name.push_str(&self.ident()?);
            }
            self.expect("(")?;
            let (args, named) = self.arguments()?;
            if !named.is_empty() {
                return Err(self.error("positional kernel arguments"));
            }
            return Ok(ExprKind::Kernel { name, args });
        }
        if self.eat("(") {
            if let Some(kind) = match name.as_str() {
                "sum" => Some(ReduceKind::Sum),
                "prod" => Some(ReduceKind::Prod),
                "integral" => Some(ReduceKind::Integral),
                _ => None,
            } {
                return self.reduce(kind);
            }
            if name == "d" {
                let body = self.expr()?;
                self.expect(")")?;
                self.expect("/")?;
                let domain = self.ident()?;
                let wrt = if domain == "d" {
                    self.path()?
                } else if let Some(domain) =
                    domain.strip_prefix('d').filter(|name| !name.is_empty())
                {
                    self.path_tail(domain.to_owned())?
                } else {
                    return Err(self.error("d followed by a differentiation domain"));
                };
                return Ok(ExprKind::Derivative {
                    body: Box::new(body),
                    wrt,
                });
            }
            let function = Function::parse(&name)
                .ok_or_else(|| syntax(start, "declared function or kernel.name", &name))?;
            let (args, named) = self.arguments()?;
            if function == Function::WeightedMean
                && (args.is_empty() || args.len() % 2 != 0 || !named.is_empty())
            {
                return Err(DslError::WeightedMeanArity { offset: start });
            }
            if !named.is_empty() && !function.has_epsilon() {
                return Err(syntax(start, "function without named epsilon", &name));
            }
            return Ok(ExprKind::Call {
                function,
                args,
                named,
            });
        }
        Ok(ExprKind::Path(self.path_tail(name)?))
    }
    fn arguments(&mut self) -> Result<(Vec<Expr>, Vec<NamedArg>), DslError> {
        let mut args = Vec::new();
        let mut named = Vec::new();
        if self.eat(")") {
            return Ok((args, named));
        }
        loop {
            if self.is("eps")
                && self
                    .tokens
                    .get(self.position + 1)
                    .is_some_and(|token| token.text == "=")
            {
                self.position += 2;
                named.push(NamedArg {
                    name: "eps".to_owned(),
                    value: self.expr()?,
                });
                self.expect(")")?;
                break;
            }
            args.push(self.expr()?);
            if self.eat(")") {
                break;
            }
            self.expect(",")?;
        }
        Ok((args, named))
    }
    fn reduce(&mut self, kind: ReduceKind) -> Result<ExprKind, DslError> {
        let var = self.ident()?;
        self.expect("in")?;
        let domain = self.path()?;
        let filter = if self.eat("where") {
            Some(Box::new(self.predicate(0)?))
        } else {
            None
        };
        self.expect("|")?;
        let body = self.expr()?;
        self.expect(")")?;
        Ok(ExprKind::Reduce {
            kind,
            binder: Box::new(Binder {
                var,
                domain,
                filter,
            }),
            body: Box::new(body),
        })
    }
    fn path(&mut self) -> Result<Path, DslError> {
        let name = self.ident()?;
        self.path_tail(name)
    }
    fn path_tail(&mut self, mut name: String) -> Result<Path, DslError> {
        let mut segments = Vec::new();
        loop {
            let mut indices = Vec::new();
            if self.eat("[") {
                if self.is("]") {
                    return Err(self.error("nonempty index"));
                }
                loop {
                    indices.push(self.expr()?);
                    if self.eat("]") {
                        break;
                    }
                    self.expect(",")?;
                }
            }
            segments.push(PathSegment { name, indices });
            if !self.eat(".") {
                break;
            }
            name = self.ident()?;
        }
        Ok(Path { segments })
    }
    fn unit(&mut self) -> Result<String, DslError> {
        let mut text = String::new();
        let mut depth = 0_u32;
        let mut operand = true;
        loop {
            let token = self
                .token()
                .ok_or_else(|| self.error("unit expression followed by }"))?;
            if token.text == "}" {
                if operand || depth != 0 {
                    return Err(self.error("complete unit expression"));
                }
                self.position += 1;
                break;
            }
            match token.text {
                "(" if operand => {
                    depth += 1;
                    if depth > 64 {
                        return Err(DslError::Budget {
                            limit: "unit depth",
                            allowed: 64,
                            needed: u64::from(depth),
                        });
                    }
                }
                ")" if !operand && depth > 0 => {
                    depth -= 1;
                }
                "*" | "/" | "^" if !operand => {
                    operand = true;
                }
                "-" | "+" if operand && text.ends_with('^') => {}
                _ if operand && (token.kind == Kind::Identifier || token.kind == Kind::Number) => {
                    operand = false;
                }
                _ => return Err(self.error("unit name, product, quotient or power")),
            }
            text.push_str(token.text);
            self.position += 1;
        }
        Ok(text)
    }
    fn predicate(&mut self, min: u8) -> Result<Predicate, DslError> {
        self.enter()?;
        let mut lhs = self.predicate_atom()?;
        loop {
            let precedence = if self.is("or") {
                1
            } else if self.is("and") {
                2
            } else {
                break;
            };
            if precedence < min {
                break;
            }
            self.position += 1;
            let rhs = self.predicate(precedence + 1)?;
            let span = Span {
                start: lhs.span.start,
                end: rhs.span.end,
            };
            lhs = Predicate {
                kind: if precedence == 1 {
                    PredicateKind::Or(Box::new(lhs), Box::new(rhs))
                } else {
                    PredicateKind::And(Box::new(lhs), Box::new(rhs))
                },
                span,
            };
        }
        self.depth -= 1;
        Ok(lhs)
    }
    fn predicate_atom(&mut self) -> Result<Predicate, DslError> {
        let start = self.at();
        let kind = if self.eat("not") {
            PredicateKind::Not(Box::new(self.predicate(3)?))
        } else if self.eat("true") {
            PredicateKind::Bool(true)
        } else if self.eat("false") {
            PredicateKind::Bool(false)
        } else if self.eat("null") {
            PredicateKind::Null
        } else {
            let saved = self.position;
            let saved_depth = self.depth;
            if self.eat("(") {
                let candidate = self.predicate(0);
                if matches!(candidate, Err(DslError::Budget { .. })) {
                    return candidate;
                }
                if let Ok(mut predicate) = candidate
                    && self.eat(")")
                    && !self.token().is_some_and(|token| {
                        matches!(
                            token.text,
                            "+" | "-"
                                | "*"
                                | "/"
                                | "^"
                                | "=="
                                | "!="
                                | "<"
                                | "<="
                                | ">"
                                | ">="
                                | "in"
                        )
                    })
                {
                    predicate.span = Span {
                        start,
                        end: self.end(),
                    };
                    return Ok(predicate);
                }
                self.position = saved;
                self.depth = saved_depth;
            }
            let lhs = self.binary(0)?;
            if self.eat("in") {
                PredicateKind::In {
                    expr: Box::new(lhs),
                    domain: self.path()?,
                }
            } else if let Some(op) = self.token().and_then(|token| match token.text {
                "==" => Some(CompareOp::Eq),
                "!=" => Some(CompareOp::NotEq),
                "<" => Some(CompareOp::Lt),
                "<=" => Some(CompareOp::Le),
                ">" => Some(CompareOp::Gt),
                ">=" => Some(CompareOp::Ge),
                _ => None,
            }) {
                self.position += 1;
                PredicateKind::Compare {
                    op,
                    lhs: Box::new(lhs),
                    rhs: Box::new(self.binary(0)?),
                }
            } else {
                PredicateKind::Atom(Box::new(lhs))
            }
        };
        Ok(Predicate {
            kind,
            span: Span {
                start,
                end: self.end(),
            },
        })
    }
    fn equation(&mut self) -> Result<Equation, DslError> {
        self.enter()?;
        let start = self.at();
        let kind = if self.eat("if") {
            let guard = self.predicate(0)?;
            self.expect("then")?;
            let then = self.equation()?;
            self.expect("else")?;
            let otherwise = self.equation()?;
            EquationKind::Conditional {
                guard,
                then: Box::new(then),
                otherwise: Box::new(otherwise),
            }
        } else {
            let lhs = self.expr()?;
            let sense = if self.eat("==") {
                EquationSense::Eq
            } else if self.eat("<=") {
                EquationSense::Le
            } else if self.eat(">=") {
                EquationSense::Ge
            } else {
                return Err(self.error("==, <= or >="));
            };
            let rhs = self.expr()?;
            EquationKind::Relation { lhs, sense, rhs }
        };
        self.depth -= 1;
        Ok(Equation {
            kind,
            span: Span {
                start,
                end: self.end(),
            },
        })
    }
}

/// Parse one complete expression under the fixed byte/depth envelope.
///
/// # Errors
/// Returns a typed syntax, arity, numeric or resource-budget failure.
pub fn parse_expr(text: &str) -> Result<Expr, DslError> {
    let mut cursor = Cursor::new(text)?;
    let result = cursor.expr()?;
    cursor.finish()?;
    Ok(result)
}

/// Parse one complete equation, including conditional equation branches.
///
/// # Errors
/// Returns a typed syntax or resource-budget failure.
pub fn parse_equation(text: &str) -> Result<Equation, DslError> {
    let mut cursor = Cursor::new(text)?;
    let result = cursor.equation()?;
    cursor.finish()?;
    Ok(result)
}

/// Parse one complete predicate with explicit boolean precedence.
///
/// # Errors
/// Returns a typed syntax or resource-budget failure.
pub fn parse_predicate(text: &str) -> Result<Predicate, DslError> {
    let mut cursor = Cursor::new(text)?;
    let result = cursor.predicate(0)?;
    cursor.finish()?;
    Ok(result)
}
