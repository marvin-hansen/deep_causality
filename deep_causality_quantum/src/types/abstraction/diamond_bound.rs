/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;

/// The two-sided bound a Frobenius residual on Choi operators places on the diamond distance.
///
/// For a linear map `Φ` with the crate's unnormalised Choi operator `J(Φ)`:
///
/// 1. `‖Φ‖_⋄ ≤ ‖J(Φ)‖_1`: every unit vector on `X ⊗ X` is `(I ⊗ A)|Ω̃⟩` with `‖A‖_F = 1`, so
///    `(Φ ⊗ id)(|ψ⟩⟨ψ|) = (I ⊗ A) J(Φ) (I ⊗ A)†` has trace norm at most `‖A‖_∞² ‖J‖_1 ≤ ‖J‖_1`.
/// 2. `‖J‖_1 ≤ √(rank J) · ‖J‖_F ≤ √(d_in d_out) · ‖J‖_F`, Cauchy–Schwarz on the singular values.
/// 3. `J(Φ)/d_in = (Φ ⊗ id)(ω)` for the maximally entangled state `ω`, so `‖J‖_F ≤ ‖J‖_1 ≤ d_in ‖Φ‖_⋄`.
///
/// Hence `r / d_in ≤ ‖E − F‖_⋄ ≤ √(d_in d_out) · r` for a Frobenius residual `r` between two
/// channels, and a residual of zero certifies a diamond distance of zero. On `id` against `R_z(θ)`
/// on one qubit, whose diamond distance is `2 sin(θ/2)`, the residual is `2√2 sin(θ/2)`, so the
/// upper bound exceeds the distance by the constant `2√2` and the lower bound sits at `√2 sin(θ/2)`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DiamondBound<R> {
    /// `r / d_in`.
    pub lower: R,
    /// `√(d_in d_out) · r`.
    pub upper: R,
    /// The factor `√(d_in d_out)` the upper bound used.
    pub amplification: R,
}

impl<R> DiamondBound<R>
where
    R: RealField + FromPrimitive,
{
    /// The bound for a Frobenius residual `r` between channels from `d_in` to `d_out`.
    pub fn from_frobenius(r: R, d_in: usize, d_out: usize) -> Self {
        let din = R::from_usize(d_in.max(1)).unwrap_or_else(R::one);
        let dout = R::from_usize(d_out.max(1)).unwrap_or_else(R::one);
        let amplification = (din * dout).sqrt();
        Self {
            lower: r / din,
            upper: amplification * r,
            amplification,
        }
    }
}
