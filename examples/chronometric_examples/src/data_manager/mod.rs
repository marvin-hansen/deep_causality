/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{ClockData, GnssDataResult, OrbitData};
use deep_causality_num::RealField;
use std::io;
use std::path::{Path, PathBuf};

/// Get the absolute path to the data input directory (data/gnss)
pub fn get_gnss_data_input_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("data/gnss");
    path
}

/// Unified data manager for all GQCD experiment data loading.
///
/// Provides a centralized API for loading:
/// - GNSS satellite data (clock, orbit, single/multi-satellite)
#[derive(Debug, Clone)]
pub struct DataManager {}

impl Default for DataManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DataManager {
    pub fn new() -> Self {
        Self {}
    }
}

impl DataManager {
    // ========================================================================
    // GNSS Data Loading Methods
    // ========================================================================

    /// Load GNSS data for a single satellite.
    ///
    /// # Arguments
    /// * `clk_path` - Path to the .clk (clock) file
    /// * `sp3_path` - Path to the .sp3 (orbit) file
    /// * `sat_id` - Satellite ID (e.g., "E14", "E18")
    ///
    /// # Returns
    /// Tuple of (clock_data, orbit_data) vectors
    pub fn load_gnss_single_satellite<R, P>(
        &self,
        clk_path: P,
        sp3_path: P,
        sat_id: &str,
    ) -> GnssDataResult<R>
    where
        R: RealField + From<f64>,
        P: AsRef<Path>,
    {
        crate::data_loader::load_data::load_data(clk_path, sp3_path, sat_id)
    }

    /// Load GNSS data for all Galileo satellites in the files.
    ///
    /// # Arguments
    /// * `clk_path` - Path to the .clk (clock) file
    /// * `sp3_path` - Path to the .sp3 (orbit) file
    ///
    /// # Returns
    /// Tuple of (clock_data, orbit_data) vectors containing all satellites
    pub fn load_gnss_all_satellites<R, P>(&self, clk_path: P, sp3_path: P) -> GnssDataResult<R>
    where
        R: RealField + From<f64>,
        P: AsRef<Path>,
    {
        crate::data_loader::load_all_satellites::load_all_satellites(clk_path, sp3_path)
    }

    /// Load only GNSS clock data for a single satellite.
    ///
    /// # Arguments
    /// * `clk_path` - Path to the .clk (clock) file
    /// * `sat_id` - Satellite ID (e.g., "E14", "E18")
    ///
    /// # Returns
    /// Vector of clock data
    pub fn load_gnss_clock_data<R, P>(
        &self,
        clk_path: P,
        sat_id: &str,
    ) -> io::Result<Vec<ClockData<R>>>
    where
        R: RealField + From<f64>,
        P: AsRef<Path>,
    {
        crate::data_loader::load_clk::load_clock_data(clk_path, sat_id)
    }

    /// Load only GNSS orbit data for a single satellite.
    ///
    /// # Arguments
    /// * `sp3_path` - Path to the .sp3 (orbit) file
    /// * `sat_id` - Satellite ID (e.g., "E14", "E18")
    ///
    /// # Returns
    /// Vector of orbit data
    pub fn load_gnss_orbit_data<R, P>(
        &self,
        sp3_path: P,
        sat_id: &str,
    ) -> io::Result<Vec<OrbitData<R>>>
    where
        R: RealField + From<f64>,
        P: AsRef<Path>,
    {
        crate::data_loader::load_sp3::load_orbit_data(sp3_path, sat_id)
    }
}
