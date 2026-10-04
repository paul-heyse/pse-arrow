// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
#include "pse_uno.h"
#include "Uno_C_API.h"
#include <chrono>
#include <cmath>
#include <cstring>
#include <exception>
#include <memory>
#include <mutex>
#include <new>
#include <set>
#include <stdexcept>
#include <thread>
#include <utility>
#include <vector>
#include <limits>

// Supplied by docker/solvers/uno-pse.patch at the reviewed Uno revision.
extern "C" void uno_pse_set_scope_callback(void (*callback)(int32_t, void*), void* data);
extern "C" void uno_pse_set_materialization_limit(uint64_t);
extern "C" void uno_pse_get_materialization(uint64_t*, uint64_t*, uint64_t*);
extern "C" void uno_pse_get_accuracy(double*);
extern "C" void uno_pse_get_feasibility(uint64_t*, uint64_t*, int32_t*, int32_t*, int32_t*, int32_t*, double*);
extern "C" double uno_pse_minimum_tolerance();
static_assert(sizeof(uno_int) == sizeof(int32_t), "Uno integer ABI must be 32-bit");
struct pse_uno_handle {
    void* model = nullptr;
    void* solver = nullptr;
    int32_t n = 0, m = 0, nnz = 0;
    pse_uno_callbacks callbacks{};
    int32_t terminal = PSE_UNO_OK;
    bool started = false, native_finished = false, result = false;
    std::vector<double> lower, upper;
    std::chrono::steady_clock::time_point created;
    double remaining_seconds = 0.;
    uint64_t materialization_limit = 0, materialization_count = 0, materialization_columns = 0, materialization_numeric_bytes = 0;
    double highs_accuracy[4] = {NAN, NAN, NAN, NAN};
    uint64_t feasibility_calls = 0, feasibility_iterations = 0;
    int32_t feasibility_status = 0, feasibility_status_available = 0, feasibility_iterations_available = 1, qp_hot_start = -1;
    double feasibility_primal_tolerance = NAN;
    ~pse_uno_handle() noexcept {
        try { if (solver) uno_destroy_solver(solver); } catch (...) {}
        try { if (model) uno_destroy_model(model); } catch (...) {}
    }
};
namespace {
std::mutex uno_process_mutex;
struct Terminal final : std::exception {
    const char* what() const noexcept override { return "terminal callback or execution scope failure"; }
};
void message(char* output, size_t capacity, const char* text) noexcept {
    if (!output || !capacity) return;
    size_t length = std::strlen(text);
    size_t count = length < capacity - 1 ? length : capacity - 1;
    std::memcpy(output, text, count);
    output[count] = '\0';
}
template<class Function> int32_t shield(pse_uno_handle* handle, char* error, size_t capacity, Function function) noexcept {
    message(error, capacity, "");
    try { function(); return PSE_UNO_OK; }
    catch (const Terminal& exception) {
        message(error, capacity, exception.what());
        return handle && handle->terminal != PSE_UNO_OK ? handle->terminal : PSE_UNO_CALLBACK_TERMINAL;
    }
    catch (const std::bad_alloc&) { message(error, capacity, "native allocation failed"); return PSE_UNO_MEMORY; }
    catch (const std::invalid_argument& exception) { message(error, capacity, exception.what()); return PSE_UNO_INVALID; }
    catch (const std::exception& exception) { message(error, capacity, exception.what()); return PSE_UNO_EXCEPTION; }
    catch (...) { message(error, capacity, "unknown foreign exception"); return PSE_UNO_EXCEPTION; }
}
void require(bool value, const char* reason) { if (!value) throw std::invalid_argument(reason); }
void accepted(bool value) { if (!value) throw std::runtime_error("Uno refused a declared model or option"); }
void poll(pse_uno_handle& h) {
    if (h.terminal != PSE_UNO_OK) throw Terminal{};
    switch (h.callbacks.poll(h.callbacks.user)) {
        case PSE_UNO_SCOPE_CONTINUE: return;
        case PSE_UNO_SCOPE_CANCEL: h.terminal = PSE_UNO_CANCELLED; break;
        case PSE_UNO_SCOPE_DEADLINE: h.terminal = PSE_UNO_DEADLINE; break;
        case PSE_UNO_SCOPE_ABANDON: h.terminal = PSE_UNO_ABANDONED; break;
        default: h.terminal = PSE_UNO_CALLBACK_TERMINAL; break;
    }
    throw Terminal{};
}
void subproblem_scope(int32_t reason, void* data) {
    auto& h = *static_cast<pse_uno_handle*>(data);
    if (reason == 1 && h.terminal == PSE_UNO_OK) h.terminal = PSE_UNO_DEADLINE;
    if (reason == 2 && h.terminal == PSE_UNO_OK) h.terminal = PSE_UNO_MEMORY;
    poll(h);
}
struct SubproblemScope {
    pse_uno_handle* owner;
    explicit SubproblemScope(pse_uno_handle* h): owner(h) {
        uno_pse_set_scope_callback(subproblem_scope, h); uno_pse_set_materialization_limit(h->materialization_limit);
    }
    ~SubproblemScope() noexcept {
        uno_pse_get_materialization(&owner->materialization_count,&owner->materialization_columns,&owner->materialization_numeric_bytes);
        uno_pse_get_accuracy(owner->highs_accuracy);
        uno_pse_get_feasibility(&owner->feasibility_calls, &owner->feasibility_iterations,
            &owner->feasibility_status, &owner->feasibility_status_available,
            &owner->feasibility_iterations_available, &owner->qp_hot_start, &owner->feasibility_primal_tolerance);
        uno_pse_set_scope_callback(nullptr, nullptr); uno_pse_set_materialization_limit(0);
    }
};
template<class Function> int32_t evaluate(pse_uno_handle& h, Function function) {
    poll(h);
    int32_t status = function();
    // A terminal Rust cause outranks a scope observation made after the call.
    if (status != PSE_UNO_EVAL_OK && status != PSE_UNO_EVAL_TRIAL) {
        h.terminal = PSE_UNO_CALLBACK_TERMINAL;
        throw Terminal{};
    }
    poll(h);
    return status; // Only +1 is handed to Uno's recoverable EvaluationError route.
}
int32_t objective(int32_t n, const double* x, double* out, void* user) {
    auto& h = *static_cast<pse_uno_handle*>(user);
    return evaluate(h, [&] { return h.callbacks.objective(n, x, out, h.callbacks.user); });
}
int32_t constraints(int32_t n, int32_t m, const double* x, double* out, void* user) {
    auto& h = *static_cast<pse_uno_handle*>(user);
    return evaluate(h, [&] { return h.callbacks.constraints(n, m, x, out, h.callbacks.user); });
}
int32_t gradient(int32_t n, const double* x, double* out, void* user) {
    auto& h = *static_cast<pse_uno_handle*>(user);
    return evaluate(h, [&] { return h.callbacks.gradient(n, x, out, h.callbacks.user); });
}
int32_t jacobian(int32_t n, int32_t nnz, const double* x, double* out, void* user) {
    auto& h = *static_cast<pse_uno_handle*>(user);
    return evaluate(h, [&] { return h.callbacks.jacobian(n, nnz, x, out, h.callbacks.user); });
}
int32_t termination(int32_t, int32_t, const double*, const double*, const double*, const double*,
    double, double, double, double, void* user) {
    poll(*static_cast<pse_uno_handle*>(user));
    return 1; // Uno's C termination callback uses zero to request termination.
}
void bounds(int32_t length, const double* lower, const double* upper) {
    require(length == 0 || (lower && upper), "missing bounds");
    for (int32_t i = 0; i < length; ++i) {
        require(!std::isnan(lower[i]) && !std::isnan(upper[i]) && lower[i] <= upper[i]
            && lower[i] != INFINITY && upper[i] != -INFINITY, "invalid bounds");
    }
}
void result_available(pse_uno_handle* h) {
    require(h, "null handle");
    if (!h->result) throw std::logic_error("no result for this handle");
}
}
extern "C" double pse_uno_minimum_tolerance() noexcept { return uno_pse_minimum_tolerance(); }
extern "C" int32_t pse_uno_create(int32_t n, int32_t m, int32_t nnz,
    const double* lower, const double* upper, const double* row_lower, const double* row_upper,
    const int32_t* rows, const int32_t* columns, const pse_uno_config* c, const pse_uno_callbacks* callbacks,
    pse_uno_handle** output, char* error, size_t capacity) noexcept {
    const auto created = std::chrono::steady_clock::now();
    if (output) *output = nullptr;
    return shield(nullptr, error, capacity, [&] {
        require(output && c && callbacks, "missing create inputs");
        require(n > 0 && m >= 0 && nnz >= 0 && (m > 0 || nnz == 0), "invalid dimensions");
        require(callbacks->objective && callbacks->gradient && callbacks->poll
            && (m == 0 || (callbacks->constraints && callbacks->jacobian)), "missing callback");
        require(c->profile == PSE_UNO_SQP_LBFGS || c->profile == PSE_UNO_SLP, "unsupported profile");
        require(c->max_iterations > 0, "invalid finite iteration limit");
        require(c->profile == PSE_UNO_SQP_LBFGS ? c->lbfgs_memory > 0 : c->lbfgs_memory == 0,
            "LBFGS memory requires SQP and must be absent for SLP");
        require(std::isfinite(c->remaining_seconds) && c->remaining_seconds > 0
            && std::isfinite(c->primal_tolerance) && c->primal_tolerance > 0
            && std::isfinite(c->dual_tolerance) && c->dual_tolerance > 0
            && std::isfinite(c->trust_radius) && c->trust_radius > 0, "invalid numeric configuration");
        require(c->primal_tolerance >= uno_pse_minimum_tolerance() && c->dual_tolerance >= uno_pse_minimum_tolerance(),
            "required accuracy below HiGHS 1.15.0 supported 1e-10 tolerance floor");
        bounds(n, lower, upper); bounds(m, row_lower, row_upper);
        require(nnz == 0 || (rows && columns), "missing Jacobian COO");
        std::set<std::pair<int32_t, int32_t>> coordinates;
        for (int32_t k = 0; k < nnz; ++k) {
            require(rows[k] >= 0 && rows[k] < m && columns[k] >= 0 && columns[k] < n, "COO index out of range");
            require(coordinates.emplace(rows[k], columns[k]).second, "duplicate COO index");
        }
        auto h = std::make_unique<pse_uno_handle>();
        h->n = n; h->m = m; h->nnz = nnz; h->callbacks = *callbacks;
        h->created = created; h->remaining_seconds = c->remaining_seconds;
        if (c->profile == PSE_UNO_SQP_LBFGS) {
            // The pinned joint factory constructs both optimality/restoration
            // L-BFGS histories on problem.model, whose extent is original n.
            // Reserve both S/Y/U/V histories, dense memory workspaces and vectors.
            // Expanded restoration/slack dimensions are checked separately against
            // the remaining allowance by the actual Subproblem materialization.
            const uint64_t variables=static_cast<uint64_t>(n), memory=static_cast<uint64_t>(c->lbfgs_memory);
            const uint64_t ceiling=std::numeric_limits<uint64_t>::max();
            if (!c->materialization_limit_bytes || memory > ceiling/64/memory || variables > ceiling/64/memory)
                throw std::bad_alloc();
            const uint64_t history=memory*memory*64;
            const uint64_t vectors=variables*memory*64;
            const uint64_t scalars=variables*128+memory*64;
            if (history > ceiling-vectors || history+vectors > ceiling-scalars) throw std::bad_alloc();
            const uint64_t allowance=history+vectors+scalars;
            if (allowance >= c->materialization_limit_bytes) throw std::bad_alloc();
            h->materialization_limit=c->materialization_limit_bytes-allowance;
        }
        h->lower.assign(lower, lower + n); h->upper.assign(upper, upper + n);
        h->model = uno_create_model("NLP", n, lower, upper, UNO_ZERO_BASED_INDEXING);
        require(h->model, "Uno did not create model");
        accepted(uno_set_user_data(h->model, h.get()));
        accepted(uno_set_objective(h->model, UNO_MINIMIZE, objective, gradient));
        accepted(uno_set_lagrangian_sign_convention(h->model, UNO_MULTIPLIER_POSITIVE));
        if (m) accepted(uno_set_constraints(h->model, m, constraints, row_lower, row_upper, nnz, rows, columns, jacobian));
        h->solver = uno_create_solver();
        require(h->solver, "Uno did not create solver");
        accepted(uno_set_solver_preset(h->solver, c->profile == PSE_UNO_SQP_LBFGS ? "filtersqp" : "filterslp"));
        accepted(uno_set_solver_string_option(h->solver, "hessian_model", c->profile == PSE_UNO_SQP_LBFGS ? "LBFGS" : "zero"));
        accepted(uno_set_solver_string_option(h->solver, "inertia_correction_strategy", "none"));
        accepted(uno_set_solver_string_option(h->solver, "QP_solver", "HiGHS"));
        accepted(uno_set_solver_string_option(h->solver, "LP_solver", "HiGHS"));
        accepted(uno_set_solver_string_option(h->solver, "logger", "SILENT"));
        accepted(uno_set_solver_bool_option(h->solver, "use_function_scaling", false));
        accepted(uno_set_solver_integer_option(h->solver, "max_iterations", c->max_iterations));
        if (c->profile == PSE_UNO_SQP_LBFGS)
            accepted(uno_set_solver_integer_option(h->solver, "quasi_newton_memory_size", c->lbfgs_memory));
        accepted(uno_set_solver_double_option(h->solver, "time_limit", c->remaining_seconds));
        accepted(uno_set_solver_double_option(h->solver, "primal_tolerance", c->primal_tolerance));
        accepted(uno_set_solver_double_option(h->solver, "dual_tolerance", c->dual_tolerance));
        accepted(uno_set_solver_double_option(h->solver, "loose_primal_tolerance", c->primal_tolerance));
        accepted(uno_set_solver_double_option(h->solver, "loose_dual_tolerance", c->dual_tolerance));
        accepted(uno_set_solver_double_option(h->solver, "TR_radius", c->trust_radius));
        accepted(uno_set_solver_callbacks(h->solver, nullptr, termination, h.get()));
        *output = h.release();
    });
}
extern "C" int32_t pse_uno_solve(pse_uno_handle* h, const double* initial, int32_t n, char* error, size_t capacity) noexcept {
    return shield(h, error, capacity, [&] {
        require(h && initial && n == h->n, "invalid initial iterate dimensions");
        require(!h->started, "handle already solved; construct a fresh declared attempt");
        h->started = true;
        for (int32_t i = 0; i < n; ++i) require(std::isfinite(initial[i]) && initial[i] >= h->lower[i]
            && initial[i] <= h->upper[i], "initial iterate outside bounds");
        std::unique_lock<std::mutex> permit(uno_process_mutex, std::defer_lock);
        while (!permit.try_lock()) { poll(*h); std::this_thread::sleep_for(std::chrono::milliseconds(1)); }
        poll(*h);
        const double remaining = h->remaining_seconds - std::chrono::duration<double>(
            std::chrono::steady_clock::now() - h->created).count();
        if (remaining <= 0.) { h->terminal = PSE_UNO_DEADLINE; throw Terminal{}; }
        accepted(uno_set_solver_double_option(h->solver, "time_limit", remaining));
        SubproblemScope subproblem_scope(h);
        accepted(uno_set_initial_primal_iterate(h->model, initial));
        uno_optimize(h->solver, h->model);
        h->native_finished = true;
        // Uno may catch Terminal in its major loop. Never expose that as a result.
        poll(*h);
        h->result = true;
    });
}
extern "C" int32_t pse_uno_get_result(pse_uno_handle* h, pse_uno_result* out, char* error, size_t capacity) noexcept {
    if (h && !h->native_finished) { message(error, capacity, "no native result"); return PSE_UNO_NO_RESULT; }
    return shield(h, error, capacity, [&] {
        require(h && h->native_finished && out, "no native scalar result or null output");
        pse_uno_result result{};
        result.optimization_status = uno_get_optimization_status(h->solver);
        result.iterate_status = uno_get_solution_status(h->solver);
        result.iterations = uno_get_number_iterations(h->solver);
        result.objective_evaluations = uno_get_number_objective_evaluations(h->solver);
        result.constraint_evaluations = uno_get_number_constraint_evaluations(h->solver);
        result.gradient_evaluations = uno_get_number_objective_gradient_evaluations(h->solver);
        result.jacobian_evaluations = uno_get_number_jacobian_evaluations(h->solver);
        result.subproblems = uno_get_number_subproblem_solved_evaluations(h->solver);
        result.objective = uno_get_solution_objective(h->solver);
        result.primal_feasibility = uno_get_solution_primal_feasibility(h->solver);
        result.stationarity = uno_get_solution_stationarity(h->solver);
        result.complementarity = uno_get_solution_complementarity(h->solver);
        result.cpu_seconds = uno_get_cpu_time(h->solver);
        result.materialization_count=h->materialization_count;
        result.materialization_columns=h->materialization_columns;
        result.materialization_numeric_bytes=h->materialization_numeric_bytes;
        result.highs_primal_tolerance=h->highs_accuracy[0];
        result.highs_dual_tolerance=h->highs_accuracy[1];
        result.highs_optimality_tolerance=h->highs_accuracy[2];
        result.highs_qp_regularization=h->highs_accuracy[3];
        result.feasibility_calls=h->feasibility_calls;
        result.feasibility_iterations=h->feasibility_iterations;
        result.feasibility_status=h->feasibility_status;
        result.feasibility_status_available=h->feasibility_status_available;
        result.feasibility_iterations_available=h->feasibility_iterations_available;
        result.qp_hot_start=h->qp_hot_start;
        result.feasibility_primal_tolerance=h->feasibility_primal_tolerance;
        *out = result;
    });
}
extern "C" int32_t pse_uno_get_vectors(pse_uno_handle* h, double* primal, double* lower, double* upper, int32_t n,
    double* rows, double* row_dual, int32_t m, char* error, size_t capacity) noexcept {
    if (h && !h->result) { message(error, capacity, "no native result"); return PSE_UNO_NO_RESULT; }
    return shield(h, error, capacity, [&] {
        result_available(h);
        require(n == h->n && m == h->m && primal && lower && upper && (m == 0 || (rows && row_dual)), "invalid vector buffers");
        uno_get_primal_solution(h->solver, primal);
        uno_get_lower_bound_dual_solution(h->solver, lower);
        uno_get_upper_bound_dual_solution(h->solver, upper);
        if (m) { uno_get_solution_constraints(h->solver, rows); uno_get_constraint_dual_solution(h->solver, row_dual); }
    });
}
extern "C" int32_t pse_uno_get_string_option(pse_uno_handle* h, const char* key, char* value, size_t value_capacity,
    char* error, size_t capacity) noexcept {
    return shield(h, error, capacity, [&] {
        require(h && key && value && value_capacity, "invalid option output");
        require(uno_get_solver_option_type(h->solver, key) == UNO_OPTION_TYPE_STRING, "not a declared string option");
        const char* source = uno_get_solver_string_option(h->solver, key);
        require(source && std::strlen(source) < value_capacity, "option output buffer too small");
        message(value, value_capacity, source);
    });
}
extern "C" int32_t pse_uno_get_double_option(pse_uno_handle* h, const char* key, double* value, char* error, size_t capacity) noexcept {
    return shield(h, error, capacity, [&] {
        require(h && key && value, "invalid option output");
        require(uno_get_solver_option_type(h->solver, key) == UNO_OPTION_TYPE_DOUBLE, "not a declared double option");
        *value = uno_get_solver_double_option(h->solver, key);
    });
}
extern "C" int32_t pse_uno_get_bool_option(pse_uno_handle* h, const char* key, int32_t* value, char* error, size_t capacity) noexcept {
    return shield(h, error, capacity, [&] {
        require(h && key && value, "invalid option output");
        require(uno_get_solver_option_type(h->solver, key) == UNO_OPTION_TYPE_BOOL, "not a declared bool option");
        *value = uno_get_solver_bool_option(h->solver, key) ? 1 : 0;
    });
}
extern "C" int32_t pse_uno_get_method(pse_uno_handle* h, char* value, size_t value_capacity, char* error, size_t capacity) noexcept {
    if (h && !h->result) { message(error, capacity, "no native result"); return PSE_UNO_NO_RESULT; }
    return shield(h, error, capacity, [&] {
        result_available(h); require(value && value_capacity, "invalid method output");
        const char* source = uno_get_method_description(h->solver);
        require(source && std::strlen(source) < value_capacity, "method output buffer too small");
        message(value, value_capacity, source);
    });
}
extern "C" int32_t pse_uno_get_integer_option(pse_uno_handle* h, const char* key, int32_t* value, char* error, size_t capacity) noexcept {
    return shield(h, error, capacity, [&] {
        require(h && key && value, "invalid option output");
        require(uno_get_solver_option_type(h->solver, key) == UNO_OPTION_TYPE_INTEGER, "not a declared integer option");
        *value = uno_get_solver_integer_option(h->solver, key);
    });
}
extern "C" int32_t pse_uno_destroy(pse_uno_handle** pointer, char* error, size_t capacity) noexcept {
    return shield(nullptr, error, capacity, [&] {
        require(pointer, "null handle pointer");
        std::unique_ptr<pse_uno_handle> h(*pointer); *pointer = nullptr;
        if (!h) return;
        std::exception_ptr failure;
        void* solver = h->solver; h->solver = nullptr;
        void* model = h->model; h->model = nullptr;
        try { if (solver) uno_destroy_solver(solver); } catch (...) { failure = std::current_exception(); }
        try { if (model) uno_destroy_model(model); } catch (...) { if (!failure) failure = std::current_exception(); }
        if (failure) std::rethrow_exception(failure);
    });
}
