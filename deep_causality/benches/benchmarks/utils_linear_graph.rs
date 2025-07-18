/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::{BaseCausalGraph, CausableGraph, CausaloidGraph, IdentificationValue};

use crate::benchmarks::utils_shared;

pub fn build_linear_graph(k: usize) -> BaseCausalGraph {
    // Builds a linear graph: root -> a -> b -> c
    let mut g = CausaloidGraph::new(0 as IdentificationValue);

    let root_causaloid = utils_shared::get_test_causaloid();
    let root_index = g
        .add_root_causaloid(root_causaloid)
        .expect("Failed to add root causaloid");

    let mut previous_idx = root_index;

    for _ in 0..k {
        // add a new causaloid and set current idx to it
        let causaloid = utils_shared::get_test_causaloid();
        let current_idx = g.add_causaloid(causaloid).expect("Failed to add causaloid");

        // link current causaloid to previos causaloid
        g.add_edge(previous_idx, current_idx)
            .expect("Failed to add edge");

        previous_idx = current_idx;
    }

    g.freeze();

    g
}

pub fn get_small_multi_cause_graph_and_data() -> (BaseCausalGraph, [f64; 4]) {
    (
        build_multi_cause_graph(),
        utils_shared::generate_sample_data(),
    )
}

fn build_multi_cause_graph() -> BaseCausalGraph {
    // Builds a multi cause graph:
    //  root
    //  / \
    //  A B
    //  \ /
    //   C

    let mut g = CausaloidGraph::new(0 as IdentificationValue);

    // Add root causaloid
    let root_causaloid = utils_shared::get_test_causaloid();
    let root_index = g
        .add_root_causaloid(root_causaloid)
        .expect("Failed to add root causaloid");

    // Add causaloid A
    let causaloid = utils_shared::get_test_causaloid();
    let idx_a = g.add_causaloid(causaloid).expect("Failed to add causaloid");

    // Link causaloid A to root causaloid
    g.add_edge(root_index, idx_a)
        .expect("Failed to add edge between root and A");

    // Add causaloid B
    let causaloid = utils_shared::get_test_causaloid();
    let idx_b = g.add_causaloid(causaloid).expect("Failed to add causaloid");

    // Link causaloid B to root causaloid
    g.add_edge(root_index, idx_b)
        .expect("Failed to add edge between root and B");

    // Add causaloid C
    let causaloid = utils_shared::get_test_causaloid();
    let idx_c = g.add_causaloid(causaloid).expect("Failed to add causaloid");

    // Link causaloid C to A
    g.add_edge(idx_a, idx_c)
        .expect("Failed to add edge between A and C");

    // Link causaloid C to B
    g.add_edge(idx_b, idx_c)
        .expect("Failed to add edge between C and B");

    g.freeze();

    g
}
