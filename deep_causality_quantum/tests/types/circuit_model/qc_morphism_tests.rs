/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The Kraus-level morphism carrier and its two caps.
//!
//! Provenance of the literals: the Choi of the identity on one qubit is `Σ_{ik} |ii⟩⟨kk|`, the
//! unnormalised maximally entangled projector, with Frobenius norm `2` and trace `2`; `Z` has the
//! same Choi with signs `(−1)^{i+k}`, so `‖J(id) − J(Z)‖_F = ‖2(|01⟩⟨01|-like block)‖` works out to
//! `√8 = 2√2` (four entries of modulus 2 off the `i = k` diagonal). Corner cases: (A) an empty
//! family, (B) a single scalar block, (D) mismatched types, (F) the entry cap at exactly the limit
//! and one above, (G) a classical value at its outcome count.

use deep_causality_num_complex::Complex;
use deep_causality_quantum::{
    Channel, NumericCaps, QcMorphism, QuantumErrorEnum, QubitOperator, choi_from_kraus,
};
use deep_causality_tensor::CausalTensor;

type C = Complex<f64>;

fn z() -> CausalTensor<C> {
    CausalTensor::from_slice(
        &[
            C::new(1.0, 0.0),
            C::new(0.0, 0.0),
            C::new(0.0, 0.0),
            C::new(-1.0, 0.0),
        ],
        &[2, 2],
    )
}

#[test]
fn test_identity_equals_itself_and_differs_from_z_by_two_root_two() {
    let caps = NumericCaps::default();
    let id = QcMorphism::<f64>::identity(2).unwrap();
    let zc = QcMorphism::from_kraus(&[z()]).unwrap();
    let (same, entries) = id.frobenius_distance(&id, &caps).unwrap();
    assert!(same.abs() < 1e-15);
    assert_eq!(entries, 32, "two Choi operators of 16 entries each");
    let (diff, _) = id.frobenius_distance(&zc, &caps).unwrap();
    assert!((diff - 8f64.sqrt()).abs() < 1e-12, "{diff}");
    assert_eq!(id.d_in(), 2);
    assert_eq!(id.d_out(), 2);
    assert_eq!(id.input_qubits(), 1);
    assert_eq!(id.output_qubits(), 1);
    assert_eq!(id.operator_count(), 1);
    assert_eq!(id.choi_entries(), 16);
}

#[test]
fn test_from_channel_agrees_with_choi_from_kraus() {
    let ch = Channel::unitary(&QubitOperator::hadamard()).unwrap();
    let m = QcMorphism::from_channel(&ch).unwrap();
    let (blocks, _) = m.choi_blocks(&NumericCaps::default()).unwrap();
    let j: &CausalTensor<C> = blocks.get(&(vec![], vec![])).unwrap();
    let expect: CausalTensor<C> = choi_from_kraus(ch.kraus().unwrap()).unwrap();
    for (a, b) in j.as_slice().iter().zip(expect.as_slice()) {
        assert!((a.re - b.re).abs() < 1e-15 && (a.im - b.im).abs() < 1e-15);
    }
    // A composed channel holds only its Choi; the Kraus family is recovered.
    let composed = ch.compose(&ch).unwrap();
    let m2 = QcMorphism::from_channel(&composed).unwrap();
    let id = QcMorphism::<f64>::identity(2).unwrap();
    let (d, _) = m2.frobenius_distance(&id, &NumericCaps::default()).unwrap();
    assert!(d < 1e-12, "H·H = I, {d}");
}

#[test]
fn test_then_composes_kraus_products_and_classical_wiring() {
    let caps = NumericCaps::default();
    let zc = QcMorphism::from_kraus(&[z()]).unwrap();
    let zz = zc.then(&zc, &caps).unwrap();
    let id = QcMorphism::<f64>::identity(2).unwrap();
    assert!(zz.frobenius_distance(&id, &caps).unwrap().0 < 1e-15);
    // Classical: a bit-flip stochastic matrix as scalar blocks, composed with itself.
    let one = || CausalTensor::from_slice(&[C::new(1.0, 0.0)], &[1, 1]);
    let mut flip = QcMorphism::<f64>::new(1, 1, vec![2], vec![2]).unwrap();
    flip.push(vec![0], vec![1], vec![one()]).unwrap();
    flip.push(vec![1], vec![0], vec![one()]).unwrap();
    let twice = flip.then(&flip, &caps).unwrap();
    assert!(twice.blocks().contains_key(&(vec![0], vec![0])));
    assert!(twice.blocks().contains_key(&(vec![1], vec![1])));
    assert!(!twice.blocks().contains_key(&(vec![0], vec![1])));
    assert_eq!(twice.classical_in(), &[2]);
    assert_eq!(twice.classical_out(), &[2]);
    // Mismatched types cannot compose or compare.
    let err = flip.then(&zc, &caps).unwrap_err();
    assert!(matches!(err.0, QuantumErrorEnum::DimensionMismatch(_)));
    let err = flip.frobenius_distance(&zc, &caps).unwrap_err();
    assert!(matches!(err.0, QuantumErrorEnum::DimensionMismatch(_)));
    let tight = NumericCaps {
        max_entries: 1 << 24,
        max_operators: 0,
    };
    let err = zc.then(&zc, &tight).unwrap_err();
    assert!(matches!(
        err.0,
        QuantumErrorEnum::KrausFamilyExceeded {
            operators: 1,
            cap: 0
        }
    ));
}

#[test]
fn test_entry_cap_is_exact_at_the_boundary() {
    let id = QcMorphism::<f64>::identity(2).unwrap();
    let at = NumericCaps {
        max_entries: 16,
        max_operators: 1 << 12,
    };
    assert!(id.choi_blocks(&at).is_ok());
    let below = NumericCaps {
        max_entries: 15,
        max_operators: 1 << 12,
    };
    let err = id.choi_blocks(&below).unwrap_err();
    assert!(matches!(
        err.0,
        QuantumErrorEnum::NaturalityDimensionExceeded {
            n: 1,
            k: 1,
            entries: 16,
            cap: 15
        }
    ));
}

#[test]
fn test_push_validates_values_and_shapes() {
    let mut m = QcMorphism::<f64>::new(2, 2, vec![2], vec![]).unwrap();
    let bad_len = m.push(vec![], vec![], vec![z()]).unwrap_err();
    assert!(matches!(bad_len.0, QuantumErrorEnum::DimensionMismatch(_)));
    let at_count = m.push(vec![2], vec![], vec![z()]).unwrap_err();
    assert!(matches!(at_count.0, QuantumErrorEnum::DimensionMismatch(_)));
    let bad_shape = m
        .push(
            vec![0],
            vec![],
            vec![CausalTensor::from_slice(&[C::new(1.0, 0.0)], &[1, 1])],
        )
        .unwrap_err();
    assert!(matches!(
        bad_shape.0,
        QuantumErrorEnum::DimensionMismatch(_)
    ));
    assert!(m.push(vec![1], vec![], vec![z()]).is_ok());
    assert!(QcMorphism::<f64>::from_kraus(&[]).is_err());
    assert!(QcMorphism::<f64>::new(0, 1, vec![], vec![]).is_err());
    assert!(QcMorphism::<f64>::new(1, 1, vec![0], vec![]).is_err());
}

#[test]
fn test_entry_count_multiplies_by_the_block_count() {
    // Two scalar blocks: one entry each, two in all; a cap of one refuses them.
    let one = || CausalTensor::from_slice(&[C::new(1.0, 0.0)], &[1, 1]);
    let mut flip = QcMorphism::<f64>::new(1, 1, vec![2], vec![2]).unwrap();
    flip.push(vec![0], vec![1], vec![one()]).unwrap();
    flip.push(vec![1], vec![0], vec![one()]).unwrap();
    assert_eq!(flip.choi_entries(), 2);
    let caps = NumericCaps {
        max_entries: 1,
        max_operators: 1 << 12,
    };
    let err = flip.choi_blocks(&caps).unwrap_err();
    assert!(matches!(
        err.0,
        QuantumErrorEnum::NaturalityDimensionExceeded {
            n: 0,
            k: 0,
            entries: 2,
            cap: 1
        }
    ));
    let (_, entries) = flip.choi_blocks(&NumericCaps::default()).unwrap();
    assert_eq!(entries, 2);
}

#[test]
fn test_default_caps_are_the_design_values() {
    let caps = NumericCaps::default();
    assert_eq!(caps.max_entries, 1 << 24);
    assert_eq!(caps.max_operators, 1 << 12);
}
