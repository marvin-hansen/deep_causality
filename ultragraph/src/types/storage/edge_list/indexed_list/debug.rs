/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use core::fmt;

use crate::types::storage::edge_list::EntryKey;
use crate::types::storage::edge_list::indexed_list::IndexedList;

/// Prints the entries as a list of `(other, value)` pairs, as the `Vec` list does; the index
/// is derived from them and is left out.
impl<V: fmt::Debug, S: EntryKey<V>> fmt::Debug for IndexedList<V, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.entries.iter()).finish()
    }
}
