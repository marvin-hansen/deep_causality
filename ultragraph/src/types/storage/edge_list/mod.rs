/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The edge lists of the Dynamic state, one per node and direction, behind one trait, so the
//! storage candidates of `ultragraph-live-state-maintained-views` are measured against the same
//! graph code. The temporary Cargo feature `edge-log` selects the snapshot-log arrays and
//! `edge-index` the `Vec` lists indexed on high-degree nodes; without either the per-node `Vec`
//! lists are used. The two features together do not build.

use alloc::vec::Vec;

use crate::EdgeKey;

#[cfg(all(feature = "edge-log", feature = "edge-index"))]
compile_error!("`edge-log` and `edge-index` each select one edge storage; enable at most one");

#[cfg(feature = "edge-log")]
pub(crate) mod edge_array;
#[cfg(feature = "edge-index")]
pub(crate) mod indexed_list;
#[cfg(not(any(feature = "edge-log", feature = "edge-index")))]
pub(crate) mod vec_list;

#[cfg(feature = "edge-log")]
pub(crate) use edge_array::EdgeArray;
#[cfg(feature = "edge-index")]
pub(crate) use indexed_list::IndexedList;
#[cfg(not(any(feature = "edge-log", feature = "edge-index")))]
pub(crate) use vec_list::VecList;

/// The list type behind every node's forward and backward edges.
#[cfg(not(any(feature = "edge-log", feature = "edge-index")))]
pub(crate) type Lists<V, S> = VecList<V, S>;
#[cfg(feature = "edge-log")]
pub(crate) type Lists<V, S> = EdgeArray<V, S>;
#[cfg(feature = "edge-index")]
pub(crate) type Lists<V, S> = IndexedList<V, S>;

/// A node's outgoing edges as `(target, payload)`, keyed by the payload's `edge_key`.
pub(crate) type Forward<W> = Lists<W, ByPayload>;
/// A node's incoming edges as `(source, key)`, keyed by the key itself.
pub(crate) type Backward<W> = Lists<<W as EdgeKey>::Key, ByValue>;

/// How a list entry's value yields the key that, with the entry's other end, identifies the
/// edge: two entries of one list never share `(other, key)`.
pub(crate) trait EntryKey<V> {
    type Key: Copy + Ord;
    fn key(value: &V) -> Self::Key;
}

/// The forward list holds the payload; the key is the payload's `edge_key`.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct ByPayload;

impl<W: EdgeKey> EntryKey<W> for ByPayload {
    type Key = W::Key;
    fn key(value: &W) -> W::Key {
        value.edge_key()
    }
}

/// The backward list holds the key itself.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct ByValue;

impl<K: Copy + Ord> EntryKey<K> for ByValue {
    type Key = K;
    fn key(value: &K) -> K {
        *value
    }
}

/// One node's edges in one direction: entries `(other, value)` with no two sharing
/// `(other, key)`. The order of the entries is not specified.
pub(crate) trait EdgeList<V, S: EntryKey<V>>: Clone + Default {
    /// An empty list with room for `capacity` entries.
    fn with_capacity(capacity: usize) -> Self;
    /// A list holding `entries`, which share no `(other, key)`.
    fn from_entries(entries: Vec<(usize, V)>) -> Self;
    /// The current entries, consuming the list.
    fn into_entries(self) -> Vec<(usize, V)>;
    /// The current entries.
    fn entries<'a>(&'a self) -> impl Iterator<Item = (usize, &'a V)> + 'a
    where
        V: 'a;
    /// Whether any entry has `other` as its other end.
    fn contains(&self, other: usize) -> bool;
    /// Whether the entry `(other, key)` exists.
    fn contains_keyed(&self, other: usize, key: S::Key) -> bool;
    /// Inserts `(other, value)`, or replaces the value of the entry with the same
    /// `(other, key)`; true when inserted, false when replaced.
    fn put(&mut self, other: usize, value: V) -> bool;
    /// Removes every entry with `other` as its other end; the number removed.
    fn remove_all(&mut self, other: usize) -> usize;
    /// Removes the entry `(other, key)`; true when it existed.
    fn remove_keyed(&mut self, other: usize, key: S::Key) -> bool;
    /// The bytes the list's buffers hold: each buffer's capacity times its element size.
    fn storage_bytes(&self) -> usize;
}
