// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Type expressions as post-order arenas (ADR-0123 Outcome 1).
//!
//! A declared type is a list of nodes in which every node's children precede it and the
//! last node is the root. Names occur only as path segments of `named` and `identifier`
//! nodes and as the names of `variable` and `argument` nodes; nothing here resolves them.
//! [`TypeTree::new`] is the one validation every consumer walks through, the parser in
//! [`super::parse`] is the one producer of source syntax, and [`render_type`] prints the
//! canonical spelling that parses back to the same arena.
use crate::AuthoringError;
pub use pse_model::generated::enums::ModelingTypeNode as TypeNodeKind;
pub use pse_model::generated::structures::{
    ModelingTypeArenaNode as TypeNode, ModelingTypeArenaNodeExponent as TypeExponent,
};

/// Why an arena is not a well-formed type. Callers report it through their own coded
/// error ([`AuthoringError::Contract`] here, the modeling contract error in the checker).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeArenaViolation {
    /// The arena has no root.
    Empty,
    /// A node names a child at or after its own position.
    ForwardChild {
        /// The parent's position.
        node: usize,
        /// The offending child position.
        child: usize,
    },
    /// A node other than the root is not exactly one node's child.
    Detached {
        /// The node's position.
        node: usize,
    },
    /// A node's fields or children disagree with its kind.
    Malformed {
        /// The node's position.
        node: usize,
        /// The node kind.
        kind: &'static str,
        /// The violated rule.
        reason: &'static str,
    },
}

impl std::fmt::Display for TypeArenaViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => f.write_str("a type has at least one node"),
            Self::ForwardChild { node, child } => write!(
                f,
                "type node {node} names child {child}, which does not precede it"
            ),
            Self::Detached { node } => {
                write!(f, "type node {node} is not exactly one node's child")
            }
            Self::Malformed { node, kind, reason } => {
                write!(f, "type node {node} ({kind}): {reason}")
            }
        }
    }
}

impl From<TypeArenaViolation> for AuthoringError {
    fn from(error: TypeArenaViolation) -> Self {
        Self::Contract {
            at: None,
            reason: error.to_string(),
        }
    }
}

/// A validated arena, viewed from its root.
#[derive(Clone, Copy, Debug)]
pub struct TypeTree<'a> {
    nodes: &'a [TypeNode],
}

/// One node of a validated arena.
#[derive(Clone, Copy, Debug)]
pub struct TypeRef<'a> {
    nodes: &'a [TypeNode],
    index: usize,
}

impl<'a> TypeTree<'a> {
    /// Validate an arena: children precede their parent, every node but the root is
    /// exactly one node's child, and each node carries exactly the fields of its kind.
    ///
    /// # Errors
    /// The first violated rule, naming the node.
    pub fn new(nodes: &'a [TypeNode]) -> Result<Self, TypeArenaViolation> {
        if nodes.is_empty() {
            return Err(TypeArenaViolation::Empty);
        }
        let mut parents = vec![0_u8; nodes.len()];
        for (index, node) in nodes.iter().enumerate() {
            for child in &node.children {
                let child = *child as usize;
                if child >= index {
                    return Err(TypeArenaViolation::ForwardChild { node: index, child });
                }
                parents[child] = parents[child].saturating_add(1);
            }
            shape(nodes, index)?;
        }
        let root = nodes.len() - 1;
        if let Some(node) = parents
            .iter()
            .enumerate()
            .find(|(i, count)| **count != u8::from(*i != root))
            .map(|(i, _)| i)
        {
            return Err(TypeArenaViolation::Detached { node });
        }
        if nodes[root].kind == TypeNodeKind::Argument {
            return Err(malformed(
                root,
                TypeNodeKind::Argument,
                "an argument is not a type",
            ));
        }
        Ok(Self { nodes })
    }
    /// The root node.
    pub fn root(self) -> TypeRef<'a> {
        TypeRef {
            nodes: self.nodes,
            index: self.nodes.len() - 1,
        }
    }
}

impl<'a> TypeRef<'a> {
    fn node(self) -> &'a TypeNode {
        &self.nodes[self.index]
    }
    /// The node kind.
    pub fn kind(self) -> TypeNodeKind {
        self.node().kind
    }
    /// The path segments of a `named` or `identifier` node; empty otherwise.
    pub fn path(self) -> &'a [String] {
        self.node().path.as_deref().unwrap_or_default()
    }
    /// The name of a `variable` or `argument` node; empty otherwise.
    pub fn name(self) -> &'a str {
        self.node().name.as_deref().unwrap_or_default()
    }
    /// The reduced rational exponent of a `power` node.
    pub fn exponent(self) -> Option<(i16, i16)> {
        self.node().exponent.as_ref().map(|e| (e.num, e.den))
    }
    /// The children, in order.
    pub fn children(self) -> impl ExactSizeIterator<Item = TypeRef<'a>> + 'a {
        let nodes = self.nodes;
        self.node().children.iter().map(move |child| TypeRef {
            nodes,
            index: *child as usize,
        })
    }
    /// The child at `position`.
    pub fn child(self, position: usize) -> Option<TypeRef<'a>> {
        self.children().nth(position)
    }
}

fn malformed(node: usize, kind: TypeNodeKind, reason: &'static str) -> TypeArenaViolation {
    TypeArenaViolation::Malformed {
        node,
        kind: kind.as_str(),
        reason,
    }
}

/// The fields and children one node kind carries, and nothing else.
fn shape(nodes: &[TypeNode], index: usize) -> Result<(), TypeArenaViolation> {
    use TypeNodeKind as K;
    let node = &nodes[index];
    let bad = |reason| Err(malformed(index, node.kind, reason));
    let path = node.path.as_deref();
    let named_path = path.is_some_and(|p| !p.is_empty() && p.iter().all(|s| !s.is_empty()));
    let name = node.name.as_deref().is_some_and(|n| !n.is_empty());
    let children = node.children.len();
    let child_kind = |position: usize| nodes[node.children[position] as usize].kind;
    let (wants_path, wants_name, wants_exponent) = match node.kind {
        K::Named | K::Identifier | K::Coordinate | K::ReducedLaw => (true, false, false),
        K::Variable | K::Argument => (false, true, false),
        K::Power => (false, false, true),
        _ => (false, false, false),
    };
    if wants_path != path.is_some() || wants_path && !named_path {
        return bad("a path of nonempty segments exactly on named and identifier nodes");
    }
    if wants_name != node.name.is_some() || wants_name && !name {
        return bad("a nonempty name exactly on variable and argument nodes");
    }
    if wants_exponent != node.exponent.is_some() {
        return bad("an exponent exactly on power nodes");
    }
    let arity_ok = match node.kind {
        K::Boolean
        | K::Integer
        | K::Text
        | K::Named
        | K::Variable
        | K::Identifier
        | K::Coordinate
        | K::ReducedLaw
        | K::QuantityType
        | K::ReferenceState => children == 0,
        K::Optional | K::Set | K::Row | K::Table | K::Delta | K::Argument | K::Power => {
            children == 1
        }
        K::Product | K::Quotient => children == 2,
        K::Transfer => children == 3 && child_kind(1) == K::Named && child_kind(2) == K::Named,
        K::Tuple | K::Function => children >= 1,
        K::Indexed => children >= 2,
    };
    if !arity_ok {
        return bad("the child count of its kind");
    }
    if let Some((num, den)) = node.exponent.as_ref().map(|e| (e.num, e.den))
        && pse_quantity::Ratio::from_parts(num, den).is_err()
    {
        return bad("a reduced rational exponent with a positive denominator");
    }
    let arguments = node
        .children
        .iter()
        .enumerate()
        .filter(|(_, c)| nodes[**c as usize].kind == K::Argument)
        .map(|(position, _)| position)
        .collect::<Vec<_>>();
    match node.kind {
        K::Function => {
            if arguments != (0..children - 1).collect::<Vec<_>>() {
                return bad("argument children followed by exactly one result");
            }
            let mut names = std::collections::BTreeSet::new();
            if node.children[..children - 1]
                .iter()
                .any(|c| !names.insert(nodes[*c as usize].name.as_deref()))
            {
                return bad("distinct argument names");
            }
        }
        _ if !arguments.is_empty() => return bad("an argument belongs to a function"),
        K::Indexed if (1..children).any(|p| child_kind(p) != K::Named) => {
            return bad("an element followed by named axes");
        }
        _ => {}
    }
    Ok(())
}

/// A node appended to an arena under construction, returning its position.
pub(crate) fn push(nodes: &mut Vec<TypeNode>, node: TypeNode) -> u32 {
    nodes.push(node);
    u32::try_from(nodes.len() - 1).unwrap_or(u32::MAX)
}

/// A node of `kind` with the given children and no other field.
pub fn node(kind: TypeNodeKind, children: Vec<u32>) -> TypeNode {
    TypeNode {
        kind,
        path: None,
        name: None,
        exponent: None,
        children,
    }
}

/// The joined path of every `named` and `identifier` node, in arena order: the names a
/// checker resolves once to identities.
pub fn type_paths(nodes: &[TypeNode]) -> impl Iterator<Item = String> + '_ {
    nodes
        .iter()
        .filter_map(|node| node.path.as_ref().map(|path| path.join(".")))
}

/// Binding strength of a node in source syntax, loosest first.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Level {
    Type,
    Indexed,
    Product,
    Primary,
}

fn level(kind: TypeNodeKind) -> Level {
    use TypeNodeKind as K;
    match kind {
        K::Function | K::Optional => Level::Type,
        K::Indexed => Level::Indexed,
        K::Product | K::Quotient | K::Power => Level::Product,
        _ => Level::Primary,
    }
}

/// The canonical spelling of a validated arena.
///
/// # Errors
/// The arena is not a well-formed type.
pub fn render_type(nodes: &[TypeNode]) -> Result<String, TypeArenaViolation> {
    let tree = TypeTree::new(nodes)?;
    let mut out = String::new();
    write(tree.root(), &mut out);
    Ok(out)
}

fn path(segments: &[String]) -> String {
    segments
        .iter()
        .map(|s| crate::grammar::render_name(s))
        .collect::<Vec<_>>()
        .join(".")
}

/// Write `node` where the grammar accepts `at_least` or tighter, parenthesizing it
/// otherwise.
fn operand(node: TypeRef<'_>, at_least: Level, out: &mut String) {
    if level(node.kind()) < at_least {
        out.push('(');
        write(node, out);
        out.push(')');
    } else {
        write(node, out);
    }
}

fn write(node: TypeRef<'_>, out: &mut String) {
    use TypeNodeKind as K;
    let generic = |keyword: &str, node: TypeRef<'_>, out: &mut String| {
        out.push_str(keyword);
        out.push('<');
        for (i, child) in node.children().enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            write(child, out);
        }
        out.push('>');
    };
    match node.kind() {
        K::Boolean => out.push_str("Boolean"),
        K::Integer => out.push_str("Integer"),
        K::Text => out.push_str("Text"),
        K::QuantityType => out.push_str("QuantityType"),
        K::ReferenceState => out.push_str("ReferenceState"),
        K::Named => out.push_str(&path(node.path())),
        K::Variable | K::Argument => out.push_str(&crate::grammar::render_name(node.name())),
        K::Identifier | K::Coordinate | K::ReducedLaw => {
            out.push_str(match node.kind() {
                K::Coordinate => "Coordinate<",
                K::ReducedLaw => "Reduced<",
                _ => "Id<",
            });
            out.push_str(&path(node.path()));
            out.push('>');
        }
        K::Set => generic("Set", node, out),
        K::Row => generic("Row", node, out),
        K::Table => generic("Table", node, out),
        K::Tuple => generic("Tuple", node, out),
        K::Delta => generic("Delta", node, out),
        K::Transfer => generic("Transfer", node, out),
        K::Optional => {
            // `?` applies to an indexed type or tighter; a function or optional operand
            // is parenthesized.
            let [inner] = node.children().collect::<Vec<_>>()[..] else {
                return;
            };
            operand(inner, Level::Indexed, out);
            out.push('?');
        }
        K::Indexed => {
            let mut children = node.children();
            if let Some(element) = children.next() {
                operand(element, Level::Product, out);
            }
            out.push('[');
            for (i, axis) in children.enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                write(axis, out);
            }
            out.push(']');
        }
        K::Product | K::Quotient => {
            // Left-associative: the left operand may itself be a product or quotient,
            // the right operand is a power or tighter.
            let [left, right] = node.children().collect::<Vec<_>>()[..] else {
                return;
            };
            operand(left, Level::Product, out);
            out.push(if node.kind() == K::Product { '*' } else { '/' });
            if matches!(right.kind(), K::Product | K::Quotient) {
                out.push('(');
                write(right, out);
                out.push(')');
            } else {
                operand(right, Level::Product, out);
            }
        }
        K::Power => {
            let [base] = node.children().collect::<Vec<_>>()[..] else {
                return;
            };
            operand(base, Level::Primary, out);
            match node.exponent() {
                Some((num, 1)) => out.push_str(&format!("^{num}")),
                Some((num, den)) => out.push_str(&format!("^({num}/{den})")),
                None => {}
            }
        }
        K::Function => {
            let children = node.children().collect::<Vec<_>>();
            let Some((result, arguments)) = children.split_last() else {
                return;
            };
            out.push_str("Fn(");
            for (i, argument) in arguments.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                out.push_str(&crate::grammar::render_name(argument.name()));
                out.push_str(": ");
                if let Some(ty) = argument.child(0) {
                    write(ty, out);
                }
            }
            out.push_str(")->");
            write(*result, out);
        }
    }
}

/// Parse one standalone type expression; `variables` are the type parameters in scope.
///
/// # Errors
/// The text is not exactly one type expression.
pub fn parse_type(text: &str, variables: &[&str]) -> Result<Vec<TypeNode>, AuthoringError> {
    super::parser::parse_type(text, variables)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use proptest::test_runner::RngSeed;

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 1024,
            rng_seed: RngSeed::Fixed(0x5053_452d_5459_5031),
            failure_persistence: None,
            ..ProptestConfig::default()
        })]
        /// Every well-formed arena over every node kind prints to a spelling that parses
        /// back to the same arena: the renderer parenthesizes exactly where the grammar
        /// would otherwise build a different tree.
        #[test]
        fn type_arena_render_parse_roundtrip(nodes in crate::dsl::type_arenas()) {
            let rendered = render_type(&nodes)
                .map_err(|e| TestCaseError::fail(format!("{nodes:?}: {e}")))?;
            let parsed = parse_type(&rendered, &crate::dsl::TYPE_VARIABLES)
                .map_err(|e| TestCaseError::fail(format!("{rendered}: {e}")))?;
            prop_assert_eq!(&parsed, &nodes, "{}", rendered);
        }
    }

    /// Source spellings of one type are one arena; the canonical spelling is stable.
    #[test]
    fn type_spellings_parse_to_one_arena() {
        let canonical = |text: &str| render_type(&parse_type(text, &["Q"]).unwrap()).unwrap();
        for (text, expected) in [
            ("MolarCp/Temperature^2", "MolarCp/Temperature^2"),
            ("MolarCp / (Temperature ^ (2))", "MolarCp/Temperature^2"),
            ("Temperature^(-1/2)", "Temperature^(-1/2)"),
            ("Temperature^-2", "Temperature^-2"),
            (
                "Scalar[chemistry.species, kinds.phase]",
                "Scalar[chemistry.species, kinds.phase]",
            ),
            (
                "Fn(T:Temperature,j:chemistry.species)->DeltaH",
                "Fn(T: Temperature, j: chemistry.species)->DeltaH",
            ),
            ("Δ<Q>", "Delta<Q>"),
            ("Set<Row<t>>", "Set<Row<t>>"),
            ("Tuple<Mass,Q?>", "Tuple<Mass, Q?>"),
            ("Id<doi>?", "Id<doi>?"),
            ("A*(B/C)", "A*(B/C)"),
            ("(A*B)/C", "A*B/C"),
        ] {
            assert_eq!(canonical(text), expected, "{text}");
        }
        let arena = parse_type("Fn(x: Q)->Q^2", &["Q"]).unwrap();
        let kinds = arena.iter().map(|n| n.kind).collect::<Vec<_>>();
        use TypeNodeKind as K;
        assert_eq!(
            kinds,
            [K::Variable, K::Argument, K::Variable, K::Power, K::Function]
        );
        // Without `Q` in scope the same spelling names a declaration.
        assert_eq!(parse_type("Q", &[]).unwrap()[0].kind, K::Named);
        for invalid in [
            "Set<Mass",
            "Mass^2.5",
            "Mass^(1/0)",
            "Fn(x: A, x: B)->C",
            "Mass Time",
            "",
        ] {
            assert!(parse_type(invalid, &[]).is_err(), "{invalid}");
        }
    }

    fn named(segments: &[&str]) -> TypeNode {
        TypeNode {
            path: Some(segments.iter().map(|s| (*s).to_owned()).collect()),
            ..node(TypeNodeKind::Named, vec![])
        }
    }

    /// A child must precede its parent: an arena naming a later node is refused before
    /// anything walks it, and so is a node that is two nodes' child or nobody's.
    #[test]
    fn type_arena_rejects_forward_child() {
        let forward = vec![node(TypeNodeKind::Set, vec![1]), named(&["Mass"])];
        assert_eq!(
            TypeTree::new(&forward).unwrap_err(),
            TypeArenaViolation::ForwardChild { node: 0, child: 1 }
        );
        let own = vec![node(TypeNodeKind::Set, vec![0])];
        assert_eq!(
            TypeTree::new(&own).unwrap_err(),
            TypeArenaViolation::ForwardChild { node: 0, child: 0 }
        );
        let shared = vec![named(&["Mass"]), node(TypeNodeKind::Product, vec![0, 0])];
        assert_eq!(
            TypeTree::new(&shared).unwrap_err(),
            TypeArenaViolation::Detached { node: 0 }
        );
        let orphan = vec![named(&["Mass"]), named(&["Time"])];
        assert_eq!(
            TypeTree::new(&orphan).unwrap_err(),
            TypeArenaViolation::Detached { node: 0 }
        );
        // Control: the same nodes in post order are a type.
        let ordered = vec![named(&["Mass"]), node(TypeNodeKind::Set, vec![0])];
        assert_eq!(render_type(&ordered).unwrap(), "Set<Mass>");
        for malformed in [
            vec![node(TypeNodeKind::Named, vec![])],
            vec![named(&["Mass"]), node(TypeNodeKind::Power, vec![0])],
            vec![
                named(&["Mass"]),
                TypeNode {
                    exponent: Some(TypeExponent { num: 2, den: 4 }),
                    ..node(TypeNodeKind::Power, vec![0])
                },
            ],
            vec![
                named(&["Mass"]),
                node(TypeNodeKind::Set, vec![0]),
                node(TypeNodeKind::Indexed, vec![0, 1]),
            ],
            vec![
                named(&["Mass"]),
                TypeNode {
                    name: Some("x".into()),
                    ..node(TypeNodeKind::Argument, vec![0])
                },
            ],
        ] {
            assert!(
                matches!(
                    TypeTree::new(&malformed),
                    Err(TypeArenaViolation::Malformed { .. })
                ),
                "{malformed:?}"
            );
        }
    }
}
