"""Audit the validation deliverable, not a simulator or thermodynamic backend.

The register is an evidence overlay. This checks trace preservation, recorded
results, source hashes, and reachable document links. It does not infer numerical
or physical coverage from the existence of a cross-reference.
"""
from __future__ import annotations
import ast
import hashlib
import json
import re
from collections import Counter
from pathlib import Path
from urllib.parse import unquote, urlsplit

ROOT = Path(__file__).resolve().parents[1]
STEM = "thermodynamics_blueprint_validation_v0_1"
AUDIT = ROOT / f"{STEM}_audit.json"
MANIFEST = ROOT / "validation_artifact_manifest.json"

def load(path: str):
    return json.loads((ROOT / path).read_text(encoding="utf-8"))

def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()

reg = load(f"{STEM}_register.json")
b4 = load("baselines/thermodynamics_functional_requirements_v0_1_register.json")
b6 = load("baselines/thermodynamics_semantic_information_dictionary_v0_1_register.json")
b7 = load("baselines/thermodynamics_action_workflow_lifecycle_blueprint_v0_1_register.json")
b8 = load("baselines/thermodynamics_cross_library_solver_integration_v0_1_register.json")
num = load("results/numerical_probes.json")
model = load("results/publication_model_checks.json")
native = load("results/native_probe_dispatch.json")
sources = load("source_manifest.json")
checks: list[dict] = []

def check(name: str, condition: bool, detail=""):
    checks.append({"name": name, "passed": bool(condition), "detail": detail})

def byid(rows, field="id"):
    return {r[field]: r for r in rows}

req = byid(reg["requirements_review"])
oldreq = byid(b4["requirements"])
alloc = byid(b8["requirement_integration_allocation"], "requirement_id")
sc = byid(reg["scenario_walkthroughs"], "scenario")
oldsc = byid(b8["scenario_profile_allocation"], "scenario")
profile = byid(reg["native_profile_disposition"])
oldprofile = byid(b8["candidate_profile_templates"])
ie = byid(reg["integration_experiment_disposition"])
oldie = byid(b8["qualification_experiments"])
acts = byid(b7["actions"])
concepts = byid(b6["concepts"])
rules = byid(b8["integration_rules"])
probes = byid(num["tests"])

check("94 unique original requirements preserved", len(req) == len(reg["requirements_review"]) == 94 and set(req) == set(oldreq))
check("Obligation classes preserved", all(req[k]["obligation"] == oldreq[k]["obligation"] for k in req))
check("65 H / 25 C / 4 R3 preserved", Counter(r["obligation"] for r in req.values()) == Counter({"H": 65, "C": 25, "R3": 4}))
check("Primary semantic owners preserved", all(req[k]["primary_owner"] == alloc[k]["primary_package"] for k in req))
check("Primary actions preserved", all(req[k]["primary_action"] == alloc[k]["primary_action"] and req[k]["primary_action"] in acts for k in req))
check("Primary information concepts preserved", all(req[k]["primary_information_concept"] == alloc[k]["primary_information_concept"] and req[k]["primary_information_concept"] in concepts for k in req))
check("Original acceptance witness identifiers preserved", all(req[k]["inherited_acceptance_tests"] == alloc[k]["acceptance_test_ids"] for k in req))
check("No requirement conformance or scientific promotion", all(r["implementation_conformance"] == "not_established" and r["independent_validation"] == "not_established" for r in req.values()))
check("Requirement rule crosslinks valid", all(set(r["integration_rules"]) <= set(rules) for r in req.values()))
check("34 unique scenario walkthroughs", len(sc) == len(reg["scenario_walkthroughs"]) == 34 and set(sc) == set(oldsc))
check("Scenario profiles retained", all(sc[k]["profile"] == oldsc[k]["profile"] for k in sc))
check("14 P1 / 16 P2 / 4 P3 retained", Counter(s["profile"] for s in sc.values()) == Counter({"P1": 14, "P2": 16, "P3": 4}))
review_fields = ["specified_input_case", "semantic_and_action_route", "adversarial_condition", "remaining_gate", "coverage_judgment", "review_kind"]
check("Every scenario has a substantive review and residual gate", all(all(s.get(k) for k in review_fields) for s in sc.values()))
check("Scenario action / information / requirement crosslinks valid", all(set(s["required_actions"]) <= set(acts) and set(s["information_concepts"]) <= set(concepts) and set(s["requirements"]) <= set(req) for s in sc.values()))
check("Scenario numerical evidence references valid", all(set(s["executed_contributing_probes"]) <= set(probes) for s in sc.values()))
check("Scenario full execution and independent validation unpromoted", all(s["E_full_scenario"] == "not_established" and s["V_independent_physical"] == "not_established" for s in sc.values()))
check("Independent representability not asserted", all("independent_acceptance_pending" in s["R_evidence"] for s in sc.values()))
check("14 native profile IDs retained", len(profile) == len(reg["native_profile_disposition"]) == 14 and set(profile) == set(oldprofile))
check("Native profile scenario and mode assignments unchanged", all(profile[k]["scenarios"] == oldprofile[k]["scenarios"] and profile[k]["solver_modes"] == oldprofile[k]["solver_modes"] for k in profile))
check("Native profiles not instantiated or admitted", all(not p["native_configuration_instantiated"] and p["native_execution"] == "not_executed" and p["production_admission"] == "not_approved" for p in profile.values()))
check("Profile review and probe references valid", all(set(p["review_routes"]) <= {s["id"] for s in sc.values()} and set(p["contributing_probe_candidates"]) <= set(probes) for p in profile.values()))
check("18 investigation IDs retained", len(ie) == len(reg["integration_experiment_disposition"]) == 18 and set(ie) == set(oldie))
check("Investigation profile and inherited probe assignments retained", all(ie[k]["profiles"] == oldie[k]["profiles"] and ie[k]["inherited_probes"] == oldie[k]["inherited_probes"] for k in ie))
check("All 38 inherited native probe IDs retained", len({p for e in ie.values() for p in e["inherited_probes"]}) == 38)
check("Native investigations unpromoted", all(e["native_experiment_status"] == "not_executed" for e in ie.values()))
check("Investigation contributing probe references valid", all(set(e["executed_contributing_probes"]) <= set(probes) for e in ie.values()))
check("Eight findings have owners and evidence", len(reg["findings"]) == 8 and all(f.get("evidence") and f.get("owners") and f.get("observation") and f.get("closure") for f in reg["findings"]))
check("Finding requirement and integration rule references valid", all(set(f["requirements"]) <= set(req) and set(f["rules"]) <= set(rules) for f in reg["findings"]))
check("30 new probe IDs unique and passing", len(probes) == len(num["tests"]) == 30 and all(p["status"] == "PASS" for p in probes.values()))
check("Numerical count summary agrees with records", num["counts"] == {"tests": 30, "passed": 30, "failed": 0})
check("Recorded fixture hash matches actual fixture", num["fixture_sha256"] == sha(ROOT / "fixtures.json"))
check("Every probe states evidence kind and result", all(t.get("evidence_kind") and t.get("detail") for t in probes.values()))
check("Extracted module fingerprint matches manifest", sources["extracted_module_sha256"] == sha(ROOT / "scripts/extracted_chedl_kernels.py"))
check("Two extracted upstream source pins retained", len([s for s in sources["sources"] if s["kind"] == "source_extracted_numerical_body"]) == 2)
notice=(ROOT / "scripts/extracted_chedl_kernels.py").read_text()
check("Extracted bodies retain provenance and MIT notice", "Permission is hereby granted" in notice and "Caleb Bell" in notice and "NOT" in notice.upper())
check("1,120 bounded schedules / 8,960 steps recorded", model["legal_schedules"] == 1120 and model["examined_event_steps"] == 8960 and model["events_per_schedule"] == 8)
check("No correct-reference-model schedule violation", model["status"] == "PASS" and not model["violations_correct_model"])
check("128 guard combinations and eight detected unsafe controls", model["guard_truth_cases"] == 128 and model["detected_mutations"] == 8 and all(m["detected"] for m in model["mutation_controls"]))
check("Native-active and quarantined cleanup cases never reusable", all(not c["reusable"] for c in model["cleanup_cases"] if c["native_active"] or c["health"] == "quarantined"))
check("Six native probes blocked, zero actual workers executed", native["blocked"] == 6 and native["executed"] == native["passed"] == 0 and all(t["status"] == "BLOCKED_DEPENDENCY" and not t["executed"] for t in native["tests"]))
check("Environment and retrieval evidence retained", all((ROOT / f"results/{n}").is_file() for n in ["environment_preflight.json", "pip_preflight.txt", "pip_public_preflight.txt", "native_probe_dispatch.log"]))
base_errors=[]
for row in reg["source_baselines"]:
    path=ROOT / "baselines" / row["filename"]
    if not path.exists() or sha(path) != row["sha256"] or path.stat().st_size != row["bytes"]: base_errors.append(row["filename"])
check("All bundled baseline bytes match recorded manifest", not base_errors, base_errors)
older=[]
for row in b8["sources"]:
    path=ROOT / "baselines" / row["filename"]
    if not path.exists() or sha(path)!=row["sha256"]: older.append(row["filename"])
check("Step-8 predecessor hashes match bundled files", not older, older)
actioncat=load("results/inherited_action_catalog.json")
integcat=load("results/inherited_integration_catalog.json")
actionref=load("results/inherited_action_reference.json")
integref=load("results/inherited_integration_reference.json")
check("Inherited structural reruns passed 35 / 42", actioncat["passed"] and len(actioncat["checks"]) == 35 and integcat["passed"] == 42 and integcat["failed"] == 0)
check("Inherited reference reruns passed 26 / 43", actionref["passed"] and actionref["tests_run"] == 26 and actionref["failures"] == actionref["errors"] == 0 and integref["passed"] == 43 and integref["failed"] == 0)
syntax=[]
for path in (ROOT / "scripts").glob("*.py"):
    try: ast.parse(path.read_text(), filename=str(path))
    except SyntaxError as exc: syntax.append(str(exc))
check("All supplied new Python scripts parse", not syntax, syntax)
check("No wheels / libraries / native executables included", not any(p.suffix.lower() in {".whl", ".so", ".dll", ".dylib", ".exe"} for p in ROOT.rglob("*") if p.is_file()))

# Write a noncircular fingerprint manifest of primary payloads. Derived audit and
# manifest files, plus transient bytecode, are explicitly excluded.
manifest_rows=[]
for path in sorted(ROOT.rglob("*")):
    if not path.is_file() or "__pycache__" in path.parts or path.suffix == ".pyc" or path in {AUDIT, MANIFEST}: continue
    manifest_rows.append({"path": path.relative_to(ROOT).as_posix(), "bytes": path.stat().st_size, "sha256": sha(path)})
MANIFEST.write_text(json.dumps({"document": "THERMO-VALIDATION-009", "kind": "artifact_payload_fingerprints", "excluded": [AUDIT.name, MANIFEST.name, "transient __pycache__ / .pyc"], "files": manifest_rows}, indent=2)+"\n")
check("Artifact payload manifest entries reproduce current bytes", all(sha(ROOT / r["path"]) == r["sha256"] for r in manifest_rows))

# Check only newly authored document references. Predecessor bytes and their
# original relative references are retained unchanged, not rewritten here.
def anchors(path: Path) -> set[str]:
    text=path.read_text(encoding="utf-8")
    ids=set(re.findall(r'<a\s+id=[\"\']([^\"\']+)[\"\']',text))
    counts=Counter()
    for heading in re.findall(r"^#{1,6}\s+(.+)$",text,re.M):
        slug=heading.lower().strip()
        slug=re.sub(r"<[^>]*>","",slug)
        slug=re.sub(r"[^\w\s-]","",slug)
        slug=re.sub(r"\s","-",slug)
        n=counts[slug];counts[slug]+=1
        ids.add(slug if n==0 else f"{slug}-{n}")
    return ids
link_errors=[];link_count=0;cache={}
for doc in [ROOT/f"{STEM}.md", ROOT/"README.md"]:
    for target in re.findall(r"\[[^\]]*\]\(([^\s)]+)(?:\s+[^)]*)?\)",doc.read_text()):
        if urlsplit(target).scheme or target.startswith("//"): continue
        link_count+=1
        raw, _, fragment=target.partition("#")
        p=(doc.parent/unquote(raw)).resolve() if raw else doc
        if p == AUDIT: continue  # The derived report is written below.
        if not p.is_file(): link_errors.append({"document":doc.name,"target":target,"error":"missing file"});continue
        if fragment and p.suffix.lower()==".md":
            if p not in cache:cache[p]=anchors(p)
            if unquote(fragment) not in cache[p]:link_errors.append({"document":doc.name,"target":target,"error":"missing anchor"})
check("All new Markdown local links and anchors resolve", not link_errors, {"links_checked":link_count,"errors":link_errors})

report={"document":"THERMO-VALIDATION-009", "kind":"structural_trace_and_artifact_audit_not_thermodynamic_or_simulator_conformance", "checks":checks,
        "count":len(checks),"passed":sum(c["passed"] for c in checks),"failed":sum(not c["passed"] for c in checks),
        "counts":{"requirements":len(req),"scenarios":len(sc),"native_profiles":len(profile),"investigations":len(ie),"new_numerical_probes":len(probes),"local_links":link_count,"manifest_payload_files":len(manifest_rows)},
        "limitations":["Counts and links are not proof of semantic sufficiency.","Recorded PASS belongs to its named source-extract or authored test subject.","No complete native package or production simulator executed.","Original R/E/V and AT/AW/P/PV records are not promoted by this audit."]}
AUDIT.write_text(json.dumps(report,indent=2)+"\n")
print(json.dumps({k:v for k,v in report.items() if k!="checks"},indent=2))
for c in checks:
    if not c["passed"]: print("FAIL:",c["name"],c["detail"])
raise SystemExit(1 if report["failed"] else 0)
