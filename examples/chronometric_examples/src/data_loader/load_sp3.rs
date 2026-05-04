use crate::{OrbitData, SatId};
use chrono::{NaiveDate, NaiveDateTime};
use deep_causality_num::RealField;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::path::Path;

pub fn load_orbit_data<R, P>(filename: P, target_sat: &str) -> io::Result<Vec<OrbitData<R>>>
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

        // Skip header lines or empty lines
        if line.len() < 2 {
            continue;
        }

        if line.starts_with('*') {
            // Epoch Line: "*  2016  7  1  0  0  0.00000000"
            let parts: Vec<&str> = line.split_whitespace().collect();
            // Safety check: ensure we have enough parts for a date
            if parts.len() < 7 {
                continue;
            }

            let year = parts[1].parse::<i32>().unwrap();
            let month = parts[2].parse::<u32>().unwrap();
            let day = parts[3].parse::<u32>().unwrap();
            let hour = parts[4].parse::<u32>().unwrap();
            let min = parts[5].parse::<u32>().unwrap();
            let sec = parts[6].parse::<f64>().unwrap() as u32;

            current_time = Some(
                NaiveDate::from_ymd_opt(year, month, day)
                    .unwrap()
                    .and_hms_opt(hour, min, sec)
                    .unwrap(),
            );
        } else if line.starts_with("P") {
            // Position Line Logic
            // It handles "P E14" (Standard) AND "PE14" (Compact)
            if let Some(time) = current_time {
                let parts: Vec<&str> = line.split_whitespace().collect();

                // Determine where the ID and Coordinates are located
                let (sat_id_str, coord_start_index) = if parts[0] == "P" {
                    // Format: "P" "E14" "12345.678" ...
                    (parts[1], 2)
                } else {
                    // Format: "PE14" "12345.678" ...
                    // We slice off the first char 'P' to get "E14"
                    (&parts[0][1..], 1)
                };

                if sat_id_str == target_sat {
                    // SP3 is in KM. Convert to METERS, then to R.
                    let x_f64 = parts[coord_start_index].parse::<f64>().unwrap() * 1000.0;
                    let y_f64 = parts[coord_start_index + 1].parse::<f64>().unwrap() * 1000.0;
                    let z_f64 = parts[coord_start_index + 2].parse::<f64>().unwrap() * 1000.0;

                    let x: R = R::from(x_f64);
                    let y: R = R::from(y_f64);
                    let z: R = R::from(z_f64);

                    let sat_id = SatId::try_from(target_sat).expect("Invalid sat_id");

                    data.push(OrbitData::new(time, sat_id, x, y, z));
                }
            }
        }
    }
    Ok(data)
}
