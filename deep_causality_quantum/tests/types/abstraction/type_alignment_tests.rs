/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Type alignments: the section check and the assembly of `τ` on a type in wire order.
//!
//! `H · H = I` makes `(τ, E) = (H, H)` a valid entry and `(H, id)` an invalid one with residual
//! `‖J(H) − J(I)‖_F = √8`: each unitary Choi operator has squared Frobenius norm `d² = 4`, and the
//! cross term `Tr J(H)†J(I) = |Tr H|²` is zero.
//! The partial trace `Tr_B` with the preparation `|0⟩_B` as section is the code-like case.

use deep_causality_num_complex::Complex;
use deep_causality_quantum::{
    AlignmentSide, NumericCaps, QcMorphism, QuantumErrorEnum, QubitOperator, TypeAlignment,
};
use deep_causality_tensor::CausalTensor;

type C = Complex<f64>;

fn id() -> QcMorphism<f64> {
    QcMorphism::identity(2).unwrap()
}

fn h() -> QcMorphism<f64> {
    QcMorphism::from_kraus(&[QubitOperator::<f64>::hadamard().matrix().clone()]).unwrap()
}

/// `Tr_B` on two qubits as a channel `4 → 2`: Kraus operators `⟨0|_B` and `⟨1|_B`.
fn trace_b() -> QcMorphism<f64> {
    let zero = C::new(0.0, 0.0);
    let one = C::new(1.0, 0.0);
    let k0 = CausalTensor::from_slice(&[one, zero, zero, zero, zero, zero, one, zero], &[2, 4]);
    let k1 = CausalTensor::from_slice(&[zero, one, zero, zero, zero, zero, zero, one], &[2, 4]);
    QcMorphism::from_kraus(&[k0, k1]).unwrap()
}

/// Prepare `|0⟩_B` beside the input: a channel `2 → 4`.
fn prepare_b() -> QcMorphism<f64> {
    let zero = C::new(0.0, 0.0);
    let one = C::new(1.0, 0.0);
    QcMorphism::from_kraus(&[CausalTensor::from_slice(
        &[one, zero, zero, zero, zero, one, zero, zero],
        &[4, 2],
    )])
    .unwrap()
}

#[test]
fn test_a_section_that_inverts_its_channel_is_admitted() {
    let a = TypeAlignment::new(vec![(vec![0], vec![0], h(), h())]).unwrap();
    assert_eq!(a.entries().len(), 1);
    assert_eq!(a.entries()[0].high(), &[0]);
    assert_eq!(a.entries()[0].tau().d_in(), 2);
    let code_like =
        TypeAlignment::new(vec![(vec![0], vec![0, 1], trace_b(), prepare_b())]).unwrap();
    assert_eq!(code_like.entries()[0].low(), &[0, 1]);
    assert_eq!(code_like.low_for(&[0]).unwrap(), vec![0, 1]);
}

#[test]
fn test_a_section_that_does_not_invert_is_refused_with_the_residual() {
    let err = TypeAlignment::new(vec![(vec![0], vec![0], h(), id())]).unwrap_err();
    match err.0 {
        QuantumErrorEnum::SectionNotInverse(msg) => {
            assert!(msg.contains("2.828427124746"), "√8, {msg}")
        }
        other => panic!("{other:?}"),
    }
    let mismatch = TypeAlignment::new(vec![(vec![0], vec![0, 1], trace_b(), id())]).unwrap_err();
    assert!(matches!(mismatch.0, QuantumErrorEnum::DimensionMismatch(_)));
    let twice = TypeAlignment::new(vec![
        (vec![0], vec![0], id(), id()),
        (vec![0], vec![1], id(), id()),
    ])
    .unwrap_err();
    assert!(
        matches!(twice.0, QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("high-level wire 0"))
    );
    let twice_low = TypeAlignment::new(vec![
        (vec![0], vec![0], id(), id()),
        (vec![1], vec![0], id(), id()),
    ])
    .unwrap_err();
    assert!(
        matches!(twice_low.0, QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("low-level wire 0"))
    );
}

#[test]
fn test_tau_is_assembled_in_ascending_wire_order() {
    let caps = NumericCaps::default();
    // Entries given out of order: high 1 ↔ low 1 with H, high 0 ↔ low 0 with the identity.
    let a = TypeAlignment::new(vec![
        (vec![1], vec![1], h(), h()),
        (vec![0], vec![0], id(), id()),
    ])
    .unwrap();
    let tau = a.tau_for(&[0, 1], &caps).unwrap();
    let expect = id().tensor(&h(), &caps).unwrap();
    assert!(tau.frobenius_distance(&expect, &caps).unwrap().0 < 1e-15);
    let section = a.section_for(&[1, 0], &caps).unwrap();
    assert!(
        section.frobenius_distance(&expect, &caps).unwrap().0 < 1e-15,
        "H is its own section here"
    );
    // Crossed wire order: high 0 ↔ low 1, high 1 ↔ low 0 both with H; τ on (0,1) must swap legs.
    let crossed = TypeAlignment::new(vec![
        (vec![0], vec![1], h(), h()),
        (vec![1], vec![0], id(), id()),
    ])
    .unwrap();
    let tau = crossed.tau_for(&[0, 1], &caps).unwrap();
    // Low legs ascending (0, 1) carry (id, H) but land on high (1, 0): the morphism is swap ∘ (id ⊗ H).
    let swap =
        QcMorphism::from_channel(&deep_causality_quantum::swap_channel::<f64>(2).unwrap()).unwrap();
    let expect = id()
        .tensor(&h(), &caps)
        .unwrap()
        .then(&swap, &caps)
        .unwrap();
    assert!(tau.frobenius_distance(&expect, &caps).unwrap().0 < 1e-15);
    // A partial type and an unaligned wire are refused; the empty type is the trivial morphism.
    let code_like =
        TypeAlignment::new(vec![(vec![0, 1], vec![0, 1, 2], id_on(4).0, id_on(4).1)]).unwrap();
    assert!(
        matches!(code_like.tau_for(&[0], &caps).unwrap_err().0, QuantumErrorEnum::CalculationError(ref m) if m.contains("as a whole"))
    );
    assert!(
        matches!(a.tau_for(&[7], &caps).unwrap_err().0, QuantumErrorEnum::CalculationError(ref m) if m.contains("not aligned"))
    );
    assert_eq!(a.tau_for(&[], &caps).unwrap().d_in(), 1);
}

/// An identity-like pair `8 → 4` and `4 → 8`: trace out one of three qubits, prepare it back.
fn id_on(_: usize) -> (QcMorphism<f64>, QcMorphism<f64>) {
    let zero = C::new(0.0, 0.0);
    let one = C::new(1.0, 0.0);
    // τ: 8 → 4 traces the last qubit; E: 4 → 8 prepares it in |0⟩.
    let mut k0 = vec![zero; 4 * 8];
    let mut k1 = vec![zero; 4 * 8];
    let mut e = vec![zero; 8 * 4];
    for i in 0..4 {
        k0[i * 8 + 2 * i] = one;
        k1[i * 8 + 2 * i + 1] = one;
        e[(2 * i) * 4 + i] = one;
    }
    (
        QcMorphism::from_kraus(&[
            CausalTensor::from_slice(&k0, &[4, 8]),
            CausalTensor::from_slice(&k1, &[4, 8]),
        ])
        .unwrap(),
        QcMorphism::from_kraus(&[CausalTensor::from_slice(&e, &[8, 4])]).unwrap(),
    )
}

#[test]
fn test_extension_follows_a_renaming_as_a_whole() {
    let a = TypeAlignment::new(vec![(vec![0], vec![0, 1], trace_b(), prepare_b())]).unwrap();
    let ext = a.extended(&[(0, 5)], &[(0, 7), (1, 8)]).unwrap();
    assert_eq!(ext.entries().len(), 2);
    assert_eq!(ext.entries()[1].high(), &[5]);
    assert_eq!(ext.entries()[1].low(), &[7, 8]);
    let partial = a.extended(&[(0, 5)], &[(0, 7)]).unwrap_err();
    assert!(
        matches!(partial.0, QuantumErrorEnum::CalculationError(ref m) if m.contains("only in part"))
    );
    let untouched = a.extended(&[(3, 4)], &[]).unwrap();
    assert_eq!(untouched.entries().len(), 1);
}

/// Sided entries: the input side aligns wire 0 by the identity and the output side aligns the same
/// high-level wire with the two low-level wires through the trace. Each side sees only its entries;
/// the same wire on the same side twice is refused.
#[test]
fn test_sided_entries_apply_to_their_side_only() {
    let caps = NumericCaps::default();
    let a = TypeAlignment::new_sided(vec![
        (AlignmentSide::Input, (vec![0], vec![0], id(), id())),
        (
            AlignmentSide::Output,
            (vec![0], vec![0, 1], trace_b(), prepare_b()),
        ),
    ])
    .unwrap();
    assert_eq!(a.entries()[0].side(), AlignmentSide::Input);
    assert_eq!(a.entries()[1].side(), AlignmentSide::Output);
    assert_eq!(a.low_for_side(&[0], AlignmentSide::Input).unwrap(), vec![0]);
    assert_eq!(
        a.low_for_side(&[0], AlignmentSide::Output).unwrap(),
        vec![0, 1]
    );
    assert_eq!(
        a.tau_for_side(&[0], AlignmentSide::Input, &caps)
            .unwrap()
            .d_in(),
        2
    );
    assert_eq!(
        a.tau_for_side(&[0], AlignmentSide::Output, &caps)
            .unwrap()
            .d_in(),
        4
    );
    assert_eq!(
        a.section_for_side(&[0], AlignmentSide::Output, &caps)
            .unwrap()
            .d_out(),
        4
    );

    let same_side = TypeAlignment::new_sided(vec![
        (AlignmentSide::Input, (vec![0], vec![0], id(), id())),
        (AlignmentSide::Input, (vec![0], vec![1], id(), id())),
    ])
    .unwrap_err();
    assert!(
        matches!(same_side.0, QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("on one side"))
    );
    let any_overlaps_input = TypeAlignment::new_sided(vec![
        (AlignmentSide::Any, (vec![0], vec![0], id(), id())),
        (AlignmentSide::Input, (vec![1], vec![0], id(), id())),
    ])
    .unwrap_err();
    assert!(
        matches!(any_overlaps_input.0, QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("low-level wire 0"))
    );
    // An unsided alignment answers every side.
    let plain = TypeAlignment::new(vec![(vec![0], vec![0, 1], trace_b(), prepare_b())]).unwrap();
    assert_eq!(plain.entries()[0].side(), AlignmentSide::Any);
    assert_eq!(
        plain.low_for_side(&[0], AlignmentSide::Input).unwrap(),
        vec![0, 1]
    );
    assert_eq!(
        plain.low_for_side(&[0], AlignmentSide::Output).unwrap(),
        vec![0, 1]
    );
}
