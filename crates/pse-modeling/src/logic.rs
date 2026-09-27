// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Logic propositions over binary variables (ADR-0104).
//!
//! `implies` binds loosest and associates right, then `or`, `xor` and `and`; `not` binds
//! tightest. `exactly(k, p, q, …)` counts true operands. Atoms are member paths.

/// A parsed proposition; atoms keep their authored path text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Proposition {
    /// A binary variable path.
    Atom(String),
    /// Complement.
    Not(Box<Proposition>),
    /// Conjunction of two or more operands.
    And(Vec<Proposition>),
    /// Disjunction of two or more operands.
    Or(Vec<Proposition>),
    /// Exclusive disjunction of two operands.
    Xor(Box<Proposition>, Box<Proposition>),
    /// Material implication.
    Implies(Box<Proposition>, Box<Proposition>),
    /// Exactly `count` operands are true; the count is authored text.
    Exactly(String, Vec<Proposition>),
}

/// Parse a proposition, or say which token is wrong.
/// # Errors
/// Unbalanced parentheses or brackets, a missing operand or trailing text.
pub fn parse(text: &str) -> Result<Proposition, String> {
    let tokens = tokenize(text)?;
    let mut parser = Parser {
        tokens,
        position: 0,
        depth: 0,
    };
    let proposition = parser.implies()?;
    match parser.tokens.get(parser.position) {
        None => Ok(proposition),
        Some(token) => Err(format!("unexpected `{token}` in proposition")),
    }
}

const KEYWORDS: [&str; 6] = ["and", "or", "xor", "implies", "not", "exactly"];
const MAX_DEPTH: usize = 64;

fn tokenize(text: &str) -> Result<Vec<String>, String> {
    let mut tokens = Vec::new();
    let mut chars = text.chars().peekable();
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
        } else if matches!(c, '(' | ')' | ',') {
            tokens.push(c.to_string());
            chars.next();
        } else if c.is_alphanumeric() || matches!(c, '_' | '.' | '\'') {
            let mut word = String::new();
            let mut brackets = 0usize;
            while let Some(&c) = chars.peek() {
                if c == '[' {
                    brackets += 1;
                } else if c == ']' {
                    brackets = brackets
                        .checked_sub(1)
                        .ok_or_else(|| "unbalanced `]` in proposition".to_owned())?;
                } else if brackets == 0
                    && !(c.is_alphanumeric() || matches!(c, '_' | '.' | '\'' | '-' | '+'))
                {
                    break;
                }
                word.push(c);
                chars.next();
            }
            if brackets != 0 {
                return Err("unbalanced `[` in proposition".into());
            }
            tokens.push(word);
        } else {
            return Err(format!("unexpected `{c}` in proposition"));
        }
    }
    Ok(tokens)
}

struct Parser {
    tokens: Vec<String>,
    position: usize,
    depth: usize,
}
impl Parser {
    fn peek(&self) -> Option<&str> {
        self.tokens.get(self.position).map(String::as_str)
    }
    fn eat(&mut self, token: &str) -> bool {
        if self.peek() == Some(token) {
            self.position += 1;
            true
        } else {
            false
        }
    }
    fn expect(&mut self, token: &str) -> Result<(), String> {
        if self.eat(token) {
            Ok(())
        } else {
            Err(format!(
                "expected `{token}`, found `{}`",
                self.peek().unwrap_or("end")
            ))
        }
    }
    fn enter(&mut self) -> Result<(), String> {
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            return Err("proposition nesting exceeds 64".into());
        }
        Ok(())
    }
    fn implies(&mut self) -> Result<Proposition, String> {
        self.enter()?;
        let lhs = self.or()?;
        let result = if self.eat("implies") {
            Proposition::Implies(Box::new(lhs), Box::new(self.implies()?))
        } else {
            lhs
        };
        self.depth -= 1;
        Ok(result)
    }
    fn or(&mut self) -> Result<Proposition, String> {
        let mut operands = vec![self.xor()?];
        while self.eat("or") {
            operands.push(self.xor()?);
        }
        Ok(if operands.len() == 1 {
            operands.remove(0)
        } else {
            Proposition::Or(operands)
        })
    }
    fn xor(&mut self) -> Result<Proposition, String> {
        let mut lhs = self.and()?;
        while self.eat("xor") {
            lhs = Proposition::Xor(Box::new(lhs), Box::new(self.and()?));
        }
        Ok(lhs)
    }
    fn and(&mut self) -> Result<Proposition, String> {
        let mut operands = vec![self.unary()?];
        while self.eat("and") {
            operands.push(self.unary()?);
        }
        Ok(if operands.len() == 1 {
            operands.remove(0)
        } else {
            Proposition::And(operands)
        })
    }
    fn unary(&mut self) -> Result<Proposition, String> {
        self.enter()?;
        let result = if self.eat("not") {
            Proposition::Not(Box::new(self.unary()?))
        } else if self.eat("(") {
            let inner = self.implies()?;
            self.expect(")")?;
            inner
        } else if self.eat("exactly") {
            self.expect("(")?;
            let count = self
                .tokens
                .get(self.position)
                .filter(|t| {
                    !KEYWORDS.contains(&t.as_str()) && !matches!(t.as_str(), "(" | ")" | ",")
                })
                .cloned()
                .ok_or_else(|| "exactly(k, ...) needs a count".to_owned())?;
            self.position += 1;
            let mut operands = Vec::new();
            while self.eat(",") {
                operands.push(self.implies()?);
            }
            self.expect(")")?;
            if operands.is_empty() {
                return Err("exactly(k, ...) needs operands".into());
            }
            Proposition::Exactly(count, operands)
        } else {
            let token = self
                .peek()
                .filter(|t| !KEYWORDS.contains(t) && !matches!(*t, "(" | ")" | ","))
                .ok_or_else(|| {
                    format!(
                        "expected a variable, found `{}`",
                        self.peek().unwrap_or("end")
                    )
                })?
                .to_owned();
            self.position += 1;
            Proposition::Atom(token)
        };
        self.depth -= 1;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::{Proposition as P, parse};
    fn atom(name: &str) -> P {
        P::Atom(name.into())
    }
    #[test]
    fn precedence_and_grouping_follow_the_declared_order() {
        assert_eq!(
            parse("a implies b or not c and d").unwrap(),
            P::Implies(
                Box::new(atom("a")),
                Box::new(P::Or(vec![
                    atom("b"),
                    P::And(vec![P::Not(Box::new(atom("c"))), atom("d")])
                ]))
            )
        );
        assert_eq!(
            parse("exactly(2, y[i], route.left, (a xor b))").unwrap(),
            P::Exactly(
                "2".into(),
                vec![
                    atom("y[i]"),
                    atom("route.left"),
                    P::Xor(Box::new(atom("a")), Box::new(atom("b")))
                ]
            )
        );
        for invalid in ["", "a and", "(a or b", "a b", "exactly(2)", "y[i"] {
            assert!(parse(invalid).is_err(), "{invalid}");
        }
    }
}
