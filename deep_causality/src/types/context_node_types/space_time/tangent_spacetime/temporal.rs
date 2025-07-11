/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{TangentSpacetime, Temporal, TimeScale};

impl Temporal<f64> for TangentSpacetime {
    fn time_scale(&self) -> TimeScale {
        TimeScale::Second
    }
    fn time_unit(&self) -> f64 {
        self.t
    }
}
