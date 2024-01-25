// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use std::hash::Hash;
use std::ops::{Add, Mul, Sub};

use deep_causality_macros::Constructor;

mod display;
mod identifiable;
mod spatial;

/// Space struct representing spatial contextoid payload.
///
/// # Type Parameters
///
/// - `T` - Type for spatial coordinate values
///
/// # Fields
///
/// - `id` - Unique ID for this space contextoid
/// - `x` - X coordinate
/// - `y` - Y coordinate
/// - `z` - Z coordinate
///
/// # Trait Implementations
///
/// - `Debug`, `Copy`, `Clone`, `Hash`, `Eq`, `PartialEq` - Derive macros
/// - `Add`, `Sub`, `Mul` - For coordinate math
///
#[derive(Constructor, Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Space<T>
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
    x: T,
    y: T,
    z: T,
}
