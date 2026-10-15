/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Data {
    pub a: u8,
    pub b: u8,
    pub z: bool,
}

impl Default for Data {
    fn default() -> Self {
        Self {
            a: 0,
            b: 1,
            z: false,
        }
    }
}

/// An edge payload whose key is its kind, so a source can carry several edges to one target.
/// `Default` is kind 0.
#[derive(Debug, Copy, Clone, Default, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct Kind(pub u8);

impl ultragraph::EdgeKey for Kind {
    type Key = u8;
    fn edge_key(&self) -> u8 {
        self.0
    }
}
