// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
// IBEX 2.9.1/FILIB binding. Solver owns covering and bisection.
#include <ibex.h>
#include <algorithm>
#include <chrono>
#include <cmath>
#include <cstdint>
#include <limits>
#include <memory>
#include <stdexcept>
#include <vector>
#ifdef __FAST_MATH__
#error "validated interval transport requires strict floating-point arithmetic"
#endif

extern "C" {
struct PseRootNode { uint32_t op, a, b, begin, count; int32_t power; double lower, upper; };
struct PseRootGuard { uint32_t node, kind, chart_only, domain; double lower, upper; };
struct PseRootRequest {
    const PseRootNode* nodes; uint32_t node_count;
    const uint32_t* edges; uint32_t edge_count;
    const uint32_t* residuals; uint32_t unknown_count, parameter_count;
    uint32_t score, tolerance;
    const PseRootGuard* guards; uint32_t guard_count;
    const double* lower; const double* upper; const double* parameters; const double* candidate;
    uint64_t max_cells; double seconds;
    int32_t (*cancelled)(const void*); const void* cancel_context;
};
struct PseChartChainResult { uint32_t status; uint64_t proof_cells, charts, connections; };
struct PseRootResult {
    uint32_t status; uint64_t cells, solution, boundary, unknown, pending;
};
// status: 0 unique selection, 2 chart, 3 coverage, 4 resource, 5 boundary, 6 unsupported.
int32_t pse_ibex_certify(const PseRootRequest*, uint32_t count, uint32_t winner,
    PseRootResult*, double* parameter_lower,
    double* parameter_upper, double* existence_lower, double* existence_upper,
    double* uniqueness_lower, double* uniqueness_upper) noexcept;
int32_t pse_ibex_promote(const PseRootRequest*, PseRootResult*,
    const double* parameter_lower, const double* parameter_upper,
    const double* uniqueness_lower, const double* uniqueness_upper) noexcept;
int32_t pse_ibex_connect(const PseRootRequest*, PseRootResult*,
    const double* intersection_lower, const double* intersection_upper) noexcept;
}
namespace {
using namespace ibex;
struct Resource {};
struct Boundary {};
using Clock=std::chrono::steady_clock;
struct Budget {
    const PseRootRequest& r; Clock::time_point started=Clock::now(); uint64_t cells=0;
    void check() const {
        if ((r.cancelled && r.cancelled(r.cancel_context)) ||
            std::chrono::duration<double>(Clock::now()-started).count()>=r.seconds) throw Resource();
    }
    double remaining() const {
        return std::max(0.0,r.seconds-std::chrono::duration<double>(Clock::now()-started).count());
    }
};
// cleanup deduplicates shared nodes. The final Function owns the entire original DAG.
struct Arena {
    std::vector<const ExprNode*> nodes;
    const ExprNode& keep(const ExprNode& e) { nodes.push_back(&e); return e; }
    ~Arena() {
        if (!nodes.empty()) {
            Array<const ExprNode> roots(static_cast<int>(nodes.size()));
            for (size_t i=0;i<nodes.size();++i) roots.set_ref(static_cast<int>(i),*nodes[i]);
            cleanup(roots,true);
        }
    }
};
struct Graph {
    Arena arena; std::vector<const ExprNode*> expr;
    std::unique_ptr<Function> values;
    std::unique_ptr<System> residual;
    std::unique_ptr<System> equations;
    std::vector<std::unique_ptr<Function>> guard_values;
    std::unique_ptr<Function> score_value, tolerance_value;
    Graph(const PseRootRequest& r,Budget& budget,bool competitive=false,double threshold=0.0) {
        const int dimension=static_cast<int>(r.unknown_count+r.parameter_count);
        const Dim input_dimension=Dim::col_vec(dimension);
        const ExprSymbol& x=ExprSymbol::new_("coordinates",input_dimension);
        arena.keep(x); expr.reserve(r.node_count);
        for (uint32_t i=0;i<r.node_count;++i) {
            budget.check();
            const auto& n=r.nodes[i]; const ExprNode* e=nullptr;
            auto child=[&](uint32_t id)->const ExprNode& {
                if (id>=i) throw std::invalid_argument("non-topological node");
                return *expr[id];
            };
            switch(n.op) {
            case 0: if(n.a>=static_cast<uint32_t>(dimension)) throw std::invalid_argument("coordinate"); e=&arena.keep(x[static_cast<int>(n.a)]); break;
            case 1: if(!std::isfinite(n.lower)||!std::isfinite(n.upper)||n.lower>n.upper) throw std::invalid_argument("constant"); e=&arena.keep(ExprConstant::new_scalar(Interval(n.lower,n.upper))); break;
            case 2: case 3: {
                if(n.begin>r.edge_count||n.count>r.edge_count-n.begin) throw std::invalid_argument("edges");
                std::vector<const ExprNode*> layer; layer.reserve(n.count);
                for(uint32_t j=0;j<n.count;++j) layer.push_back(&child(r.edges[n.begin+j]));
                if(layer.empty()) layer.push_back(&arena.keep(ExprConstant::new_scalar(Interval(n.op==2?0.0:1.0))));
                while(layer.size()>1) {
                    std::vector<const ExprNode*> next; next.reserve((layer.size()+1)/2);
                    for(size_t j=0;j<layer.size();j+=2) {
                        if(j+1==layer.size()) next.push_back(layer[j]);
                        else if(n.op==2) next.push_back(&arena.keep(*layer[j]+*layer[j+1]));
                        else next.push_back(&arena.keep(*layer[j]* *layer[j+1]));
                    }
                    layer=std::move(next);
                }
                e=layer[0]; break;
            }
            case 4: e=&arena.keep(pow(child(n.a),n.power)); break;
            case 5: e=&arena.keep(sqrt(child(n.a))); break;
            case 6: e=&arena.keep(exp(child(n.a))); break;
            case 7: e=&arena.keep(log(child(n.a))); break;
            case 8: e=&arena.keep(sin(child(n.a))); break;
            case 9: e=&arena.keep(cos(child(n.a))); break;
            default: throw std::invalid_argument("unsupported operation");
            }
            expr.push_back(e);
        }
        SystemFactory factory; factory.set_simplification_level(0); factory.add_var(x);
        for(uint32_t i=0;i<r.unknown_count;++i) {
            if(r.residuals[i]>=expr.size()) throw std::invalid_argument("residual");
            factory.add_ctr(ExprCtr(*expr[r.residuals[i]],LEQ));
            factory.add_ctr(ExprCtr(*expr[r.residuals[i]],GEQ));
        }
        std::vector<const ExprNode*> closure;
        for(uint32_t i=0;i<r.guard_count;++i) {
            budget.check();
            const auto& g=r.guards[i];
            if(g.node>=expr.size()||g.kind>4) throw std::invalid_argument("guard");
            if(g.chart_only||g.kind==2) continue;
            const double lower=g.kind>=3?0.0:g.lower;
            const double upper=g.kind>=3?POS_INFINITY:g.upper;
            if(std::isfinite(lower)) {
                const auto& bound=arena.keep(*expr[g.node]-lower); closure.push_back(&bound);
                factory.add_ctr(ExprCtr(bound,GEQ));
            }
            if(std::isfinite(upper)) {
                const auto& bound=arena.keep(*expr[g.node]-upper); closure.push_back(&bound);
                factory.add_ctr(ExprCtr(bound,LEQ));
            }
        }
        if(r.score>=expr.size()||r.tolerance>=expr.size()) throw std::invalid_argument("selection expression");
        if(competitive) {
            if(!std::isfinite(threshold)) throw std::invalid_argument("competitive threshold");
            const auto& bound=arena.keep(*expr[r.score]-threshold); closure.push_back(&bound);
            // Keep the equality face: a tie or any better root defeats selection.
            factory.add_ctr(ExprCtr(bound,LEQ));
        }
        residual=std::make_unique<System>(factory);
        SystemFactory equations_factory; equations_factory.set_simplification_level(0); equations_factory.add_var(x);
        for(uint32_t i=0;i<r.unknown_count;++i) equations_factory.add_ctr_eq(*expr[r.residuals[i]]);
        equations=std::make_unique<System>(equations_factory);
        Array<const ExprNode> outputs(static_cast<int>(expr.size()+closure.size()));
        for(size_t i=0;i<expr.size();++i) outputs.set_ref(static_cast<int>(i),*expr[i]);
        for(size_t i=0;i<closure.size();++i) outputs.set_ref(static_cast<int>(expr.size()+i),*closure[i]);
        const auto& vector=arena.keep(ExprVector::new_col(outputs));
        values=std::make_unique<Function>(x,vector);
        arena.nodes.clear();
        auto scalar=[&](uint32_t node)->std::unique_ptr<Function> {
            budget.check();
            const auto& input=ExprSymbol::new_("scalar_coordinates",input_dimension);
            arena.keep(input);
            const auto& body=arena.keep(ExprCopy().copy(Array<const ExprSymbol>(x),
                Array<const ExprSymbol>(input),*expr[node]));
            auto result=std::make_unique<Function>(input,body);
            arena.nodes.clear();
            return result;
        };
        // Evaluate only the relevant scalar DAG. Unrelated physical outputs can
        // cross their domains on B without invalidating a constant tolerance.
        score_value=scalar(r.score); tolerance_value=scalar(r.tolerance);
        guard_values.reserve(r.guard_count);
        for(uint32_t i=0;i<r.guard_count;++i) {
            guard_values.push_back(scalar(r.guards[i].node));
        }
    }
};
enum class GuardState { Admitted, Invalid, Crossing };
GuardState guards(const PseRootRequest& r, Graph& graph, const IntervalVector& box, bool chart,bool domain_only=false) {
    bool crossing=false;
    for(uint32_t i=0;i<r.guard_count;++i) {
        const auto& g=r.guards[i]; if((g.chart_only&&!chart)||(domain_only&&!g.domain)) continue;
        if(g.node>=r.node_count||g.kind>4) throw std::invalid_argument("guard");
        Interval image;
        try { image=graph.guard_values[i]->eval(box); }
        catch (const std::bad_alloc&) { throw; }
        catch (...) { throw Boundary(); }
        if(image.is_empty()||!std::isfinite(image.lb())||!std::isfinite(image.ub())) {
            crossing=true; continue;
        }
        if(g.kind==2) { // nonzero
            if(image==Interval(0.0)) return GuardState::Invalid;
            if(image.contains(0.0)) crossing=true;
        } else {
            const bool strict=g.kind==1||g.kind==3; // strict interval or positive
            const double lo=g.kind>=3?0.0:g.lower;
            const double hi=g.kind>=3?POS_INFINITY:g.upper;
            if(strict ? (image.ub()<=lo||image.lb()>=hi) : (image.ub()<lo||image.lb()>hi)) return GuardState::Invalid;
            if(strict ? (image.lb()<=lo||image.ub()>=hi) : (image.lb()<lo||image.ub()>hi)) crossing=true;
        }
    }
    return crossing?GuardState::Crossing:GuardState::Admitted;
}
struct Covering : Solver {
    using Solver::Solver;
    uint64_t visited_cells() const { return nb_cells; }
};
// A derivative-based split priority is useful only on its actual C1 domain.
// Neither priority establishes root/chart evidence; Solver still owns covering.
struct GuardBisector : Bsc {
    const PseRootRequest& r; Graph& graph; Budget& budget;
    LargestFirst fallback;
    SmearSumRelative smear;
    GuardBisector(const PseRootRequest& req,Graph& g,Budget& b,const Vector& precision)
        : Bsc(precision),r(req),graph(g),budget(b),fallback(precision),
          smear(*g.residual,precision,fallback) {}
    void add_property(const IntervalVector& box,BoxProperties& prop) override {
        // Smear forwards its property requirements to the borrowed fallback.
        budget.check(); smear.add_property(box,prop); budget.check();
    }
    BisectionPoint choose_var(const Cell& cell) override {
        budget.check();
        const bool admitted=guards(r,graph,cell.box,true,true)==GuardState::Admitted;
        budget.check();
        BisectionPoint split=admitted?smear.choose_var(cell):fallback.choose_var(cell);
        budget.check();
        return split;
    }
};
// Reuse one HC4 owner in the initial contraction, every library shaving slice,
// and the LP fixpoint. A covering cell remains the accounting unit; each HC4
// subcall observes the same absolute deadline and cancellation owner.
struct BudgetHC4 : Ctc {
    Budget& budget;
    CtcHC4 hc4;
    BudgetHC4(const System& system,Budget& b)
        : Ctc(system.nb_var),budget(b),hc4(system.ctrs,0.01) {
        input=hc4.input; output=hc4.output;
    }
    void add_property(const IntervalVector& box,BoxProperties& prop) override {
        budget.check(); hc4.add_property(box,prop); budget.check();
    }
    void contract(IntervalVector& box) override {
        ContractContext context(box); contract(box,context);
    }
    void contract(IntervalVector& box,ContractContext& context) override {
        budget.check(); hc4.contract(box,context); budget.check();
    }
};
struct GuardContractor : Ctc {
    const PseRootRequest& r; Graph& graph; Budget& budget;
    BudgetHC4 hc4;
    Ctc3BCid shaving;
    std::unique_ptr<CtcNewton> newton;
    LinearizerXTaylor linearizer; CtcPolytopeHull polytope; CtcCompo lp_hc4; CtcFixPoint lp;
    GuardContractor(const PseRootRequest& req,Graph& g,Budget& b,const VarSet& variables)
        : Ctc(g.residual->nb_var),r(req),graph(g),budget(b),hc4(*g.residual,b),
          shaving(variables.is_var,hc4),
          newton(req.parameter_count?std::make_unique<CtcNewton>(g.equations->f_ctrs,variables,5e8):
              std::make_unique<CtcNewton>(g.equations->f_ctrs,5e8)),
          // Library Taylor relaxation evaluates one interval Jacobian for both
          // corners; RELAX still encloses every nonlinear feasible point.
          linearizer(*g.residual,LinearizerXTaylor::RELAX,
              LinearizerXTaylor::RANDOM_OPP,LinearizerXTaylor::TAYLOR),
          polytope(linearizer,100,1),lp_hc4(polytope,hc4),lp(lp_hc4) {}
    void add_property(const IntervalVector& box,BoxProperties& prop) override {
        shaving.add_property(box,prop); newton->add_property(box,prop); lp.add_property(box,prop);
    }
    void contract(IntervalVector& box) override { ContractContext context(box); contract(box,context); }
    void contract(IntervalVector& box,ContractContext& context) override {
        budget.check();
        if(guards(r,graph,box,false)==GuardState::Invalid) { box.set_empty(); return; }
        // Library-owned existential HC4 contraction preserves the admitted
        // portion of crossing log/sqrt/nonzero domains.
        try { hc4.contract(box,context); } catch(const Resource&) { throw; } catch(const std::bad_alloc&) { throw; } catch(...) { throw Boundary(); }
        budget.check();
        if(box.is_empty()) return;
        // Only unknown coordinates are selected for shaving. HC4's existential
        // subbox contractions may also soundly narrow parameters; no parametric
        // Newton or C1 premise is used by this library-owned disjunction.
        try { shaving.contract(box,context); } catch(const Resource&) { throw; } catch(const std::bad_alloc&) { throw; } catch(...) { throw Boundary(); }
        budget.check();
        if(box.is_empty()) return;
        // Every C1 mean-value premise is checked before Newton and Taylor/LP.
        // Crossing cells proceed through the native Solver's ordinary bisection.
        if(guards(r,graph,box,true,true)==GuardState::Admitted) {
            // Parametric interval Newton contracts unknowns only; it does not
            // turn a numerical proposal into root existence or selection evidence.
            try { newton->contract(box,context); } catch(const Resource&) { throw; } catch(const std::bad_alloc&) { throw; } catch(...) { throw Boundary(); }
            budget.check();
            if(box.is_empty()) return;
            try { lp.contract(box,context); } catch(const Resource&) { throw; } catch(const std::bad_alloc&) { throw; } catch(...) { throw Boundary(); }
            budget.check();
        }
    }
};
bool exclude(const PseRootRequest& r,Graph& graph,const VarSet& variables,
    const IntervalVector& domain,Budget& budget,PseRootResult& out) {
    budget.check(); if(budget.cells>=r.max_cells) throw Resource();
    Vector precision(domain.size(),1e-10);
    for(uint32_t i=r.unknown_count;i<static_cast<uint32_t>(domain.size());++i) precision[i]=2.0*domain[i].diam();
    GuardContractor guarded(r,graph,budget,variables);
    GuardBisector bisector(r,graph,budget,precision); CellStack buffer;
    // Both inequalities encode each residual equality exactly. Hence m=0:
    // Solver::check_sol never performs hidden inflating Newton on a crossing cell.
    Covering solver(*graph.residual,guarded,bisector,buffer,precision,Vector(domain.size(),POS_INFINITY));
    solver.cell_limit=static_cast<long>(r.max_cells-budget.cells);
    solver.time_limit=budget.remaining(); solver.start(domain);
    try {
        CovSolverData::BoxStatus status;
        while(solver.next(status)) {
            budget.check();
            // Any retained cell defeats complete exclusion; avoid retaining many.
            solver.flush(); budget.cells+=solver.visited_cells(); out.cells=budget.cells;
            const auto& d=solver.get_data(); out.solution=d.nb_inner()+d.nb_solution(); out.boundary=d.nb_boundary();
            out.unknown=d.nb_unknown(); out.pending=d.nb_pending(); return false;
        }
    } catch(...) {
        solver.flush(); budget.cells+=solver.visited_cells(); out.cells=budget.cells;
        const auto& d=solver.get_data(); out.solution=d.nb_inner()+d.nb_solution(); out.boundary=d.nb_boundary();
        out.unknown=d.nb_unknown(); out.pending=d.nb_pending(); throw;
    }
    solver.flush(); budget.cells+=solver.visited_cells(); out.cells=budget.cells;
    const auto& d=solver.get_data(); out.solution=d.nb_inner()+d.nb_solution(); out.boundary=d.nb_boundary();
    out.unknown=d.nb_unknown(); out.pending=d.nb_pending();
    if(budget.cells>r.max_cells) throw Resource();
    return d.nb_inner()==0&&d.nb_solution()==0&&d.nb_boundary()==0&&d.nb_unknown()==0&&d.nb_pending()==0;
}

Interval scalar_image(Function& function,const IntervalVector& box,Budget& budget) {
    budget.check();
    Interval image;
    try { image=function.eval(box); }
    catch(const std::bad_alloc&) { throw; }
    catch(...) { throw Boundary(); }
    budget.check();
    return image;
}
double tolerance_upper(Graph& graph,const IntervalVector& box,Budget& budget) {
    const Interval image=scalar_image(*graph.tolerance_value,box,budget);
    // The first binding requires a uniform finite nonnegative enclosure over B.
    // It does not reinterpret an unresolved tolerance as zero.
    if(image.is_empty()||!std::isfinite(image.lb())||!std::isfinite(image.ub())||image.lb()<0.0)
        throw std::invalid_argument("nonuniform selection tolerance");
    return image.ub();
}
IntervalVector physical_domain(const PseRootRequest& r,const IntervalVector& parameters) {
    IntervalVector domain(static_cast<int>(r.unknown_count+r.parameter_count));
    for(uint32_t i=0;i<r.unknown_count;++i) domain[i]=Interval(r.lower[i],r.upper[i]);
    for(uint32_t j=0;j<r.parameter_count;++j) domain[r.unknown_count+j]=parameters[j];
    return domain;
}
VarSet unknown_variables(const PseRootRequest& r) {
    const int dimension=static_cast<int>(r.unknown_count+r.parameter_count);
    BitSet bits=BitSet::empty(dimension);
    for(uint32_t i=0;i<r.unknown_count;++i) bits.add(static_cast<int>(i));
    return VarSet(dimension,bits);
}
double competitive_threshold(double score_upper,double winner_tolerance,double rival_tolerance) {
    const Interval threshold=Interval(score_upper)+Interval(std::max(winner_tolerance,rival_tolerance));
    if(threshold.is_empty()||!std::isfinite(threshold.ub())) throw std::invalid_argument("selection threshold overflow");
    return threshold.ub();
}
// A coordinate-sensitive Newton-image proposal, not a root or selection proof.
// IBEX supplies the interval derivatives, numerical preconditioner and arithmetic.
// The centered parameter image reduces dependency without proving a root. Every
// accepted seed still passes uniform Newton and full original competitive covering.
bool predictive_seed(const PseRootRequest& r,Graph& graph,const IntervalVector& domain,
    Budget& budget,IntervalVector& seed) {
    budget.check();
    const int n=static_cast<int>(r.unknown_count);
    const int p=static_cast<int>(r.parameter_count);
    IntervalVector point(domain),candidate(n);
    for(int i=0;i<n;++i) point[i]=candidate[i]=Interval(r.candidate[i]);
    const auto point_guards=guards(r,graph,point,true);budget.check();
    if(point_guards!=GuardState::Admitted) return false;
    const auto jacobian=graph.equations->f_ctrs.jacobian(point);budget.check();
    if(jacobian.is_empty()) return false;
    IntervalMatrix coefficients(n,n);
    Matrix inverse(n,n);
    for(int i=0;i<n;++i) for(int j=0;j<n;++j) coefficients[i][j]=jacobian[i][j];
    try {
        // A seed preconditioner needs no interval regularity certificate. IBEX's
        // approximate real inverse is used only to propose a box for re-proof.
        budget.check();real_inverse(coefficients.mid(),inverse);budget.check();
    } catch(const LinearException&) { budget.check();return false; }
    auto values=graph.equations->f_ctrs.eval_vector(point);budget.check();
    if(values.is_empty()) return false;
    if(p) {
        IntervalMatrix parameter_jacobian(n,p);
        IntervalVector midpoint(point),delta(p);
        for(int j=0;j<p;++j) {
            midpoint[n+j]=Interval(domain[n+j].mid());
            delta[j]=domain[n+j]-midpoint[n+j];
            for(int i=0;i<n;++i) parameter_jacobian[i][j]=jacobian[i][n+j];
        }
        // The preceding C1 guard admission covers the entire parameter box.
        // Intersect the natural residual with its library mean-value enclosure,
        // as IBEX's parametric inflating Newton does for its own Fmid.
        budget.check();
        const auto midpoint_values=graph.equations->f_ctrs.eval_vector(midpoint);budget.check();
        if(midpoint_values.is_empty()) return false;
        const IntervalVector centered=midpoint_values+parameter_jacobian*delta;budget.check();
        values&=centered;
        if(values.is_empty()) return false;
    }
    const IntervalVector prediction=candidate-inverse*values;budget.check();
    seed=domain;
    for(int i=0;i<n;++i) {
        seed[i]=candidate[i]|prediction[i];
        if(seed[i].is_empty()||!std::isfinite(seed[i].lb())||!std::isfinite(seed[i].ub())) return false;
        seed[i].inflate(1.1,1e-12);
        if(!seed[i].is_subset(domain[i])) return false;
    }
    budget.check();
    const bool admitted=guards(r,graph,seed,true)==GuardState::Admitted;
    budget.check();return admitted;
}
}
extern "C" int32_t pse_ibex_certify(const PseRootRequest* requests,uint32_t count,uint32_t winner,PseRootResult* out,
    double* pl,double* pu,double* el,double* eu,double* ul,double* uu) noexcept {
    if(!requests||!out) return -1;
    *out={6,0,0,0,0,0};
    try {
        if(count==0||count>64||winner>=count||!pl||!pu||!el||!eu||!ul||!uu) return 0;
        const auto& r=requests[winner];
        if(!r.candidate) return 0;
        for(uint32_t k=0;k<count;++k) {
            const auto& q=requests[k];
            if(!q.nodes||!q.edges||!q.residuals||!q.lower||!q.upper||!q.parameters||(q.guard_count&&!q.guards)||
               q.node_count==0||q.node_count>65536||q.unknown_count==0||q.unknown_count>128||
               q.parameter_count>128-q.unknown_count||q.score>=q.node_count||q.tolerance>=q.node_count||
               q.edge_count>1048576||q.guard_count>8192||q.max_cells<2||q.max_cells>1048576||
               !std::isfinite(q.seconds)||q.seconds<=0||q.unknown_count!=r.unknown_count||q.parameter_count!=r.parameter_count||
               q.max_cells!=r.max_cells||q.seconds!=r.seconds||q.cancelled!=r.cancelled||q.cancel_context!=r.cancel_context) return 0;
            for(uint32_t i=0;i<q.unknown_count;++i)
                if(!std::isfinite(q.lower[i])||!std::isfinite(q.upper[i])||q.lower[i]>=q.upper[i]) return 0;
            for(uint32_t j=0;j<q.parameter_count;++j)
                if(!std::isfinite(q.parameters[j])||q.parameters[j]!=r.parameters[j]) return 0;
        }
        for(uint32_t i=0;i<r.unknown_count;++i)
            if(!std::isfinite(r.candidate[i])||r.candidate[i]<r.lower[i]||r.candidate[i]>r.upper[i]) return 0;
        // One account includes chart retries, all winning slabs and every rival.
        Budget budget{r}; budget.check();
        const int dimension=static_cast<int>(r.unknown_count+r.parameter_count);
        const VarSet variables=unknown_variables(r);
        bool chart_boundary=false,any_chart=false;
        // Propose a useful parameter neighborhood before the narrow fallbacks.
        // This changes only the proof proposal: every admitted width still needs
        // uniform Newton, all guards and complete competitive exclusion under
        // this same account. Nearby evaluations can reuse only the proved box.
        for(double width:{1e-5,1e-6,1e-8,1e-10,1e-12}) {
            budget.check();
            // IBEX vectors require a positive size; no placeholder coordinate is
            // inserted into any domain when the authored program has no parameters.
            IntervalVector parameters(static_cast<int>(std::max(1u,r.parameter_count)));
            for(uint32_t j=0;j<r.parameter_count;++j) {
                const double p=r.parameters[j];
                const Interval neighborhood=Interval(p)+Interval(-width,width)*Interval(std::max(1.0,std::abs(p)));
                if(!std::isfinite(neighborhood.lb())||!std::isfinite(neighborhood.ub())||neighborhood.lb()>=p||neighborhood.ub()<=p) return 0;
                parameters[j]=neighborhood;
            }
            const IntervalVector domain=physical_domain(r,parameters);
            IntervalVector seed(domain),existence(dimension),uniqueness(dimension);
            auto graph=std::make_unique<Graph>(r,budget);
            auto prove_seed=[&](double chi) {
                budget.check();
                bool seed_inside=true;
                for(uint32_t i=0;i<r.unknown_count;++i) {
                    seed_inside&=seed[i].is_subset(domain[i]);
                }
                if(!seed_inside) return false;
                if(guards(r,*graph,seed,true,true)!=GuardState::Admitted) { chart_boundary=true; return false; }
                const bool chart_proved=r.parameter_count?
                    inflating_newton(graph->equations->f_ctrs,variables,seed,existence,uniqueness,100,1.0,1.1,chi):
                    inflating_newton(graph->equations->f_ctrs,seed,existence,uniqueness,100,1.0,1.1,chi);
                budget.check(); if(!chart_proved) return false;
                bool valid=true;
                for(uint32_t i=0;i<r.unknown_count;++i)
                    valid&=existence[i].is_interior_subset(uniqueness[i])&&uniqueness[i].is_subset(domain[i])&&
                        existence[i].is_interior_subset(domain[i])&&uniqueness[i].contains(r.candidate[i]);
                for(uint32_t i=r.unknown_count;i<static_cast<uint32_t>(dimension);++i)
                    valid&=domain[i].is_subset(existence[i])&&domain[i].is_subset(uniqueness[i]);
                if(!valid) return false;
                if(guards(r,*graph,existence,true)!=GuardState::Admitted||guards(r,*graph,uniqueness,true)!=GuardState::Admitted) { chart_boundary=true; return false; }
                budget.check();return true;
            };
            bool valid=predictive_seed(r,*graph,domain,budget,seed)&&prove_seed(1e-12);
            for(double chi:{1e-3,1e-5,1e-8,1e-12}) {
                if(valid) break;
                budget.check();
                for(uint32_t i=0;i<r.unknown_count;++i) {
                    // An exact point root can otherwise yield E=U={candidate}.
                    // Supply a local box; IBEX still owns all Newton inflation.
                    seed[i]=Interval(r.candidate[i])+Interval(-chi,chi);
                }
                valid=prove_seed(chi);
            }
            if(!valid) continue;
            any_chart=true;
            const Interval winner_score=scalar_image(*graph->score_value,existence,budget);
            if(winner_score.is_empty()||!std::isfinite(winner_score.lb())||!std::isfinite(winner_score.ub())) throw Boundary();
            const double winner_tolerance=tolerance_upper(*graph,existence,budget);
            std::vector<double> physical_tolerances(count);
            physical_tolerances[winner]=tolerance_upper(*graph,domain,budget);
            // SystemFactory copies constraints; the original DAG remains owned
            // by values. Destroy it before building the constrained replacement.
            graph.reset();
            for(uint32_t k=0;k<count;++k) {
                if(k==winner) continue;
                const auto& rival=requests[k];
                graph=std::make_unique<Graph>(rival,budget);
                physical_tolerances[k]=tolerance_upper(*graph,physical_domain(rival,parameters),budget);
                graph.reset();
            }
            const double winner_threshold=competitive_threshold(winner_score.ub(),winner_tolerance,physical_tolerances[winner]);
            graph=std::make_unique<Graph>(r,budget,true,winner_threshold);
            IntervalVector core(domain); bool complete=true;
            for(uint32_t i=0;i<r.unknown_count&&complete;++i) {
                for(int side=0;side<2&&complete;++side) {
                    IntervalVector slab(core);
                    // Closed slabs include shared chart faces. There are at most 2*n.
                    slab[i]=side==0?Interval(domain[i].lb(),uniqueness[i].lb()):Interval(uniqueness[i].ub(),domain[i].ub());
                    complete=exclude(r,*graph,variables,slab,budget,*out);
                }
                core[i]&=uniqueness[i];
            }
            graph.reset();
            if(!complete) continue;
            for(uint32_t k=0;k<count&&complete;++k) {
                if(k==winner) continue;
                const auto& rival=requests[k];
                const IntervalVector rival_domain=physical_domain(rival,parameters);
                const VarSet rival_variables=unknown_variables(rival);
                const double threshold=competitive_threshold(winner_score.ub(),winner_tolerance,physical_tolerances[k]);
                graph=std::make_unique<Graph>(rival,budget,true,threshold);
                complete=exclude(rival,*graph,rival_variables,rival_domain,budget,*out);
                graph.reset();
            }
            if(!complete) continue;
            for(uint32_t j=0;j<r.parameter_count;++j) { pl[j]=domain[r.unknown_count+j].lb(); pu[j]=domain[r.unknown_count+j].ub(); }
            for(uint32_t i=0;i<r.unknown_count;++i) { el[i]=existence[i].lb(); eu[i]=existence[i].ub(); ul[i]=uniqueness[i].lb(); uu[i]=uniqueness[i].ub(); }
            budget.check(); out->solution=1; out->boundary=0; out->unknown=0; out->pending=0; out->status=0; return 0;
        }
        out->status=any_chart?3:(chart_boundary?5:2);

    } catch(const Resource&) { out->status=4; }
      catch(const TimeOutException&) { out->status=4; }
      catch(const CellLimitException&) { out->status=4; }
      catch(const Boundary&) { out->status=5; }
      catch(const std::bad_alloc&) { out->status=4; }
      catch(const std::invalid_argument&) { out->status=6; }
      catch(...) { out->status=3; }
    return 0;
}

// The Rust owner validates exact unchanged source/domain/selection scope first.
// Existence, regularity, uniform uniqueness and full competitive exclusion remain
// the previous chart's proof. This operation checks the newly encoded guard/order
// obligations on that same uniform uniqueness box, without any covering search.
extern "C" int32_t pse_ibex_promote(const PseRootRequest* request,PseRootResult* out,
    const double* pl,const double* pu,const double* ul,const double* uu) noexcept {
    if(!request||!out) return -1;
    *out={6,0,0,0,0,0};
    try {
        const auto& r=*request;
        if(!pl||!pu||!ul||!uu||!r.nodes||!r.edges||!r.residuals||!r.lower||!r.upper||
           !r.parameters||!r.candidate||(r.guard_count&&!r.guards)||
           r.unknown_count==0||r.unknown_count>128||r.parameter_count>128-r.unknown_count||
           r.node_count==0||r.node_count>65536||r.edge_count>1048576||r.guard_count>8192||
           r.score>=r.node_count||r.tolerance>=r.node_count||!std::isfinite(r.seconds)||r.seconds<=0) return 0;
        Budget budget{r};budget.check();
        IntervalVector box(static_cast<int>(r.unknown_count+r.parameter_count));
        for(uint32_t i=0;i<r.unknown_count;++i) {
            if(!std::isfinite(ul[i])||!std::isfinite(uu[i])||ul[i]>=uu[i]||
               ul[i]<r.lower[i]||uu[i]>r.upper[i]||!std::isfinite(r.candidate[i])||
               r.candidate[i]<ul[i]||r.candidate[i]>uu[i]) return 0;
            box[i]=Interval(ul[i],uu[i]);
        }
        for(uint32_t j=0;j<r.parameter_count;++j) {
            if(!std::isfinite(pl[j])||!std::isfinite(pu[j])||pl[j]>=pu[j]||
               !std::isfinite(r.parameters[j])||r.parameters[j]<=pl[j]||r.parameters[j]>=pu[j]) return 0;
            box[r.unknown_count+j]=Interval(pl[j],pu[j]);
        }
        Graph graph(r,budget);
        if(guards(r,graph,box,true)!=GuardState::Admitted) { out->status=5;return 0; }
        budget.check();out->status=0;out->solution=1;
    } catch(const Resource&) { out->status=4; }
      catch(const Boundary&) { out->status=5; }
      catch(const std::bad_alloc&) { out->status=4; }
      catch(const std::invalid_argument&) { out->status=6; }
      catch(...) { out->status=2; }
    return 0;
}
extern "C" int32_t pse_ibex_connect(const PseRootRequest* request,PseRootResult* out,
    const double* lower,const double* upper) noexcept {
    if(!request||!out) return -1;
    *out={6,0,0,0,0,0};
    try {
        const auto& r=*request;
        if(!lower||!upper||!r.nodes||!r.edges||!r.residuals||!r.lower||!r.upper||
           !r.parameters||!r.candidate||(r.guard_count&&!r.guards)||r.unknown_count==0||
           r.unknown_count>128||r.parameter_count>128-r.unknown_count||r.node_count==0||
           r.node_count>65536||r.edge_count>1048576||r.guard_count>8192||
           r.score>=r.node_count||r.tolerance>=r.node_count||!std::isfinite(r.seconds)||r.seconds<=0) return 0;
        Budget budget{r};budget.check();
        const int dimension=static_cast<int>(r.unknown_count+r.parameter_count);
        IntervalVector domain(dimension),existence(dimension),uniqueness(dimension);
        for(uint32_t i=0;i<r.unknown_count;++i) {
            if(!std::isfinite(lower[i])||!std::isfinite(upper[i])||lower[i]>=upper[i]||
               lower[i]<r.lower[i]||upper[i]>r.upper[i]) return 0;
            domain[i]=Interval(lower[i],upper[i]);
        }
        for(uint32_t j=0;j<r.parameter_count;++j) {
            if(!std::isfinite(r.parameters[j])) return 0;
            domain[r.unknown_count+j]=Interval(r.parameters[j]);
        }
        Graph graph(r,budget);
        if(guards(r,graph,domain,true,true)!=GuardState::Admitted) { out->status=5;return 0; }
        const VarSet variables=unknown_variables(r);
        const bool proved=r.parameter_count?
            inflating_newton(graph.equations->f_ctrs,variables,domain,existence,uniqueness,100,1.0,1.1,1e-12):
            inflating_newton(graph.equations->f_ctrs,domain,existence,uniqueness,100,1.0,1.1,1e-12);
        budget.check();if(!proved) { out->status=2;return 0; }
        for(uint32_t i=0;i<r.unknown_count;++i)
            if(!existence[i].is_interior_subset(domain[i])) { out->status=2;return 0; }
        if(guards(r,graph,existence,true)!=GuardState::Admitted) { out->status=5;return 0; }
        budget.check();out->status=0;out->solution=1;
    } catch(const Resource&) { out->status=4; }
      catch(const Boundary&) { out->status=5; }
      catch(const std::bad_alloc&) { out->status=4; }
      catch(const std::invalid_argument&) { out->status=6; }
      catch(...) { out->status=2; }
    return 0;
}

// Root-sheet proof only: uniform regularity/eligibility and common-root bridges.
// Competitive exclusion belongs to the independently certified endpoint charts;
// this operation makes no selected-winner claim on intermediate parameter cells.
extern "C" int32_t pse_ibex_connect_chain(const PseRootRequest* request,PseChartChainResult* out,
    const double* origin,const double* previous_pl,const double* previous_pu,
    const double* previous_el,const double* previous_eu,const double* previous_ul,const double* previous_uu,
    const double* next_pl,const double* next_pu,const double* next_el,const double* next_eu,
    const double* next_ul,const double* next_uu) noexcept {
    if(!request||!out) return -1;
    *out={6,0,0,0};
    try {
        const auto& r=*request;
        if(!origin||!previous_pl||!previous_pu||!previous_el||!previous_eu||!previous_ul||!previous_uu||
           !next_pl||!next_pu||!next_el||!next_eu||!next_ul||!next_uu||
           !r.nodes||!r.edges||!r.residuals||!r.lower||!r.upper||!r.parameters||!r.candidate||
           (r.guard_count&&!r.guards)||r.unknown_count==0||r.unknown_count>128||
           r.parameter_count>128-r.unknown_count||r.node_count==0||r.node_count>65536||
           r.edge_count>1048576||r.guard_count>8192||r.score>=r.node_count||r.tolerance>=r.node_count||
           r.max_cells<2||r.max_cells>1048576||!std::isfinite(r.seconds)||r.seconds<=0) return 0;
        Budget budget{r};budget.check();
        const int dimension=static_cast<int>(r.unknown_count+r.parameter_count);
        struct Chart {
            IntervalVector box,existence,uniqueness;
            explicit Chart(int n):box(n),existence(n),uniqueness(n){}
        };
        Chart current(dimension),endpoint(dimension);
        for(uint32_t i=0;i<r.unknown_count;++i) {
            if(!std::isfinite(r.lower[i])||!std::isfinite(r.upper[i])||r.lower[i]>=r.upper[i]||
               !std::isfinite(previous_el[i])||!std::isfinite(previous_eu[i])||
               !std::isfinite(previous_ul[i])||!std::isfinite(previous_uu[i])||
               !std::isfinite(next_el[i])||!std::isfinite(next_eu[i])||
               !std::isfinite(next_ul[i])||!std::isfinite(next_uu[i])||
               previous_ul[i]>=previous_el[i]||previous_el[i]>previous_eu[i]||previous_eu[i]>=previous_uu[i]||
               next_ul[i]>=next_el[i]||next_el[i]>next_eu[i]||next_eu[i]>=next_uu[i]||
               previous_ul[i]<r.lower[i]||previous_uu[i]>r.upper[i]||
               next_ul[i]<r.lower[i]||next_uu[i]>r.upper[i]) return 0;
            current.box[i]=endpoint.box[i]=Interval(r.lower[i],r.upper[i]);
            current.existence[i]=Interval(previous_el[i],previous_eu[i]); current.uniqueness[i]=Interval(previous_ul[i],previous_uu[i]);
            endpoint.existence[i]=Interval(next_el[i],next_eu[i]); endpoint.uniqueness[i]=Interval(next_ul[i],next_uu[i]);
        }
        std::vector<Interval> margins; margins.reserve(r.parameter_count);
        for(uint32_t j=0;j<r.parameter_count;++j) {
            if(!std::isfinite(origin[j])||!std::isfinite(r.parameters[j])||
               !std::isfinite(previous_pl[j])||!std::isfinite(previous_pu[j])||
               !std::isfinite(next_pl[j])||!std::isfinite(next_pu[j])||
               origin[j]<=previous_pl[j]||origin[j]>=previous_pu[j]||
               r.parameters[j]<=next_pl[j]||r.parameters[j]>=next_pu[j]) return 0;
            const int k=static_cast<int>(r.unknown_count+j);
            current.box[k]=current.existence[k]=current.uniqueness[k]=Interval(previous_pl[j],previous_pu[j]);
            endpoint.box[k]=endpoint.existence[k]=endpoint.uniqueness[k]=Interval(next_pl[j],next_pu[j]);
            const double margin=std::min({(Interval(origin[j])-Interval(previous_pl[j])).lb(),
                (Interval(previous_pu[j])-Interval(origin[j])).lb(),
                (Interval(r.parameters[j])-Interval(next_pl[j])).lb(),
                (Interval(next_pu[j])-Interval(r.parameters[j])).lb()});
            if(!(margin>0.0)) { out->status=5;return 0; }
            margins.push_back(Interval(-margin,margin)/Interval(2.0));
        }
        Graph graph(r,budget); const VarSet variables=unknown_variables(r);
        auto proof_cell=[&]() {
            budget.check();if(budget.cells>=r.max_cells) throw Resource();
            ++budget.cells;out->proof_cells=budget.cells;
        };
        auto parameters_at=[&](double t) {
            std::vector<double> p;p.reserve(r.parameter_count);
            for(uint32_t j=0;j<r.parameter_count;++j) {
                if(t==0.0) p.push_back(origin[j]);
                else if(t==1.0) p.push_back(r.parameters[j]);
                else p.push_back((Interval(origin[j])+Interval(t)*(Interval(r.parameters[j])-Interval(origin[j]))).mid());
            }
            return p;
        };
        auto common_root=[&](const Chart& a,const Chart& b,double t) {
            proof_cell();const auto parameters=parameters_at(t);
            IntervalVector domain(dimension),existence(dimension),uniqueness(dimension);
            for(uint32_t i=0;i<r.unknown_count;++i) {
                domain[i]=a.uniqueness[i]&b.uniqueness[i];
                if(domain[i].is_empty()||domain[i].diam()<=0) return false;
            }
            for(uint32_t j=0;j<r.parameter_count;++j) {
                const int k=static_cast<int>(r.unknown_count+j); const double p=parameters[j];
                if(p<=a.box[k].lb()||p>=a.box[k].ub()||p<=b.box[k].lb()||p>=b.box[k].ub()) return false;
                domain[k]=Interval(p);
            }
            if(guards(r,graph,domain,true,true)!=GuardState::Admitted) return false;
            const bool proved=r.parameter_count?
                inflating_newton(graph.equations->f_ctrs,variables,domain,existence,uniqueness,100,1.0,1.1,1e-12):
                inflating_newton(graph.equations->f_ctrs,domain,existence,uniqueness,100,1.0,1.1,1e-12);
            budget.check();if(!proved) return false;
            for(uint32_t i=0;i<r.unknown_count;++i) if(!existence[i].is_interior_subset(domain[i])) return false;
            return guards(r,graph,existence,true)==GuardState::Admitted;
        };
        enum class ChartSeed { PreviousExistence, EndpointExistenceHull, EndpointUniquenessHull };
        auto uniform_chart=[&](const Interval& path,Chart& chart) {
            auto attempt=[&](ChartSeed proposal) {
                proof_cell();IntervalVector seed(dimension);
                for(uint32_t i=0;i<r.unknown_count;++i) {
                    chart.box[i]=Interval(r.lower[i],r.upper[i]);
                    switch(proposal) {
                        case ChartSeed::PreviousExistence: seed[i]=current.existence[i]; break;
                        case ChartSeed::EndpointExistenceHull: seed[i]=current.existence[i]|endpoint.existence[i]; break;
                        case ChartSeed::EndpointUniquenessHull: seed[i]=current.uniqueness[i]|endpoint.uniqueness[i]; break;
                    }
                    if(!seed[i].is_subset(chart.box[i])) return false;
                }
                for(uint32_t j=0;j<r.parameter_count;++j) {
                    const int k=static_cast<int>(r.unknown_count+j);
                    chart.box[k]=seed[k]=Interval(origin[j])+path*(Interval(r.parameters[j])-Interval(origin[j]))+margins[j];
                    if(!std::isfinite(seed[k].lb())||!std::isfinite(seed[k].ub())||seed[k].diam()<=0) return false;
                }
                if(guards(r,graph,seed,true,true)!=GuardState::Admitted) return false;
                const bool proved=r.parameter_count?
                    inflating_newton(graph.equations->f_ctrs,variables,seed,chart.existence,chart.uniqueness,100,1.0,1.1,1e-12):
                    inflating_newton(graph.equations->f_ctrs,seed,chart.existence,chart.uniqueness,100,1.0,1.1,1e-12);
                budget.check();if(!proved) return false;
                for(uint32_t i=0;i<r.unknown_count;++i) if(!chart.existence[i].is_interior_subset(chart.uniqueness[i])||
                    !chart.uniqueness[i].is_subset(chart.box[i])||!chart.existence[i].is_interior_subset(chart.box[i])) return false;
                for(uint32_t j=0;j<r.parameter_count;++j) {
                    const int k=static_cast<int>(r.unknown_count+j);
                    if(!chart.box[k].is_subset(chart.existence[k])||!chart.box[k].is_subset(chart.uniqueness[k])) return false;
                }
                return guards(r,graph,chart.existence,true)==GuardState::Admitted&&guards(r,graph,chart.uniqueness,true)==GuardState::Admitted;
            };
            // The old root enclosure is the cheap first proposal. If parameter
            // movement defeats inflation from that tiny seed, the hull of the
            // two certified endpoint enclosures is another Newton proposal. If
            // the root barely moves, that hull can still be too small to absorb
            // interval dependency; the endpoint uniqueness hull supplies a
            // broader proposal without granting existence or selection evidence.
            // Every attempt consumes the same proof account and must establish
            // every guard, domain, existence and uniqueness check above.
            return attempt(ChartSeed::PreviousExistence)||
                attempt(ChartSeed::EndpointExistenceHull)||
                attempt(ChartSeed::EndpointUniquenessHull);
        };
        // IBEX owns outward interval arithmetic and bisection. A finite stack covers
        // [0,1] left-to-right; failed proof cells consume this same account. Successful
        // charts replace the previous one, so no unaccounted retained chain appears.
        std::vector<Interval> pending{Interval(0.0,1.0)};
        while(!pending.empty()) {
            budget.check();const Interval path=pending.back();pending.pop_back();Chart chart(dimension);
            bool proved=uniform_chart(path,chart)&&common_root(current,chart,path.lb());
            if(proved&&path.ub()==1.0) proved=common_root(chart,endpoint,1.0);
            if(proved) {
                ++out->charts;++out->connections;
                if(path.ub()==1.0) ++out->connections;
                current=std::move(chart);continue;
            }
            if(!path.is_bisectable()) { out->status=2;return 0; }
            const auto halves=path.bisect();pending.push_back(halves.second);pending.push_back(halves.first);
        }
        budget.check();out->status=0;
    } catch(const Resource&) { out->status=4; }
      catch(const Boundary&) { out->status=5; }
      catch(const std::bad_alloc&) { out->status=4; }
      catch(const std::invalid_argument&) { out->status=6; }
      catch(...) { out->status=2; }
    return 0;
}

namespace {
bool point_request(const PseRootRequest& r) {
    return r.nodes&&r.edges&&r.residuals&&r.lower&&r.upper&&r.parameters&&r.candidate&&
        (!r.guard_count||r.guards)&&r.unknown_count>0&&r.unknown_count<=128&&
        r.parameter_count<=128-r.unknown_count&&r.node_count>0&&r.node_count<=65536&&
        r.edge_count<=1048576&&r.guard_count<=8192&&r.score<r.node_count&&r.tolerance<r.node_count&&
        r.max_cells>0&&r.max_cells<=1048576&&std::isfinite(r.seconds)&&r.seconds>0;
}
void point_cell(Budget& budget,PseRootResult& out) {
    budget.check();if(budget.cells>=budget.r.max_cells) throw Resource();
    ++budget.cells;out.cells=budget.cells;
}
// Root existence is established anew at singleton parameters by the bounded
// library operator. The immutable original chart still owns selected-root meaning.
bool fixed_root(const PseRootRequest& r,Graph& graph,Budget& budget,PseRootResult& out,
    const double* el,const double* eu,const double* ul,const double* uu,
    IntervalVector& local,IntervalVector& hull) {
    const int dimension=static_cast<int>(r.unknown_count+r.parameter_count);
    IntervalVector seed(dimension),unique(dimension),proved_unique(dimension);
    for(uint32_t i=0;i<r.unknown_count;++i) {
        if(!std::isfinite(el[i])||!std::isfinite(eu[i])||!std::isfinite(ul[i])||!std::isfinite(uu[i])||
           el[i]>eu[i]||ul[i]>=el[i]||eu[i]>=uu[i]||ul[i]<r.lower[i]||uu[i]>r.upper[i]||
           !std::isfinite(r.candidate[i])||r.candidate[i]<ul[i]||r.candidate[i]>uu[i]) return false;
        seed[i]=Interval(el[i],eu[i]);unique[i]=Interval(ul[i],uu[i]);
    }
    for(uint32_t j=0;j<r.parameter_count;++j) {
        if(!std::isfinite(r.parameters[j])) return false;
        seed[r.unknown_count+j]=unique[r.unknown_count+j]=Interval(r.parameters[j]);
    }
    if(guards(r,graph,seed,true,true)!=GuardState::Admitted) throw Boundary();
    point_cell(budget,out);
    const VarSet variables=unknown_variables(r);
    const bool proved=r.parameter_count?
        inflating_newton(graph.equations->f_ctrs,variables,seed,local,proved_unique,64,1.0,1.1,1e-12):
        inflating_newton(graph.equations->f_ctrs,seed,local,proved_unique,64,1.0,1.1,1e-12);
    budget.check();if(!proved||local.is_empty()) return false;
    for(uint32_t i=0;i<r.unknown_count;++i) {
        if(!local[i].is_interior_subset(unique[i])||!std::isfinite(local[i].lb())||!std::isfinite(local[i].ub())) return false;
    }
    // Only after the newly proved root lies inside the retained uniqueness region
    // do both enclosures identify the same root. Their intersection preserves it.
    local &= seed;
    if(local.is_empty()) return false;
    hull=local;
    for(uint32_t i=0;i<r.unknown_count;++i) {
        hull[i] |= Interval(r.candidate[i]);
        if(!hull[i].is_subset(unique[i])) return false;
    }
    if(guards(r,graph,local,true)!=GuardState::Admitted||
       guards(r,graph,hull,true)!=GuardState::Admitted) throw Boundary();
    budget.check();return true;
}
IntervalMatrix root_inverse(const PseRootRequest& r,Graph& graph,const IntervalVector& box,
    Budget& budget,PseRootResult& out) {
    point_cell(budget,out);
    const auto jacobian=graph.equations->f_ctrs.jacobian(box);budget.check();
    const int n=static_cast<int>(r.unknown_count);
    IntervalMatrix coefficients(n,n),inverse(n,n);
    for(int i=0;i<n;++i) for(int j=0;j<n;++j) coefficients[i][j]=jacobian[i][j];
    neumaier_inverse(coefficients,inverse);budget.check();return inverse;
}
}
extern "C" int32_t pse_ibex_refine_point(const PseRootRequest* request,PseRootResult* out,
    const double* existence_lower,const double* existence_upper,
    const double* uniqueness_lower,const double* uniqueness_upper,
    const double* unknown_scales,const double* row_scales,
    double* root_lower,double* root_upper,double* inverse_norm_upper) noexcept {
    if(!request||!out) return -1;
    *out={6,0,0,0,0,0};
    try {
        const auto& r=*request;
        if(!point_request(r)||!existence_lower||!existence_upper||!uniqueness_lower||!uniqueness_upper||
           !unknown_scales||!row_scales||!root_lower||!root_upper||!inverse_norm_upper) return 0;
        for(uint32_t i=0;i<r.unknown_count;++i)
            if(!std::isfinite(unknown_scales[i])||unknown_scales[i]<=0||
               !std::isfinite(row_scales[i])||row_scales[i]<=0) return 0;
        Budget budget{r};budget.check();Graph graph(r,budget);
        const int dimension=static_cast<int>(r.unknown_count+r.parameter_count);
        IntervalVector local(dimension),hull(dimension);
        if(!fixed_root(r,graph,budget,*out,existence_lower,existence_upper,
            uniqueness_lower,uniqueness_upper,local,hull)) {out->status=2;return 0;}
        const auto inverse=root_inverse(r,graph,hull,budget,*out);
        Interval bound(0.0);
        for(uint32_t i=0;i<r.unknown_count;++i) {
            Interval row(0.0);
            for(uint32_t j=0;j<r.unknown_count;++j)
                row+=abs(inverse[i][j])*Interval(row_scales[j])/Interval(unknown_scales[i]);
            if(row.is_empty()||!std::isfinite(row.ub())) {out->status=2;return 0;}
            bound=max(bound,row);
        }
        if(bound.is_empty()||!std::isfinite(bound.ub())||bound.ub()<=0) {out->status=2;return 0;}
        for(uint32_t i=0;i<r.unknown_count;++i) {root_lower[i]=local[i].lb();root_upper[i]=local[i].ub();}
        *inverse_norm_upper=bound.ub();budget.check();out->solution=1;out->status=0;
    } catch(const Resource&) {out->status=4;}
      catch(const Boundary&) {out->status=5;}
      catch(const std::bad_alloc&) {out->status=4;}
      catch(const std::invalid_argument&) {out->status=6;}
      catch(...) {out->status=2;}
    return 0;
}
// The actual IFT action uses a genuinely refined fixed-parameter root enclosure.
extern "C" int32_t pse_ibex_enclose_action(const PseRootRequest* request,PseRootResult* out,
    const double* existence_lower,const double* existence_upper,
    const double* uniqueness_lower,const double* uniqueness_upper,const double* direction,
    double* action_lower,double* action_upper) noexcept {
    if(!request||!out) return -1;
    *out={6,0,0,0,0,0};
    try {
        const auto& r=*request;
        if(!point_request(r)||!existence_lower||!existence_upper||!uniqueness_lower||!uniqueness_upper||
           !direction||!action_lower||!action_upper) return 0;
        Budget budget{r};budget.check();
        IntervalVector box(static_cast<int>(r.unknown_count+r.parameter_count));
        for(uint32_t j=0;j<r.parameter_count;++j) if(!std::isfinite(direction[j])) return 0;
        Graph graph(r,budget);IntervalVector hull(box.size());
        if(!fixed_root(r,graph,budget,*out,existence_lower,existence_upper,
            uniqueness_lower,uniqueness_upper,box,hull)) {out->status=2;return 0;}
        const auto inverse=root_inverse(r,graph,box,budget,*out);
        const auto jacobian=graph.equations->f_ctrs.jacobian(box);budget.check();
        const int n=static_cast<int>(r.unknown_count);
        IntervalVector rhs(n);
        for(int i=0;i<n;++i) {
            rhs[i]=Interval(0.0);
            for(uint32_t j=0;j<r.parameter_count;++j) rhs[i]-=jacobian[i][n+static_cast<int>(j)]*Interval(direction[j]);
        }
        const auto action=inverse*rhs;
        for(int i=0;i<n;++i) {
            if(action[i].is_empty()||!std::isfinite(action[i].lb())||!std::isfinite(action[i].ub())) {out->status=2;return 0;}
            action_lower[i]=action[i].lb();action_upper[i]=action[i].ub();
        }
        budget.check();out->solution=1;out->status=0;
    } catch(const Resource&) {out->status=4;}
      catch(const Boundary&) {out->status=5;}
      catch(const std::bad_alloc&) {out->status=4;}
      catch(const std::invalid_argument&) {out->status=6;}
      catch(...) {out->status=2;}
    return 0;
}
