// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use std::fmt::{Debug, Display};

/// RelationKind enum representing the different kinds of relations between contextoids.
///
/// # Variants
///
/// - `Datial` - Relation between data contextoids
/// - `Temporal` - Relation between time contextoids
/// - `Spatial` - Relation between space contextoids
/// - `SpaceTemporal` - Relation between space-time contextoids
///
/// # Trait Implementations
///
/// - `Copy`, `Clone`, `Debug`, `Eq`, `PartialEq`, `Hash` - Derive macros
/// - `repr(u8)` - Fixed size representation for compactness
///
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[repr(u8)]
pub enum RelationKind {
    Datial,
    Temporal,
    Spatial,
    SpaceTemporal,
}

impl Display for RelationKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
