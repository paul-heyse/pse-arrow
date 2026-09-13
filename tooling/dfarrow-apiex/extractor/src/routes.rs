//! Public-route index: every path by which an item is *reachable* from the crate root.
//!
//! Why this exists: filtering on `Visibility::Public` both over-reports (a `pub`
//! item inside a private module is unreachable) and under-reports (an item
//! re-exported through `pub use` is public at a path that is not its defining
//! path). Reachability through `Use` items is the real criterion, so `Use` items
//! must be preserved and followed.

use rustdoc_types::*;
use std::collections::{BTreeMap, BTreeSet, HashSet};

#[derive(Debug, Default)]
pub struct RouteIndex {
    /// item id -> every public path it is reachable at, lexicographically ordered.
    pub routes: BTreeMap<Id, BTreeSet<String>>,
    /// `Use` targets that could not be resolved inside this document.
    pub unresolved: BTreeSet<String>,
    /// Glob re-exports whose target module is not in this document.
    pub unresolved_globs: BTreeSet<String>,
}

impl RouteIndex {
    pub fn state(&self) -> &'static str {
        if self.unresolved.is_empty() && self.unresolved_globs.is_empty() {
            "complete"
        } else {
            "partial"
        }
    }
    pub fn for_id(&self, id: &Id) -> Vec<String> {
        self.routes.get(id).map(|s| s.iter().cloned().collect()).unwrap_or_default()
    }
    /// Shortest route, tie-broken lexicographically: the canonical `use` path.
    pub fn canonical(&self, id: &Id) -> Option<String> {
        self.routes.get(id).and_then(|s| {
            s.iter()
                .min_by_key(|p| (p.matches("::").count(), p.len(), (*p).clone()))
                .cloned()
        })
    }
}

fn is_public(v: &Visibility) -> bool {
    matches!(v, Visibility::Public)
}

pub fn build(krate: &Crate, crate_name: &str) -> RouteIndex {
    let mut idx = RouteIndex::default();
    let mut seen: HashSet<(Id, usize)> = HashSet::new();
    walk(krate, &krate.root, crate_name, &mut idx, &mut seen, 0);
    idx
}

fn walk(
    krate: &Crate,
    module_id: &Id,
    prefix: &str,
    idx: &mut RouteIndex,
    seen: &mut HashSet<(Id, usize)>,
    depth: usize,
) {
    // Re-export cycles are legal Rust; bound the walk rather than trusting acyclicity.
    if depth > 24 || !seen.insert((*module_id, depth)) {
        return;
    }
    let Some(item) = krate.index.get(module_id) else { return };
    let ItemEnum::Module(m) = &item.inner else { return };

    for child_id in &m.items {
        let Some(child) = krate.index.get(child_id) else {
            // Not in this document: an external or stripped item.
            continue;
        };
        match &child.inner {
            ItemEnum::Use(u) => {
                // A `use` is the public face of something defined elsewhere.
                let Some(target) = &u.id else {
                    idx.unresolved.insert(format!("{prefix}::{}", u.name));
                    continue;
                };
                if u.is_glob {
                    // `pub use foo::*` lifts every public child of foo to *this* path.
                    match krate.index.get(target) {
                        Some(Item { inner: ItemEnum::Module(_), .. }) => {
                            walk(krate, target, prefix, idx, seen, depth + 1);
                        }
                        _ => {
                            idx.unresolved_globs.insert(format!("{prefix}::{}::*", u.source));
                        }
                    }
                } else {
                    let route = format!("{prefix}::{}", u.name);
                    idx.routes.entry(*target).or_default().insert(route.clone());
                    // A re-exported module also exposes everything beneath it.
                    if let Some(Item { inner: ItemEnum::Module(_), .. }) = krate.index.get(target) {
                        walk(krate, target, &route, idx, seen, depth + 1);
                    }
                }
            }
            _ => {
                if !is_public(&child.visibility) {
                    continue;
                }
                let Some(name) = &child.name else { continue };
                let route = format!("{prefix}::{name}");
                idx.routes.entry(*child_id).or_default().insert(route.clone());
                if matches!(child.inner, ItemEnum::Module(_)) {
                    walk(krate, child_id, &route, idx, seen, depth + 1);
                }
            }
        }
    }
}
