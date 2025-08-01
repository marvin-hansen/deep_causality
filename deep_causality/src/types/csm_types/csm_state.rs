/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::traits::contextuable::space_temporal::SpaceTemporal;
use crate::traits::contextuable::spatial::Spatial;
use crate::traits::contextuable::temporal::Temporal;
use crate::{Causable, CausalityError, Causaloid, Datable, PropagatingEffect, Symbolic};
use deep_causality_macros::{Constructor, Getters};
use std::fmt::{Display, Formatter};

/// A `CausalState` represents a state in a causal state machine (CSM) that can be evaluated
/// based on causal conditions.
///
/// In a CSM, states are paired with actions. When a state's conditions are met (evaluated to true),
/// the associated action is triggered.
///
/// # Purpose
/// `CausalState` encapsulates a reference to a causaloid that defines when the state should be
/// considered active, along with:
/// - An identifier for the state
/// - A version number for tracking changes
/// - Data that can be used for evaluation
///
/// # Usage
/// `CausalState` is typically used in conjunction with `CausalAction` in a state-action pair
/// within a causal state machine (CSM). The CSM evaluates states and, when conditions are met,
/// fires the associated actions.
///
#[allow(clippy::type_complexity)]
#[derive(Getters, Constructor, Clone, Debug)]
pub struct CausalState<D, S, T, ST, SYM, VS, VT>
where
    D: Datable + Clone,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
{
    /// Unique identifier for the state
    id: usize,
    /// Version number for tracking changes to the state
    version: usize,
    /// Numerical data used for state evaluation
    data: PropagatingEffect,
    /// Reference to a causaloid that defines when this state is active
    causaloid: Causaloid<D, S, T, ST, SYM, VS, VT>,
}

impl<D, S, T, ST, SYM, VS, VT> CausalState<D, S, T, ST, SYM, VS, VT>
where
    D: Datable + Clone,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
{
    /// Evaluates the state using its internal data.
    ///
    /// This method uses the state's causaloid to determine if the state's conditions
    /// are met based on the internal data value.
    ///
    /// # Returns
    /// - `Ok(true)` if the state's conditions are met
    /// - `Ok(false)` if the state's conditions are not met
    /// - `Err(CausalityError)` if an error occurs during evaluation
    ///
    pub fn eval(&self) -> Result<PropagatingEffect, CausalityError> {
        self.causaloid.evaluate(&self.data)
    }

    /// Evaluates the state using provided external data.
    ///
    /// This method uses the state's causaloid to determine if the state's conditions
    /// are met based on the provided data value, rather than the internal data.
    ///
    /// # Parameters
    /// - `data`: The numerical value to use for evaluation
    ///
    /// # Returns
    /// - `Ok(true)` if the state's conditions are met with the provided data
    /// - `Ok(false)` if the state's conditions are not met with the provided data
    /// - `Err(CausalityError)` if an error occurs during evaluation
    ///
    /// ```texttext
    pub fn eval_with_data(
        &self,
        data: &PropagatingEffect,
    ) -> Result<PropagatingEffect, CausalityError> {
        self.causaloid.evaluate(data)
    }
}

impl<D, S, T, ST, SYM, VS, VT> Display for CausalState<D, S, T, ST, SYM, VS, VT>
where
    D: Datable + Clone,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CausalState: \n id: {} version: {} \n data: {:?} causaloid: {}",
            self.id, self.version, self.data, self.causaloid,
        )
    }
}
