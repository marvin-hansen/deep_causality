/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use core::marker::PhantomData;

use alloc::vec::Vec;

use crate::types::storage::edge_list::EntryKey;

mod clone;
mod debug;
mod default;
mod edge_list;

/// The log compacts once it holds `max(snapshot.len(), MIN_LOG)` entries.
pub(crate) const MIN_LOG: usize = 16;

/// The snapshot-log candidate (design D5): a snapshot sorted by `(other, key)` and a log of
/// later operations, appended after a lookup; the newest entry for `(other, key)` decides.
/// The log compacts into a new snapshot once it reaches `max(snapshot.len(), MIN_LOG)`.
pub(crate) struct EdgeArray<V, S: EntryKey<V>> {
    snapshot: Vec<(usize, V)>,
    log: Vec<LogEntry<V, S::Key>>,
    /// The number of current entries, kept exact across puts and removals.
    count: usize,
    keying: PhantomData<S>,
}

/// One logged operation on the entries with `other` as their other end.
pub(crate) struct LogEntry<V, K> {
    other: usize,
    op: LogOp<V, K>,
}

/// A put of a value, the removal of one key, or the removal of every key; a removal is an
/// operation, never a value, so a weight of zero stays a weight.
pub(crate) enum LogOp<V, K> {
    Put(V),
    Remove(K),
    RemoveAll,
}
