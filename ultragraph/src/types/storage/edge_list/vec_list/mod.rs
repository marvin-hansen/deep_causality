/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use core::marker::PhantomData;

use alloc::vec::Vec;

mod clone;
mod debug;
mod default;
mod edge_list;

/// The incumbent: the entries in one `Vec`, in insertion order until a removal or a
/// `from_entries` reorders them; every lookup scans the list.
pub(crate) struct VecList<V, S> {
    entries: Vec<(usize, V)>,
    keying: PhantomData<S>,
}
