// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use deep_causality_macros::Constructor;

use crate::prelude::{DescriptionValue, IdentificationValue, NumericalValue};

mod display;
mod identifiable;
mod inferable;

/// Inference struct representing an inference query.
///
/// # Fields
///
/// - `id` - Unique ID for the inference
/// - `question` - The question or goal of the inference
/// - `observation` - The given observation or evidence
/// - `threshold` - The decision threshold
/// - `effect` - The observed effect value
/// - `target` - The target variable to infer
///
#[derive(Constructor, Debug, Clone, PartialEq, PartialOrd)]
pub struct Inference {
    id: IdentificationValue,
    question: DescriptionValue,
    observation: NumericalValue,
    threshold: NumericalValue,
    effect: NumericalValue,
    target: NumericalValue,
}
