/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

#![forbid(unsafe_code)]

pub use crate::grid_type::ArrayGrid;
pub use crate::grid_type::ArrayType;
pub use crate::grid_type::ArrayType::*;
// Array Grid types
pub use crate::grid_type::grid::Grid;
pub use crate::grid_type::point::PointIndex;
pub use crate::grid_type::point::PointIndexType;
pub use crate::grid_type::storage::Storage;
// window types
pub use crate::window_type;
pub use crate::window_type::SlidingWindow;

pub use crate::window_type::storage::WindowStorage;
pub use crate::window_type::storage_safe::storage_array::ArrayStorage;
pub use crate::window_type::storage_safe::storage_vec::VectorStorage;

#[cfg(feature = "unsafe")]
pub use crate::window_type::storage_unsafe::{UnsafeArrayStorage, UnsafeVectorStorage};
