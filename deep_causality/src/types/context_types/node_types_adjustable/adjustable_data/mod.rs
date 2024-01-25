use std::hash::Hash;

use deep_causality_macros::{Constructor, Getters};

// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use crate::prelude::Datable;

mod adjustable;
mod display;
mod identifiable;

/// AdjustableData struct representing adjustable data contextoid payload.
///
/// # Type Parameters
///
/// - `T` - Type of the adjustable data payload
///
/// # Fields
///
/// - `id` - Unique ID for this adjustable data contextoid
/// - `data` - The adjustable data payload
///
/// # Trait Implementations
///
/// - `Datable` - Required for use in context
/// - `Debug`, `Copy`, `Clone`, `Hash`, `Eq`, `PartialEq` - Derive macros
///
#[derive(Getters, Constructor, Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct AdjustableData<T>
where
    T: Default + Copy + Clone + Hash + Eq + PartialEq,
{
    #[getter(name = data_id)] // Rename ID getter to prevent conflict impl with identifiable
    id: u64,
    data: T,
}

// Type tag required for context.
impl<T> Datable for AdjustableData<T> where T: Default + Copy + Clone + Hash + Eq + PartialEq {}
