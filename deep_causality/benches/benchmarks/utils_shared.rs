// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use deep_causality::errors::CausalityError;
use deep_causality::prelude::{Causaloid, Dataoid, IdentificationValue, NumericalValue, Spaceoid, SpaceTempoid, Tempoid};

// Generates a fixed sized array with sample data

pub fn get_test_causaloid<'l>()
    -> Causaloid<'l, Dataoid, Spaceoid, Tempoid, SpaceTempoid>
{
    let id: IdentificationValue = 1;
    let description = "tests whether data exceeds threshold of 0.55";
    fn causal_fn(obs: NumericalValue) -> Result<bool, CausalityError> {
        if obs.is_sign_negative() {
            return Err(CausalityError("Observation is negative".into()));
        }

        let threshold: NumericalValue = 0.55;
        if !obs.ge(&threshold) {
            Ok(false)
        } else {
            Ok(true)
        }
    }

    Causaloid::new(id, causal_fn, description)
}

pub fn generate_sample_data<const N: usize>()
    -> [f64; N]
{
    [0.99; N]
}
