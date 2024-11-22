// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use crate::prelude::{Assumable, DescriptionValue, EvalFn, NumericalValue};
use crate::types::reasoning_types::assumption::Assumption;

impl Assumable for Assumption {
    /// Gets the text description of this assumption.
    ///
    /// Converts the internal DescriptionValue to a String and returns it.
    ///
    /// # Returns
    ///
    /// The text description as a String.
    ///
    fn description(&self) -> DescriptionValue {
        self.description.to_string() as DescriptionValue
    }

    /// Gets the evaluation function for this assumption.
    ///
    /// Returns the closure representing the evaluation logic for this assumption.
    ///
    /// # Returns
    ///
    /// The evaluation function as an EvalFn closure.
    ///
    fn assumption_fn(&self) -> EvalFn {
        self.assumption_fn
    }

    /// Gets whether this assumption has been tested.
    ///
    /// Reads the assumption_tested flag.
    ///
    /// # Returns
    ///
    /// true if this assumption has been tested, false otherwise.
    ///
    fn assumption_tested(&self) -> bool {
        *self.assumption_tested.read().unwrap()
    }

    /// Gets whether this assumption is valid.
    ///
    /// Reads the assumption_valid flag.
    ///
    /// # Returns
    ///
    /// true if this assumption is valid, false otherwise.
    ///
    fn assumption_valid(&self) -> bool {
        *self.assumption_valid.read().unwrap()
    }

    /// Verifies this assumption against the provided data.
    ///
    /// Evaluates the assumption function against the data.
    /// Sets the assumption_tested flag to true.
    /// If evaluation succeeds, sets the assumption_valid flag to true.
    ///
    /// # Arguments
    ///
    /// * `data` - Input data to evaluate assumption against
    ///
    /// # Returns
    ///
    /// true if assumption holds for given data, false otherwise
    ///
    fn verify_assumption(&self, data: &[NumericalValue]) -> bool {
        let res = (self.assumption_fn)(data);
        let mut guard_tested = self.assumption_tested.write().unwrap();
        *guard_tested = true;

        if res {
            let mut guard_valid = self.assumption_valid.write().unwrap();
            *guard_valid = true;
        }
        res
    }
}
