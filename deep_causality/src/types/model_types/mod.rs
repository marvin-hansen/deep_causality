// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
use std::hash::Hash;
use std::ops::*;

use deep_causality_macros::Getters;

use crate::prelude::{
    Assumption, Causaloid, Context, Datable, Identifiable, SpaceTemporal, Spatial, Temporable,
};

/// Model struct containing a causal model definition.
///
/// # Type Parameters
///
/// - `'l` - Lifetime of references
/// - `D` - Datable type for observations
/// - `S` - Spatial type
/// - `T` - Temporal type
/// - `ST` - SpaceTemporal type
/// - `V` - Value type
///
/// # Fields
///
/// - `id` - Unique model ID
/// - `author` - Author of the model
/// - `description` - Description of the model
/// - `assumptions` - Optional list of assumptions
/// - `causaloid` - Causaloid representing the model logic
/// - `context` - Optional context
///
#[derive(Getters, Clone, Debug)]
pub struct Model<'l, D, S, T, ST, V>
where
    D: Datable + Clone,
    S: Spatial<V> + Clone,
    T: Temporable<V> + Clone,
    ST: SpaceTemporal<V> + Clone,
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
    #[getter(name = model_id)] // Rename ID getter to prevent conflict impl with identifiable
    id: u64,
    author: &'l str,
    description: &'l str,
    assumptions: Option<Vec<&'l Assumption>>,
    causaloid: Causaloid<'l, D, S, T, ST, V>,
    context: Option<&'l Context<'l, D, S, T, ST, V>>,
}

impl<'l, D, S, T, ST, V> Model<'l, D, S, T, ST, V>
where
    D: Datable + Clone,
    S: Spatial<V> + Clone,
    T: Temporable<V> + Clone,
    ST: SpaceTemporal<V> + Clone,
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
    /// Creates a new Model instance.
    ///
    /// # Parameters
    ///
    /// - `id` - Unique model ID
    /// - `author` - Author of the model
    /// - `description` - Description of the model
    /// - `assumptions` - Optional list of assumptions
    /// - `causaloid` - Causaloid representing the model logic
    /// - `context` - Optional context
    ///
    /// # Returns
    ///
    /// A new Model instance
    ///
    pub fn new(
        id: u64,
        author: &'l str,
        description: &'l str,
        assumptions: Option<Vec<&'l Assumption>>,
        causaloid: Causaloid<'l, D, S, T, ST, V>,
        context: Option<&'l Context<'l, D, S, T, ST, V>>,
    ) -> Self {
        Self {
            id,
            author,
            description,
            assumptions,
            causaloid,
            context,
        }
    }
}

impl<'l, D, S, T, ST, V> Identifiable for Model<'l, D, S, T, ST, V>
where
    D: Datable + Clone,
    S: Spatial<V> + Clone,
    T: Temporable<V> + Clone,
    ST: SpaceTemporal<V> + Clone,
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
    /// Gets the unique ID of this Model.
    ///
    /// # Returns
    ///
    /// u64 - The unique ID of this Model
    ///
    fn id(&self) -> u64 {
        self.id
    }
}
