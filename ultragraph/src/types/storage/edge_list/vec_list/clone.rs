/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use core::marker::PhantomData;

use crate::types::storage::edge_list::VecList;

impl<V: Clone, S> Clone for VecList<V, S> {
    fn clone(&self) -> Self {
        Self {
            entries: self.entries.clone(),
            keying: PhantomData,
        }
    }
}
