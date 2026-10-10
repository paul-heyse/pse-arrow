// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! One complete grammar enumeration for immutable field and function bodies.
use super::{
    admission::ExpressionOccurrence,
    occurrences::{StaticPart, Syntax, static_parts},
};
use pse_authoring::dsl::{self, Expr, Predicate, PredicateKind as P};
use std::collections::BTreeMap;

/// One expression root under its exact owning field path.
#[derive(Debug)]
pub struct Root<'a> {
    /// Relative source field role, also used to associate checked function copies.
    pub role: &'static str,
    /// Repeated field position.
    pub repeated: usize,
    /// Root position inside that field's grammar.
    pub part: usize,
    /// Retained source-bearing syntax.
    pub expression: &'a Expr,
}

/// Enumerate expression roots, never all descendants, in predicate grammar order.
pub fn predicate<'a>(value: &'a Predicate, roots: &mut Vec<&'a Expr>) {
    match &value.kind {
        P::Compare { lhs, rhs, .. } => roots.extend([lhs.as_ref(), rhs.as_ref()]),
        P::In { expr, domain } => {
            roots.push(expr);
            path(domain, roots);
        }
        P::Atom(expr) => roots.push(expr),
        P::And(a, b) | P::Or(a, b) => {
            predicate(a, roots);
            predicate(b, roots);
        }
        P::Not(inner) => predicate(inner, roots),
        P::Bool(_) | P::Null => {}
    }
}
fn path<'a>(value: &'a dsl::Path, roots: &mut Vec<&'a Expr>) {
    roots.extend(value.segments.iter().flat_map(|segment| &segment.indices));
}
fn equation<'a>(value: &'a dsl::Equation, roots: &mut Vec<&'a Expr>) {
    match &value.kind {
        dsl::EquationKind::Relation { lhs, rhs, .. } => roots.extend([lhs, rhs]),
        dsl::EquationKind::Conditional {
            guard,
            then,
            otherwise,
        } => {
            predicate(guard, roots);
            equation(then, roots);
            equation(otherwise, roots);
        }
    }
}
/// Complete retained field inventory, including static and cardinality grammars.
pub fn field(value: &Syntax) -> Vec<&Expr> {
    let mut roots = Vec::new();
    match value {
        Syntax::Expression(expr) => roots.push(expr),
        Syntax::Predicate(value) => predicate(value, &mut roots),
        Syntax::Equation(value) => equation(value, &mut roots),
        Syntax::Logic(value) => value.expressions(&mut |expr| roots.push(expr)),
        Syntax::Static(value) => static_parts(value, &mut |part| match part {
            StaticPart::Expression(expr) => roots.push(expr),
            StaticPart::Predicate(value) => predicate(value, &mut roots),
            StaticPart::Name(_) => {}
        }),
    }
    roots
}
/// Complete function inventory. Ordering is independent of checker execution order.
pub fn function(value: &crate::Function) -> Vec<Root<'_>> {
    let mut roots = Vec::new();
    fn append<'a>(
        out: &mut Vec<Root<'a>>,
        role: &'static str,
        repeated: usize,
        expressions: Vec<&'a Expr>,
    ) {
        out.extend(
            expressions
                .into_iter()
                .enumerate()
                .map(|(part, expression)| Root {
                    role,
                    repeated,
                    part,
                    expression,
                }),
        );
    }
    append(&mut roots, "function.body", 0, value.body.iter().collect());
    append(
        &mut roots,
        "function.external.output",
        0,
        value.external.iter().map(|value| &value.output).collect(),
    );
    if let Some(value) = &value.validity {
        let mut expressions = Vec::new();
        predicate(value, &mut expressions);
        append(&mut roots, "function.validity", 0, expressions);
    }
    for (index, guard) in value.envelopes.iter().enumerate() {
        let mut expressions = Vec::new();
        predicate(&guard.predicate, &mut expressions);
        append(&mut roots, "function.envelopes", index, expressions);
    }
    for (index, expression) in value.applicability.iter().enumerate() {
        append(
            &mut roots,
            "function.applicability",
            index,
            vec![expression],
        );
    }
    for (index, usage) in value.applicability_uses.iter().enumerate() {
        let mut expressions = Vec::new();
        for value in &usage.predicates {
            predicate(value, &mut expressions);
        }
        expressions.extend(&usage.inputs);
        append(
            &mut roots,
            "function.applicability_uses",
            index,
            expressions,
        );
    }
    roots
}
/// Temporary addresses never enter checked products or portable coordinates.
pub fn positions<'a>(
    roots: impl IntoIterator<Item = &'a Expr>,
) -> BTreeMap<usize, ExpressionOccurrence> {
    let mut positions = BTreeMap::new();
    for (body, root) in roots.into_iter().enumerate() {
        let mut position = 0;
        root.walk(|node| {
            positions.insert(
                std::ptr::from_ref(node) as usize,
                ExpressionOccurrence::Node { body, position },
            );
            position += 1;
        });
    }
    positions
}
/// Prepare coordinate bounds once, without an individual subtree walk per receipt.
pub fn node_counts(roots: &[&Expr]) -> Vec<usize> {
    roots
        .iter()
        .map(|root| {
            let mut count = 0;
            root.walk(|_| count += 1);
            count
        })
        .collect()
}
/// Resolve one selected coordinate for attribution or diagnostics.
pub fn node<'a>(roots: &[&'a Expr], occurrence: &ExpressionOccurrence) -> Option<&'a Expr> {
    let ExpressionOccurrence::Node { body, position } = occurrence else {
        return None;
    };
    let root = roots.get(*body)?;
    let mut found = None;
    let mut index = 0;
    root.walk(|node| {
        if index == *position {
            found = Some(node);
        }
        index += 1;
    });
    found
}

/// Span-independent structural equality on original ASTs, without cloning subtrees.
pub fn structural_eq(a: &Expr, b: &Expr) -> bool {
    use dsl::ExprKind as E;
    fn list(a: &[Expr], b: &[Expr]) -> bool {
        a.len() == b.len() && a.iter().zip(b).all(|(a, b)| structural_eq(a, b))
    }
    match (&a.kind, &b.kind) {
        (E::Number(a), E::Number(b)) => a == b,
        (E::Path(a), E::Path(b)) => path_eq(a, b),
        (E::Neg(a), E::Neg(b)) => structural_eq(a, b),
        (
            E::Binary {
                op: a,
                lhs: al,
                rhs: ar,
            },
            E::Binary {
                op: b,
                lhs: bl,
                rhs: br,
            },
        ) => a == b && structural_eq(al, bl) && structural_eq(ar, br),
        (
            E::Call {
                function: a,
                args: aa,
            },
            E::Call {
                function: b,
                args: ba,
            },
        ) => a == b && list(aa, ba),
        (E::NamedCall { name: a, args: aa }, E::NamedCall { name: b, args: ba }) => {
            path_eq(a, b) && list(aa, ba)
        }
        (E::Kernel { name: a, args: aa }, E::Kernel { name: b, args: ba }) => {
            a == b && list(aa, ba)
        }
        (
            E::Partial {
                function: a,
                args: aa,
                wrt: aw,
            },
            E::Partial {
                function: b,
                args: ba,
                wrt: bw,
            },
        ) => {
            path_eq(a, b)
                && list(aa, ba)
                && aw.len() == bw.len()
                && aw.iter().zip(bw).all(|(a, b)| path_eq(a, b))
        }
        (
            E::Reduce {
                kind: a,
                binder: ab,
                body: ae,
            },
            E::Reduce {
                kind: b,
                binder: bb,
                body: be,
            },
        ) => a == b && binder_eq(ab, bb) && structural_eq(ae, be),
        (
            E::Fold {
                accumulator: a,
                item: ai,
                binder: ab,
                value: av,
                step: as_,
            },
            E::Fold {
                accumulator: b,
                item: bi,
                binder: bb,
                value: bv,
                step: bs,
            },
        ) => {
            a == b
                && ai == bi
                && binder_eq(ab, bb)
                && structural_eq(av, bv)
                && structural_eq(as_, bs)
        }
        (E::Derivative { body: a, wrt: aw }, E::Derivative { body: b, wrt: bw }) => {
            structural_eq(a, b) && path_eq(aw, bw)
        }
        (
            E::Conditional {
                guard: a,
                then: at,
                otherwise: ao,
            },
            E::Conditional {
                guard: b,
                then: bt,
                otherwise: bo,
            },
        ) => predicate_eq(a, b) && structural_eq(at, bt) && structural_eq(ao, bo),
        (
            E::Let {
                bindings: a,
                body: ae,
            },
            E::Let {
                bindings: b,
                body: be,
            },
        ) => {
            a.len() == b.len()
                && a.iter()
                    .zip(b)
                    .all(|((an, ae), (bn, be))| an == bn && structural_eq(ae, be))
                && structural_eq(ae, be)
        }
        _ => false,
    }
}
fn path_eq(a: &dsl::Path, b: &dsl::Path) -> bool {
    a.segments.len() == b.segments.len()
        && a.segments.iter().zip(&b.segments).all(|(a, b)| {
            a.name == b.name
                && a.indices.len() == b.indices.len()
                && a.indices
                    .iter()
                    .zip(&b.indices)
                    .all(|(a, b)| structural_eq(a, b))
        })
}
fn binder_eq(a: &dsl::Binder, b: &dsl::Binder) -> bool {
    a.var == b.var
        && path_eq(&a.domain, &b.domain)
        && match (&a.filter, &b.filter) {
            (None, None) => true,
            (Some(a), Some(b)) => predicate_eq(a, b),
            _ => false,
        }
}
fn predicate_eq(a: &Predicate, b: &Predicate) -> bool {
    match (&a.kind, &b.kind) {
        (
            P::Compare {
                op: a,
                lhs: al,
                rhs: ar,
            },
            P::Compare {
                op: b,
                lhs: bl,
                rhs: br,
            },
        ) => a == b && structural_eq(al, bl) && structural_eq(ar, br),
        (
            P::In {
                expr: a,
                domain: ad,
            },
            P::In {
                expr: b,
                domain: bd,
            },
        ) => structural_eq(a, b) && path_eq(ad, bd),
        (P::And(a, b), P::And(c, d)) | (P::Or(a, b), P::Or(c, d)) => {
            predicate_eq(a, c) && predicate_eq(b, d)
        }
        (P::Not(a), P::Not(b)) => predicate_eq(a, b),
        (P::Atom(a), P::Atom(b)) => structural_eq(a, b),
        (P::Bool(a), P::Bool(b)) => a == b,
        (P::Null, P::Null) => true,
        _ => false,
    }
}

/// Prepared root index. Prehashes are temporary candidate filters, never membership.
pub(crate) struct StructuralIndex<'a> {
    roots: Vec<&'a Expr>,
    hashes: Vec<pse_ids::ContentHash>,
}
impl<'a> StructuralIndex<'a> {
    pub(crate) fn new(roots: Vec<&'a Expr>) -> Self {
        let hashes = roots.iter().map(|root| prehash(root)).collect();
        Self { roots, hashes }
    }
    pub(crate) fn matches(&self, slot: usize, other: &Self, other_slot: usize) -> bool {
        self.hashes
            .get(slot)
            .zip(other.hashes.get(other_slot))
            .is_some_and(|(a, b)| a == b)
            && self
                .roots
                .get(slot)
                .zip(other.roots.get(other_slot))
                .is_some_and(|(a, b)| structural_eq(a, b))
    }
}
// Every node is visited once bottom-up. Only immediate payloads and child digests
// enter its frame; no subtree text, normalization or recursive signature is retained.
#[cfg(test)]
thread_local! { static PREHASH_VISITS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) }; }
fn prehash(value: &Expr) -> pse_ids::ContentHash {
    #[cfg(test)]
    PREHASH_VISITS.with(|visits| visits.set(visits.get() + 1));
    use dsl::ExprKind as E;
    let mut h = pse_ids::FramedHasher::new(pse_ids::Frame::MathStructuralBodyIndexV1);
    fn expressions(h: &mut pse_ids::FramedHasher, values: &[Expr]) {
        h.u64(values.len() as u64);
        for value in values {
            h.hash(&prehash(value));
        }
    }
    match &value.kind {
        E::Number(n) => {
            h.str("number").u64(n.value.to_bits());
            match n.integer() {
                Some(n) => {
                    h.bool(true).part(&n.to_le_bytes());
                }
                None => {
                    h.bool(false);
                }
            }
            match &n.unit {
                Some(unit) => {
                    h.bool(true).u64(unit.factors().len() as u64);
                    for (name, power) in unit.factors() {
                        h.str(name)
                            .part(&power.num().to_le_bytes())
                            .part(&power.den().to_le_bytes());
                    }
                }
                None => {
                    h.bool(false);
                }
            }
        }
        E::Path(p) => {
            h.str("path");
            hash_path(&mut h, p);
        }
        E::Neg(v) => {
            h.str("neg").hash(&prehash(v));
        }
        E::Binary { op, lhs, rhs } => {
            h.str("binary")
                .str(op.as_str())
                .hash(&prehash(lhs))
                .hash(&prehash(rhs));
        }
        E::Call { function, args } => {
            h.str("call").str(function.as_str());
            expressions(&mut h, args);
        }
        E::NamedCall { name, args } => {
            h.str("named");
            hash_path(&mut h, name);
            expressions(&mut h, args);
        }
        E::Kernel { name, args } => {
            h.str("kernel").str(name);
            expressions(&mut h, args);
        }
        E::Partial {
            function,
            args,
            wrt,
        } => {
            h.str("partial");
            hash_path(&mut h, function);
            expressions(&mut h, args);
            h.u64(wrt.len() as u64);
            for p in wrt {
                hash_path(&mut h, p);
            }
        }
        E::Reduce { kind, binder, body } => {
            h.str("reduce").str(kind.as_str());
            hash_binder(&mut h, binder);
            h.hash(&prehash(body));
        }
        E::Fold {
            accumulator,
            item,
            binder,
            value,
            step,
        } => {
            h.str("fold").str(accumulator).str(item);
            hash_binder(&mut h, binder);
            h.hash(&prehash(value)).hash(&prehash(step));
        }
        E::Derivative { body, wrt } => {
            h.str("derivative").hash(&prehash(body));
            hash_path(&mut h, wrt);
        }
        E::Conditional {
            guard,
            then,
            otherwise,
        } => {
            h.str("conditional");
            hash_predicate(&mut h, guard);
            h.hash(&prehash(then)).hash(&prehash(otherwise));
        }
        E::Let { bindings, body } => {
            h.str("let").u64(bindings.len() as u64);
            for (name, value) in bindings {
                h.str(name).hash(&prehash(value));
            }
            h.hash(&prehash(body));
        }
    }
    h.finish_hash()
}
fn hash_path(h: &mut pse_ids::FramedHasher, value: &dsl::Path) {
    h.u64(value.segments.len() as u64);
    for segment in &value.segments {
        h.str(&segment.name).u64(segment.indices.len() as u64);
        for index in &segment.indices {
            h.hash(&prehash(index));
        }
    }
}
fn hash_binder(h: &mut pse_ids::FramedHasher, value: &dsl::Binder) {
    h.str(&value.var);
    hash_path(h, &value.domain);
    h.bool(value.filter.is_some());
    if let Some(value) = &value.filter {
        hash_predicate(h, value);
    }
}
fn hash_predicate(h: &mut pse_ids::FramedHasher, value: &Predicate) {
    match &value.kind {
        P::Compare { op, lhs, rhs } => {
            h.str("compare")
                .str(op.as_str())
                .hash(&prehash(lhs))
                .hash(&prehash(rhs));
        }
        P::In { expr, domain } => {
            h.str("in").hash(&prehash(expr));
            hash_path(h, domain);
        }
        P::And(a, b) => {
            h.str("and");
            hash_predicate(h, a);
            hash_predicate(h, b);
        }
        P::Or(a, b) => {
            h.str("or");
            hash_predicate(h, a);
            hash_predicate(h, b);
        }
        P::Not(v) => {
            h.str("not");
            hash_predicate(h, v);
        }
        P::Atom(v) => {
            h.str("atom").hash(&prehash(v));
        }
        P::Bool(v) => {
            h.str("bool").bool(*v);
        }
        P::Null => {
            h.str("null");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pse_authoring::dsl::ExprKind as E;

    #[test]
    fn body_inventory_let_positions_follow_structural_order_not_checking_order() {
        let root = dsl::parse_expr("a+b where a=x/x,b=y/y").unwrap();
        let E::Let { body, bindings } = &root.kind else {
            panic!("let")
        };
        let positions = positions([&root]);
        assert_eq!(
            positions[&(std::ptr::from_ref(body.as_ref()) as usize)],
            ExpressionOccurrence::Node {
                body: 0,
                position: 1
            }
        );
        assert_eq!(
            positions[&(std::ptr::from_ref(&bindings[0].1) as usize)],
            ExpressionOccurrence::Node {
                body: 0,
                position: 4
            }
        );
        assert_eq!(
            positions[&(std::ptr::from_ref(&bindings[1].1) as usize)],
            ExpressionOccurrence::Node {
                body: 0,
                position: 7
            }
        );
    }
    #[test]
    fn body_inventory_membership_domain_indices_are_distinct_roots() {
        let syntax = Syntax::Predicate(dsl::parse_predicate("x in domain[x/y]").unwrap());
        let roots = field(&syntax);
        assert_eq!(roots.len(), 2);
        assert!(matches!(roots[0].kind, E::Path(_)));
        assert!(matches!(
            roots[1].kind,
            E::Binary {
                op: dsl::BinaryOp::Div,
                ..
            }
        ));
        let positions = positions(roots.iter().copied());
        assert_eq!(
            positions[&(std::ptr::from_ref(roots[1]) as usize)],
            ExpressionOccurrence::Node {
                body: 1,
                position: 0
            }
        );
    }
    #[test]
    fn body_inventory_collision_prefilter_cannot_merge_structure_or_binders() {
        let a = dsl::parse_expr("x/x where x=a").unwrap();
        let mut equal = a.clone();
        equal.strip_spans();
        let others = [
            "x*x where x=a",
            "x/x where x=b",
            "y/y where y=a",
            "x/(x+1) where x=a",
        ]
        .map(|text| dsl::parse_expr(text).unwrap());
        let collision = pse_ids::ContentHash::from_bytes([7; 32]);
        let left = StructuralIndex {
            roots: vec![&a],
            hashes: vec![collision],
        };
        assert!(left.matches(
            0,
            &StructuralIndex {
                roots: vec![&equal],
                hashes: vec![collision]
            },
            0
        ));
        for other in &others {
            assert!(!left.matches(
                0,
                &StructuralIndex {
                    roots: vec![other],
                    hashes: vec![collision]
                },
                0
            ));
        }
    }
    #[test]
    fn body_inventory_exact_literals_units_and_path_segment_indices_survive_collision() {
        let collision = pse_ids::ContentHash::from_bytes([9; 32]);
        for (a, b) in [
            ("1{m}", "1{s}"),
            ("9007199254740993", "9007199254740994"),
            ("a[x].b[y]", "a[y].b[x]"),
            ("if x>0 then x else y", "if x>=0 then x else y"),
        ] {
            let a = dsl::parse_expr(a).unwrap();
            let b = dsl::parse_expr(b).unwrap();
            let left = StructuralIndex {
                roots: vec![&a],
                hashes: vec![collision],
            };
            assert!(!left.matches(
                0,
                &StructuralIndex {
                    roots: vec![&b],
                    hashes: vec![collision]
                },
                0
            ));
        }
    }
    #[test]
    fn body_inventory_coordinates_are_constant_extent_and_owner_local() {
        let roots = [
            dsl::parse_expr("x/x").unwrap(),
            dsl::parse_expr("x/x").unwrap(),
        ];
        let positions = positions(roots.iter());
        assert_eq!(positions.len(), 6);
        assert_ne!(
            positions[&(std::ptr::from_ref(&roots[0]) as usize)],
            positions[&(std::ptr::from_ref(&roots[1]) as usize)]
        );
        assert_eq!(node_counts(&roots.iter().collect::<Vec<_>>()), [3, 3]);
        assert!(size_of::<ExpressionOccurrence>() <= 3 * size_of::<usize>());
        assert!(
            node(
                &roots.iter().collect::<Vec<_>>(),
                &ExpressionOccurrence::Node {
                    body: 2,
                    position: 0
                }
            )
            .is_none()
        );
        assert!(
            node(
                &roots.iter().collect::<Vec<_>>(),
                &ExpressionOccurrence::Node {
                    body: 0,
                    position: 3
                }
            )
            .is_none()
        );
    }
    #[test]
    fn body_inventory_complete_expression_grammar_uses_original_nodes() {
        for text in [
            "1{m}",
            "a[x].b[y]",
            "-x",
            "x+y",
            "sqrt(x)",
            "f[i](x)",
            "kernel.foreign(x)",
            "partial(f[i],x[j])(y)",
            "sum(i in s[x] where i in t[y] | z[i])",
            "fold(a,b;i in s[x] where i>0 | y; a+b)",
            "d(x)/dt[i]",
            "if x in s[y] then a else b",
            "a where a=x",
        ] {
            let root = dsl::parse_expr(text).unwrap();
            let mut equal = root.clone();
            equal.strip_spans();
            let mut count = 0;
            root.walk(|_| count += 1);
            let coordinates = positions([&root]);
            assert_eq!(coordinates.len(), count, "{text}");
            assert!(
                StructuralIndex::new(vec![&root]).matches(
                    0,
                    &StructuralIndex::new(vec![&equal]),
                    0
                ),
                "{text}"
            );
        }
    }
    #[test]
    fn body_inventory_static_comprehension_and_logic_cardinality_roots() {
        use pse_authoring::language::StaticValue as S;
        let e = |text| S::Expression(dsl::parse_expr(text).unwrap());
        let syntax = Syntax::Static(S::Comprehension {
            bindings: vec![("i".into(), S::Set(vec![e("a/b")]))],
            filter: Some(dsl::parse_predicate("i in s[x/y]").unwrap()),
            body: Box::new(S::Apply {
                name: "Root".into(),
                arguments: vec![(
                    "x".into(),
                    S::Tuple(vec![e("z/w"), S::Text("ignored".into())]),
                )],
            }),
        });
        let roots = field(&syntax);
        assert_eq!(roots.len(), 4);
        assert_eq!(
            roots
                .iter()
                .map(|root| dsl::render_expr(root))
                .collect::<Vec<_>>(),
            ["(a / b)", "i", "(x / y)", "(z / w)"]
        );
        let logic = Syntax::Logic(dsl::parse_proposition("exactly(1+1, a, b)").unwrap());
        let roots = field(&logic);
        assert_eq!(roots.len(), 3);
        assert!(matches!(roots[0].kind, E::Binary { .. }));
    }
    #[test]
    fn body_inventory_preparation_work_and_coordinate_extent_grow_with_nodes() {
        for length in [16, 64, 256] {
            let mut root = dsl::parse_expr("x").unwrap();
            for _ in 0..length {
                root = Expr {
                    kind: E::Binary {
                        op: dsl::BinaryOp::Add,
                        lhs: Box::new(root),
                        rhs: Box::new(dsl::parse_expr("x").unwrap()),
                    },
                    span: Default::default(),
                };
            }
            let expected = 2 * length + 1;
            PREHASH_VISITS.with(|visits| visits.set(0));
            let _index = StructuralIndex::new(vec![&root]);
            assert_eq!(PREHASH_VISITS.with(std::cell::Cell::get), expected);
            assert_eq!(positions([&root]).len(), expected);
            assert_eq!(node_counts(&[&root]), [expected]);
        }
    }
}
