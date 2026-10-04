// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
#include "pse_ipopt_sequence.h"
#include "IpIpoptApplication.hpp"
#include "IpTNLP.hpp"
#include "IpOptionsList.hpp"
#include <algorithm>
#include <cmath>
#include <cstring>
#include <exception>
#include <memory>
#include <new>
#include <set>
#include <stdexcept>
#include <utility>
#include <vector>
static_assert(sizeof(Ipopt::Index) == sizeof(int32_t), "Ipopt sequence requires 32-bit indices");
static_assert(sizeof(Ipopt::Number) == sizeof(double), "Ipopt sequence requires double precision");
namespace {
struct Terminal final : std::exception {
    const char* what() const noexcept override { return "terminal callback or execution scope failure"; }
};
void message(char* output, size_t capacity, const char* source) noexcept {
    if (!output || !capacity) return;
    size_t count = std::min(std::strlen(source), capacity - 1);
    std::memcpy(output, source, count); output[count] = '\0';
}
void require(bool value, const char* reason) { if (!value) throw std::invalid_argument(reason); }
void accepted(bool value) { if (!value) throw std::invalid_argument("Ipopt refused declared option"); }
void bounds(int32_t length, const double* lower, const double* upper) {
    require(length == 0 || (lower && upper), "missing bounds");
    for (int32_t i = 0; i < length; ++i) require(!std::isnan(lower[i]) && !std::isnan(upper[i])
        && lower[i] <= upper[i] && lower[i] != INFINITY && upper[i] != -INFINITY, "invalid bounds");
}
void coo(int32_t count, int32_t nrows, int32_t ncolumns, const int32_t* rows, const int32_t* columns, bool triangular) {
    require(count == 0 || (rows && columns), "missing COO");
    std::set<std::pair<int32_t, int32_t>> seen;
    for (int32_t k = 0; k < count; ++k) {
        require(rows[k] >= 0 && rows[k] < nrows && columns[k] >= 0 && columns[k] < ncolumns
            && (!triangular || rows[k] >= columns[k]), "invalid COO index");
        require(seen.emplace(rows[k], columns[k]).second, "duplicate COO index");
    }
}
class Sequence final : public Ipopt::TNLP {
public:
    int32_t n, m, jnnz, hnnz, terminal = 0;
    bool started = false, reusable = false, returned = false, finalized = false, has_duals = false;
    pse_ipopt_sequence_callbacks callbacks;
    pse_ipopt_sequence_result result{};
    std::vector<double> lower, upper, row_lower, row_upper, initial, initial_lower, initial_upper, initial_row;
    std::vector<int32_t> jr, jc, hr, hc;
    std::vector<double> primal, lower_dual, upper_dual, constraints, row_dual;
    Sequence(int32_t n_, int32_t m_, int32_t j_, int32_t h_, const pse_ipopt_sequence_callbacks& callbacks_)
        : n(n_), m(m_), jnnz(j_), hnnz(h_), callbacks(callbacks_), initial(n), initial_lower(n), initial_upper(n),
          initial_row(m), primal(n), lower_dual(n), upper_dual(n), constraints(m), row_dual(m) {}
    void poll() {
        if (terminal) throw Terminal{};
        switch (callbacks.poll(callbacks.user)) {
            case 0: return;
            case 1: terminal = PSE_IPOPT_SEQUENCE_CANCEL; break;
            case 2: terminal = PSE_IPOPT_SEQUENCE_DEADLINE; break;
            case 3: terminal = PSE_IPOPT_SEQUENCE_ABANDON; break;
            default: terminal = PSE_IPOPT_SEQUENCE_TERMINAL; break;
        }
        finalized = false; reusable = false;
        throw Terminal{};
    }
    template<class Function> bool evaluate(Function function) {
        poll(); int32_t status = function();
        if (status != 0 && status != 1) { terminal = PSE_IPOPT_SEQUENCE_TERMINAL; throw Terminal{}; }
        poll(); return status == 0;
    }
    bool get_nlp_info(Ipopt::Index& variables, Ipopt::Index& rows, Ipopt::Index& jacobian,
        Ipopt::Index& hessian, IndexStyleEnum& style) override {
        poll(); variables = n; rows = m; jacobian = jnnz; hessian = hnnz; style = C_STYLE; return true;
    }
    bool get_bounds_info(Ipopt::Index, double* lo, double* up, Ipopt::Index, double* rlo, double* rup) override {
        poll(); std::copy(lower.begin(), lower.end(), lo); std::copy(upper.begin(), upper.end(), up);
        if (m) { std::copy(row_lower.begin(), row_lower.end(), rlo); std::copy(row_upper.begin(), row_upper.end(), rup); }
        return true;
    }
    bool get_starting_point(Ipopt::Index, bool init_x, double* x, bool init_z, double* zl, double* zu,
        Ipopt::Index, bool init_lambda, double* lambda) override {
        poll();
        if ((init_z || init_lambda) && !has_duals) return false;
        if (init_x) std::copy(initial.begin(), initial.end(), x);
        if (init_z) { std::copy(initial_lower.begin(), initial_lower.end(), zl); std::copy(initial_upper.begin(), initial_upper.end(), zu); }
        if (init_lambda && m) std::copy(initial_row.begin(), initial_row.end(), lambda);
        return true;
    }
    bool eval_f(Ipopt::Index, const double* x, bool, double& value) override {
        return evaluate([&] { return callbacks.objective(n, x, &value, callbacks.user); });
    }
    bool eval_grad_f(Ipopt::Index, const double* x, bool, double* out) override {
        return evaluate([&] { return callbacks.gradient(n, x, out, callbacks.user); });
    }
    bool eval_g(Ipopt::Index, const double* x, bool, Ipopt::Index, double* out) override {
        if (!m) { poll(); return true; }
        return evaluate([&] { return callbacks.constraints(n, m, x, out, callbacks.user); });
    }
    bool eval_jac_g(Ipopt::Index, const double* x, bool, Ipopt::Index, Ipopt::Index,
        Ipopt::Index* rows, Ipopt::Index* columns, double* out) override {
        if (!out) { poll(); std::copy(jr.begin(), jr.end(), rows); std::copy(jc.begin(), jc.end(), columns); return true; }
        if (!jnnz) { poll(); return true; }
        return evaluate([&] { return callbacks.jacobian(n, jnnz, x, out, callbacks.user); });
    }
    bool eval_h(Ipopt::Index, const double* x, bool, double objective_multiplier, Ipopt::Index,
        const double* lambda, bool, Ipopt::Index, Ipopt::Index* rows, Ipopt::Index* columns, double* out) override {
        if (!out) { poll(); std::copy(hr.begin(), hr.end(), rows); std::copy(hc.begin(), hc.end(), columns); return true; }
        if (!hnnz) { poll(); return true; }
        return evaluate([&] { return callbacks.hessian(n, m, hnnz, x, objective_multiplier, lambda, out, callbacks.user); });
    }
    bool intermediate_callback(Ipopt::AlgorithmMode mode, Ipopt::Index iteration, double objective,
        double inf_pr, double inf_du, double mu, double step, double regularization,
        double alpha_du, double alpha_pr, Ipopt::Index trials, const Ipopt::IpoptData* data,
        Ipopt::IpoptCalculatedQuantities* quantities) override {
        poll(); result.iterations = iteration; result.iterations_available = 1;
        if (std::isfinite(mu) && mu >= 0.) { result.barrier = mu; result.barrier_available = 1; }
        if (callbacks.iteration) {
            std::vector<double> current(n), lagrangian(n);
            bool has_current = get_curr_iterate(data, quantities, false, n, current.data(), nullptr, nullptr, m, nullptr, nullptr);
            bool has_lagrangian = get_curr_violations(data, quantities, false, n, nullptr, nullptr, nullptr, nullptr,
                lagrangian.data(), m, nullptr, nullptr);
            int32_t status = callbacks.iteration(static_cast<int32_t>(mode), iteration, objective, inf_pr, inf_du,
                mu, step, regularization, alpha_du, alpha_pr, trials, n,
                has_current ? current.data() : nullptr, has_lagrangian ? lagrangian.data() : nullptr, callbacks.user);
            if (status != 0) { terminal = PSE_IPOPT_SEQUENCE_TERMINAL; finalized = false; reusable = false; throw Terminal{}; }
            poll();
        }
        return true;
    }
    void finalize_solution(Ipopt::SolverReturn status, Ipopt::Index, const double* x, const double* zl,
        const double* zu, Ipopt::Index, const double* rows, const double* lambda, double objective,
        const Ipopt::IpoptData*, Ipopt::IpoptCalculatedQuantities*) override {
        if (terminal) return;
        std::copy(x, x + n, primal.begin()); std::copy(zl, zl + n, lower_dual.begin());
        std::copy(zu, zu + n, upper_dual.begin());
        if (m) { std::copy(rows, rows + m, constraints.begin()); std::copy(lambda, lambda + m, row_dual.begin()); }
        result.solver_status = static_cast<int32_t>(status); result.objective = objective; finalized = true;
    }
};
}
struct pse_ipopt_sequence_handle {
    Ipopt::SmartPtr<Sequence> problem;
    Ipopt::SmartPtr<Ipopt::IpoptApplication> application;
};
namespace {
template<class Function> int32_t shield(pse_ipopt_sequence_handle* h, char* error, size_t capacity, Function function) noexcept {
    message(error, capacity, "");
    try { function(); return PSE_IPOPT_SEQUENCE_OK; }
    catch (const Terminal& exception) { message(error, capacity, exception.what());
        return h && Ipopt::IsValid(h->problem) && h->problem->terminal ? h->problem->terminal : PSE_IPOPT_SEQUENCE_TERMINAL; }
    catch (const std::bad_alloc&) { message(error, capacity, "native allocation failed"); return PSE_IPOPT_SEQUENCE_MEMORY; }
    catch (const std::invalid_argument& exception) { message(error, capacity, exception.what()); return PSE_IPOPT_SEQUENCE_INVALID; }
    catch (const std::exception& exception) { message(error, capacity, exception.what()); return PSE_IPOPT_SEQUENCE_EXCEPTION; }
    catch (...) { message(error, capacity, "unknown foreign exception"); return PSE_IPOPT_SEQUENCE_EXCEPTION; }
}
void option_allowed(pse_ipopt_sequence_handle* h, const char* key, bool numeric_restart = false) {
    require(h && key, "missing option handle or key");
    bool restart = !std::strcmp(key, "mu_init") || !std::strcmp(key, "warm_start_bound_push")
        || !std::strcmp(key, "warm_start_bound_frac") || !std::strcmp(key, "warm_start_slack_bound_push")
        || !std::strcmp(key, "warm_start_slack_bound_frac") || !std::strcmp(key, "warm_start_mult_bound_push");
    require(!h->problem->started || (numeric_restart && restart), "structural options are immutable after first optimize");
    require(std::strcmp(key, "warm_start_same_structure") && std::strcmp(key, "warm_start_init_point")
        && std::strcmp(key, "max_wall_time") && std::strcmp(key, "option_file_name"), "sequence-owned option");
}
}
extern "C" int32_t pse_ipopt_sequence_create(int32_t n, int32_t m, int32_t jnnz, int32_t hnnz,
    const double* lower, const double* upper, const double* row_lower, const double* row_upper,
    const int32_t* jr, const int32_t* jc, const int32_t* hr, const int32_t* hc,
    const pse_ipopt_sequence_callbacks* callbacks, pse_ipopt_sequence_handle** out, char* error, size_t capacity) noexcept {
    if (out) *out = nullptr;
    return shield(nullptr, error, capacity, [&] {
        require(out && callbacks, "missing create inputs");
        require(n > 0 && m >= 0 && jnnz >= 0 && hnnz >= 0 && (m || !jnnz), "invalid dimensions");
        require(callbacks->objective && callbacks->gradient && callbacks->poll && (!m || callbacks->constraints)
            && (!jnnz || callbacks->jacobian) && (!hnnz || callbacks->hessian), "missing callback");
        bounds(n, lower, upper); bounds(m, row_lower, row_upper);
        coo(jnnz, m, n, jr, jc, false); coo(hnnz, n, n, hr, hc, true);
        auto h = std::make_unique<pse_ipopt_sequence_handle>();
        h->problem = new Sequence(n, m, jnnz, hnnz, *callbacks);
        h->problem->lower.assign(lower, lower + n); h->problem->upper.assign(upper, upper + n);
        if (m) { h->problem->row_lower.assign(row_lower, row_lower + m); h->problem->row_upper.assign(row_upper, row_upper + m); }
        if (jnnz) { h->problem->jr.assign(jr, jr + jnnz); h->problem->jc.assign(jc, jc + jnnz); }
        if (hnnz) { h->problem->hr.assign(hr, hr + hnnz); h->problem->hc.assign(hc, hc + hnnz); }
        h->application = IpoptApplicationFactory();
        require(Ipopt::IsValid(h->application), "Ipopt application allocation failed");
        // Do not read undeclared host ipopt.opt files.
        accepted(h->application->Options()->SetStringValue("option_file_name", ""));
        *out = h.release();
    });
}
extern "C" int32_t pse_ipopt_sequence_set_string(pse_ipopt_sequence_handle* h, const char* key, const char* value,
    char* error, size_t capacity) noexcept {
    return shield(h, error, capacity, [&] { option_allowed(h, key); require(value, "null option value");
        accepted(h->application->Options()->SetStringValue(key, value)); });
}
extern "C" int32_t pse_ipopt_sequence_set_double(pse_ipopt_sequence_handle* h, const char* key, double value,
    char* error, size_t capacity) noexcept {
    return shield(h, error, capacity, [&] { option_allowed(h, key, true); require(std::isfinite(value), "nonfinite option value");
        accepted(h->application->Options()->SetNumericValue(key, value)); });
}
extern "C" int32_t pse_ipopt_sequence_set_integer(pse_ipopt_sequence_handle* h, const char* key, int32_t value,
    char* error, size_t capacity) noexcept {
    return shield(h, error, capacity, [&] { option_allowed(h, key);
        accepted(h->application->Options()->SetIntegerValue(key, value)); });
}
extern "C" int32_t pse_ipopt_sequence_reset_restart(pse_ipopt_sequence_handle* h, char* error, size_t capacity) noexcept {
    return shield(h, error, capacity, [&] {
        require(h, "missing restart handle");
        // Absent values already use the registered native defaults. Clear prior seed
        // data before applying this attempt's restart, including warm-to-cold reuse.
        for (const char* key : {"mu_init", "warm_start_bound_push", "warm_start_bound_frac",
            "warm_start_slack_bound_push", "warm_start_slack_bound_frac", "warm_start_mult_bound_push"})
            h->application->Options()->UnsetValue(key);
    });
}
extern "C" int32_t pse_ipopt_sequence_solve(pse_ipopt_sequence_handle* h, int32_t reuse, double seconds,
    int32_t n, const double* initial, const double* zl, const double* zu, int32_t m, const double* lambda,
    char* error, size_t capacity) noexcept {
    return shield(h, error, capacity, [&] {
        require(h && initial && (reuse == 0 || reuse == 1), "invalid solve inputs");
        auto& p = *h->problem;
        p.returned = false; p.finalized = false; p.result = {};
        require(n == p.n && m == p.m, "initial dimensions changed");
        require(reuse ? p.reusable : !p.started, "invalid OptimizeTNLP/ReOptimizeTNLP lifecycle");
        require(std::isfinite(seconds) && seconds > 0, "finite remaining time required");
        bool duals = zl || zu || lambda;
        require(!duals || (zl && zu && (!m || lambda)), "dual starts must be supplied together");
        for (int32_t i = 0; i < n; ++i) {
            require(std::isfinite(initial[i]), "invalid initial primal");
            if (duals) require(std::isfinite(zl[i]) && std::isfinite(zu[i]) && zl[i] >= 0 && zu[i] >= 0, "invalid initial bound dual");
        }
        if (duals) for (int32_t i = 0; i < m; ++i) require(std::isfinite(lambda[i]), "invalid initial row dual");
        std::copy(initial, initial + n, p.initial.begin());
        if (duals) { std::copy(zl, zl + n, p.initial_lower.begin()); std::copy(zu, zu + n, p.initial_upper.begin());
            if (m) std::copy(lambda, lambda + m, p.initial_row.begin()); }
        p.has_duals = duals; p.poll();
        accepted(h->application->Options()->SetNumericValue("max_wall_time", seconds));
        accepted(h->application->Options()->SetStringValue("warm_start_same_structure", reuse ? "yes" : "no"));
        accepted(h->application->Options()->SetStringValue("warm_start_init_point", duals ? "yes" : "no"));
        p.started = true; p.reusable = false;
        if (!reuse) {
            auto status = h->application->Initialize("");
            require(status == Ipopt::Solve_Succeeded, "Ipopt initialization refused declared options");
        }
        Ipopt::SmartPtr<Ipopt::TNLP> problem = h->problem;
        auto status = reuse ? h->application->ReOptimizeTNLP(problem) : h->application->OptimizeTNLP(problem);
        p.result.application_status = static_cast<int32_t>(status); p.result.reoptimized = reuse; p.returned = true;
        p.poll(); // Preserve known native return even when the terminal latch vetoes its candidate.
        p.reusable = p.finalized && (status == Ipopt::Solve_Succeeded || status == Ipopt::Solved_To_Acceptable_Level);
    });
}
extern "C" int32_t pse_ipopt_sequence_get_result(pse_ipopt_sequence_handle* h, pse_ipopt_sequence_result* out,
    char* error, size_t capacity) noexcept {
    if (h && !h->problem->returned) { message(error, capacity, "no native result"); return PSE_IPOPT_SEQUENCE_NO_RESULT; }
    return shield(h, error, capacity, [&] { require(h && out, "invalid result buffer"); *out = h->problem->result;
        out->candidate_available = h->problem->finalized && !h->problem->terminal ? 1 : 0; });
}
extern "C" int32_t pse_ipopt_sequence_get_vectors(pse_ipopt_sequence_handle* h, int32_t n, double* primal, double* zl,
    double* zu, int32_t m, double* rows, double* lambda, char* error, size_t capacity) noexcept {
    if (h && !h->problem->finalized) { message(error, capacity, "no native result"); return PSE_IPOPT_SEQUENCE_NO_RESULT; }
    return shield(h, error, capacity, [&] {
        require(h && primal && zl && zu, "invalid vector buffers"); auto& p = *h->problem;
        require(n == p.n && m == p.m && (!m || (rows && lambda)), "invalid vector dimensions");
        std::copy(p.primal.begin(), p.primal.end(), primal); std::copy(p.lower_dual.begin(), p.lower_dual.end(), zl);
        std::copy(p.upper_dual.begin(), p.upper_dual.end(), zu);
        if (m) { std::copy(p.constraints.begin(), p.constraints.end(), rows); std::copy(p.row_dual.begin(), p.row_dual.end(), lambda); }
    });
}
extern "C" int32_t pse_ipopt_sequence_destroy(pse_ipopt_sequence_handle** pointer, char* error, size_t capacity) noexcept {
    return shield(nullptr, error, capacity, [&] { require(pointer, "null handle pointer");
        auto* h = *pointer; *pointer = nullptr; delete h; });
}
