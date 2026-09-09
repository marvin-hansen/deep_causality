<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

## Context

`deep_causality_quantum` 0.2.5 ships QCL v1: the decision form (`Check<R>`, `CheckReport<R>`,
`Tolerance<R>`), the carriers (`Channel` CPTP-checked once), the hypothesis layer
(`ProcessFactors`, `FactorSupports`, `Hypothesis` with `intervene_mechanism`, `compose` and the
`Inherited | Rederived` provenance), the exact code checks (`LogicalBasis` with both stabilizer
families, `check_class_invariance` over the code space, `clifford_conjugate` and
`check_clifford_action`), the Table 1 emitters, the reified `QuantumCircuit`, and the builder
`QclBuilder` with three subjects and the `validate` → `Screened<R>` → `control` hand-off. Three
consumers run at three precisions. Every prerequisite the road map lists under Phase 0 is in code
(verification V-10).

Two things bound this design that the road map did not weigh. First, no dense operator on more
than a handful of qubits exists anywhere in the crate, and none can: the fixtures the code path
runs on are 18 and 32 qubits, and every check that reaches them is an 𝔽₂ or rational computation
over supports. Second, the theory the road map imports is stated for classical causal models in
one place it matters (Theorem 51) and stated exactly, with no approximate notion, throughout. The
register [`qcl2-roadmap-verification.md`](../../notes/quantum/qcl2-roadmap-verification.md) records
fifteen corrections; the decisions below apply them. The road map's own decisions D2-1 to D2-5 are
kept where the register does not touch them and cited by their numbers.

The paper is [`CausalandCompositionalAbstraction-2602.16612v1.pdf`](../../notes/quantum/CausalandCompositionalAbstraction-2602.16612v1.pdf).
Definitions and results are cited by number as they appear there.

## Goals / Non-Goals

**Goals:**

- Represent the relation between a physical process and its logical description as a value the
  pipeline holds, composes and perturbs: `Abstraction<L, H>`.
- Decide naturality on every register the code path already reaches, exactly, and on small
  registers numerically, with the report saying which path decided and to what residual.
- Make fault tolerance a predicate over an enlarged query signature rather than a new kind of
  check, and produce the Haruna filter as the first result the substrate alone could not.
- Replace certificate inheritance with abstraction composition under a stated, computed law.
- Validate a decoder's classical picture against the physical circuit as a causal statement.

**Non-Goals:**

- Implementing a decoder. `τ` is validated, never built (D2-5).
- Diamond-norm evaluation. There is no SDP in the workspace; the report states the Frobenius
  proxy and its amplification (D2-2).
- Claiming thresholds, distances or asymptotic suppression. The fault-tolerance claim is "holds
  under fault set `F` to residual `ε`" (D2-4).
- Making a general Barrett–Lorenz–Oreshkov process operator a compositional model. The circuit
  is carried and BLO's dilation theorem is cited (road map §9 rule 5; paper §8).
- Cyclic structures, device models, graph traversal and topology ownership, as in v1 §8.
- Editing the road map. The register is its errata.

## Decisions

### D1. Two semantics paths, and the report names which one decided

**Measured.** The Choi operator of the composite `τ ∘ U`, a channel `2^n → 2^k`, has `2^(2n+2k)`
entries: `1.0e6` on `[[8,2,2]]`, `1.1e12` (17 TB) on `[[18,2,3]]`, `2.9e20` on `[[32,2,4]]`. The Choi
of the low-level unitary alone is `2^(4n)`. The road map's `check_naturality` cannot be formed on
the fixtures its exit criteria name, on any machine.

`CircuitModel<R>` therefore carries two semantics functors and `check_naturality` runs whichever
the query admits:

- **Exact.** A program of Pauli and Clifford gates is carried as its symplectic action on
  `(x, z)`, through `clifford_conjugate`; a diagonal Table 1 gate is carried as a `DiagonalPhase`
  with its `Rational<i64>` phase polynomial. A square commutes exactly when the images agree up
  to stabilizers through `LogicalBasis::are_logically_equivalent` and the phase ratio is integral
  through `DiagonalPhase::phase_at`. No width limit; the residual is zero or it is not.
- **Numeric.** A general program with noise boxes is evolved as a density matrix by embedding each
  gate's unitary or each noise box's Kraus family on the register through `embed_on_legs` and
  `apply_kraus`; both sides of the square are formed as Choi operators of the composite
  `2^n → 2^k` channel and compared in Frobenius norm against `Tolerance::state()`. The path carries
  a cap on the entry count, default `2^24` (so `n + k ≤ 12`), refuses above it with
  `NaturalityDimensionExceeded { n, k, entries, cap }` before allocating, and reports the entries
  it formed. This is the discipline D1 and D7 of `add-qcl` apply to the code-space enumeration and
  the design cover.

The report carries `SemanticsPath::{Exact, Numeric}` beside the norm and the amplification, and a
scenario asserts that a query decidable by both paths gets the same verdict on a fixture the
numeric path reaches.

*Alternative rejected.* State-vector simulation of the low-level side only. `SimQpu` caps at 24
qubits and produces samples, not channels; a pure-state path cannot carry noise boxes and cannot
form the composite channel the square compares.

### D2. The generation regression is a reduction, and it is written first

On the exact path the strict `CodeAbstraction`'s naturality check *is* v1's two predicates:
`Z̄(γ) ↦ X̄(γ̃)` up to stabilizers is `check_clifford_action`, and the phase ratio integral on the
code space is `check_class_invariance`. The regression therefore does not compare two independent
computations. It pins the reduction: for a noiseless physical model and every Table 1 gate on every
fixture, `check_naturality` on the strict code abstraction and the v1 stage return the same verdict
and, where the v1 stage names a witness, the same witness. It is the first test Phase 3 writes, and
nothing about a noisy or composite abstraction is claimed until it passes. The numeric path's
agreement with the exact path on the small fixture (D1) is the second half of the same bridge.

### D3. The dilation fixes a leg convention the flat store does not have

`FactorSupports` declares one leg per node under `support(A) = {A} ∪ Pa(A)` with every leg
defaulting to a qubit, and `structure_from_supports` reads the DAG off that. Barrett, Lorenz and
Oreshkov's factor `ρ_{A|Pa(A)}` acts on `A^in ⊗ ⨂_{P∈Pa(A)} P^out`. One leg per node cannot name
both spaces.

`Dilation` fixes the convention: leg `A` has dimension `d_A^in · d_A^out`, declared through
`set_leg_dim`, with the input index outer and the output index inner in the row-major layout;
`ρ_{A|Pa(A)}` is embedded as the identity on `A^out` and on every `P^in`. The commutation check then
runs on the operators BLO's proposition says commute, and the DAG the supports encode is the
circuit's induced DAG (paper Example 61: a vertex per encoder input, per unitary box and per
measurement output, an edge per wire). A fixture asserts that the dilation of a two-node unitary
circuit is Markov for its induced DAG at Q-TOL.

*Consequence for v1's open question.* A composite of two circuits is a circuit, and its dilation is
the induced factorization D9 and X-2 could not construct. `CircuitModel::glue` followed by
`Dilation` supplies it; `CertificateNotInherited` remains the v1 behaviour for a caller who holds
only marginals.

### D4. `τ` carries a section, and surjectivity is its witness

Definition 14 asks for an epic `τ_X`; §7 does not restate the condition for QC; Proposition 18,
which derives the upward abstraction, needs every high-level sharp state to be `τ ∘ s` for a
low-level `s`. `TypeAlignment` therefore carries, beside each `τ_X : π(X) → X`, a section
`E_X : X → π(X)` and checks `τ_X ∘ E_X = id_X` at construction against `Tolerance::state()`,
refusing with `SectionNotInverse` otherwise. For a code `E` is the code-space isometry and `τ` the
ideal decoder built from the stabilizer generators; the composite is the identity on `2^k`. On the
exact path the section is the statement that `τ` inverts the encoding on the code space, which
`LogicalBasis` decides. Upward abstractions are derived from the downward one by composing with the
low-level sharp states per Proposition 18 and are not stored (road map §3).

*Alternative rejected.* A rank test on `τ`'s Choi operator. Rank is a numeric-path quantity, has no
exact-path form, and does not exhibit the state Proposition 18 needs.

### D5. `check_alignment_structure` states what Theorem 51 licenses

Theorem 51 is stated for causal models in a Markov, cd or Cartesian structure category, each of
which has copy maps as syntax, and its proof runs through Lemma 66 on normalised network diagrams.
Quantum compositional models of DAGs (Definition 59) have no copy maps, and the paper proves no
characterisation of component-level abstraction for them (§8 names it future work).

The stage computes Definition 49's `α(X)`, the low-level vertices with a directed path to `π(X)`
avoiding `π(Pa(X))`, and the three predicates *simple*, *extra-simple* and *full*, as a graph
computation on the two DAGs, and reports which held with the offending pair as witness. Its
documentation states the scope in one sentence each way: for two classical models it is Theorem 51
and decides mechanism-level abstraction; for a quantum low-level model it is a necessary condition,
on Remark 56's argument that no network diagram for the opened model with inputs `π(Pa(X))` exists
when `α(X)` meets the low-level inputs, and the report reads `Necessary`, not `Equivalent`. The
paper's Examples 54 and 55 are the classical fixtures. It runs in `validate` before any operator is
formed, as the road map has it.

### D6. The ε-law has two constants, and both are computed

With `τ = τ₂ ∘ τ₁` and `π = π₁ ∘ π₂`, the composite defect splits as
`τ₂ (τ₁ ⟦π₁ Q_M⟧_L − ⟦Q_M⟧_M τ₁) + (τ₂ ⟦Q_M⟧_M − ⟦Q⟧_H τ₂) τ₁`, so
`ε ≤ ‖τ₂‖_post · ε₁ + ‖τ₁‖_pre · ε₂`, with the constants the induced norms of post-composition by
`τ₂` and pre-composition by `τ₁` in the norm the residuals are measured in. In the diamond norm both
are one. In Frobenius norm on Choi operators neither is one in general, and the road map's law
dropped the second factor.

`Abstraction::compose` computes both constants as the largest singular values of the composition
superoperators on Choi space, which are finite matrices under the D1 cap, and records them with the
norm in the composite report's provenance. The exact case `ε₁ = ε₂ = 0 ⇒ ε = 0` is Proposition 17
and is the Lean statement. The tightness test is on a constructed pair where both constants exceed
one, so a law with either constant assumed to be one fails it.

### D7. Faults propagate in the Pauli basis, to a cap

`clifford_conjugate` refuses `T`, `Tdg`, `Csdg`, `Ccz` and `Cmz` on three or more qubits, correctly:
their conjugation action is not a Pauli. `logical_t` emits all three over the support, its pairs and
its triples. A single Pauli fault before a `CS†` or `CCZ` in `T̄`'s program therefore leaves the
tableau, and D1 rules out simulating an 18-qubit register.

A fault is carried through the program as a linear combination of Paulis on the qubits it has
touched, with coefficients in `Complex<R>`. Through a Clifford gate it stays one term. Through `T`,
`CS†` or `CCZ` it branches, and the term count is bounded by `4^w` for the support weight `w` it
can spread over; the propagator counts first and refuses above `PauliTermCountExceeded` (default
`2^16` terms) before allocating. Correctability of the resulting error set is decided against the
stabilizer generators `LogicalBasis` carries: a term in the normalizer that is not a stabilizer is a
logical fault, and the report's witness names the fault location and that term. The Haruna filter's
verdict carries `Exact` for the Clifford subset (`Z̄`, `X̄`, `S̄`, `CZ̄`, `H̄`) and `PauliBasisToCap`
for `T̄`, `CS̄†` and `CC̄Z`. The external-oracle facts the road map lists are derived by hand in the
change's notes before the asserting test is written.

### D8. The Frobenius-to-diamond factor is derived before it is documented

For a Hermiticity-preserving `Φ` with unnormalised Choi operator `J(Φ)` on `d_in · d_out`
dimensions, `‖Φ‖_⋄ ≤ d_in · ‖J(Φ)‖_1 ≤ d_in · √(d_in d_out) · ‖J(Φ)‖_F`. The naturality report
carries the factor it used. One test compares the bound against a pair whose diamond distance has
a closed form, two single-qubit unitary channels differing by a rotation by `θ`. If the derivation
at implementation time yields a different constant, the docstring follows the derivation; the
requirement is that a stated factor be recorded and tested, not that this one be it.

### D9. Interchange queries are validated at construction

`Inc(S₁, …, Sₙ)` is defined only on pairwise disjoint, each parallelisable, subsets of non-input
vertices (paper §7.2: no directed path between two members of one set). `QuerySignature` checks
that on the low-level DAG when the query is built and refuses with `NotParallelisable` naming the
path, so no semantics is ever asked for a query the paper does not define. `Open(S)` is the
mechanism-level intervention v1 names `intervene_mechanism`; the query wrapper adds a name and
changes nothing (road map §3).

### D10. The decoder is a black box, and the detector model is a `CausaloidGraph`

`DecoderAbstraction` is `Abstraction<CircuitModel, DemModel>` with `τ` the decoder's channel from
syndromes to logical outcomes, supplied by the caller as a `Channel` or a classical stochastic
matrix. `DemModel` is a `CausaloidGraph` over detector and observable variables with error
mechanisms as latent parents, and the Stim detector-error-model text format is one constructor
behind the `dem` feature. Nothing in the crate decodes, and no `Decoder` trait appears; if one does,
D2-5 has been violated. The logical attribution query enumerates the fault-set queries whose squares
fail and ranks them by residual; because the low-level model is a circuit, entanglement-mediated
correlations are represented as such rather than as a classical common cause.

### D11. Feature placement

`CircuitModel`, `TypeAlignment`, `QuerySignature`, `Abstraction`, `check_naturality`,
`check_alignment_structure`, `CodeAbstraction` and `FaultSet` need `alloc`, the tensor stack, the
homology stack and the crate's carriers; they compile in the default and `no-std` builds and are
not gated. `Dilation`, `.over_circuit`, `CircuitModel::glue`'s factorization and `DemModel` are
`qcm`-gated, because `ProcessFactors` and `CausaloidGraph` are, and `qcm` implies `std`. The Stim
parser is `dem`, which implies `qcm`. `BUILD.bazel` enables `dem` beside `qpu`.

### D12. The seven live specifications are restored before implementation

The live `openspec/specs/qcl-*/` directories are empty; the 57 requirements live only in the
archived `add-qcl` deltas. Task 0 restores them from the archive as `ADDED` requirements, exactly
as archived, and validates. Every QCL-2 requirement against an existing capability is written as
`ADDED` so the change validates with or without task 0 having run.

### D13. The crosstalk exit criterion is the decision, not the numbers

The crosstalk consumer's two-leg factor `diag(0.85, 0.05, 0.05, 0.05)` traces over the child to
`diag(0.9, 0.1)`, not the identity; it is not the Choi operator of a trace-preserving channel and
no circuit's dilation produces it. The Phase 1 exit criterion is re-stated: with each admitted
candidate re-expressed as a `CircuitModel` whose dilation yields normalised factors with the same
parental structure, the pipeline admits the same three by Markov and C₃, refuses the cyclic fourth at
`build()`, plans `{do(Q1), do(Q2)}` at cost 2 against tomography at 200, and names H₁ the
survivor. The factor values change; the screen and the plan do not.

### D14. Fixtures

The exact path runs on `[[18,2,3]]` (`square_torus(3)`) and `[[32,2,4]]` (`square_torus(4)`) as the
code path does today. The numeric path runs on `[[8,2,2]]` from `square_torus(2)` if a 2×2 periodic
lattice builds as a valid complex, which the tree does not exercise; the fallback is a hand-built
`[[4,2,2]]` chain complex with one 2-cell (`∂₂` the all-ones column), four 1-cells and one 0-cell
(`δ₀` the all-ones row), which satisfies `∂₁∂₂ = 4 ≡ 0` and has `k = 2`. The classical fixtures for
D5 are the paper's Examples 54 and 55. Every published wall-clock figure carries the machine
(M3 Max, 16 cores, 128 GB).

### D15. Counts and caps

Every cap and every count is ℕ on `NumberType`, every dimension product is `checked_mul`, and every
real quantity follows `FloatType`, as `add-qcl` D6 has it. Configuration literals enter through
`lift`; `f64` appears at the display boundary and nowhere else.

## Risks / Trade-offs

**[The numeric path reaches almost nothing physical]** → It is the bridge to the exact path, not the
workhorse. Its job is to agree with the exact path on the small fixture and to carry noise boxes and
general channels where the exact path cannot. The cap makes the limit visible before it is a hang.

**[The Pauli-basis propagator's cap bites on qLDPC representatives]** → `4^w` at weight `w` in the
tens is out of reach. The cap errors and names the count, as `TUPLE_ENUMERATION_CAP` does for the
same family; a per-gate `PauliBasisToCap` label says which verdicts were reached.

**[Theorem 51's quantum scope is a necessary condition only]** → Stated on the type, in the report
and in the docs (D5). A layout that fails the precheck fails for a reason the paper proves; one that
passes has not been shown to support a mechanism-level abstraction, and the report does not say it
has.

**[The ε-law's constants can be large in Frobenius]** → They are computed, not assumed, and the
report shows them. A caller who needs the tight bound needs the diamond norm, which is optional and
not on the critical path.

**[The dilation is bigger than the process operator]** → A leg of dimension `d_in · d_out` per node
squares the per-node dimension. The design-time positioning absorbs it; the cap reports it.

**[Restoring the live specifications from the archive re-opens a closed change]** → It copies text
that was reviewed and archived, adds nothing, and is one mechanical task with `openspec validate`
as its check.

**[The square_torus(2) fixture may not build]** → The hand-built `[[4,2,2]]` complex is the
fallback and is four lines of `CsrMatrix`.

**[Stim's format changes]** → The parser handles the `error(p) D… L…` and `detector`/`logical_observable`
lines of the current text format, sits behind `dem`, and `DemModel` takes any `CausaloidGraph`.

## Migration Plan

Additive. No shipped signature changes. `Validate` on the model and plant subjects is unchanged;
the circuit subject is a fourth constructor. Callers holding only marginals keep v1's behaviour,
including `CertificateNotInherited`. release-plz derives the bump from the commit messages.

## Open Questions

- Whether `LatticeComplex::<2, _>::square_torus(2)` is a valid complex with `β₁ = 2` (D14). Decided
  at the first task of the abstraction group; the fallback is fixed.
- The exact Frobenius-to-diamond constant (D8). The requirement is that a derived, tested factor be
  recorded.
- Whether the Pauli-basis propagator's default term cap should follow the support weight rather
  than a fixed count. Decided from the `[[18,2,3]]` `T̄` measurement in the fault group.
