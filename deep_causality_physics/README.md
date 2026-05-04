# DeepCausality Physics

**A library of physics formulas and engineering primitives for DeepCausality.**

`deep_causality_physics` provides a comprehensive collection of physics kernels, causal wrappers, and physical
quantities designed for use within the DeepCausality hyper-graph simulation engine. It leverages Geometric Algebra (via
`deep_causality_multivector`) and Causal Tensors to model complex physical interactions with high fidelity.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
deep_causality_physics = { version = "0.5" }

# For QCD hadronization (Lund string fragmentation) - use os-random feature
# deep_causality_physics = { version = "0.3.0", features = ["os-random"] }
```

## Features

This crate is organized into modular domains, each providing low-level computation kernels and high-level causal
wrappers:

* **Astro**: Astrophysics kernels (Schwarzschild radius, orbital velocity, luminosity, Hubble's law, etc.).
* **Chronometric**: Chronometric geodesy kernels — invert the J2-corrected weak-field 1PN clock
  equation (Bjerhammar 1975, Vermeer 1983) to recover gravitational parameters such as $GM_\oplus$
  and the derived planetary mass $M_\oplus = GM_\oplus / G$ from satellite clock time-dilation
  measurements. Includes `CentralBody` (gravity-model parameters), `SpaceTimeCoordinate` (clock +
  kinematic state), and the `solve_gm_analytical` kernel and wrapper.
* **Condensed**: Condensed matter physics (Quantum Geometry, Twistronics, Phase Field Models, e.g., Quantum Geometric
  Tensor, Bistritzer-MacDonald Hamiltonian, Ginzburg-Landau equations).
* **Dynamics**: Classical mechanics (Kinematics, Newton's laws), state estimation (Kalman filters), and Euler
  integration.
* **Electromagnetism**: Maxwell's equations, Lorentz force, Poynting vectors, and gauge fields using Geometric Algebra.
* **Fluids**: Fluid dynamics (Bernoulli's principle, Reynolds number, viscosity, pressure).
* **Materials**: Material science properties (Stress, Strain, Hooke's Law, Young's modulus, thermal expansion).
* **MHD**: Magnetohydrodynamics (Alfven waves, Magnetic Pressure, Ideal Induction on Manifolds, General Relativistic MHD
  with Manifold-based current computation, Plasma parameters).
* **Nuclear**: Nuclear physics (Binding energy, radioactive decay, PDG particle database, **Lund String Fragmentation**
  for QCD hadronization).
* **Photonics**: Ray Optics, Polarization Calculus, Gaussian Beam Optics, and Diffraction.
* **Quantum**: Quantum mechanics primitives (Wavefunctions, operators, gates, expectation values, Haruna's Gauge Field
  gates).
* **Relativity**: Special and General Relativity (Spacetime intervals, time dilation, Einstein tensor, geodesic
  deviation).
* **Thermodynamics**: Statistical mechanics (Entropy, Carnot efficiency, Ideal Gas Law, heat diffusion).
* **Units**: Type-safe physical units (Time, Mass, Length, ElectricCurrent, Temperature, etc.) to prevent dimensional
  errors.
* **Waves**: Wave mechanics (Doppler effect, wave speed, frequency/wavelength relations).

## Kernels vs. Theories

This library is designed with a clear distinction between **computational kernels** and **theoretical frameworks**:

1. **Kernels for Isolation**: If you need to solve a specific equation (e.g., `schwarzschild_radius`, `lorentz_force`,
   `cahn_hilliard_flux`) in isolation, use the standalone **kernels**. These are pure functions, stateless, and
   efficient.
2. **Theories for Frameworks**: If you are working within a full physical theory (e.g., General Relativity, Electroweak
   Theory, etc), use the **Physics Theories** modules (`src/theories`). These leverage **Geometric Algebra (GA)** and *
   *Gauge Fields** to model the entire coherent system, ensuring mathematical consistency and high precision across all operations.

>  **Read More**: [**Understanding Gauge Theories & Geometric Algebra**](./README_GAUGE_THEORIES.md) — A detailed guide
> on how Gravity, Electromagnetism, and Weak Force have been implmented using a shared topological backend.

## Architecture

The library follows a functional and causal architecture:

1. **Kernels (`*::mechanics`, `*::gravity`, etc.)**: Pure functions that perform the raw physical computations. They
   operate on `CausalTensor`, `CausalMultiVector`, `Manifold`, or primitive types. They are stateless and side-effect
   free.
    * *Example*: `klein_gordon_kernel` computes $(\\Delta + m^2)\\psi$.

2. **Theories (`theories/*`)**: High-level implementations of complete physical theories (GR, EM, Electroweak) that
   integrate with `deep_causality_topology`. They use **Gauge Fields** and **Geometric Algebra** to unify disparate
   physical phenomena under a single mathematical roof.
    * *Example*: `GR` struct implements `GrOps` by calling topology-level `CurvatureTensor` methods.

3. **Wrappers (`*::wrappers`)**: Monadic wrappers that lift kernels into the `PropagatingEffect` monad. These allow
   physics functions to be directly embedded into `CausalEffect` functions within a DeepCausality graph.
    * *Example*: `apply_gate` wraps `apply_gate_kernel` to validly propagate state changes in the causal graph.

4. **Quantities (`*::quantities`, `units::*`)**: Newtype wrappers (e.g., `Speed`, `Mass`, `Temperature`, `FourMomentum`,
   `Hadron`) that enforce physical invariants (e.g., mass cannot be negative) and type safety.

5. **Metric Types**: Re-exports from `deep_causality_metric` for sign convention handling (`LorentzianMetric`,
   `EastCoastMetric`, `WestCoastMetric`, `PhysicsMetric`).

### Example: Relativistic Dynamics

```rust
use deep_causality_physics::{
    time_dilation_angle, spacetime_interval,
    Speed, Time, Length
};
use deep_causality_multivector::{CausalMultiVector, Metric};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Define Spacetime Events using Geometric Algebra
    // Minkowski Metric (+---) for 4D spacetime
    let metric = Metric::Minkowski(4);

    // Event A at origin
    let event_a = CausalMultiVector::new(vec![0.0; 16], metric).unwrap();

    // Event B at (t=10, x=5, y=0, z=0)
    // Multivector data layout depends on dimension, assume standard layout
    let mut data_b = vec![0.0; 16];
    data_b[1] = 10.0; // Time basis
    data_b[2] = 5.0;  // X basis
    let event_b = CausalMultiVector::new(data_b, metric).unwrap();

    // 2. Compute Spacetime Interval (Wrapper returns PropagatingEffect)
    let interval_effect = spacetime_interval(&event_b, &metric);

    if let Ok(s2) = interval_effect {
        println!("Spacetime Interval s^2: {}", s2);
    }

    Ok(())
}
```

### Example: Chronometric GM Recovery

```rust
use deep_causality_physics::{
    CentralBody, SpaceTimeCoordinate, solve_gm_analytical,
};

fn main() {
    // Two space-time coordinate samples (position in ECI, inertial velocity,
    // IGS clock bias and drift rate). In practice these come from preprocessed
    // satellite broadcast clock + SP3 orbit data.
    let coord_a: SpaceTimeCoordinate<f64> = /* ... */;
    let coord_b: SpaceTimeCoordinate<f64> = /* ... */;

    // Earth parameters: JGM-3 J2 with WGS-84 equatorial radius. The struct
    // pairs J2 with its reference radius — they are only meaningful together.
    let body = CentralBody::EARTH_JGM3;

    // Wrapper returns PropagatingEffect<f64> for use in causaloid graphs.
    let effect = solve_gm_analytical(&coord_a, &coord_b, &body);
    // → recovered GM in m³/s²; divide by NEWTONIAN_CONSTANT_OF_GRAVITATION
    //   to get Earth's mass in kg.
}
```

A complete worked example processing one full GPS week of Galileo broadcast clock data
(satellite E14) is in [`examples/chronometric_examples/gm_recovery`](../examples/chronometric_examples/gm_recovery).
It demonstrates the framework's `CausalMonad` bind chain end-to-end and recovers
$GM_\oplus$ and Earth's mass to ~0.2% relative error against published JGM-3 / IERS 2010
references — *the planet weighed by clock time-dilation alone.*

### Example: Lund String Fragmentation (QCD Hadronization)

```rust
use deep_causality_physics::{
    FourMomentum, LundParameters, lund_string_fragmentation_kernel
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure Lund parameters (defaults tuned to LEP e+e- data)
    let params = LundParameters::default();

    // Define a quark-antiquark string (e.g., from e+e- → qq̄)
    // Quark moving in +z, antiquark in -z
    let quark = FourMomentum::new(50.0, 0.0, 0.0, 50.0);
    let antiquark = FourMomentum::new(50.0, 0.0, 0.0, -50.0);
    let endpoints = vec![(quark, antiquark)];

    // Fragment the string into hadrons
    let mut rng = rand::thread_rng();
    let hadrons = lund_string_fragmentation_kernel(&endpoints, &params, &mut rng)?;

    println!("Produced {} hadrons:", hadrons.len());
    for h in &hadrons {
        println!("  PDG ID: {}, Energy: {:.2} GeV", h.pdg_id(), h.energy());
    }

    Ok(())
}
```

Code examples are in the [repo example folder](../examples/physics_examples).

## Configuration

The crate supports `no_std` environments via feature flags.

* `default`: Enables `std`.
* `std`: Usage of standard library (includes `alloc`).
* `alloc`: Usage of allocation (Vec, String) without full `std`.
* `os-random`: Enables OS-based secure random number generator and Lund string fragmentation for QCD hadronization.

## License

Licensed under MIT. Copyright (c) 2025 DeepCausality Authors.
