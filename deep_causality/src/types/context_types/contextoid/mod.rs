// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::*;

use crate::prelude::{ContextoidType, Datable, SpaceTemporal, Spatial, Temporable};

pub mod contextoid_type;
mod contextuable;
mod display;
mod identifiable;

/// Contextoid struct representing a node in a context graph.
///
/// # Type Parameters
///
/// - `D` - Datable trait object for node data
/// - `S` - Spatial trait object for spatial properties
/// - `T` - Temporal trait object for temporal properties
/// - `ST` - SpaceTemporal trait object for spatio-temporal properties
/// - `V` - Type for vertex values
///
/// # Fields
///
/// - `id` - Unique identifier for the contextoid
/// - `vertex_type` - The type of this contextoid
///
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Contextoid<D, S, T, ST, V>
where
    D: Datable,
    S: Spatial<V>,
    T: Temporable<V>,
    ST: SpaceTemporal<V>,
    V: Default
        + Copy
        + Clone
        + Hash
        + Eq
        + PartialEq
        + Add<V, Output = V>
        + Sub<V, Output = V>
        + Mul<V, Output = V>,
{
    id: u64,
    vertex_type: ContextoidType<D, S, T, ST, V>,
    ty: PhantomData<V>,
}

impl<D, S, T, ST, V> Contextoid<D, S, T, ST, V>
where
    D: Datable,
    S: Spatial<V>,
    T: Temporable<V>,
    ST: SpaceTemporal<V>,
    V: Default
        + Copy
        + Clone
        + Hash
        + Eq
        + PartialEq
        + Add<V, Output = V>
        + Sub<V, Output = V>
        + Mul<V, Output = V>,
{
    /// Creates a new Contextoid instance.
    ///
    /// # Parameters
    ///
    /// - `id` - The unique ID for this contextoid
    /// - `vertex_type` - The type of this contextoid
    ///
    /// # Returns
    ///
    /// A new Contextoid instance
    ///
    pub fn new(id: u64, vertex_type: ContextoidType<D, S, T, ST, V>) -> Self {
        Self {
            id,
            vertex_type,
            ty: PhantomData,
        }
    }
}
