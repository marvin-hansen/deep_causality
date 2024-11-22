use std::hash::Hash;
// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use std::ops::*;

use deep_causality_macros::Constructor;

use crate::prelude::TimeScale;

mod display;
mod identifiable;
mod temporable;

/// Time struct representing temporal contextoid payload.
///
/// # Type Parameters
///
/// - `T` - Type for time values
///
/// # Fields
///
/// - `id` - Unique ID for this time contextoid
/// - `time_scale` - The time scale
/// - `time_unit` - The time value
///
/// # Trait Implementations
///
/// - `Debug`, `Copy`, `Clone`, `Hash`, `Eq`, `PartialEq` - Derive macros
/// - `Add`, `Sub`, `Mul` - For math on time values
///
#[derive(Constructor, Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Time<T>
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
    id: u64,
    time_scale: TimeScale,
    time_unit: T,
}
