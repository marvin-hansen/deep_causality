/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{ClockData, SatId};
use chrono::NaiveDate;
use deep_causality_num::RealField;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::path::Path;

pub fn load_clock_data<R, P>(filename: P, target_sat: &str) -> io::Result<Vec<ClockData<R>>>
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
            let sat_id = parts[1];

            if sat_id == target_sat {
                let year = parts[2].parse::<i32>().unwrap();
                let month = parts[3].parse::<u32>().unwrap();
                let day = parts[4].parse::<u32>().unwrap();
                let hour = parts[5].parse::<u32>().unwrap();
                let min = parts[6].parse::<u32>().unwrap();
                let sec = parts[7].parse::<f64>().unwrap() as u32;

                let time = NaiveDate::from_ymd_opt(year, month, day)
                    .unwrap()
                    .and_hms_opt(hour, min, sec)
                    .unwrap();

                // Column 9 is usually the bias (check if Column 8 is '2' which means 2 data values follows)
                // In standard RINEX CLK: Code, SatID, Time(YMDHMS), NumberOfValues, Bias, Sigma
                let bias_f64 = parts[9].parse::<f64>().unwrap();

                // SAFETY FILTER: IGS uses 999999.999999 to denote invalid/missing/broken clock data.
                // We filter out anything > 900,000 ns (typical bias is < 1,000,000 ns unless broken).
                // E24 specifically has 999999.999999.
                if bias_f64 > 900_000.0 {
                    continue;
                }

                let sat_id = SatId::try_from(sat_id).expect("Invalid sat_id");
                let bias: R = R::from(bias_f64);

                data.push(ClockData::new(time, sat_id, bias));
            }
        }
    }
    Ok(data)
}
