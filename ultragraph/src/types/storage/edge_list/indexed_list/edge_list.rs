/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use core::marker::PhantomData;
use core::mem::size_of;

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::types::storage::edge_list::indexed_list::{
    BTREE_NODE_ENTRIES, BTREE_NODE_HEADER, HUB_DEGREE, IndexedList,
};
use crate::types::storage::edge_list::{EdgeList, EntryKey};

/// The bytes counted for a `BTreeMap` of `len` entries whose keys and values have the given
/// sizes: each entry beside one child pointer, and one node header per `BTREE_NODE_ENTRIES`.
fn btree_bytes(len: usize, key: usize, value: usize) -> usize {
    len * (key + value + size_of::<usize>()) + len.div_ceil(BTREE_NODE_ENTRIES) * BTREE_NODE_HEADER
}

impl<V: Clone, S: EntryKey<V>> IndexedList<V, S> {
    /// The position of the entry `(other, key)`: through the index when the list has one, by a
    /// scan otherwise.
    fn position_of(&self, other: usize, key: S::Key) -> Option<usize> {
        match &self.index {
            Some(index) => index.get(&other).and_then(|keys| keys.get(&key)).copied(),
            None => self
                .entries
                .iter()
                .position(|(o, value)| *o == other && S::key(value) == key),
        }
    }

    /// Builds the index from the entries.
    fn build_index(entries: &[(usize, V)]) -> BTreeMap<usize, BTreeMap<S::Key, usize>> {
        let mut index: BTreeMap<usize, BTreeMap<S::Key, usize>> = BTreeMap::new();
        for (position, (other, value)) in entries.iter().enumerate() {
            index
                .entry(*other)
                .or_default()
                .insert(S::key(value), position);
        }
        index
    }

    /// Builds the index once the list reaches `HUB_DEGREE` entries, and drops it once the list
    /// falls below half of that.
    fn follow_threshold(&mut self) {
        let len = self.entries.len();
        if self.index.is_none() && len >= HUB_DEGREE {
            self.index = Some(Self::build_index(&self.entries));
        } else if self.index.is_some() && len < HUB_DEGREE / 2 {
            self.index = None;
        }
    }

    /// Swap-removes the entry at `position` and re-records the entry moved into its place.
    fn swap_remove(&mut self, position: usize) {
        self.entries.swap_remove(position);
        if let Some(index) = &mut self.index
            && let Some((other, value)) = self.entries.get(position)
        {
            index
                .entry(*other)
                .or_default()
                .insert(S::key(value), position);
        }
    }
}

impl<V: Clone, S: EntryKey<V>> EdgeList<V, S> for IndexedList<V, S> {
    fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: Vec::with_capacity(capacity),
            index: None,
            keying: PhantomData,
        }
    }

    fn from_entries(entries: Vec<(usize, V)>) -> Self {
        let mut list = Self {
            entries,
            index: None,
            keying: PhantomData,
        };
        list.follow_threshold();
        list
    }

    fn into_entries(self) -> Vec<(usize, V)> {
        self.entries
    }

    fn entries<'a>(&'a self) -> impl Iterator<Item = (usize, &'a V)> + 'a
    where
        V: 'a,
    {
        self.entries.iter().map(|(other, value)| (*other, value))
    }

    fn contains(&self, other: usize) -> bool {
        match &self.index {
            Some(index) => index.contains_key(&other),
            None => self.entries.iter().any(|(o, _)| *o == other),
        }
    }

    fn contains_keyed(&self, other: usize, key: S::Key) -> bool {
        self.position_of(other, key).is_some()
    }

    fn put(&mut self, other: usize, value: V) -> bool {
        let key = S::key(&value);
        match self.position_of(other, key) {
            Some(position) => {
                self.entries[position].1 = value;
                false
            }
            None => {
                let position = self.entries.len();
                self.entries.push((other, value));
                if let Some(index) = &mut self.index {
                    index.entry(other).or_default().insert(key, position);
                }
                self.follow_threshold();
                true
            }
        }
    }

    fn remove_all(&mut self, other: usize) -> usize {
        let removed = match &mut self.index {
            Some(index) => {
                let Some(keys) = index.remove(&other) else {
                    return 0;
                };
                // Highest position first, so a swap moves no entry of `other` into a position
                // still to be removed: every entry of `other` above it is already gone.
                let mut positions: Vec<usize> = keys.into_values().collect();
                positions.sort_unstable_by(|a, b| b.cmp(a));
                for position in &positions {
                    self.swap_remove(*position);
                }
                positions.len()
            }
            None => {
                let before = self.entries.len();
                self.entries.retain(|(o, _)| *o != other);
                before - self.entries.len()
            }
        };
        self.follow_threshold();
        removed
    }

    fn remove_keyed(&mut self, other: usize, key: S::Key) -> bool {
        let Some(position) = self.position_of(other, key) else {
            return false;
        };
        if let Some(index) = &mut self.index {
            let keys = index
                .get_mut(&other)
                .expect("an indexed entry's other end is indexed");
            keys.remove(&key);
            if keys.is_empty() {
                index.remove(&other);
            }
        }
        self.swap_remove(position);
        self.follow_threshold();
        true
    }

    fn storage_bytes(&self) -> usize {
        let index = self.index.as_ref().map_or(0, |index| {
            btree_bytes(
                index.len(),
                size_of::<usize>(),
                size_of::<BTreeMap<S::Key, usize>>(),
            ) + index
                .values()
                .map(|keys| btree_bytes(keys.len(), size_of::<S::Key>(), size_of::<usize>()))
                .sum::<usize>()
        });
        self.entries.capacity() * size_of::<(usize, V)>() + index
    }
}
