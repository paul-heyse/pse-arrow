---
title: "2. Kinetic parameters from four batch experiments"
notion_source: "https://app.notion.com/p/40a141b5836981c5bac0f11765815d8d"
notion_last_edited: "2026-11-01T20:36:22.695Z"
---

# 2. Kinetic parameters from four batch experiments

Source: [Notion](https://app.notion.com/p/40a141b5836981c5bac0f11765815d8d). Imported 2026-11-02.

**Version 3** · 2026-11-01

Four constant-volume batch experiments for consecutive reactions A → B → C are fitted for Arrhenius parameters. A shared physical reactor model is replicated across experiments; case data and observation models determine the inference task.

**Profile:** Infer, point estimate. **Scope:** process dynamics and parameter estimation.

## Model

| Part | Definition |
| --- | --- |
| Material scope | Abstract components A, B, C and the declared stoichiometric basis; one liquid phase |
| Reaction laws | Arrhenius rate expressions for R1 and R2, with the rate/concentration units and reference conventions specified |
| Domain | Time from zero to each experiment's supplied end time |
| Quantities | Concentrations c\[A,B,C\](t), temperature T(t), rates r\[R1,R2\](t), parameters k0 and Ea |
| Physics | Constant-volume component balances and the selected kinetic relations |
| Capability | A dynamic model with initial-condition and parameter-sensitivity contracts |

The constant-volume and imposed-temperature assumptions are part of this model. A later non-isothermal jacketed experiment would require the additional energy/inventory physics rather than merely another role assignment.

## Specification and observations

There are four scenarios E1–E4. The kinetic parameters are shared; initial concentrations and imposed temperature histories are experiment-specific. Each experiment supplies its own time coverage, for example 0–4 h.

The dataset contains experiment identifiers, sample times, measured concentrations, and temperature. Identifier columns are typed labels; physical measurement columns carry units and quantity kinds. Observation alignment maps each experiment to its scenario and each sample to the relevant time.

Temperature is treated as a known input trajectory in this illustration. If its measurement error materially affects inference, the model instead needs an explicit latent-input or errors-in-variables treatment; it is not automatically accounted for by concentration noise.

The concentration observation model specifies instrument noise. A relative-error model includes a positive absolute noise floor or another defined behavior near zero concentration. Correlations and heteroscedasticity are included when justified rather than assuming independence by default.

## Query

The Arrhenius quantities are Estimands with declared admissible domains and any chosen parameterization. For example, estimating log(k0) can enforce positivity; an Ea interval is a study assumption, not a universal physical law.

The Infer request identifies:

- Observations and noise/loss model.
- Whether priors are active, distinguishing likelihood-based estimation from MAP.
- Shared parameters and experiment-specific nuisance quantities.
- Enforced physical/operating constraints.
- Outputs: parameter estimates, predicted trajectories, residuals, fit diagnostics, and uncertainty/identifiability evidence.

A structural-identifiability analysis is requested where supported. Lack of a general proof does not by itself forbid fitting; known non-identifiable directions and limitations must be reported.

## Execution

The task may lower to a constrained estimation problem over replicated dynamic models. Shooting or collocation is an execution choice subject to the model's evaluation capabilities and the desired evidence.

A reduced-Hessian or sensitivity-based covariance estimate is local and assumption-dependent. It is not automatically a globally valid confidence region. The result names the noise assumptions, parameterization, rank/conditioning diagnostics, and approximation used; profile likelihood, bootstrap, or posterior methods are separate supported analyses when requested.

The fitted result can become a nominal binding or an uncertainty description in a later study, with parameter identity, units, covariance, and correlation preserved.
