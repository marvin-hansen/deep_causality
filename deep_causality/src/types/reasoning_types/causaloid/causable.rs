// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use std::collections::HashMap;
use std::hash::Hash;
use std::ops::*;

use crate::errors::CausalityError;
use crate::prelude::{
    Causable, CausableGraph, CausableGraphExplaining, CausableGraphReasoning, CausableReasoning,
    Causaloid, Datable, IdentificationValue, NumericalValue, SpaceTemporal, Spatial, Temporable,
};
use crate::types::reasoning_types::causaloid::causal_type::CausalType;

impl<'l, D, S, T, ST, V> Causable for Causaloid<'l, D, S, T, ST, V>
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
        + Mul<V, Output = V>
        + Clone,
{
    /// Generates an explanation for why this Causaloid is active.
    ///
    /// # Returns
    ///
    /// Result<String, CausalityError>
    ///
    /// * Ok(String) - The explanation if successful.
    /// * Err(CausalityError) - If an error occurs generating the explanation.
    ///
    /// # Behavior
    ///
    /// If this Causaloid is not active, returns an error.
    /// Otherwise:
    /// - For a singleton, returns a simple string explanation.
    /// - For a collection, calls explain() on the collection.
    /// - For a graph, calls explain_all_causes() on the graph.
    ///
    fn explain(&self) -> Result<String, CausalityError> {
        return if self.is_active() {
            match self.causal_type {
                CausalType::Singleton => {
                    let reason = format!(
                        "Causaloid: {} {} evaluated to {}",
                        self.id,
                        self.description,
                        self.is_active()
                    );
                    Ok(reason)
                }

                CausalType::Collection => Ok(self.causal_coll.as_ref().unwrap().explain()),

                CausalType::Graph => {
                    match self.causal_graph.as_ref().unwrap().explain_all_causes() {
                        Ok(str) => Ok(str),
                        Err(e) => Err(CausalityError(e.to_string())),
                    }
                }
            }
        } else {
            // Return an error message that the causaloid is not active
            let reason = format!(
                "Causaloid: {} has not been evaluated. Call verify() to activate it",
                self.id
            );

            Err(CausalityError(reason))
        };
    }

    fn is_active(&self) -> bool {
        match self.causal_type {
            CausalType::Singleton => *self.active.read().unwrap(),
            CausalType::Collection => self.causal_coll.as_ref().unwrap().number_active() > 0f64,
            CausalType::Graph => self.causal_graph.as_ref().unwrap().number_active() > 0f64,
        }
    }

    /// Checks if this Causaloid is a singleton.
    ///
    /// # Returns
    ///
    /// bool - True if this is a singleton Causaloid, false otherwise.
    ///
    /// # Behavior
    ///
    /// Returns true if the causal_type is CausalType::Singleton.
    /// Returns false for CausalType::Collection or CausalType::Graph.
    ///
    fn is_singleton(&self) -> bool {
        match self.causal_type {
            CausalType::Singleton => true,
            CausalType::Collection => false,
            CausalType::Graph => false,
        }
    }

    /// Verifies that the single cause in this Causaloid is satisfied by the given observation.
    ///
    /// This should only be called on singleton Causaloids.
    ///
    /// # Parameters
    ///
    /// * `obs` - The input observation to verify against the cause.
    ///
    /// # Returns
    ///
    /// Result<bool, CausalityError>
    ///
    /// * Ok(bool) - True if the cause is satisfied, false otherwise.
    /// * Err(CausalityError) - If an error occurs during reasoning.
    ///
    /// # Behavior
    ///
    /// If this Causaloid has context, calls the contextual causal function.
    /// Otherwise, calls the standard causal function.
    /// Updates the internal `active` state with the result.
    ///
    fn verify_single_cause(&self, obs: &NumericalValue) -> Result<bool, CausalityError> {
        return if self.has_context {
            let contextual_causal_fn = match self.context_causal_fn {
                Some(ref causal_fn) => causal_fn,
                None => {
                    return Err(CausalityError(
                        "Causaloid::verify_single_cause: contextual_causal_fn is None".to_string(),
                    ))
                }
            };

            let context = match self.context {
                Some(context) => context,
                None => {
                    return Err(CausalityError(
                        "Causaloid::verify_single_cause: context is None".to_string(),
                    ))
                }
            };

            let res = match (contextual_causal_fn)(obs.to_owned(), context) {
                Ok(res) => res,
                Err(e) => return Err(e),
            };

            let mut guard = self.active.write().unwrap();
            *guard = res;

            Ok(res)
        } else {
            let causal_fn = self
                .causal_fn
                .expect("Causaloid::verify_single_cause: causal_fn is None");

            //
            let res = match (causal_fn)(obs.to_owned()) {
                Ok(res) => res,
                Err(e) => return Err(e),
            };

            let mut guard = self.active.write().unwrap();
            *guard = res;

            Ok(res)
        };
    }

    /// Verifies that all causes in this Causaloid are satisfied by the given data.
    ///
    /// # Parameters
    ///
    /// * `data` - The input data to verify against the causes.
    /// * `data_index` - Optional index mapping data IDs to indices.
    ///
    /// # Returns
    ///
    /// Result<bool, CausalityError>
    ///
    /// * Ok(bool) - True if all causes are satisfied, false otherwise.
    /// * Err(CausalityError) - If an error occurs during reasoning.
    ///
    /// # Behavior
    ///
    /// If this is a singleton Causaloid, returns an error.
    /// If the causal collection or graph is None, returns an error.
    /// Otherwise, calls `reason_all_causes` on the collection or graph.
    ///
    fn verify_all_causes(
        &self,
        data: &[NumericalValue],
        data_index: Option<&HashMap<IdentificationValue, IdentificationValue>>,
    ) -> Result<bool, CausalityError> {
        match self.causal_type {
            CausalType::Singleton => Err(CausalityError(
                "Causaloid is singleton. Call verify_single_cause instead.".into(),
            )),

            CausalType::Collection => match &self.causal_coll {
                None => Err(CausalityError(
                    "Causaloid::verify_all_causes: causal collection is None".into(),
                )),
                Some(coll) => {
                    let res = match coll.reason_all_causes(data) {
                        Ok(res) => res,
                        Err(e) => return Err(e),
                    };

                    Ok(res)
                }
            },

            CausalType::Graph => match &self.causal_graph {
                None => Err(CausalityError(
                    "Causaloid::verify_all_causes: Causal graph is None".into(),
                )),
                Some(graph) => {
                    let res = match graph.reason_all_causes(data, data_index) {
                        Ok(res) => res,
                        Err(e) => return Err(CausalityError(e.to_string())),
                    };

                    Ok(res)
                }
            },
        }
    }
}
