<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

Ordering follows the road map's phases as corrected by `qcl2-roadmap-verification.md`, with group 0
ahead of them because nothing else validates against empty specifications. Every group ends with a
verification task, and no group is done until `bazel test //...` is green for it. Every new numeric
kernel (groups 1, 3, 4 and 6) follows the unified-math TDD protocol: literals with stated
provenance, corner-case rows A to K in the test module doc, the defect audit, then `cargo mutants`.
A commit message is prepared at each group boundary; nothing is committed by the agent.

## 0. Restore the live specifications

- [ ] 0.1 Copy the seven archived `add-qcl` deltas into `openspec/specs/qcl-*/spec.md` as their
      live text, requirement counts 7, 11, 7, 7, 7, 9 and 9
- [ ] 0.2 Verify: `openspec validate --specs` green; this change still validates with the
      `qcl-pipeline` delta as `ADDED`

## 1. The circuit model and its two semantics

- [ ] 1.1 Add `CircuitModel<R>` under `types/circuit_model/`: boxes (encoder, unitary, channel,
      instrument, measurement), typed wires, the box-to-node grouping, declared outputs, and
      `induced_dag()` per Example 61; construction rejects mis-dimensioned wires, double writers
      and a grouping that is not a partition
- [ ] 1.2 Add the exact semantics: a Clifford program as its symplectic action through
      `clifford_conjugate`, a diagonal Table 1 gate as a `DiagonalPhase`; `SemanticsPath::Exact`
- [ ] 1.3 Add the numeric semantics kernel: density-matrix evolution of a `GateOp` program with
      `Channel` boxes through `embed_on_legs` and `apply_kraus`, and the Choi of the composite
      `2^n → 2^k` channel; `SemanticsPath::Numeric`
- [ ] 1.4 Add the entry-count cap on the numeric path, default `2^24`, counted on `NumberType`
      with a checked product, `NaturalityDimensionExceeded { n, k, entries, cap }` before
      allocating, and the examined count on success
- [ ] 1.5 Decide the small fixture: build `LatticeComplex::<2, _>::square_torus(2)` and assert
      `β₁ = 2` and `∂₁∂₂ = 0`; on failure add the hand-built `[[4,2,2]]` chain complex to
      `utils_tests` and record the decision in the change's notes
- [ ] 1.6 Verify: the numeric kernel against `apply_kraus` on one qubit and against a hand-computed
      two-qubit `CZ` Choi; the 18-qubit request refused with the exact entry count; both semantics
      agree on `Z̄` and `H̄` over the small fixture; defect audit and `cargo mutants` on the kernel

## 2. The dilation and the circuit subject

- [ ] 2.1 Add `Dilation` under `qcm`: leg dimension `d_in · d_out` per node through `set_leg_dim`,
      input outer and output inner, `ρ_{A|Pa(A)}` embedded as the identity on the unused halves,
      supports encoding the induced DAG
- [ ] 2.2 Add `CircuitModel::glue` along a shared wire, and its dilation as the induced
      factorization of the composite
- [ ] 2.3 Add `CircuitSubject` and `.over_circuit` to `QclBuilder`; `build()` rejects a cyclic
      induced DAG as `CyclicStructureUnsupported`; `Screened<R>` records its origin
- [ ] 2.4 Add `NoCompositionalModel` and return it from every abstraction constructor handed a
      model subject
- [ ] 2.5 Verify: the dilation of a two-node unitary circuit is Markov for its induced DAG at
      Q-TOL with provenance `Rederived`; the glued dilation produces no `CertificateNotInherited`;
      `.over_model` behaviour unchanged against the existing pipeline tests

## 3. The abstraction object and the naturality check

- [ ] 3.1 Add `TypeAlignment` with `τ_X` as `Channel`, the section `E_X`, the `τ ∘ E = id` check
      against `Tolerance::state()`, `SectionNotInverse`, and monoidal products of types
- [ ] 3.2 Add `QuerySignature` with `Io`, `Open(S)`, `Inc(S₁…Sₙ)` and `Observe(O)`; the
      parallelisable check at construction with `NotParallelisable` naming the path; `Open` given
      semantics as box deletion and shown equal to `intervene_mechanism` with the identity
      instrument on the dilation
- [ ] 3.3 Add `Abstraction<L, H>` with the total query map, and the derived upward abstraction per
      Proposition 18 computed on demand
- [ ] 3.4 Add `check_naturality` as a `Check<R>`: one record per query, the `SemanticsPath`, the
      norm, the amplification factor and the examined count beside the report; the ε-abstraction
      definition in the doc block with the paper cited for the exact case
- [ ] 3.5 Derive the Frobenius-to-diamond factor in `notes/` of this change, write it into the
      docstring, and test it against the `2 sin(θ/2)` closed form over a `θ` sweep
- [ ] 3.6 Verify: vacuous signature reads `Vacuous`; a swapped program rejects and names the query;
      both paths agree on the small fixture; defect audit and `cargo mutants` on the residual kernel

## 4. The structural precheck and the code as an abstraction

- [ ] 4.1 Add `check_alignment_structure`: `α(X)` by blocked reachability on the low-level DAG, the
      simple, extra-simple and full predicates, the offending pair as witness, and the scope field
      `Equivalent | Necessary` with the doc stating Theorem 51's classical scope and Remark 56's
      argument for the quantum case
- [ ] 4.2 Add the paper's Examples 54 and 55 as fixtures and pin their predicate values
- [ ] 4.3 Write the generation regression first: `check_naturality` on the strict
      `CodeAbstraction` against `check_class_invariance` and `check_clifford_action` on
      `[[18,2,3]]` and `[[32,2,4]]`, every emitted gate, verdict and witness
- [ ] 4.4 Add `CodeAbstraction` from a `LogicalBasis`: `π` to the block, `τ` the ideal decoder from
      the stabilizer generators, `E` the code-space isometry, the query map through the Table 1
      emitters, `Open(S̄) ↦ Open(π(S̄))`, `Observe(Ō)` to the logical measurement
- [ ] 4.5 Wire `check_alignment_structure` and `check_naturality` into `Validate` on the circuit
      subject, in that order, sticky failure, named stages
- [ ] 4.6 Verify: the regression passes on both fixtures; omitting `S̄`'s CZ pairs fails it; the
      precheck on a `CircuitModel` reads `Necessary`; a rejecting precheck stops `check_naturality`
      from forming a matrix

## 5. Fault sets and the fault-tolerance predicate

- [ ] 5.1 Add `FaultSet` with `pauli_weight(t)`, `declared` and `from_dem`, counted on
      `NumberType` as `C(n, t) · 3^t` and refused above the cap before allocating
- [ ] 5.2 Add the Pauli-basis propagator: one term through Clifford gates by the tableau rule,
      branching through `T`, `T†`, `CS†`, `CCZ` and wide `Cmz`, coefficients in `Complex<R>`, term
      count checked against `PauliTermCountExceeded` before allocating
- [ ] 5.3 Add correctability against `LogicalBasis`'s stabilizer generators: a normalizer term that
      is not a stabilizer is a logical fault
- [ ] 5.4 Add `check_fault_tolerance` over the enlarged signature: per-fault residuals, the worst,
      the count, the witness `(location, Pauli, term)`, the `SemanticsPath` per record
- [ ] 5.5 Derive the oracle facts by hand in `notes/` of this change (`Z̄`, `X̄` weight-one FT;
      `S̄` with CZ pairs not, on `[[18,2,3]]`), then add the Haruna filter with per-gate labels
      `Exact` and `PauliBasisToCap`
- [ ] 5.6 Verify: `X` through `S̄` gives `X Z Z` up to phase; `X` through `T` gives two terms of
      modulus `1/√2`; the filter matches the derived oracle; `T̄`'s record is labelled; empty set
      reads `Vacuous`; defect audit and `cargo mutants` on the propagator

## 6. Composition and the three chain consumers

- [ ] 6.1 Add `Abstraction::compose`: `π = π₁ ∘ π₂`, `τ = τ₂ ∘ τ₁`, both constants as largest
      singular values of the composition superoperators on Choi space, the norm and the bound in the
      report's provenance
- [ ] 6.2 Add `lean/DeepCausalityFormal/Quantum/Abstraction.lean` with Proposition 17 in the exact
      case over the pair-indexed matrix model, and bind it in `lean/THEOREM_MAP.md` to the exact
      composition test; register the Bazel `lean_test` target
- [ ] 6.3 Add the three chain consumers under `examples/quantum_examples/qcl_examples/`:
      concatenated `[[4,2,2]]`, code switching with the gadget as low-level query, and a distillation
      round labelled as an example; each with a `rust_binary` in `BUILD.bazel`, a `FloatType` alias
      in `main.rs`, and the lifts from `deep_causality_num`
- [ ] 6.4 Verify: exact links compose to residual zero; the tightness pair exceeds a bound with
      either constant set to one; each consumer's measured residual is at most its recorded bound;
      the consumers run at `f32`, `f64` and `Float106`

## 7. The decoder as an abstraction

- [ ] 7.1 Add `DemModel::from_graph` over a frozen `CausaloidGraph` under `qcm`, with detectors,
      observables and latent mechanisms, and `induced_dag()`
- [ ] 7.2 Add the `dem` feature implying `qcm`, `DemModel::from_stim_text` for `error`, `detector`
      and `logical_observable` lines, unknown lines refused by name; enable `dem` in `BUILD.bazel`
- [ ] 7.3 Add `DecoderAbstraction` with `τ` as a caller-supplied channel or stochastic matrix lifted
      through the FStoch embedding; no `Decoder` trait
- [ ] 7.4 Add the logical attribution query over a `FaultSet`, ranked by residual
- [ ] 7.5 Build the small memory-experiment fixture with one injected correlated two-qubit error
      and its two `DemModel`s, with and without the mechanism
- [ ] 7.6 Verify: the omitted mechanism is exposed at the injected location; the complete model
      passes; attribution ranks the injected location first; the three-line Stim text parses and the
      `repeat` line is refused

## 8. The crosstalk consumer over circuits, and close-out

- [ ] 8.1 Re-express the crosstalk consumer's candidates as `CircuitModel` values with normalised
      dilations and the same parental structure, run `.over_circuit`, and keep the v1 example beside
      it
- [ ] 8.2 Verify: three admitted, the cyclic fourth refused at `build()`, the plan `{do(Q1),
      do(Q2)}` at cost 2 against tomography at 200, H₁ the survivor
- [ ] 8.3 Register every new test file in its `mod.rs` and in `tests/BUILD.bazel`; add every new
      example's `rust_binary`; `make check_examples` green
- [ ] 8.4 Update `qcl-design-note.md` §9 with a QCL-2 row per group, each check's witness through
      `lean/THEOREM_MAP.md` or the statement that it has none, and `LEAN_QUANTUM.md` with the new
      Lean file
- [ ] 8.5 Verify: `bazel test //...` green, `cargo clippy --workspace --all-targets` clean,
      `cargo fmt --check` clean, `openspec validate --specs` green, the default and `no-std` builds
      of `deep_causality_quantum` compile the ungated abstraction layer
