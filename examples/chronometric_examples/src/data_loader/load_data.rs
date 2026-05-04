/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use super::load_clk::load_clock_data;
use super::load_sp3::load_orbit_data;
use crate::GnssDataResult;
use deep_causality_num::RealField;

use std::path::Path;

pub fn load_data<R, P>(clk_path: P, sp3_path: P, target_sat: &str) -> GnssDataResult<R>
where
    R: RealField + From<f64>,
    P: AsRef<Path>,
{
    let clocks = load_clock_data(clk_path, target_sat)?;
    let orbits = load_orbit_data(sp3_path, target_sat)?;
    Ok((clocks, orbits))
}
