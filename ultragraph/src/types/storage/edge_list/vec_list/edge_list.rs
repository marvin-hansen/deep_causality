/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use core::marker::PhantomData;
use core::mem::size_of;

use alloc::vec::Vec;

use crate::types::storage::edge_list::{EdgeList, EntryKey, VecList};

impl<V: Clone, S: EntryKey<V>> EdgeList<V, S> for VecList<V, S> {
    fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: Vec::with_capacity(capacity),
            keying: PhantomData,
        }
    }

    fn from_entries(entries: Vec<(usize, V)>) -> Self {
        Self {
            entries,
            keying: PhantomData,
        }
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
        self.entries.iter().any(|(o, _)| *o == other)
    }

    fn contains_keyed(&self, other: usize, key: S::Key) -> bool {
        self.entries
            .iter()
            .any(|(o, value)| *o == other && S::key(value) == key)
    }

    fn put(&mut self, other: usize, value: V) -> bool {
        let key = S::key(&value);
        let existing = self
            .entries
            .iter()
            .position(|(o, v)| *o == other && S::key(v) == key);
        match existing {
            Some(position) => {
                self.entries[position].1 = value;
                false
            }
            None => {
                self.entries.push((other, value));
                true
            }
        }
    }

    fn remove_all(&mut self, other: usize) -> usize {
        let before = self.entries.len();
        self.entries.retain(|(o, _)| *o != other);
        before - self.entries.len()
    }

    fn remove_keyed(&mut self, other: usize, key: S::Key) -> bool {
        let position = self
            .entries
            .iter()
            .position(|(o, v)| *o == other && S::key(v) == key);
        match position {
            Some(position) => {
                self.entries.remove(position);
                true
            }
            None => false,
        }
    }

    fn storage_bytes(&self) -> usize {
        self.entries.capacity() * size_of::<(usize, V)>()
    }
}
