/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_macros::Constructor;
use std::error::Error;
use std::fmt;

#[derive(Constructor, Debug)]
pub struct IndexError(pub String);

impl Error for IndexError {}

impl fmt::Display for IndexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "IndexError: {}", self.0)
    }
}

impl From<String> for IndexError {
    fn from(s: String) -> Self {
        IndexError(s)
    }
}

impl From<&str> for IndexError {
    fn from(s: &str) -> Self {
        IndexError(s.to_string())
    }
}
