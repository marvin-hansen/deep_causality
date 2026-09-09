/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Type alignments, Lorenz & Tull, arXiv:2602.16612, Definition 14: for each high-level type `X`
//! a list `π(X)` of low-level types and an epic channel `τ_X : π(X) → X`. In QC the witness of
//! surjectivity is a section `E_X : X → π(X)` with `τ_X ∘ E_X = id_X`, which is what Proposition 18
//! needs to derive an upward abstraction from a downward one. Types are lists of wires; products of
//! types are tensor products of the entries (Eq. 14).

use crate::QuantumError;
use crate::types::circuit_model::{NumericCaps, QcMorphism, WireId};
use crate::types::decision::Tolerance;
use alloc::collections::BTreeSet;
use alloc::format;
use alloc::vec::Vec;
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;

/// Which side of a query's type an entry aligns. A query has an input type and an output type
/// (Lorenz & Tull, footnote 19), and a code's low-level model in the shape of Example 58, encoder
/// then gate, aligns its logical inputs by the identity and its physical outputs by the decoder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AlignmentSide {
    /// Applies on both sides.
    Any,
    /// Applies to input types only.
    Input,
    /// Applies to output types only.
    Output,
}

impl AlignmentSide {
    fn covers(self, requested: AlignmentSide) -> bool {
        self == AlignmentSide::Any || requested == AlignmentSide::Any || self == requested
    }
}

/// One aligned type: high-level wires, the low-level wires they abstract, the channel and its
/// section.
#[derive(Debug, Clone, PartialEq)]
pub struct AlignmentEntry<R: RealField> {
    side: AlignmentSide,
    high: Vec<WireId>,
    low: Vec<WireId>,
    tau: QcMorphism<R>,
    section: QcMorphism<R>,
}

impl<R: RealField> AlignmentEntry<R> {
    /// The side the entry applies to.
    pub fn side(&self) -> AlignmentSide {
        self.side
    }

    /// The high-level wires, ascending.
    pub fn high(&self) -> &[WireId] {
        &self.high
    }

    /// The low-level wires, ascending.
    pub fn low(&self) -> &[WireId] {
        &self.low
    }

    /// `τ_X : π(X) → X`.
    pub fn tau(&self) -> &QcMorphism<R> {
        &self.tau
    }

    /// `E_X : X → π(X)`.
    pub fn section(&self) -> &QcMorphism<R> {
        &self.section
    }
}

/// One alignment entry as given to [`TypeAlignment::new`]: `(high wires, low wires, τ, E)`.
pub type AlignmentSpec<R> = (Vec<WireId>, Vec<WireId>, QcMorphism<R>, QcMorphism<R>);

/// The type alignment of an abstraction. See the module documentation.
#[derive(Debug, Clone, PartialEq)]
pub struct TypeAlignment<R: RealField> {
    entries: Vec<AlignmentEntry<R>>,
}

impl<R> TypeAlignment<R>
where
    R: RealField + FromPrimitive + Default + core::fmt::Debug,
{
    /// An alignment from its entries: `(high wires, low wires, τ, E)`. Each `τ` is a [`Channel`],
    /// so it was validated CPTP once at its own construction; here `τ ∘ E = id` is checked against
    /// `Tolerance::state()` and the wire lists are checked disjoint across entries.
    ///
    /// # Errors
    ///
    /// [`QuantumError::SectionNotInverse`] carrying the residual and the tolerance when
    /// `τ_X ∘ E_X` differs from the identity; [`QuantumError::DimensionMismatch`] when a channel's
    /// dimensions disagree with its section's, or a wire appears in two entries.
    pub fn new(entries: Vec<AlignmentSpec<R>>) -> Result<Self, QuantumError> {
        Self::new_sided(
            entries
                .into_iter()
                .map(|e| (AlignmentSide::Any, e))
                .collect(),
        )
    }

    /// An alignment whose entries name the side they apply to. Wires must be disjoint among the
    /// entries that can apply to one side.
    ///
    /// # Errors
    ///
    /// As [`new`](Self::new).
    pub fn new_sided(
        entries: Vec<(AlignmentSide, AlignmentSpec<R>)>,
    ) -> Result<Self, QuantumError> {
        let caps = NumericCaps::default();
        let mut placed: Vec<(AlignmentSide, Vec<WireId>, Vec<WireId>)> = Vec::new();
        let mut out = Vec::with_capacity(entries.len());
        for (i, (side, (mut high, mut low, tau, section))) in entries.into_iter().enumerate() {
            high.sort_unstable();
            low.sort_unstable();
            for (other_side, other_high, other_low) in &placed {
                if !side.covers(*other_side) {
                    continue;
                }
                if let Some(w) = high.iter().find(|w| other_high.contains(w)) {
                    return Err(QuantumError::DimensionMismatch(format!(
                        "high-level wire {w} appears in two alignment entries on one side"
                    )));
                }
                if let Some(w) = low.iter().find(|w| other_low.contains(w)) {
                    return Err(QuantumError::DimensionMismatch(format!(
                        "low-level wire {w} appears in two alignment entries on one side"
                    )));
                }
            }
            placed.push((side, high.clone(), low.clone()));
            if tau.d_in() != section.d_out() || tau.d_out() != section.d_in() {
                return Err(QuantumError::DimensionMismatch(format!(
                    "entry {i}: τ is {} → {} but its section is {} → {}",
                    tau.d_in(),
                    tau.d_out(),
                    section.d_in(),
                    section.d_out()
                )));
            }
            let composite = section.then(&tau, &caps)?;
            let identity = QcMorphism::<R>::identity(tau.d_out())?;
            let (residual, _) = composite.frobenius_distance(&identity, &caps)?;
            let tolerance = Tolerance::<R>::state()
                .threshold(tau.d_out() * tau.d_out(), R::one())
                .unwrap_or_else(|| R::epsilon().sqrt());
            if residual > tolerance {
                return Err(QuantumError::SectionNotInverse(format!(
                    "entry {i}: ‖τ ∘ E − id‖_F = {residual:?} exceeds the state tolerance {tolerance:?}"
                )));
            }
            out.push(AlignmentEntry {
                side,
                high,
                low,
                tau,
                section,
            });
        }
        Ok(Self { entries: out })
    }

    /// The entries.
    pub fn entries(&self) -> &[AlignmentEntry<R>] {
        &self.entries
    }

    /// The alignment extended along a query's renamings: for every entry whose high-level wires
    /// were all renamed, an entry on the fresh names with the same `τ` and `E`, provided the aligned
    /// low-level wires were all renamed too. This is `π` applied to the opened type.
    ///
    /// # Errors
    ///
    /// [`QuantumError::CalculationError`] if an entry is renamed on one side or in part only.
    pub fn extended(
        &self,
        high_map: &[(WireId, WireId)],
        low_map: &[(WireId, WireId)],
    ) -> Result<Self, QuantumError> {
        let rename = |map: &[(WireId, WireId)], w: WireId| {
            map.iter().find(|(o, _)| *o == w).map(|(_, n)| *n)
        };
        let mut entries = self.entries.clone();
        for e in &self.entries {
            // A fresh input replaces the outputs of an opened mechanism, so it carries the output
            // type: input-side entries are never copied, and a copied output-side entry serves
            // both sides of the opened query.
            if e.side == AlignmentSide::Input {
                continue;
            }
            let high_hits = e
                .high
                .iter()
                .filter(|&&w| rename(high_map, w).is_some())
                .count();
            let low_hits = e
                .low
                .iter()
                .filter(|&&w| rename(low_map, w).is_some())
                .count();
            if high_hits == 0 && low_hits == 0 {
                continue;
            }
            if high_hits != e.high.len() || low_hits != e.low.len() {
                return Err(QuantumError::CalculationError(format!(
                    "the query renames the aligned type {:?} ↔ {:?} only in part ({high_hits} of {} high, {low_hits} of {} low wires)",
                    e.high,
                    e.low,
                    e.high.len(),
                    e.low.len()
                )));
            }
            let mut high: Vec<WireId> = e
                .high
                .iter()
                .map(|&w| rename(high_map, w).expect("hit"))
                .collect();
            let mut low: Vec<WireId> = e
                .low
                .iter()
                .map(|&w| rename(low_map, w).expect("hit"))
                .collect();
            high.sort_unstable();
            low.sort_unstable();
            entries.push(AlignmentEntry {
                side: AlignmentSide::Any,
                high,
                low,
                tau: e.tau.clone(),
                section: e.section.clone(),
            });
        }
        Ok(Self { entries })
    }

    /// `τ` on a list of high-level quantum wires, as a morphism from the aligned low-level wires
    /// (ascending) to the high-level wires (ascending). The list must be a union of whole entries.
    ///
    /// # Errors
    ///
    /// [`QuantumError::CalculationError`] if a wire is not aligned or an entry is only partly
    /// requested; the caps' errors.
    pub fn tau_for(
        &self,
        high: &[WireId],
        caps: &NumericCaps,
    ) -> Result<QcMorphism<R>, QuantumError> {
        self.assemble(high, AlignmentSide::Any, caps, false)
    }

    /// [`tau_for`](Self::tau_for) restricted to the entries that apply on one side.
    ///
    /// # Errors
    ///
    /// As [`tau_for`](Self::tau_for).
    pub fn tau_for_side(
        &self,
        high: &[WireId],
        side: AlignmentSide,
        caps: &NumericCaps,
    ) -> Result<QcMorphism<R>, QuantumError> {
        self.assemble(high, side, caps, false)
    }

    /// `E` on a list of high-level quantum wires, from the high wires (ascending) to the aligned
    /// low-level wires (ascending).
    ///
    /// # Errors
    ///
    /// As [`tau_for`](Self::tau_for).
    pub fn section_for(
        &self,
        high: &[WireId],
        caps: &NumericCaps,
    ) -> Result<QcMorphism<R>, QuantumError> {
        self.assemble(high, AlignmentSide::Any, caps, true)
    }

    /// [`section_for`](Self::section_for) restricted to the entries that apply on one side.
    ///
    /// # Errors
    ///
    /// As [`tau_for`](Self::tau_for).
    pub fn section_for_side(
        &self,
        high: &[WireId],
        side: AlignmentSide,
        caps: &NumericCaps,
    ) -> Result<QcMorphism<R>, QuantumError> {
        self.assemble(high, side, caps, true)
    }

    /// The low-level wires aligned with a list of high-level wires, ascending.
    ///
    /// # Errors
    ///
    /// [`QuantumError::CalculationError`] if a wire is not aligned.
    pub fn low_for(&self, high: &[WireId]) -> Result<Vec<WireId>, QuantumError> {
        self.low_for_side(high, AlignmentSide::Any)
    }

    /// [`low_for`](Self::low_for) restricted to the entries of one side.
    ///
    /// # Errors
    ///
    /// As [`low_for`](Self::low_for).
    pub fn low_for_side(
        &self,
        high: &[WireId],
        side: AlignmentSide,
    ) -> Result<Vec<WireId>, QuantumError> {
        let mut low = Vec::new();
        for e in self.covering(high, side)? {
            low.extend(&e.low);
        }
        low.sort_unstable();
        Ok(low)
    }

    fn covering(
        &self,
        high: &[WireId],
        side: AlignmentSide,
    ) -> Result<Vec<&AlignmentEntry<R>>, QuantumError> {
        let requested: BTreeSet<WireId> = high.iter().copied().collect();
        let mut covered: BTreeSet<WireId> = BTreeSet::new();
        let mut entries = Vec::new();
        for e in self.entries.iter().filter(|e| e.side.covers(side)) {
            let hit = e.high.iter().filter(|w| requested.contains(w)).count();
            if hit == 0 {
                continue;
            }
            if hit != e.high.len() {
                return Err(QuantumError::CalculationError(format!(
                    "the type {high:?} takes part of the aligned type {:?}; a type is aligned as a whole",
                    e.high
                )));
            }
            covered.extend(&e.high);
            entries.push(e);
        }
        if let Some(w) = requested.iter().find(|w| !covered.contains(w)) {
            return Err(QuantumError::CalculationError(format!(
                "high-level wire {w} is not aligned with any low-level type"
            )));
        }
        Ok(entries)
    }

    /// Tensor the covering entries in high-wire order, then permute both sides to ascending wire
    /// order.
    fn assemble(
        &self,
        high: &[WireId],
        side: AlignmentSide,
        caps: &NumericCaps,
        section: bool,
    ) -> Result<QcMorphism<R>, QuantumError> {
        let entries = self.covering(high, side)?;
        if entries.is_empty() {
            return QcMorphism::<R>::identity(1);
        }
        let mut acc: Option<QcMorphism<R>> = None;
        for e in &entries {
            let m = if section {
                e.section.clone()
            } else {
                e.tau.clone()
            };
            acc = Some(match acc {
                None => m,
                Some(a) => a.tensor(&m, caps)?,
            });
        }
        let morphism = acc.expect("at least one entry");
        // Legs as tensored: entry by entry. Each entry's channel is one leg of its full dimension.
        let high_dims: Vec<usize> = entries.iter().map(|e| e.tau.d_out()).collect();
        let low_dims: Vec<usize> = entries.iter().map(|e| e.tau.d_in()).collect();
        let high_order = ascending_order(entries.iter().map(|e| e.high[0]).collect());
        let low_order = ascending_order(entries.iter().map(|e| e.low[0]).collect());
        if section {
            morphism.permute_legs(&high_dims, &high_order, &low_dims, &low_order)
        } else {
            morphism.permute_legs(&low_dims, &low_order, &high_dims, &high_order)
        }
    }
}

/// The order that sorts `keys` ascending: `order[k]` is the index of the `k`-th smallest key.
fn ascending_order(keys: Vec<WireId>) -> Vec<usize> {
    let mut idx: Vec<usize> = (0..keys.len()).collect();
    idx.sort_by_key(|&i| keys[i]);
    idx
}
