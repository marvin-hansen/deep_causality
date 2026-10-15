/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use core::fmt;

use crate::types::storage::edge_list::VecList;

/// Prints the entries as a list of `(other, value)` pairs.
impl<V: fmt::Debug, S> fmt::Debug for VecList<V, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.entries.iter()).finish()
    }
}
