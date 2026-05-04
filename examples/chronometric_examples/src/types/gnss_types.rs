//! GNSS data type definitions and aliases.

use crate::{ClockData, OrbitData};
use std::io;

/// Result type for loading GNSS satellite data (clock and orbit).
/// This type alias reduces complexity for functions that return both clock and orbit data.
pub type GnssDataResult<R> = io::Result<(Vec<ClockData<R>>, Vec<OrbitData<R>>)>;
