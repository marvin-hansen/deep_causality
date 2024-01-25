// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use deep_causality_macros::Constructor;

use crate::prelude::{IdentificationValue, NumericalValue};

mod display;
mod identifiable;
mod observable;

/// Observation struct containing an observed data point.
///
/// # Fields
///
/// - `id` - Unique ID of the observation
/// - `observation` - The actual observed data value
/// - `observed_effect` - The observed effect or outcome corresponding to the observation
///
#[derive(Constructor, Debug, Clone, PartialEq, PartialOrd)]
pub struct Observation {
    id: IdentificationValue,
    observation: NumericalValue,
    observed_effect: NumericalValue,
}
