// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use crate::prelude::Spatial;

use super::*;

impl<T> Spatial<T> for AdjustableSpaceTime<T>
where
    T: Default
        + Copy
        + Clone
        + Hash
        + Eq
        + PartialEq
        + Add<T, Output = T>
        + Sub<T, Output = T>
        + Mul<T, Output = T>,
{
    fn x(&self) -> &T {
        &self.x
    }

    fn y(&self) -> &T {
        &self.y
    }

    fn z(&self) -> &T {
        &self.z
    }
}
