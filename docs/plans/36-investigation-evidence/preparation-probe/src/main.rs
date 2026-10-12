use salsa::Setter;
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};

static PARSE: AtomicUsize = AtomicUsize::new(0);
static CHECK: AtomicUsize = AtomicUsize::new(0);
#[salsa::db]
#[derive(Default, Clone)]
struct Db { storage: salsa::Storage<Self> }
#[salsa::db]
impl salsa::Database for Db {}
#[salsa::input]
struct Row { text: String }
#[salsa::input]
struct Context { scale: u32 }
#[salsa::input]
struct Namespace { entries: Vec<(String, Row)> }
#[salsa::tracked(returns(clone))]
fn parsed(db: &dyn salsa::Database, row: Row) -> Arc<Vec<String>> {
    PARSE.fetch_add(1, Ordering::SeqCst);
    Arc::new(row.text(db).split_whitespace().map(str::to_owned).collect())
}
#[salsa::tracked(returns(clone))]
fn checked(db: &dyn salsa::Database, row: Row, context: Context) -> Arc<(Vec<String>, u32)> {
    CHECK.fetch_add(1, Ordering::SeqCst);
    Arc::new((parsed(db, row).as_ref().clone(), *context.scale(db)))
}
#[salsa::tracked(returns(clone))]
fn lookup(db: &dyn salsa::Database, scope: Namespace, name: String) -> Result<Option<Row>, String> {
    let found: Vec<_> = scope.entries(db).iter().filter(|(n,_)| n == &name).map(|(_,r)| *r).collect();
    if found.len() > 1 { Err("duplicate".into()) } else { Ok(found.first().copied()) }
}
fn count() -> (usize,usize) { (PARSE.load(Ordering::SeqCst), CHECK.load(Ordering::SeqCst)) }
fn salsa_controls() {
    let mut db = Db::default();
    let a = Row::new(&db, "a b".to_owned());
    let b = Row::new(&db, "unrelated".to_owned());
    let context = Context::new(&db, 1);
    let scope = Namespace::new(&db, vec![("a".into(),a)]);
    let historical = checked(&db, a, context);
    assert_eq!(count(), (1,1));
    checked(&db,a,context); assert_eq!(count(),(1,1));
    b.set_text(&mut db).to("different".into());
    checked(&db,a,context); assert_eq!(count(),(1,1));
    a.set_text(&mut db).to("a  b".into());
    checked(&db,a,context); assert_eq!(count(),(2,1));
    a.set_text(&mut db).to("a c".into());
    checked(&db,a,context); assert_eq!(count(),(3,2));
    context.set_scale(&mut db).to(2);
    checked(&db,a,context); assert_eq!(count(),(3,3));
    assert_eq!(historical.as_ref(), &(vec!["a".into(),"b".into()],1));
    assert!(lookup(&db,scope,"missing".into()).unwrap().is_none());
    scope.set_entries(&mut db).to(vec![("a".into(),a),("missing".into(),b)]);
    assert!(lookup(&db,scope,"missing".into()).unwrap()==Some(b));
    scope.set_entries(&mut db).to(vec![("a".into(),a),("a".into(),b)]);
    assert!(lookup(&db,scope,"a".into()).is_err());
    use salsa::Database;
    db.cancellation_token().cancel();
    assert!(salsa::Cancelled::catch(|| checked(&db,a,context)).is_err());
    println!("salsa=0.28.4 warm/outside/equal/changed/context/history/absence/duplicate/cancel=passed parse=3 check=3");
}
fn graph_controls() {
    use petgraph::{graph::DiGraph, visit::{Dfs, Reversed}};
    // Child-to-parent edges match executable::implicit. Include a diamond, isolate and cycle.
    let mut cases = 0;
    for n in [1_usize,8,64,256] {
        for cycle in [false,true] {
            let mut g = DiGraph::<(),()>::new();
            let nodes: Vec<_> = (0..n).map(|_|g.add_node(())).collect();
            for i in 1..n.saturating_sub(1) { g.add_edge(nodes[i],nodes[(i-1)/2],()); }
            if n > 4 { g.add_edge(nodes[4],nodes[1],()); }
            if cycle && n > 4 { g.add_edge(nodes[0],nodes[4],()); }
            for parent in &nodes {
                let reversed = Reversed(&g);
                let mut dfs = Dfs::new(reversed,*parent);
                let mut descendants = std::collections::BTreeSet::new();
                while let Some(node) = dfs.next(reversed) { descendants.insert(node); }
                for child in &nodes {
                    assert_eq!(descendants.contains(child),petgraph::algo::has_path_connecting(&g,*child,*parent,None));
                    cases += 1;
                }
            }
        }
    }
    println!("petgraph=0.8.3 hierarchy reverse-traversal equivalence cases={cases} passed");
}
fn main() { salsa_controls(); graph_controls(); }
