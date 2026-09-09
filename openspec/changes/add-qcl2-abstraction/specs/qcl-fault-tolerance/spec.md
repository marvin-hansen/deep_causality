<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

## ADDED Requirements

### Requirement: A fault set is a query signature over the low-level model, counted and capped

`FaultSet` SHALL be a finite signature of low-level queries, each an opening at a wire followed by
insertion of an error channel, with constructors `pauli_weight(t)`, `declared(&[...])` and
`from_dem(...)`, and `pauli_weight(t)` on `n` locations SHALL count `C(n, t) · 3^t` queries on
`NumberType` and SHALL refuse above a cap before allocating, naming the count and the cap.

A fault is a comb in the paper's sense, a general intervention that is not a Do-query. The count
is the same exponential D7 of `add-qcl` caps for the design cover; the default weight is one, and
realistic sets come from a detector error model rather than from enumeration.

#### Scenario: Weight-one Paulis on the small torus

- **WHEN** `FaultSet::pauli_weight(1)` is built over the 8 locations of the `[[8,2,2]]` circuit
- **THEN** it holds `24` queries, one per `(location, Pauli)`, and its count reads `24`

#### Scenario: A set above the cap is refused

- **WHEN** `FaultSet::pauli_weight(3)` is built over `n` locations such that `C(n, 3) · 27` exceeds
  the cap
- **THEN** construction returns `QuantumError::CalculationError` naming the count and the cap, and
  no queries are allocated

### Requirement: Faults propagate in the Pauli basis, exactly through Clifford gates and to a cap otherwise

The fault propagator SHALL carry a Pauli fault through the low-level program as a linear
combination of Paulis with coefficients in `Complex<R>`, SHALL keep one term through every Clifford
gate by the shipped `clifford_conjugate` rule, SHALL branch through `T`, `T†`, `CS†`, `CCZ` and
`C^{m−1}Z` on three or more qubits, and SHALL count the terms it would produce and refuse above a
cap with `QuantumError::PauliTermCountExceeded { terms, cap }` before allocating.

The tableau refuses non-Clifford gates because their conjugation action is not a Pauli; the
propagator lifts that refusal to a branching with a bound. The term count is bounded by `4^w` for
the support weight `w` the fault can spread over, the default cap is `2^16`, and the count is ℕ on
`NumberType` with a checked product.

#### Scenario: A Clifford program keeps one term

- **WHEN** an `X` fault on qubit `0` is propagated through `S̄(γ)` with `γ` of weight 3 containing
  qubit `0`
- **THEN** the result is one term, `X₀ Z_a Z_b` for the other two support qubits up to phase, which
  is what `CZ` does to `X` on its control

#### Scenario: A `T` gate branches

- **WHEN** an `X` fault is propagated through a single `T` on the same qubit
- **THEN** the result has two terms, `X` and `Y`, each with coefficient of modulus `1/√2`, and no
  term of any other Pauli

#### Scenario: The cap is refused before allocation

- **WHEN** a fault is propagated through `T̄` on a representative whose weight makes `4^w` exceed
  the cap
- **THEN** the propagator returns `PauliTermCountExceeded` naming the count and the cap, and
  allocates no terms

### Requirement: Fault tolerance is the naturality check over the enlarged signature

`check_fault_tolerance(abstraction, fault_set)` SHALL run `check_naturality` over the abstraction's
signature enlarged by every fault in the set, SHALL report per-fault residuals, the worst, the
count examined and the witnessing fault as `(location, Pauli, offending term)`, and SHALL decide
correctability of a propagated error set against the stabilizer generators `LogicalBasis` carries:
a term in the normalizer that is not a stabilizer is a logical fault.

The report carries a witness rather than a margin for the reason D10 of `add-qcl` gives: which
fault broke the square is the information, and how badly is secondary. Each per-gate verdict carries
the `SemanticsPath` that decided it.

#### Scenario: A transversal gate under weight-one noise passes

- **WHEN** `check_fault_tolerance` runs on `Z̄(γ)` of the `[[18,2,3]]` torus under
  `pauli_weight(1)`
- **THEN** every fault propagates to a single-qubit Pauli, none is in the normalizer without being
  a stabilizer, the report accepts with examined count `3 · 18`, and every record reads
  `SemanticsPath::Exact`

#### Scenario: A failing fault is named

- **WHEN** a program spreads a weight-one fault to an operator logically equivalent to `X̄(γ̃)`
- **THEN** the report rejects, its witness names the location, the injected Pauli and the offending
  term, and `first_rejection` returns that record

#### Scenario: An empty fault set is vacuous

- **WHEN** `check_fault_tolerance` runs with `FaultSet::declared(&[])`
- **THEN** the report's verdict is `Vacuous` with examined count zero

### Requirement: The Haruna filter labels each gate by the path that decided it

The Haruna filter SHALL run `check_fault_tolerance` under `pauli_weight(1)` on each Table 1 gate's
emitted program over a CSS code, SHALL output the subset that holds, and SHALL label each gate's
verdict `Exact` for `Z̄`, `X̄`, `S̄`, `CZ̄` and `H̄` and `PauliBasisToCap` for `T̄`, `CS̄†` and `CC̄Z`, so a
verdict reached under the term cap is never read as one reached exactly.

The oracle facts the filter is validated against are derived by hand in the change's notes before
the asserting test is written, and each expected value in the test carries that derivation as its
provenance.

#### Scenario: The filter agrees with the derived oracle on the small torus

- **WHEN** the filter runs on `[[18,2,3]]`
- **THEN** `Z̄` and `X̄` hold under weight-one faults with label `Exact`, `S̄` with its CZ pairs does
  not, with a named witness, and each verdict's label matches its gate family

#### Scenario: A non-Clifford verdict is labelled

- **WHEN** the filter reports `T̄`
- **THEN** its record carries `PauliBasisToCap` with the term count it reached, and a reader cannot
  mistake it for an exact verdict

### Requirement: The fault-tolerance claim is narrowed, not removed

Every fault-tolerance report SHALL state its fault set, its residual and its semantics path, and the
crate SHALL make no claim of a threshold, a distance or asymptotic suppression.

#### Scenario: The report names its fault set

- **WHEN** a `check_fault_tolerance` report is displayed
- **THEN** it names the fault set's constructor and count, the worst residual, the path per gate,
  and carries no field for a threshold or a distance
