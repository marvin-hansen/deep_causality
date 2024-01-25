// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use std::hash::Hash;
use std::ops::*;

use deep_causality_macros::{Constructor, Getters};

use crate::prelude::TimeScale;

mod adjustable;
mod display;
mod identifiable;
mod space_temporal;
mod spatial;
mod temporable;

/// AdjustableSpaceTime struct representing adjustable spatio-temporal contextoid payload.
///
/// # Type Parameters
///
/// - `T` - Type for adjustable spatial and temporal coordinate values
///
/// # Fields
///
/// - `id` - Unique ID for this adjustable space-time contextoid
/// - `time_scale` - The adjustable time scale
/// - `time_unit` - The adjustable time value
/// - `x` - Adjustable X spatial coordinate
/// - `y` - Adjustable Y spatial coordinate
/// - `z` - Adjustable Z spatial coordinate
///
/// # Trait Implementations
///
/// - `Debug`, `Copy`, `Clone`, `Hash`, `Eq`, `PartialEq` - Derive macros
/// - `Add`, `Sub`, `Mul` - For coordinate math
///
#[derive(Getters, Constructor, Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct AdjustableSpaceTime<T>
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
    x: T,
    y: T,
    z: T,
}
