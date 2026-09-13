"""Probes 3, 4, 5 — the Pyomo adapter surface asserted by §21.2."""
import pyomo.environ as pyo
from pyomo.core.expr.numeric_expr import LinearExpression
from pyomo.util.check_units import assert_units_consistent, identify_inconsistent_units
from pyomo.common.errors import PyomoException

print("== PROBE 3: expression lowering fidelity (§21.2 step 2)")
m = pyo.ConcreteModel()
m.I = pyo.RangeSet(0, 2)
m.x = pyo.Var(m.I, bounds=(0, 10), initialize=1.0)
m.p = pyo.Param(initialize=2.0, mutable=True)
# Affine -> LinearExpression
le = LinearExpression(constant=3.0, linear_coefs=[1.0, -2.0], linear_vars=[m.x[0], m.x[1]])
print("   LinearExpression      :", type(le).__name__, "| value:", pyo.value(le))
print("     nargs/args ok       :", le.nargs() if hasattr(le,'nargs') else 'n/a', "| polynomial_degree:", le.polynomial_degree())
# Conditional -> Expr_if
ei = pyo.Expr_if(IF=m.x[0] >= 0.5, THEN=m.x[1], ELSE=m.p * m.x[2])
print("   Expr_if               :", type(ei).__name__, "| value:", pyo.value(ei))
# ExternalFunction (declaration only; no library loaded)
try:
    m.f = pyo.ExternalFunction(library="libpse_kernels.so", function="cubic_root_l")
    print("   ExternalFunction decl :", type(m.f).__name__, "| callable in expr:", type(m.f(m.x[0])).__name__)
except Exception as e:
    print("   ExternalFunction decl : ERROR", type(e).__name__, str(e)[:80])

print("== PROBE 4: units (§21.2 step 1)")
u = pyo.units
m2 = pyo.ConcreteModel()
m2.T = pyo.Var(units=u.K, initialize=320.0)
m2.P = pyo.Var(units=u.Pa, initialize=1e5)
m2.ok = pyo.Constraint(expr=m2.T == 320 * u.K)
try:
    assert_units_consistent(m2); print("   consistent model      : PASSES")
except Exception as e:
    print("   consistent model      : FAILED", type(e).__name__, str(e)[:80])
m2.bad = pyo.Constraint(expr=m2.T == m2.P)     # K == Pa
try:
    assert_units_consistent(m2); print("   inconsistent model    : PASSES (unexpected)")
except Exception as e:
    print("   inconsistent model    : raises", type(e).__name__)
bad = identify_inconsistent_units(m2)
print("   identify_inconsistent_units ->", [c.name for c in bad])

print("== PROBE 5: suffixes and solver availability (§21.2 steps 3-4)")
m3 = pyo.ConcreteModel()
m3.x = pyo.Var(bounds=(0, 10), initialize=1.0)
m3.c = pyo.Constraint(expr=m3.x >= 2.0)
m3.obj = pyo.Objective(expr=(m3.x - 5.0) ** 2)
m3.scaling_factor = pyo.Suffix(direction=pyo.Suffix.EXPORT)
m3.scaling_factor[m3.x] = 1e-3
m3.dual = pyo.Suffix(direction=pyo.Suffix.IMPORT)
m3.ipopt_zL_out = pyo.Suffix(direction=pyo.Suffix.IMPORT)
m3.ipopt_zU_out = pyo.Suffix(direction=pyo.Suffix.IMPORT)
print("   Suffix directions     : EXPORT/IMPORT declared ok")
for s in ("ipopt", "cbc", "glpk"):
    try:
        opt = pyo.SolverFactory(s); print(f"   SolverFactory({s:6}) available: {opt.available(False)}")
    except Exception as e:
        print(f"   SolverFactory({s:6}) ERROR {type(e).__name__}")
