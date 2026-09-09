/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The fourth subject: a circuit on the builder, screened through its dilation.

use deep_causality_quantum::{
    Axis, Channel, CircuitBox, CircuitModel, CommutatorTolerance, Factorization, QclBuilder,
    QuantumErrorEnum, QubitOperator, ScreenOrigin, WireType,
};

type FloatType = f64;
type NumberType = u64;

fn ry(theta: f64) -> CircuitBox<FloatType> {
    CircuitBox::Channel {
        wires: vec![0],
        channel: Channel::unitary(&QubitOperator::rotation(Axis::Y, theta).unwrap()).unwrap(),
    }
}

fn chain() -> CircuitModel<FloatType> {
    CircuitModel::ungrouped(
        vec![WireType::qubit()],
        vec![ry(0.7), ry(0.9)],
        vec![],
        vec![0],
    )
    .unwrap()
}

#[test]
fn test_a_circuit_builds_and_its_dilation_is_screened() {
    let cfg = QclBuilder::config::<FloatType, NumberType>()
        .over_circuit(chain())
        .build()
        .expect("an acyclic quantum circuit builds");
    let screened = QclBuilder::validate(&cfg)
        .check_markov(&CommutatorTolerance::new())
        .check_decomposable(&[0], &[1])
        .finalize()
        .expect("the dilation is Markov and C₃-free");
    assert_eq!(screened.origin(), ScreenOrigin::Circuit);
    assert!(screened.require_compositional().is_ok());
    let stages = screened.stages();
    assert_eq!(stages[0].0, "check_markov");
    assert_eq!(stages[0].1.examined(), 1);
    assert_eq!(stages[0].1.factorization(), Factorization::Rederived);
    assert_eq!(stages[1].0, "check_decomposable");
    assert!(screened.report().unwrap().accepted());
    assert_eq!(cfg.subject().model().boxes().len(), 2);
}

#[test]
fn test_a_cyclic_grouping_is_refused_at_build() {
    let cyclic = CircuitModel::new(
        vec![WireType::qubit()],
        vec![ry(0.1), ry(0.2), ry(0.3)],
        vec![vec![0, 2], vec![1]],
        vec![],
        vec![0],
    )
    .unwrap();
    let err = QclBuilder::config::<FloatType, NumberType>()
        .over_circuit(cyclic)
        .build()
        .err()
        .expect("a cyclic grouping is refused");
    assert!(matches!(
        err.0,
        QuantumErrorEnum::CyclicStructureUnsupported(_)
    ));
    let empty =
        CircuitModel::<FloatType>::ungrouped(vec![WireType::qubit()], vec![], vec![0], vec![0])
            .unwrap();
    let err = QclBuilder::config::<FloatType, NumberType>()
        .over_circuit(empty)
        .build()
        .err()
        .expect("refused");
    assert!(matches!(err.0, QuantumErrorEnum::CalculationError(_)));
    let classical = CircuitModel::<FloatType>::ungrouped(
        vec![WireType::qubit(), WireType::bit()],
        vec![CircuitBox::Measurement {
            wires: vec![0],
            outcome: 1,
        }],
        vec![0],
        vec![1],
    )
    .unwrap();
    let err = QclBuilder::config::<FloatType, NumberType>()
        .over_circuit(classical)
        .build()
        .err()
        .expect("refused");
    assert!(matches!(err.0, QuantumErrorEnum::CalculationError(_)));
}

#[test]
fn test_a_model_subject_cannot_enter_an_abstraction() {
    use deep_causality::utils_test::test_utils;
    use deep_causality::{BaseCausaloid, CausableGraph, CausaloidGraph};
    use deep_causality_num_complex::Complex;
    use deep_causality_quantum::{FactorSupports, ProcessFactors};
    use deep_causality_tensor::CausalTensor;
    let mut g: CausaloidGraph<BaseCausaloid<f64, bool>> = CausaloidGraph::new(0);
    let n0 = g
        .add_causaloid(test_utils::get_test_causaloid_deterministic(0))
        .unwrap();
    let n1 = g
        .add_causaloid(test_utils::get_test_causaloid_deterministic(1))
        .unwrap();
    g.add_edge(n0, n1).unwrap();
    g.freeze();
    let diag = |a: f64, b: f64| {
        CausalTensor::from_slice(
            &[
                Complex::new(a, 0.0),
                Complex::new(0.0, 0.0),
                Complex::new(0.0, 0.0),
                Complex::new(b, 0.0),
            ],
            &[2, 2],
        )
    };
    let mut factors = ProcessFactors::new();
    factors.insert(0, diag(0.9, 0.1));
    let mut supports = FactorSupports::new();
    supports.declare(0, &[0]);
    let cfg = QclBuilder::config::<FloatType, NumberType>()
        .over_model(g, factors, supports)
        .build()
        .unwrap();
    let screened = QclBuilder::validate(&cfg)
        .check_markov(&CommutatorTolerance::new())
        .finalize()
        .unwrap();
    assert_eq!(screened.origin(), ScreenOrigin::Marginal);
    let err = screened.require_compositional().unwrap_err();
    assert!(
        matches!(err.0, QuantumErrorEnum::NoCompositionalModel(ref m) if m.contains("Marginal"))
    );
}
