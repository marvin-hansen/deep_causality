/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The generation regression and the two-path bridge for the code abstraction.
//!
//! On the exact path every Table 1 gate holds on `[[18,2,3]]` and `[[32,2,4]]`, and the verdict
//! agrees gate by gate with v1's `check_class_invariance` (diagonal gates) and
//! `check_clifford_action_on_qubit` (`H̄`). The one constructed failure, `S̄` with its `CZ` pairs
//! omitted, is rejected by both: the program's phase `n/4` is `1/2` at overlap two where a parity
//! function gives `0`. On `[[8,2,2]]` the numeric path agrees with the exact one on every gate.

use deep_causality_homology::ChainComplex;
use deep_causality_quantum::{
    CheckVerdict, CodeAbstraction, DiagonalPhase, GateOp, LogicalGate, NumericCaps,
    QuantumErrorEnum, SemanticsPath,
};
use deep_causality_topology::LatticeComplex;

type W = u64;

fn torus(l: usize) -> LatticeComplex<2, f64> {
    LatticeComplex::<2, f64>::square_torus(l)
}

#[test]
fn test_table_one_holds_exactly_on_both_torus_fixtures() {
    for l in [3usize, 4] {
        let complex = torus(l);
        let gates = CodeAbstraction::<W>::table_one(2);
        assert_eq!(gates.len(), 11);
        let ca = CodeAbstraction::<W>::new(&complex, gates).unwrap();
        let exact = ca.check_naturality_exact::<f64>().unwrap();
        assert_eq!(exact.path, SemanticsPath::Exact);
        assert_eq!(exact.report.examined(), 11);
        assert_eq!(
            exact.report.verdict(),
            CheckVerdict::Accepted,
            "{:?}",
            exact.gates
        );
        assert!(exact.gates.iter().all(|g| g.holds && g.witness.is_none()));
        assert_eq!(ca.code().n(), complex.num_cells(1));
        assert_eq!(ca.duals().len(), 2);
    }
}

#[test]
fn test_regression_agrees_with_the_v1_predicates_on_every_gate() {
    for l in [3usize, 4] {
        let complex = torus(l);
        let ca = CodeAbstraction::<W>::new(&complex, CodeAbstraction::<W>::table_one(2)).unwrap();
        let exact = ca.check_naturality_exact::<f64>().unwrap();
        for verdict in &exact.gates {
            match &verdict.gate {
                LogicalGate::Z(i) | LogicalGate::S(i) | LogicalGate::T(i) => {
                    let gamma = ca.basis().homology()[*i].clone();
                    let phase = match verdict.gate {
                        LogicalGate::Z(_) => DiagonalPhase::z(gamma),
                        LogicalGate::S(_) => DiagonalPhase::s(gamma),
                        _ => DiagonalPhase::t(gamma),
                    };
                    let v1 = ca
                        .basis()
                        .check_class_invariance(&phase, ca.code().z_generators())
                        .unwrap();
                    assert_eq!(v1.holds, verdict.holds, "{}", verdict.gate.name());
                }
                LogicalGate::H(i) => {
                    let program = ca.program::<f64>(&verdict.gate).unwrap();
                    let v1 = ca
                        .basis()
                        .check_clifford_action_on_qubit(&program, *i, ca.duals())
                        .unwrap();
                    assert_eq!(v1.holds, verdict.holds);
                }
                LogicalGate::X(_) | LogicalGate::Cz(_, _) => assert!(verdict.holds),
            }
        }
    }
}

#[test]
fn test_omitting_the_cz_pairs_of_s_bar_fails_both_generations() {
    let complex = torus(3);
    let ca = CodeAbstraction::<W>::new(&complex, vec![LogicalGate::S(0)]).unwrap();
    let gamma = ca.basis().homology()[0].clone();
    let without_pairs: Vec<GateOp> = gamma.support().map(GateOp::S).collect();
    let verdict = ca
        .check_gate_program(&LogicalGate::S(0), &without_pairs)
        .unwrap();
    assert!(!verdict.holds);
    assert!(
        verdict
            .witness
            .as_deref()
            .unwrap()
            .contains("not a function of the block parities"),
        "{:?}",
        verdict.witness
    );
    // v1's predicate on the polynomial that program implements, n/4, rejects as well.
    let n_over_4 = DiagonalPhase::new(gamma, vec![0, 1], 2).unwrap();
    let v1 = ca
        .basis()
        .check_class_invariance(&n_over_4, ca.code().z_generators())
        .unwrap();
    assert!(!v1.holds);
    // The complete program holds.
    let complete = ca.program::<f64>(&LogicalGate::S(0)).unwrap();
    assert!(
        ca.check_gate_program(&LogicalGate::S(0), &complete)
            .unwrap()
            .holds
    );
    // The S̄ program handed in as H̄ fails the tableau side: Z̄(γ) is fixed, not sent to X̄(γ̃).
    let ca_h = CodeAbstraction::<W>::new(&complex, vec![LogicalGate::H(0)]).unwrap();
    let verdict = ca_h
        .check_gate_program(&LogicalGate::H(0), &complete)
        .unwrap();
    assert!(!verdict.holds);
    assert!(
        verdict.witness.as_deref().unwrap().contains("Z̄ ↦ X̄: false"),
        "{:?}",
        verdict.witness
    );
}

#[test]
fn test_numeric_path_agrees_with_the_exact_path_on_the_small_torus() {
    let complex = torus(2);
    let gates = vec![
        LogicalGate::Z(0),
        LogicalGate::X(1),
        LogicalGate::S(0),
        LogicalGate::T(1),
        LogicalGate::H(0),
        LogicalGate::Cz(0, 1),
    ];
    let ca = CodeAbstraction::<W>::new(&complex, gates.clone()).unwrap();
    let exact = ca.check_naturality_exact::<f64>().unwrap();
    assert_eq!(exact.report.verdict(), CheckVerdict::Accepted);
    let caps = NumericCaps::default();
    let numeric = ca.numeric_abstractions::<f64>().unwrap();
    assert_eq!(numeric.len(), gates.len());
    for ((gate, abstraction), verdict) in numeric.iter().zip(&exact.gates) {
        assert_eq!(gate, &verdict.gate);
        let r = abstraction.check_naturality(&caps).unwrap();
        assert_eq!(r.path, SemanticsPath::Numeric);
        assert_eq!(
            r.report.verdict(),
            CheckVerdict::Accepted,
            "{}: residual {}",
            gate.name(),
            r.worst_residual()
        );
        assert!(
            r.worst_residual() < 1e-8,
            "{}: {}",
            gate.name(),
            r.worst_residual()
        );
        assert_eq!(r.report.accepted(), verdict.holds);
    }
}

#[test]
fn test_construction_errors() {
    let complex = torus(3);
    let err = CodeAbstraction::<W>::new(&complex, vec![LogicalGate::T(5)]).unwrap_err();
    assert!(
        matches!(err.0, QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("logical qubit 5"))
    );
    let err = CodeAbstraction::<W>::new(&complex, vec![LogicalGate::Cz(1, 1)]).unwrap_err();
    assert!(matches!(err.0, QuantumErrorEnum::DimensionMismatch(_)));
    assert_eq!(LogicalGate::Cz(0, 1).qubits(), vec![0, 1]);
    assert_eq!(LogicalGate::H(1).name(), "H̄(1)");
    let ca = CodeAbstraction::<W>::new(&complex, vec![]).unwrap();
    let exact = ca.check_naturality_exact::<f64>().unwrap();
    assert_eq!(exact.report.verdict(), CheckVerdict::Vacuous);
    assert!(ca.gates().is_empty());
}
