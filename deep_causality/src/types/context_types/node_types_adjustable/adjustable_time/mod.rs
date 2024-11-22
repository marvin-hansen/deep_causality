// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use std::hash::Hash;
use std::ops::*;

use deep_causality_macros::{Constructor, Getters};

use crate::prelude::TimeScale;

mod adjustable;
mod display;
mod identifiable;
mod temporable;

/// AdjustableTime struct representing adjustable temporal contextoid payload.
///
/// # Type Parameters
///
/// - `T` - Type for adjustable time values
///
/// # Fields
///
/// - `id` - Unique ID for this adjustable time contextoid
/// - `time_scale` - The adjustable time scale
/// - `time_unit` - The adjustable time value
///
/// # Trait Implementations
///
/// - `Debug`, `Copy`, `Clone`, `Hash`, `Eq`, `PartialEq` - Derive macros
/// - `Add`, `Sub`, `Mul` - For math on adjustable time values
///
#[derive(Getters, Constructor, Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct AdjustableTime<T>
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
    #[getter(name = time_id)] // Rename ID getter to prevent conflict impl with identifiable
    id: u64,
    time_scale: TimeScale,
    time_unit: T,
}
