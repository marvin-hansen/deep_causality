/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Identifiable, MinkowskiSpacetime};

impl Identifiable for MinkowskiSpacetime {
    fn id(&self) -> u64 {
        self.id
    }
}
