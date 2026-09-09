/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Naturality on abstractions whose residuals have closed forms.
//!
//! Low level: two qubits, `R_y(0.7) ⊗ I` as the first node and a gate as the second. High level:
//! one qubit, `R_y(0.7)`. `τ = Tr_B`, `E` prepares `|0⟩_B`. With `X` on the second qubit the square
//! commutes exactly. With `CNOT(0 → 1)` after the rotation the residual is `2√2` for every rotation:
//! pre-composition with the unitary `R ⊗ I` conjugates both Choi operators and drops out, leaving
//! `‖J(Tr_B ∘ CNOT) − J(Tr_B)‖_F`. With Kraus operators `K_b = I ⊗ ⟨b|`, each Choi has squared
//! norm `Σ_{b,b'} |Tr K_b† K_b'|² = 2² · 2 = 8`, and the cross term is
//! `Σ_{b,b'} |Tr(CNOT (I ⊗ |b⟩⟨b'|))|² = 4 · 1²`, since `CNOT = P₀ ⊗ I + P₁ ⊗ X` contributes `1`
//! to every `(b, b')`. So the squared residual is `8 + 8 − 2·4 = 8`.

use deep_causality_num_complex::Complex;
use deep_causality_quantum::{
    Abstraction, Axis, Channel, CheckVerdict, CircuitBox, CircuitModel, FROBENIUS_ON_CHOI, GateOp,
    NumericCaps, QcMorphism, QuantumErrorEnum, QubitOperator, Query, SemanticsPath, TypeAlignment,
    WireType,
};
use deep_causality_tensor::CausalTensor;

type C = Complex<f64>;

fn ry(wire: usize, theta: f64) -> CircuitBox<f64> {
    CircuitBox::Channel {
        wires: vec![wire],
        channel: Channel::unitary(&QubitOperator::rotation(Axis::Y, theta).unwrap()).unwrap(),
    }
}

/// `R_y(θ) ⊗ I` as one box on both low-level wires, so the node covers the aligned block.
fn ry_block(theta: f64) -> CircuitBox<f64> {
    let r = QubitOperator::rotation(Axis::Y, theta).unwrap();
    let one = C::new(1.0, 0.0);
    let zero = C::new(0.0, 0.0);
    let identity = CausalTensor::from_slice(&[one, zero, zero, one], &[2, 2]);
    CircuitBox::Channel {
        wires: vec![0, 1],
        channel: Channel::from_kraus(&[r.matrix().kronecker(&identity).unwrap()]).unwrap(),
    }
}

fn trace_b() -> QcMorphism<f64> {
    let zero = C::new(0.0, 0.0);
    let one = C::new(1.0, 0.0);
    let k0 = CausalTensor::from_slice(&[one, zero, zero, zero, zero, zero, one, zero], &[2, 4]);
    let k1 = CausalTensor::from_slice(&[zero, one, zero, zero, zero, zero, zero, one], &[2, 4]);
    QcMorphism::from_kraus(&[k0, k1]).unwrap()
}

fn prepare_b() -> QcMorphism<f64> {
    let zero = C::new(0.0, 0.0);
    let one = C::new(1.0, 0.0);
    QcMorphism::from_kraus(&[CausalTensor::from_slice(
        &[one, zero, zero, zero, zero, one, zero, zero],
        &[4, 2],
    )])
    .unwrap()
}

fn low(second: GateOp) -> CircuitModel<f64> {
    CircuitModel::ungrouped(
        vec![WireType::qubit(), WireType::qubit()],
        vec![
            ry_block(0.7),
            CircuitBox::Unitary {
                wires: vec![0, 1],
                program: vec![second],
            },
        ],
        vec![0, 1],
        vec![0, 1],
    )
    .unwrap()
}

fn high() -> CircuitModel<f64> {
    CircuitModel::ungrouped(vec![WireType::qubit()], vec![ry(0, 0.7)], vec![0], vec![0]).unwrap()
}

fn alignment() -> TypeAlignment<f64> {
    TypeAlignment::new(vec![(vec![0], vec![0, 1], trace_b(), prepare_b())]).unwrap()
}

#[test]
fn test_a_commuting_square_has_zero_residual_on_io_and_open() {
    let caps = NumericCaps::default();
    let a = Abstraction::new(
        low(GateOp::X(1)),
        high(),
        alignment(),
        vec![
            (Query::Io, Query::Io),
            (Query::Open(vec![0]), Query::Open(vec![0])),
        ],
    )
    .unwrap();
    let r = a.check_naturality(&caps).unwrap();
    assert_eq!(r.report.verdict(), CheckVerdict::Accepted);
    assert_eq!(r.report.examined(), 2);
    assert_eq!(r.path, SemanticsPath::Numeric);
    assert_eq!(r.norm, FROBENIUS_ON_CHOI);
    assert!(r.worst_residual() < 1e-12);
    assert!(r.bound.upper < 1e-11);
    assert!(r.entries > 0);
    assert_eq!(a.image(&Query::Io), Some(&Query::Io));
    assert_eq!(a.signature().len(), 2);
    assert_eq!(a.query_map().len(), 2);
    assert_eq!(a.low().boxes().len(), 2);
    assert_eq!(a.high().boxes().len(), 1);
    assert_eq!(a.alignment().entries().len(), 1);
}

#[test]
fn test_an_entangling_gate_breaks_the_square_by_root_two() {
    let caps = NumericCaps::default();
    let a = Abstraction::new(
        low(GateOp::Cnot {
            control: 0,
            target: 1,
        }),
        high(),
        alignment(),
        vec![(Query::Io, Query::Io)],
    )
    .unwrap();
    let r = a.check_naturality(&caps).unwrap();
    assert_eq!(r.report.verdict(), CheckVerdict::Rejected);
    let rejected = r.report.first_rejection().unwrap();
    assert_eq!(rejected.item, deep_causality_quantum::CheckItem::Index(0));
    assert!(
        (rejected.measured - 8f64.sqrt()).abs() < 1e-10,
        "{}",
        rejected.measured
    );
    // The bound brackets the residual's diamond distance: √2/4 ≤ ⋄ ≤ √2·√8.
    assert!((r.bound.lower - 8f64.sqrt() / 4.0).abs() < 1e-10);
    assert!((r.bound.upper - 8.0).abs() < 1e-10);
    // The same low-level model with the CNOT on the other side of the rotation? Same residual:
    // the dephasing kills the same off-diagonals of any unitary's Choi operator.
    let a2 = Abstraction::new(
        CircuitModel::ungrouped(
            vec![WireType::qubit(), WireType::qubit()],
            vec![
                ry_block(1.3),
                CircuitBox::Unitary {
                    wires: vec![0, 1],
                    program: vec![GateOp::Cnot {
                        control: 0,
                        target: 1,
                    }],
                },
            ],
            vec![0, 1],
            vec![0, 1],
        )
        .unwrap(),
        CircuitModel::ungrouped(vec![WireType::qubit()], vec![ry(0, 1.3)], vec![0], vec![0])
            .unwrap(),
        alignment(),
        vec![(Query::Io, Query::Io)],
    )
    .unwrap();
    assert!((a2.check_naturality(&caps).unwrap().worst_residual() - 8f64.sqrt()).abs() < 1e-10);
}

#[test]
fn test_empty_signature_is_vacuous_and_a_double_mapping_is_refused() {
    let caps = NumericCaps::default();
    let a = Abstraction::new(low(GateOp::X(1)), high(), alignment(), vec![]).unwrap();
    let r = a.check_naturality(&caps).unwrap();
    assert_eq!(r.report.verdict(), CheckVerdict::Vacuous);
    assert_eq!(r.report.examined(), 0);
    assert_eq!(r.worst_residual(), 0.0);
    let err = Abstraction::new(
        low(GateOp::X(1)),
        high(),
        alignment(),
        vec![(Query::Io, Query::Io), (Query::Io, Query::Io)],
    )
    .unwrap_err();
    assert!(
        matches!(err.0, QuantumErrorEnum::CalculationError(ref m) if m.contains("mapped twice"))
    );
    let err = Abstraction::new(
        low(GateOp::X(1)),
        high(),
        alignment(),
        vec![(Query::Io, Query::Open(vec![9]))],
    )
    .unwrap_err();
    assert!(matches!(err.0, QuantumErrorEnum::DimensionMismatch(_)));
    let a = Abstraction::new(
        low(GateOp::X(1)),
        high(),
        alignment(),
        vec![(Query::Io, Query::Io)],
    )
    .unwrap();
    assert!(matches!(
        a.square(&Query::Observe(vec![0]), &caps).unwrap_err().0,
        QuantumErrorEnum::CalculationError(_)
    ));
}

#[test]
fn test_observe_and_a_failing_swapped_program() {
    let caps = NumericCaps::default();
    // A spectator wire the alignment does not cover: low wire 1 is fresh and never an output, so
    // the aligned type of the high qubit is low wire 0 alone and observing it on both sides gives
    // one classical bit each.
    let identity = QcMorphism::<f64>::identity(2).unwrap();
    let spectator = CircuitModel::ungrouped(
        vec![WireType::qubit(), WireType::qubit()],
        vec![
            ry(0, 0.7),
            CircuitBox::Unitary {
                wires: vec![1],
                program: vec![GateOp::X(0)],
            },
        ],
        vec![0],
        vec![0],
    )
    .unwrap();
    let a = Abstraction::new(
        spectator,
        high(),
        TypeAlignment::new(vec![(vec![0], vec![0], identity.clone(), identity)]).unwrap(),
        vec![
            (Query::Observe(vec![0]), Query::Observe(vec![0])),
            (Query::Io, Query::Io),
        ],
    )
    .unwrap();
    let r = a.check_naturality(&caps).unwrap();
    assert_eq!(
        r.report.verdict(),
        CheckVerdict::Accepted,
        "{:?}",
        r.report.worst()
    );
    // Mapping the high-level rotation to a low-level model with a different angle names the query.
    let wrong = CircuitModel::ungrouped(
        vec![WireType::qubit(), WireType::qubit()],
        vec![
            ry_block(0.2),
            CircuitBox::Unitary {
                wires: vec![0, 1],
                program: vec![GateOp::X(1)],
            },
        ],
        vec![0, 1],
        vec![0, 1],
    )
    .unwrap();
    // With the block alignment, observing low wire 0 alone leaves low wire 1 as a quantum output
    // that π of the high-level type does not have: the square is ill-typed and says so.
    let ill = Abstraction::new(
        low(GateOp::X(1)),
        high(),
        alignment(),
        vec![(Query::Observe(vec![0]), Query::Observe(vec![0]))],
    )
    .unwrap();
    assert!(matches!(
        ill.square(&Query::Observe(vec![0]), &caps).unwrap_err().0,
        QuantumErrorEnum::CalculationError(ref m) if m.contains("ill-typed")
    ));
    let a = Abstraction::new(wrong, high(), alignment(), vec![(Query::Io, Query::Io)]).unwrap();
    let r = a.check_naturality(&caps).unwrap();
    assert_eq!(r.report.verdict(), CheckVerdict::Rejected);
    assert_eq!(
        r.report.first_rejection().unwrap().item,
        deep_causality_quantum::CheckItem::Index(0)
    );
}

#[test]
fn test_concrete_do_is_derived_and_agrees_on_both_sides() {
    let caps = NumericCaps::default();
    let a = Abstraction::new(
        low(GateOp::X(1)),
        high(),
        alignment(),
        vec![(Query::Open(vec![0]), Query::Open(vec![0]))],
    )
    .unwrap();
    // The opened high-level type is (input 0, fresh 1): a two-qubit state.
    let s = std::f64::consts::FRAC_1_SQRT_2;
    let state = [
        C::new(s, 0.0),
        C::new(0.0, 0.0),
        C::new(0.0, 0.0),
        C::new(s, 0.0),
    ];
    let (left, right) = a.concrete_do(&Query::Open(vec![0]), &state, &caps).unwrap();
    assert_eq!((left.d_in(), left.d_out()), (1, 2));
    assert!(left.frobenius_distance(&right, &caps).unwrap().0 < 1e-12);
    let err = a
        .concrete_do(&Query::Open(vec![0]), &state[..2], &caps)
        .unwrap_err();
    assert!(matches!(err.0, QuantumErrorEnum::DimensionMismatch(_)));
    let a_io = Abstraction::new(
        low(GateOp::X(1)),
        high(),
        alignment(),
        vec![(Query::Io, Query::Io)],
    )
    .unwrap();
    assert!(matches!(
        a_io.concrete_do(&Query::Io, &state, &caps).unwrap_err().0,
        QuantumErrorEnum::CalculationError(_)
    ));
}
