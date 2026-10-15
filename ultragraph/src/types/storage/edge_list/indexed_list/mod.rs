/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use core::marker::PhantomData;

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::types::storage::edge_list::EntryKey;

mod clone;
mod debug;
mod default;
mod edge_list;

/// A list builds its index once it holds `HUB_DEGREE` entries and drops it once it falls below
/// half of that, so a node around the threshold does not rebuild on every edge.
pub(crate) const HUB_DEGREE: usize = 64;

/// The bytes counted for one node of a `BTreeMap` beyond its entries: the parent pointer, the
/// index in the parent and the length, padded to the pointer alignment.
pub(crate) const BTREE_NODE_HEADER: usize = 16;

/// The entries a `BTreeMap` node holds at most, B = 6: one node header is counted per that many
/// entries.
pub(crate) const BTREE_NODE_ENTRIES: usize = 11;

/// The indexed-lists candidate (design D5b): the `Vec` list of the incumbent, plus, while the
/// list holds `HUB_DEGREE` entries or more, an index from the other end and the key to the
/// entry's position, so a keyed lookup is O(log d) in place of the scan. The list stays the
/// source of truth; a removal swap-removes and re-records the moved entry.
pub(crate) struct IndexedList<V, S: EntryKey<V>> {
    entries: Vec<(usize, V)>,
    index: Option<BTreeMap<usize, BTreeMap<S::Key, usize>>>,
    keying: PhantomData<S>,
}
