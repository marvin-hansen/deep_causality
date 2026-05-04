/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Load all satellites from GNSS data files (for multi-satellite analysis like tidal).

use crate::{ClockData, GnssDataResult, OrbitData, SatId};
use chrono::{NaiveDate, NaiveDateTime};
use deep_causality_num::RealField;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

/// Loads clock and orbit data for ALL Galileo satellites in the files.
/// Returns (clocks, orbits) where each contains data from all satellites.
pub fn load_all_satellites<R, P>(clk_path: P, sp3_path: P) -> GnssDataResult<R>
where
    R: RealField + From<f64>,
    P: AsRef<Path>,
{
    let clocks = parse_clk_all(&clk_path)?;
    let orbits = parse_sp3_all(&sp3_path)?;
    Ok((clocks, orbits))
}

/// Parse all satellites from CLK file (not just a single target).
fn parse_clk_all<R, P>(filename: P) -> io::Result<Vec<ClockData<R>>>
where
    R: RealField + From<f64>,
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    let reader = io::BufReader::new(file);
    let mut data = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if line.starts_with("AS") {
            // Line: "AS E18  2016 07 01 00 00 00.000000  2  0.123456789012"
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 10 {
                continue;
            }

            let sat_id_str = parts[1];

            // Only process Galileo satellites (E prefix)
            if !sat_id_str.starts_with('E') {
                continue;
            }

            let year = parts[2].parse::<i32>().unwrap();
            let month = parts[3].parse::<u32>().unwrap();
            let day = parts[4].parse::<u32>().unwrap();
            let hour = parts[5].parse::<u32>().unwrap();
            let minute = parts[6].parse::<u32>().unwrap();

            if let Some(date) = NaiveDate::from_ymd_opt(year, month, day)
                && let Some(time) = date.and_hms_opt(hour, minute, 0)
            {
                let bias_f64 = parts[9].parse::<f64>().unwrap_or(0.0);

                // Filter invalid values
                if bias_f64.abs() > 900_000.0 {
                    continue;
                }

                if let Ok(sat_id) = SatId::try_from(sat_id_str) {
                    let bias: R = R::from(bias_f64);
                    data.push(ClockData::new(time, sat_id, bias));
                }
            }
        }
    }
    Ok(data)
}

/// Parse all satellites from SP3 file (not just a single target).
fn parse_sp3_all<R, P>(filename: P) -> io::Result<Vec<OrbitData<R>>>
where
    R: RealField + From<f64>,
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    let reader = io::BufReader::new(file);
    let mut data = Vec::new();
    let mut current_time: Option<NaiveDateTime> = None;

    for line in reader.lines() {
        let line = line?;

        if line.len() < 2 {
            continue;
        }

        if line.starts_with('*') {
            // Epoch Line: "*  2016  7  1  0  0  0.00000000"
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 7 {
                let year = parts[1].parse::<i32>().unwrap();
                let month = parts[2].parse::<u32>().unwrap();
                let day = parts[3].parse::<u32>().unwrap();
                let hour = parts[4].parse::<u32>().unwrap();
                let minute = parts[5].parse::<u32>().unwrap();

                if let Some(date) = NaiveDate::from_ymd_opt(year, month, day)
                    && let Some(dt) = date.and_hms_opt(hour, minute, 0)
                {
                    current_time = Some(dt);
                }
            }
        } else if let Some(first_char) = line.chars().next()
            && (first_char == 'P' || first_char == 'G')
            && let Some(time) = current_time
        {
            // Position Line: "PE14  12345.678901  -9876.543210  5432.109876  999.999999"
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            // Extract satellite ID (remove leading P/G)
            let sat_id_full = parts[0];
            let (coord_start_index, sat_id_str) =
                if sat_id_full.len() > 1 && sat_id_full.chars().nth(1).unwrap().is_alphabetic() {
                    (1, &sat_id_full[1..])
                } else {
                    (1, sat_id_full)
                };

            // Only process Galileo satellites (E prefix)
            if !sat_id_str.starts_with('E') {
                continue;
            }

            if parts.len() >= coord_start_index + 3 {
                // SP3 is in KM, convert to METERS, then to R
                let x_f64 = parts[coord_start_index].parse::<f64>().unwrap_or(0.0) * 1000.0;
                let y_f64 = parts[coord_start_index + 1].parse::<f64>().unwrap_or(0.0) * 1000.0;
                let z_f64 = parts[coord_start_index + 2].parse::<f64>().unwrap_or(0.0) * 1000.0;

                // Skip invalid positions
                if x_f64.abs() < 1.0 || y_f64.abs() < 1.0 || z_f64.abs() < 1.0 {
                    continue;
                }

                if let Ok(sat_id) = SatId::try_from(sat_id_str) {
                    let x: R = R::from(x_f64);
                    let y: R = R::from(y_f64);
                    let z: R = R::from(z_f64);
                    data.push(OrbitData::new(time, sat_id, x, y, z));
                }
            }
        }
    }
    Ok(data)
}
