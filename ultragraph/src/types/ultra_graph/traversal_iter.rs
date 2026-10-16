/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

/// One iterator type over the three states' traversal iterators, so the container's traversal
/// returns a single opaque type without a heap allocation.
pub(crate) enum TraversalIter<F, D, L> {
    Frozen(F),
    Dynamic(D),
    Live(L),
}

impl<F, D, L> Iterator for TraversalIter<F, D, L>
where
    F: Iterator<Item = usize>,
    D: Iterator<Item = usize>,
    L: Iterator<Item = usize>,
{
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        match self {
            Self::Frozen(iter) => iter.next(),
            Self::Dynamic(iter) => iter.next(),
            Self::Live(iter) => iter.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Frozen(iter) => iter.size_hint(),
            Self::Dynamic(iter) => iter.size_hint(),
            Self::Live(iter) => iter.size_hint(),
        }
    }
}
