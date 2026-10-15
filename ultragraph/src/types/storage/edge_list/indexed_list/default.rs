/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use core::marker::PhantomData;

use alloc::vec::Vec;

use crate::types::storage::edge_list::EntryKey;
use crate::types::storage::edge_list::indexed_list::IndexedList;

impl<V, S: EntryKey<V>> Default for IndexedList<V, S> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            index: None,
            keying: PhantomData,
        }
    }
}
