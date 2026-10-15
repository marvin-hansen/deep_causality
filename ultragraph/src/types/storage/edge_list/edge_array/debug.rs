/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use core::fmt;

use crate::types::storage::edge_list::edge_array::EdgeArray;
use crate::types::storage::edge_list::{EdgeList, EntryKey};

/// Prints the current entries as a list of `(other, value)` pairs, the log resolved, so the
/// output is the `Vec` list's.
impl<V: Clone + fmt::Debug, S: EntryKey<V>> fmt::Debug for EdgeArray<V, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.entries()).finish()
    }
}
