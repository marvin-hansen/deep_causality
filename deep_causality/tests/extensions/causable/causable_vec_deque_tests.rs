/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::collections::VecDeque;

use deep_causality::utils_test::test_utils::*;
use deep_causality::*;

// Helper function to create a standard test VecDeque.
fn get_deterministic_test_causality_vec_deque() -> VecDeque<BaseCausaloid> {
    VecDeque::from_iter(get_deterministic_test_causality_vec())
}

fn get_probabilistic_test_causality_vec_deque() -> VecDeque<BaseCausaloid> {
    VecDeque::from_iter(get_probabilistic_test_causality_vec())
}

// Helper to activate all causes in a collection for testing purposes.
fn activate_all_causes(col: &VecDeque<BaseCausaloid>) {
    // A value that ensures the default test causaloid (threshold 0.55) becomes active.
    let effect = PropagatingEffect::Numerical(0.99);
    for cause in col {
        // We call evaluate to set the internal state, but ignore the result for this setup.
        let _ = cause.evaluate(&effect);
    }
}

#[test]
fn test_add() {
    let mut col = get_deterministic_test_causality_vec_deque();
    assert_eq!(3, col.len());

    let q = get_test_causaloid_deterministic();
    col.push_back(q);
    assert_eq!(4, col.len());
}

#[test]
fn test_get_all_items() {
    let col = get_deterministic_test_causality_vec_deque();
    let all_items = col.get_all_items();

    let exp_len = col.len();
    let actual_len = all_items.len();
    assert_eq!(exp_len, actual_len);
}

#[test]
fn test_evaluate_deterministic_propagation() {
    let col = get_deterministic_test_causality_vec_deque();

    // Case 1: All succeed, chain should be deterministically true.
    let effect_success = PropagatingEffect::Numerical(0.99);
    let res = col.evaluate_deterministic(&effect_success, &AggregateLogic::All);
    assert!(res.is_ok());
    let res_success = res.unwrap();
    assert_eq!(res_success, PropagatingEffect::Deterministic(true));

    // Case 2: One fails, chain should be deterministically false.
    let effect_fail = PropagatingEffect::Numerical(0.1);
    let res = col.evaluate_deterministic(&effect_fail, &AggregateLogic::All);
    assert!(res.is_ok());
    let res_fail = res.unwrap();
    assert_eq!(res_fail, PropagatingEffect::Deterministic(false));
}

#[test]
fn test_evaluate_probabilistic_propagation() {
    let col = get_probabilistic_test_causality_vec_deque();

    // Case 1: All succeed (Deterministic(true) is treated as probability 1.0).
    // The cumulative probability should be 1.0.
    let effect_success = PropagatingEffect::Numerical(0.99);
    let res = col.evaluate_probabilistic(&effect_success, &AggregateLogic::All, 0.5);
    assert!(res.is_ok());
    let res_success = res.unwrap();
    assert_eq!(res_success, PropagatingEffect::Probabilistic(1.0));

    // Case 2: One fails (Deterministic(false) is treated as probability 0.0).
    // The chain should short-circuit and return a cumulative probability of 0.0.
    let effect_fail = PropagatingEffect::Numerical(0.1);
    let res = col.evaluate_probabilistic(&effect_fail, &AggregateLogic::All, 0.5);
    assert!(res.is_ok());
    let res_fail = res.unwrap();
    assert_eq!(res_fail, PropagatingEffect::Probabilistic(0.0));
}

#[test]
fn test_evaluate_mixed_propagation() {
    let col = get_deterministic_test_causality_vec_deque();

    // Case 1: All succeed, chain remains deterministically true.
    let effect_success = PropagatingEffect::Numerical(0.99);
    let res = col.evaluate_mixed(&effect_success, &AggregateLogic::All, 0.5);
    assert!(res.is_ok());
    let res_success = res.unwrap();
    assert_eq!(res_success, PropagatingEffect::Deterministic(true));

    // Case 2: One fails, chain becomes deterministically false.
    let effect_fail = PropagatingEffect::Numerical(0.1);
    let res = col.evaluate_mixed(&effect_fail, &AggregateLogic::All, 0.5);
    assert!(res.is_ok());
    let res_fail = res.unwrap();
    assert_eq!(res_fail, PropagatingEffect::Deterministic(false));
}

#[test]
fn test_explain() {
    let col = get_deterministic_test_causality_vec_deque();
    activate_all_causes(&col);

    let single_explanation = "\n * Causaloid: 1 'tests whether data exceeds threshold of 0.55' evaluated to: PropagatingEffect::Deterministic(true)\n";
    let expected = single_explanation.repeat(3);
    let res = col.explain();
    assert!(res.is_ok());
    let actual = res.unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn test_len() {
    let col = get_deterministic_test_causality_vec_deque();
    assert_eq!(3, col.len());
}

#[test]
fn test_is_empty() {
    let col = get_deterministic_test_causality_vec_deque();
    assert!(!col.is_empty());
}

#[test]
fn test_to_vec() {
    let col = get_deterministic_test_causality_vec_deque();
    assert_eq!(3, col.to_vec().len());
}
