/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use core::marker::PhantomData;
use core::mem::size_of;

use alloc::collections::BTreeSet;
use alloc::vec::Vec;

use crate::types::storage::edge_list::edge_array::{EdgeArray, LogEntry, LogOp, MIN_LOG};
use crate::types::storage::edge_list::{EdgeList, EntryKey};

impl<V: Clone, S: EntryKey<V>> EdgeArray<V, S> {
    /// The newest log entry deciding `(other, key)`: `Some(true)` when a put is the newest,
    /// `Some(false)` when a removal of the key or of every key is, `None` when the log holds
    /// none, so the snapshot decides.
    fn log_decides(&self, other: usize, key: S::Key) -> Option<bool> {
        self.log
            .iter()
            .rev()
            .filter(|entry| entry.other == other)
            .find_map(|entry| match &entry.op {
                LogOp::Put(value) => (S::key(value) == key).then_some(true),
                LogOp::Remove(removed) => (*removed == key).then_some(false),
                LogOp::RemoveAll => Some(false),
            })
    }

    /// The snapshot's run of entries with `other` as their other end, sorted by key.
    fn run(&self, other: usize) -> &[(usize, V)] {
        let start = self.snapshot.partition_point(|(o, _)| *o < other);
        let end = start + self.snapshot[start..].partition_point(|(o, _)| *o <= other);
        &self.snapshot[start..end]
    }

    fn exists(&self, other: usize, key: S::Key) -> bool {
        self.log_decides(other, key).unwrap_or_else(|| {
            self.run(other)
                .binary_search_by(|(_, value)| S::key(value).cmp(&key))
                .is_ok()
        })
    }

    /// The current entries with `other` as their other end, newest log puts first, then the
    /// snapshot's survivors.
    fn current_of(&self, other: usize) -> Vec<(usize, &V)> {
        let mut decided: Vec<S::Key> = Vec::new();
        let mut kept: Vec<(usize, &V)> = Vec::new();
        for entry in self.log.iter().rev().filter(|entry| entry.other == other) {
            match &entry.op {
                LogOp::Put(value) => {
                    let key = S::key(value);
                    if !decided.contains(&key) {
                        decided.push(key);
                        kept.push((other, value));
                    }
                }
                LogOp::Remove(key) => {
                    if !decided.contains(key) {
                        decided.push(*key);
                    }
                }
                LogOp::RemoveAll => {
                    return kept;
                }
            }
        }
        kept.extend(
            self.run(other)
                .iter()
                .filter(|(_, value)| !decided.contains(&S::key(value)))
                .map(|(o, value)| (*o, value)),
        );
        kept
    }

    /// Appends `entry`, and compacts once the log reaches `max(snapshot.len(), MIN_LOG)`.
    fn append(&mut self, other: usize, op: LogOp<V, S::Key>) {
        self.log.push(LogEntry { other, op });
        if self.log.len() >= self.snapshot.len().max(MIN_LOG) {
            self.compact();
        }
    }

    /// Resolves the log into a new snapshot: newest first, each `(other, key)` is decided once,
    /// a removal of every key blocks every older entry of that other end; the survivors are
    /// sorted by `(other, key)` into a snapshot of exactly the current count, and the log is
    /// cleared, keeping its capacity.
    fn compact(&mut self) {
        let mut decided: BTreeSet<(usize, S::Key)> = BTreeSet::new();
        let mut blocked: BTreeSet<usize> = BTreeSet::new();
        let mut next: Vec<(usize, V)> = Vec::with_capacity(self.count);
        for entry in self.log.drain(..).rev() {
            match entry.op {
                LogOp::Put(value) => {
                    let key = S::key(&value);
                    if !blocked.contains(&entry.other) && decided.insert((entry.other, key)) {
                        next.push((entry.other, value));
                    }
                }
                LogOp::Remove(key) => {
                    decided.insert((entry.other, key));
                }
                LogOp::RemoveAll => {
                    blocked.insert(entry.other);
                }
            }
        }
        next.extend(
            core::mem::take(&mut self.snapshot)
                .into_iter()
                .filter(|(other, value)| {
                    !blocked.contains(other) && !decided.contains(&(*other, S::key(value)))
                }),
        );
        sort_by_other_then_key::<V, S>(&mut next);
        self.snapshot = next;
    }
}

/// Sorts entries by their other end, then by key; no two entries share both.
fn sort_by_other_then_key<V, S: EntryKey<V>>(entries: &mut [(usize, V)]) {
    entries.sort_unstable_by(|(a, v), (b, w)| (*a, S::key(v)).cmp(&(*b, S::key(w))));
}

/// The current entries of an array: the snapshot alone when the log is empty, the resolved
/// list otherwise.
pub(crate) enum Entries<'a, V> {
    Snapshot(core::slice::Iter<'a, (usize, V)>),
    Resolved(alloc::vec::IntoIter<(usize, &'a V)>),
}

impl<'a, V> Iterator for Entries<'a, V> {
    type Item = (usize, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Snapshot(iter) => iter.next().map(|(other, value)| (*other, value)),
            Self::Resolved(iter) => iter.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Snapshot(iter) => iter.size_hint(),
            Self::Resolved(iter) => iter.size_hint(),
        }
    }
}

impl<V: Clone, S: EntryKey<V>> EdgeList<V, S> for EdgeArray<V, S> {
    /// Reserves the log, where every write lands; the snapshot takes its exact size at each
    /// compaction.
    fn with_capacity(capacity: usize) -> Self {
        Self {
            snapshot: Vec::new(),
            log: Vec::with_capacity(capacity),
            count: 0,
            keying: PhantomData,
        }
    }

    fn from_entries(mut entries: Vec<(usize, V)>) -> Self {
        sort_by_other_then_key::<V, S>(&mut entries);
        Self {
            count: entries.len(),
            snapshot: entries,
            log: Vec::new(),
            keying: PhantomData,
        }
    }

    fn into_entries(mut self) -> Vec<(usize, V)> {
        if !self.log.is_empty() {
            self.compact();
        }
        self.snapshot
    }

    fn entries<'a>(&'a self) -> impl Iterator<Item = (usize, &'a V)> + 'a
    where
        V: 'a,
    {
        if self.log.is_empty() {
            return Entries::Snapshot(self.snapshot.iter());
        }
        let mut decided: BTreeSet<(usize, S::Key)> = BTreeSet::new();
        let mut blocked: BTreeSet<usize> = BTreeSet::new();
        let mut resolved: Vec<(usize, &V)> = Vec::with_capacity(self.count);
        for entry in self.log.iter().rev() {
            match &entry.op {
                LogOp::Put(value) => {
                    let key = S::key(value);
                    if !blocked.contains(&entry.other) && decided.insert((entry.other, key)) {
                        resolved.push((entry.other, value));
                    }
                }
                LogOp::Remove(key) => {
                    decided.insert((entry.other, *key));
                }
                LogOp::RemoveAll => {
                    blocked.insert(entry.other);
                }
            }
        }
        resolved.extend(
            self.snapshot
                .iter()
                .filter(|(other, value)| {
                    !blocked.contains(other) && !decided.contains(&(*other, S::key(value)))
                })
                .map(|(other, value)| (*other, value)),
        );
        Entries::Resolved(resolved.into_iter())
    }

    fn contains(&self, other: usize) -> bool {
        !self.current_of(other).is_empty()
    }

    fn contains_keyed(&self, other: usize, key: S::Key) -> bool {
        self.exists(other, key)
    }

    /// The count moves before the append, so a compaction the append triggers sizes the new
    /// snapshot by the count after this operation.
    fn put(&mut self, other: usize, value: V) -> bool {
        let inserted = !self.exists(other, S::key(&value));
        if inserted {
            self.count += 1;
        }
        self.append(other, LogOp::Put(value));
        inserted
    }

    fn remove_all(&mut self, other: usize) -> usize {
        let removed = self.current_of(other).len();
        if removed > 0 {
            self.count -= removed;
            self.append(other, LogOp::RemoveAll);
        }
        removed
    }

    fn remove_keyed(&mut self, other: usize, key: S::Key) -> bool {
        if !self.exists(other, key) {
            return false;
        }
        self.count -= 1;
        self.append(other, LogOp::Remove(key));
        true
    }

    fn storage_bytes(&self) -> usize {
        self.snapshot.capacity() * size_of::<(usize, V)>()
            + self.log.capacity() * size_of::<LogEntry<V, S::Key>>()
    }
}
