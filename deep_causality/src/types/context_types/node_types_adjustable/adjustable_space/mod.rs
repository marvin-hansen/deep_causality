// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use std::hash::Hash;
use std::ops::*;

use deep_causality_macros::{Constructor, Getters};

mod adjustable;
mod display;
mod identifiable;
mod spatial;

/// AdjustableSpace struct representing adjustable spatial contextoid payload.
///
/// # Type Parameters
///
/// - `T` - Type for adjustable spatial coordinate values
///
/// # Fields
///
/// - `id` - Unique ID for this adjustable space contextoid
/// - `x` - Adjustable X coordinate
/// - `y` - Adjustable Y coordinate
/// - `z` - Adjustable Z coordinate
///
/// # Trait Implementations
///
/// - `Debug`, `Copy`, `Clone`, `Hash`, `Eq`, `PartialEq` - Derive macros
/// - `Add`, `Sub`, `Mul` - For coordinate math
///
#[derive(Getters, Constructor, Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct AdjustableSpace<T>
where
    T: Default
        + Copy
        + Clone
        + Hash
        + Eq
        + PartialEq
        + Add<T, Output = T>
        + Sub<T, Output = T>
        + Mul<T, Output = T>,
{
    #[getter(name = space_id)] // Rename ID getter to prevent conflict impl with identifiable
    id: u64,
    x: T,
    y: T,
    z: T,
}
