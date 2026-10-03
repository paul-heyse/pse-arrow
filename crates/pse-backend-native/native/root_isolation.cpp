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
struct PseRootResult {
    uint32_t status; uint64_t cells, solution, boundary, unknown, pending;
};
// status: 0 unique selection, 2 chart, 3 coverage, 4 resource, 5 boundary, 6 unsupported.
int32_t pse_ibex_certify(const PseRootRequest*, uint32_t count, uint32_t winner,
    PseRootResult*, double* parameter_lower,
    double* parameter_upper, double* existence_lower, double* existence_upper,
    double* uniqueness_lower, double* uniqueness_upper) noexcept;
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
struct GuardContractor : Ctc {
    const PseRootRequest& r; Graph& graph; Budget& budget;
    CtcHC4 hc4;
    std::unique_ptr<CtcNewton> newton;
    LinearizerXTaylor linearizer; CtcPolytopeHull polytope; CtcCompo lp_hc4; CtcFixPoint lp;
    GuardContractor(const PseRootRequest& req,Graph& g,Budget& b,const VarSet& variables)
        : Ctc(g.residual->nb_var),r(req),graph(g),budget(b),hc4(g.residual->ctrs,0.01),
          newton(req.parameter_count?std::make_unique<CtcNewton>(g.equations->f_ctrs,variables,5e8):
              std::make_unique<CtcNewton>(g.equations->f_ctrs,5e8)),
          linearizer(*g.residual),polytope(linearizer,100,1),lp_hc4(polytope,hc4),lp(lp_hc4) {}
    void add_property(const IntervalVector& box,BoxProperties& prop) override {
        hc4.add_property(box,prop); newton->add_property(box,prop); lp.add_property(box,prop);
    }
    void contract(IntervalVector& box) override { ContractContext context(box); contract(box,context); }
    void contract(IntervalVector& box,ContractContext& context) override {
        budget.check();
        if(guards(r,graph,box,false)==GuardState::Invalid) { box.set_empty(); return; }
        // Library-owned existential HC4 contraction preserves the admitted
        // portion of crossing log/sqrt/nonzero domains.
        try { hc4.contract(box,context); } catch(const std::bad_alloc&) { throw; } catch(...) { throw Boundary(); }
        budget.check();
        if(box.is_empty()) return;
        // Every C1 mean-value premise is checked before Newton and Taylor/LP.
        // Crossing cells proceed through the native Solver's ordinary bisection.
        if(guards(r,graph,box,true,true)==GuardState::Admitted) {
            // Parametric interval Newton contracts unknowns only; it does not
            // turn a numerical proposal into root existence or selection evidence.
            try { newton->contract(box,context); } catch(const std::bad_alloc&) { throw; } catch(...) { throw Boundary(); }
            budget.check();
            if(box.is_empty()) return;
            try { lp.contract(box,context); } catch(const std::bad_alloc&) { throw; } catch(...) { throw Boundary(); }
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
    LargestFirst bisector(precision); CellStack buffer;
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
        for(double width:{1e-8,1e-10,1e-12}) {
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
            bool valid=false;
            for(double chi:{1e-5,1e-8,1e-12}) {
                budget.check();
                bool seed_inside=true;
                for(uint32_t i=0;i<r.unknown_count;++i) {
                    // An exact point root can otherwise yield E=U={candidate}.
                    // Supply a local box; IBEX still owns all Newton inflation.
                    seed[i]=Interval(r.candidate[i])+Interval(-chi,chi);
                    seed_inside&=seed[i].is_subset(domain[i]);
                }
                if(!seed_inside) continue;
                if(guards(r,*graph,seed,true,true)!=GuardState::Admitted) { chart_boundary=true; continue; }
                const bool chart_proved=r.parameter_count?
                    inflating_newton(graph->equations->f_ctrs,variables,seed,existence,uniqueness,100,1.0,1.1,chi):
                    inflating_newton(graph->equations->f_ctrs,seed,existence,uniqueness,100,1.0,1.1,chi);
                budget.check(); if(!chart_proved) continue;
                valid=true;
                for(uint32_t i=0;i<r.unknown_count;++i)
                    valid&=existence[i].is_interior_subset(uniqueness[i])&&uniqueness[i].is_subset(domain[i])&&
                        existence[i].is_interior_subset(domain[i])&&uniqueness[i].contains(r.candidate[i]);
                for(uint32_t i=r.unknown_count;i<static_cast<uint32_t>(dimension);++i)
                    valid&=domain[i].is_subset(existence[i])&&domain[i].is_subset(uniqueness[i]);
                if(!valid) continue;
                if(guards(r,*graph,existence,true)!=GuardState::Admitted||guards(r,*graph,uniqueness,true)!=GuardState::Admitted) { chart_boundary=true; valid=false; continue; }
                break;
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
