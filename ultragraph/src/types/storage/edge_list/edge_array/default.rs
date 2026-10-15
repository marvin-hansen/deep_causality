/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use core::marker::PhantomData;

use alloc::vec::Vec;

use crate::types::storage::edge_list::EntryKey;
use crate::types::storage::edge_list::edge_array::EdgeArray;

impl<V, S: EntryKey<V>> Default for EdgeArray<V, S> {
    fn default() -> Self {
        Self {
            snapshot: Vec::new(),
            log: Vec::new(),
            count: 0,
            keying: PhantomData,
        }
    }
}
