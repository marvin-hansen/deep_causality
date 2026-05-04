# DeepCausality Examples Overview

This directory contains examples demonstrating various features and applications of the DeepCausality library. Each example showcases how to model and reason about causal relationships using the Effect Propagation Process (EPP) and PropagatingEffect monads.

## Example Categories

| Category | Description |
|----------|-------------|
| [Starter Example](#starter-example) | Basic introduction to DeepCausality |
| [Classical Causality](#classical-causality-examples) | Traditional causal inference methods (CATE, DBN, Granger, RCM, SCM) |
| [CSM Examples](#csm-examples) | Causal State Machine patterns |
| [Core Examples](#core-examples) | PropagatingEffect and PropagatingProcess fundamentals |
| [Avionics Examples](#avionics-examples) | High-assurance GNC and Safety Critical Systems |
| [Physics Examples](#physics-examples) | Multi-physics simulations with Geometric Algebra |
| [Medicine Examples](#medicine-examples) | Biomedical and life sciences applications |
| [Material Examples](#material-examples) | Material Science and Metamaterials |
| [Tokio Example](#tokio-example) | Async integration with tokio runtime |

---

## Starter Example

**Location:** `examples/starter_example`

Basic introduction to DeepCausality. **Start here if you are new.**

| Example | Focus | Command |
|---------|-------|---------|
| Starter | CausaloidGraph basics | `cargo run -p starter_example --example starter_example` |

---  

## Classical Causality Examples

**Location:** `examples/classical_causality_examples`

Traditional causal inference methods implemented using the DeepCausality framework.

| Example | Method | Command |
|---------|--------|---------|
| CATE | Conditional Average Treatment Effect | `cargo run -p classical_causality_examples --example cate_example` |
| DBN | Dynamic Bayesian Network | `cargo run -p classical_causality_examples --example dbn_example` |
| Granger | Granger Causality Test | `cargo run -p classical_causality_examples --example granger_example` |
| RCM | Rubin Causal Model | `cargo run -p classical_causality_examples --example rcm_example` |
| SCM | Pearl's Ladder of Causation | `cargo run -p classical_causality_examples --example scm_example` |

See [classical_causality_examples/README.md](classical_causality_examples/README.md) for detailed documentation.

---

## CSM Examples

**Location:** `examples/csm_examples`

Causal State Machine patterns for stateful causal reasoning.

| Example | Pattern | Command |
|---------|---------|---------|
| CSM Basic | Simple monitoring system | `cargo run -p csm_examples --example csm_example` |
| CSM Context | Shared mutable state via `Arc<RwLock>` | `cargo run -p csm_examples --example csm_context_example` |
| CSM Effect Ethos | Deontic reasoning integration | `cargo run -p csm_examples --example csm_effect_ethos_example` |

See [csm_examples/README.md](csm_examples/README.md) for detailed documentation.

---

## Core Examples

**Location:** `examples/core_examples`

Fundamental examples demonstrating the monadic API.

| Example | Focus | Command |
|---------|-------|---------|
| PropagatingEffect | Basic monadic composition | `cargo run -p core_examples --example propagating_effect_example` |
| PropagatingEffect Counterfactual | Counterfactual reasoning | `cargo run -p core_examples --example propagating_effect_counterfactual_example` |
| PropagatingProcess | Stateful composition | `cargo run -p core_examples --example propagating_process_example` |
| PropagatingProcess Counterfactual | Stateful counterfactuals | `cargo run -p core_examples --example propagating_process_counterfactual` |
| Control Flow Builder | Builder patterns | `cargo run -p core_examples --example control_flow_builder` |
| Control Flow Strict ZST | Zero-sized type control | `cargo run -p core_examples --example control_flow_strict_zst` |

---

## Avionics Examples

**Location:** `examples/avionics_examples`

High-assurance examples for Aerospace, Defense, and Safety Critical systems.

| Example | Domain | Description | Command |
|---------|--------|-------------|---------|
| [magnav](magnav/README.md) | Navigation | Magnetic Navigation using Causal Particle Filters (Bayesian estimation) | `cargo run -p avionics_examples --example magnav` |
| [geometric_tcas](geometric_tcas/README.md) | Collision Avoidance | NextGen TCAS using Geometric Algebra collision detection and `Intervenable` safety interlocks | `cargo run -p avionics_examples --example geometric_tcas` |
| [hypersonic_2t](hypersonic_2t/README.md) | Defense/Tracking | Tracking Hypersonic Glide Vehicles (HGV) using Dual-Time (2T) Physics in 6D phase space | `cargo run -p avionics_examples --example hypersonic_2t` |
| [flight_envelope_monitor](flight_envelope_monitor/README.md) | Health Monitoring | Three-stage stateful pipeline (sensor collection → bind chain → envelope hypergraph) demonstrating uniform composition through `PropagatingProcess<_, FlightState, AircraftConfig>` | `cargo run -p avionics_examples --example flight_envelope_monitor` |

See [avionics_examples/README.md](avionics_examples/README.md) for detailed documentation.

---

## Physics Examples

**Location:** `examples/physics_examples`

Multi-physics simulations using Geometric Algebra, Tensor operations, and Topology.

| Example | Domain | Command |
|---------|--------|---------|
| Bernoulli Flow Network | Fluid Dynamics | `cargo run -p physics_examples --example bernoulli_flow_network` |
| Carnot Cycle Engine | Thermodynamics | `cargo run -p physics_examples --example carnot_cycle_engine` |
| Laser Resonator Stability | Optics | `cargo run -p physics_examples --example laser_resonator_stability` |
| Maxwell's Unification | Electromagnetism | `cargo run -p physics_examples --example maxwell_example` |
| GRMHD | Relativity | `cargo run -p physics_examples --example grmhd_example` |
| Geometric Tilt | Robotics/IMU | `cargo run -p physics_examples --example geometric_tilt` |
| Algebraic Scanner | Abstract Algebra | `cargo run -p physics_examples --example algebraic_scanner` |
| Multi-Physics Pipeline | Particle Physics | `cargo run -p physics_examples --example multi_physics_pipeline` |
| Quantum Counterfactual | Quantum | `cargo run -p physics_examples --example quantum_counterfactual` |
| Quantum Geometric Tensor | Condensed Matter | `cargo run -p physics_examples --example quantum_geometric_tensor` |
| IKKT Matrix Model | Quantum Gravity | `cargo run -p physics_examples --example ikkt_matrix_model` |
| Gravitational Wave | Relativity | `cargo run -p physics_examples --example gravitational_wave` |
| Event Horizon Probe | Relativity | `cargo run -p physics_examples --example event_horizon_probe` |
| Gauge EM | Electromagnetism | `cargo run -p physics_examples --example gauge_em` |
| Gauge GR | General Relativity | `cargo run -p physics_examples --example gauge_gr` |
| Gauge Electroweak | Electroweak | `cargo run -p physics_examples --example gauge_electroweak` |
| Gauge Weak Force | Weak Force | `cargo run -p physics_examples --example gauge_weak_force` |
| Gauge Lattice U(1) 2D | Lattice Gauge | `cargo run -p physics_examples --example gauge_lattice_u1_2d` |

See [physics_examples/README.md](physics_examples/README.md) for detailed documentation.

---

## Medicine Examples

**Location:** `examples/medicine_examples`

Biomedical and life sciences applications using causal monads.

| Example | Domain | Command |
|---------|--------|---------|
| Protein Folding | Biophysics | `cargo run -p medicine_examples --example protein_folding` |
| MRI Tissue Classification | Medical Imaging | `cargo run -p medicine_examples --example tissue_classification` |
| Aneurysm Risk (Hemodynamics) | Cardiovascular | `cargo run -p medicine_examples --example aneurysm_risk` |
| Diving Decompression | Hyperbaric Medicine | `cargo run -p medicine_examples --example diving_decompression` |
| Epilepsy Virtual Resection | Neurology | `cargo run -p medicine_examples --example epilepsy` |
| Tumor Treatment (TTFields) | Oncology | `cargo run -p medicine_examples --example tumor_treatment` |

See [medicine_examples/README.md](medicine_examples/README.md) for detailed documentation.

---

## Material Examples

**Location:** `examples/material_examples`

Material Science and Metamaterial simulations using topology, multivectors, and causal interventions.

| Example | Domain | Command |
|---------|--------|---------|
| Hyperlens | Metamaterials | `cargo run -p material_examples --example hyperlens_example` |
| Topological Insulator | Quantum Materials | `cargo run -p material_examples --example topological_insulator_example` |
| Structural Health Monitor | Smart Materials | `cargo run -p material_examples --example structural_health_monitor_example` |

See [material_examples/README.md](material_examples/README.md) for detailed documentation.

---

## Tokio Example

**Location:** `examples/tokio_example`

Asynchronous integration with the tokio runtime.

| Example | Focus | Command |
|---------|-------|---------|
| Tokio | Async causal inference | `cargo run -p tokio_example --example tokio_example` |

---

## License

All examples are licensed under the [MIT license](LICENSE).