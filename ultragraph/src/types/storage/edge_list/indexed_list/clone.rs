/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use core::marker::PhantomData;

use crate::types::storage::edge_list::EntryKey;
use crate::types::storage::edge_list::indexed_list::IndexedList;

impl<V: Clone, S: EntryKey<V>> Clone for IndexedList<V, S> {
    fn clone(&self) -> Self {
        Self {
            entries: self.entries.clone(),
            index: self.index.clone(),
            keying: PhantomData,
        }
    }
}
