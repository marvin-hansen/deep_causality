/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

/// One row of an arena read through a mask: the entries whose other end `passes`, as
/// `(other, weight)`, in the row's order. Two slices and the mask: small enough to sit inside
/// the container's traversal iterator without growing it.
pub(crate) struct RowEntries<'a, W, F> {
    targets: &'a [usize],
    weights: &'a [W],
    passes: F,
}

impl<'a, W, F: Fn(usize) -> bool> RowEntries<'a, W, F> {
    pub(crate) fn new(targets: &'a [usize], weights: &'a [W], passes: F) -> Self {
        Self {
            targets,
            weights,
            passes,
        }
    }
}

impl<'a, W, F: Fn(usize) -> bool> Iterator for RowEntries<'a, W, F> {
    type Item = (usize, &'a W);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((&other, rest)) = self.targets.split_first() {
            let (weight, weights) = self.weights.split_first()?;
            self.targets = rest;
            self.weights = weights;
            if (self.passes)(other) {
                return Some((other, weight));
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.targets.len()))
    }
}
