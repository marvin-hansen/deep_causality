/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Construction checks and the induced DAG of `CircuitModel`.
//!
//! The DAG scenario is Lorenz & Tull, arXiv:2602.16612, Example 61 read on the wiring of the
//! `qcl-circuit-model` spec: encoder on `{0, 1}`, `U` on `{0, 1}`, `V` on `{1, 2}`, measurements on
//! `0` and `2`. Corner cases: (A) no boxes, (B) one box, (C) a box grouped twice, (D) an empty node,
//! (E) a wire out of range, (F) a cyclic grouping.

use deep_causality_num_complex::Complex;
use deep_causality_quantum::{
    Channel, CircuitBox, CircuitModel, GateOp, QuantumErrorEnum, QubitOperator, WireType,
};
use deep_causality_tensor::CausalTensor;

type C = Complex<f64>;

fn ket(amps: &[f64]) -> CausalTensor<C> {
    CausalTensor::from_slice(
        &amps.iter().map(|&a| C::new(a, 0.0)).collect::<Vec<_>>(),
        &[amps.len()],
    )
}

fn unitary(wires: &[usize], program: Vec<GateOp>) -> CircuitBox<f64> {
    CircuitBox::Unitary {
        wires: wires.to_vec(),
        program,
    }
}

/// The spec's wiring: wires 0..3 qubits, 3 a classical input, 4 and 5 classical outcomes.
fn example_61() -> CircuitModel<f64> {
    let wires = vec![
        WireType::qubit(),
        WireType::qubit(),
        WireType::qubit(),
        WireType::Classical { outcomes: 4 },
        WireType::bit(),
        WireType::bit(),
    ];
    let boxes = vec![
        CircuitBox::Encoder {
            input: 3,
            outputs: vec![0, 1],
            states: vec![
                ket(&[1.0, 0.0, 0.0, 0.0]),
                ket(&[0.0, 1.0, 0.0, 0.0]),
                ket(&[0.0, 0.0, 1.0, 0.0]),
                ket(&[0.0, 0.0, 0.0, 1.0]),
            ],
        },
        unitary(
            &[0, 1],
            vec![GateOp::Cnot {
                control: 0,
                target: 1,
            }],
        ),
        unitary(
            &[1, 2],
            vec![GateOp::Cz {
                control: 0,
                target: 1,
            }],
        ),
        CircuitBox::Measurement {
            wires: vec![0],
            outcome: 4,
        },
        CircuitBox::Measurement {
            wires: vec![2],
            outcome: 5,
        },
    ];
    CircuitModel::ungrouped(wires, boxes, vec![], vec![4, 5]).unwrap()
}

#[test]
fn test_induced_dag_follows_the_wires() {
    let m = example_61();
    let dag = m.induced_dag();
    // Nodes: 0 encoder, 1 U, 2 V, 3 M(0), 4 M(2).
    assert!(dag.has_edge(0, 1));
    assert!(dag.has_edge(1, 2), "wire 1 leaves U and enters V");
    assert!(dag.has_edge(1, 3));
    assert!(dag.has_edge(2, 4));
    assert!(!dag.has_edge(1, 4), "U never touches wire 2");
    assert!(!dag.has_edge(0, 2), "wire 1 reaches V through U");
    assert_eq!(dag.num_edges(), 4);
    assert_eq!(m.classical_inputs(), vec![3]);
    assert_eq!(m.num_qubits(), 3);
    assert_eq!(m.nodes_on_wire(1), vec![0, 1, 2]);
    assert_eq!(m.boxes_of_nodes(&[2, 0]), vec![0, 2]);
    assert_eq!(m.node_of(3), Some(3));
}

#[test]
fn test_no_boxes_and_one_box() {
    let empty =
        CircuitModel::<f64>::ungrouped(vec![WireType::qubit()], vec![], vec![0], vec![0]).unwrap();
    assert_eq!(empty.induced_dag().num_vertices(), 0);
    let one = CircuitModel::<f64>::ungrouped(
        vec![WireType::qubit()],
        vec![unitary(&[0], vec![GateOp::H(0)])],
        vec![0],
        vec![0],
    )
    .unwrap();
    assert_eq!(one.induced_dag().num_vertices(), 1);
    assert_eq!(one.induced_dag().num_edges(), 0);
}

#[test]
fn test_mis_dimensioned_channel_names_the_wire_and_the_box() {
    let err = CircuitModel::<f64>::ungrouped(
        vec![WireType::qubit(), WireType::qubit()],
        vec![CircuitBox::Channel {
            wires: vec![0, 1],
            channel: Channel::unitary(&QubitOperator::hadamard()).unwrap(),
        }],
        vec![0, 1],
        vec![0, 1],
    )
    .unwrap_err();
    match err.0 {
        QuantumErrorEnum::DimensionMismatch(msg) => {
            assert!(msg.contains("box 0") && msg.contains("[0, 1]"), "{msg}")
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn test_two_writers_of_one_classical_wire_are_refused() {
    let err = CircuitModel::<f64>::ungrouped(
        vec![WireType::qubit(), WireType::qubit(), WireType::bit()],
        vec![
            CircuitBox::Measurement {
                wires: vec![0],
                outcome: 2,
            },
            CircuitBox::Measurement {
                wires: vec![1],
                outcome: 2,
            },
        ],
        vec![],
        vec![2],
    )
    .unwrap_err();
    match err.0 {
        QuantumErrorEnum::DimensionMismatch(msg) => {
            assert!(
                msg.contains("wire 2") && msg.contains("box 0") && msg.contains("box 1"),
                "{msg}"
            )
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn test_grouping_must_partition_the_boxes() {
    let wires = vec![WireType::qubit()];
    let boxes = || {
        vec![
            unitary(&[0], vec![GateOp::H(0)]),
            unitary(&[0], vec![GateOp::S(0)]),
        ]
    };
    let twice = CircuitModel::<f64>::new(
        wires.clone(),
        boxes(),
        vec![vec![0, 1], vec![1]],
        vec![0],
        vec![0],
    )
    .unwrap_err();
    assert!(matches!(twice.0, QuantumErrorEnum::CalculationError(ref m) if m.contains("box 1")));
    let missing = CircuitModel::<f64>::new(wires.clone(), boxes(), vec![vec![0]], vec![0], vec![0])
        .unwrap_err();
    assert!(matches!(missing.0, QuantumErrorEnum::CalculationError(ref m) if m.contains("box 1")));
    let empty_node = CircuitModel::<f64>::new(
        wires.clone(),
        boxes(),
        vec![vec![0, 1], vec![]],
        vec![0],
        vec![0],
    )
    .unwrap_err();
    assert!(
        matches!(empty_node.0, QuantumErrorEnum::CalculationError(ref m) if m.contains("node 1"))
    );
    let out_of_range =
        CircuitModel::<f64>::new(wires, boxes(), vec![vec![0], vec![7]], vec![0], vec![0])
            .unwrap_err();
    assert!(matches!(
        out_of_range.0,
        QuantumErrorEnum::CalculationError(_)
    ));
}

#[test]
fn test_cyclic_grouping_shows_in_the_induced_dag() {
    // Boxes 0 and 2 in one node, box 1 in another, all on one wire: A → B → A.
    let m = CircuitModel::<f64>::new(
        vec![WireType::qubit()],
        vec![
            unitary(&[0], vec![GateOp::H(0)]),
            unitary(&[0], vec![GateOp::S(0)]),
            unitary(&[0], vec![GateOp::H(0)]),
        ],
        vec![vec![0, 2], vec![1]],
        vec![0],
        vec![0],
    )
    .unwrap();
    assert!(m.induced_dag().has_cycle());
}

#[test]
fn test_wire_kind_and_range_errors() {
    let bad_kind = CircuitModel::<f64>::ungrouped(
        vec![WireType::bit()],
        vec![unitary(&[0], vec![])],
        vec![],
        vec![],
    )
    .unwrap_err();
    assert!(
        matches!(bad_kind.0, QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("classical"))
    );
    let out_of_range = CircuitModel::<f64>::ungrouped(
        vec![WireType::qubit()],
        vec![unitary(&[1], vec![])],
        vec![],
        vec![],
    )
    .unwrap_err();
    assert!(
        matches!(out_of_range.0, QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("wire 1"))
    );
    let repeated = CircuitModel::<f64>::ungrouped(
        vec![WireType::qubit()],
        vec![unitary(&[0, 0], vec![])],
        vec![],
        vec![],
    )
    .unwrap_err();
    assert!(
        matches!(repeated.0, QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("more than once"))
    );
    let local_qubit = CircuitModel::<f64>::ungrouped(
        vec![WireType::qubit()],
        vec![unitary(&[0], vec![GateOp::H(1)])],
        vec![],
        vec![],
    )
    .unwrap_err();
    assert!(
        matches!(local_qubit.0, QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("local qubit 1"))
    );
    let qutrit_program = CircuitModel::<f64>::ungrouped(
        vec![WireType::Quantum { dim: 3 }],
        vec![unitary(&[0], vec![])],
        vec![],
        vec![],
    )
    .unwrap_err();
    assert!(
        matches!(qutrit_program.0, QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("qubits"))
    );
    let zero_wire =
        CircuitModel::<f64>::ungrouped(vec![WireType::Quantum { dim: 0 }], vec![], vec![], vec![])
            .unwrap_err();
    assert!(matches!(
        zero_wire.0,
        QuantumErrorEnum::DimensionMismatch(_)
    ));
}

#[test]
fn test_encoder_measurement_and_instrument_checks() {
    let states_ok = vec![ket(&[1.0, 0.0]), ket(&[0.0, 1.0])];
    let wrong_count = CircuitModel::<f64>::ungrouped(
        vec![WireType::qubit(), WireType::Classical { outcomes: 3 }],
        vec![CircuitBox::Encoder {
            input: 1,
            outputs: vec![0],
            states: states_ok.clone(),
        }],
        vec![],
        vec![0],
    )
    .unwrap_err();
    assert!(
        matches!(wrong_count.0, QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("3 values"))
    );
    let wrong_dim = CircuitModel::<f64>::ungrouped(
        vec![WireType::qubit(), WireType::bit()],
        vec![CircuitBox::Encoder {
            input: 1,
            outputs: vec![0],
            states: vec![ket(&[1.0, 0.0, 0.0]), ket(&[0.0, 1.0])],
        }],
        vec![],
        vec![0],
    )
    .unwrap_err();
    assert!(
        matches!(wrong_dim.0, QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("state 0"))
    );
    let encoded_input = CircuitModel::<f64>::ungrouped(
        vec![WireType::qubit(), WireType::bit()],
        vec![CircuitBox::Encoder {
            input: 1,
            outputs: vec![0],
            states: states_ok.clone(),
        }],
        vec![0],
        vec![0],
    )
    .unwrap_err();
    assert!(
        matches!(encoded_input.0, QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("encoder"))
    );
    let meas_count = CircuitModel::<f64>::ungrouped(
        vec![WireType::qubit(), WireType::Classical { outcomes: 3 }],
        vec![CircuitBox::Measurement {
            wires: vec![0],
            outcome: 1,
        }],
        vec![],
        vec![1],
    )
    .unwrap_err();
    assert!(
        matches!(meas_count.0, QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("measures dimension 2"))
    );
    // An instrument whose families do not sum to the identity.
    let p0 = CausalTensor::from_slice(
        &[
            C::new(1.0, 0.0),
            C::new(0.0, 0.0),
            C::new(0.0, 0.0),
            C::new(0.0, 0.0),
        ],
        &[2, 2],
    );
    let not_tp = CircuitModel::<f64>::ungrouped(
        vec![WireType::qubit(), WireType::bit()],
        vec![CircuitBox::Instrument {
            wires: vec![0],
            outcome: 1,
            kraus: vec![vec![p0.clone()], vec![]],
        }],
        vec![0],
        vec![0, 1],
    )
    .unwrap_err();
    assert!(
        matches!(not_tp.0, QuantumErrorEnum::CalculationError(ref m) if m.contains("trace-preserving"))
    );
    let p1 = CausalTensor::from_slice(
        &[
            C::new(0.0, 0.0),
            C::new(0.0, 0.0),
            C::new(0.0, 0.0),
            C::new(1.0, 0.0),
        ],
        &[2, 2],
    );
    let ok = CircuitModel::<f64>::ungrouped(
        vec![WireType::qubit(), WireType::bit()],
        vec![CircuitBox::Instrument {
            wires: vec![0],
            outcome: 1,
            kraus: vec![vec![p0], vec![p1]],
        }],
        vec![0],
        vec![0, 1],
    );
    assert!(ok.is_ok());
    let unwritten_output = CircuitModel::<f64>::ungrouped(
        vec![WireType::qubit(), WireType::bit()],
        vec![],
        vec![0],
        vec![1],
    )
    .unwrap_err();
    assert!(
        matches!(unwritten_output.0, QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("written by no box"))
    );
}
