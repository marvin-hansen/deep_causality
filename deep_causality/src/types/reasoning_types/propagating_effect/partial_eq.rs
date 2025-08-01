/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::PropagatingEffect;
use std::sync::Arc;

impl PartialEq for PropagatingEffect {
    fn eq(&self, _other: &Self) -> bool {
        match (self, _other) {
            (Self::None, Self::None) => true,
            (Self::Deterministic(l), Self::Deterministic(r)) => l == r,
            (Self::Numerical(l), Self::Numerical(r)) => l == r,
            (Self::Probabilistic(l), Self::Probabilistic(r)) => l == r,
            (Self::ContextualLink(l0, l1), Self::ContextualLink(r0, r1)) => l0 == r0 && l1 == r1,
            (Self::Map(l), Self::Map(r)) => l == r,
            (Self::Graph(l), Self::Graph(r)) => Arc::ptr_eq(l, r),
            (Self::RelayTo(l0, l1), Self::RelayTo(r0, r1)) => l0 == r0 && l1 == r1,
            _ => false,
        }
    }
}
