/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use core::marker::PhantomData;

use crate::types::storage::edge_list::EntryKey;
use crate::types::storage::edge_list::edge_array::{EdgeArray, LogEntry, LogOp};

impl<V: Clone, K: Copy> Clone for LogOp<V, K> {
    fn clone(&self) -> Self {
        match self {
            Self::Put(value) => Self::Put(value.clone()),
            Self::Remove(key) => Self::Remove(*key),
            Self::RemoveAll => Self::RemoveAll,
        }
    }
}

impl<V: Clone, K: Copy> Clone for LogEntry<V, K> {
    fn clone(&self) -> Self {
        Self {
            other: self.other,
            op: self.op.clone(),
        }
    }
}

impl<V: Clone, S: EntryKey<V>> Clone for EdgeArray<V, S> {
    fn clone(&self) -> Self {
        Self {
            snapshot: self.snapshot.clone(),
            log: self.log.clone(),
            count: self.count,
            keying: PhantomData,
        }
    }
}
