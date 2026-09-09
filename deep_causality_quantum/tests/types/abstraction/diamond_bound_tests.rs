/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The two-sided bound on the closed-form pair: `id` against `R_z(θ)` has diamond distance
//! `2 sin(θ/2)` (the distance from the origin to the chord between `e^{±iθ/2}` is `cos(θ/2)`, and
//! `‖U − V‖_⋄ = 2√(1 − δ²)`), and Frobenius residual `2√2 sin(θ/2)` on the unnormalised Choi
//! operators. Both ends of the bound hold at every θ and the upper bound is loose by `2√2` exactly.

use deep_causality_quantum::{Axis, Channel, DiamondBound, NumericCaps, QcMorphism, QubitOperator};

#[test]
fn test_both_bounds_hold_on_the_rotation_pair() {
    let caps = NumericCaps::default();
    let id =
        QcMorphism::from_channel(&Channel::unitary(&QubitOperator::identity()).unwrap()).unwrap();
    for k in 1..=8 {
        let theta = std::f64::consts::PI * k as f64 / 8.0;
        let rz = QcMorphism::from_channel(
            &Channel::unitary(&QubitOperator::rotation(Axis::Z, theta).unwrap()).unwrap(),
        )
        .unwrap();
        let (r, _) = id.frobenius_distance(&rz, &caps).unwrap();
        let diamond = 2.0 * (theta / 2.0).sin();
        assert!(
            (r - 2.0 * 2f64.sqrt() * (theta / 2.0).sin()).abs() < 1e-12,
            "θ = {theta}: r = {r}"
        );
        let b = DiamondBound::from_frobenius(r, 2, 2);
        assert!(b.lower <= diamond + 1e-12, "lower {} vs {diamond}", b.lower);
        assert!(b.upper >= diamond - 1e-12, "upper {} vs {diamond}", b.upper);
        assert!(
            (b.upper / diamond - 2.0 * 2f64.sqrt()).abs() < 1e-9,
            "the upper bound is loose by 2√2"
        );
        assert!((b.amplification - 2.0).abs() < 1e-15);
    }
}

#[test]
fn test_zero_residual_certifies_zero_distance_and_dimensions_enter() {
    let z = DiamondBound::from_frobenius(0.0f64, 4, 2);
    assert_eq!(z.lower, 0.0);
    assert_eq!(z.upper, 0.0);
    assert!((z.amplification - 8f64.sqrt()).abs() < 1e-15);
    let b = DiamondBound::from_frobenius(1.0f64, 4, 2);
    assert!((b.lower - 0.25).abs() < 1e-15);
    assert!((b.upper - 8f64.sqrt()).abs() < 1e-15);
}
