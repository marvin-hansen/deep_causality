/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

pub const SIZE: usize = 10;
#[cfg(feature = "unsafe")]
pub const CAPACITY: usize = MULT * SIZE;
pub const MULT: usize = 100;
