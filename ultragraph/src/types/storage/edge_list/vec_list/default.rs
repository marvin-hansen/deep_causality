/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use core::marker::PhantomData;

use alloc::vec::Vec;

use crate::types::storage::edge_list::VecList;

impl<V, S> Default for VecList<V, S> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            keying: PhantomData,
        }
    }
}
