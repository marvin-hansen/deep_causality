// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use std::sync::Arc;
use std::thread;

use dcl_data_structures::ring_buffer::prelude::{AtomicSequence, AtomicSequenceOrdered, Sequence};

#[test]
fn test_atomic_sequence_default() {
    let seq = AtomicSequenceOrdered::default();
    assert_eq!(seq.get(), 0);
}

#[test]
fn test_atomic_sequence_from() {
    let value: Sequence = 42;
    let seq = AtomicSequenceOrdered::from(value);
    assert_eq!(seq.get(), value);
}

#[test]
fn test_atomic_sequence_get_set() {
    let seq = AtomicSequenceOrdered::default();
    seq.set(100);
    assert_eq!(seq.get(), 100);
}

#[test]
fn test_atomic_sequence_compare_exchange_success() {
    let seq = AtomicSequenceOrdered::default();
    assert!(seq.compare_and_swap(0, 1));
    assert_eq!(seq.get(), 1);
}

#[test]
fn test_atomic_sequence_compare_exchange_failure() {
    let seq = AtomicSequenceOrdered::default();
    seq.set(5);
    assert!(!seq.compare_and_swap(0, 1));
    assert_eq!(seq.get(), 5);
}

#[test]
fn test_atomic_sequence_thread_safety() {
    let seq = Arc::new(AtomicSequenceOrdered::default());
    let mut handles = vec![];

    // Spawn 10 threads that increment the sequence
    for _ in 0..10 {
        let seq_clone = Arc::clone(&seq);
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                let mut current = seq_clone.get();
                while !seq_clone.compare_and_swap(current, current + 1) {
                    current = seq_clone.get();
                }
            }
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    // Each thread increments 100 times, so final value should be 1000
    assert_eq!(seq.get(), 1000);
}

#[test]
fn test_atomic_sequence_into() {
    let value: Sequence = 42;
    let seq = AtomicSequenceOrdered::from(value);
    let result: Sequence = seq.into();
    assert_eq!(result, value);
}

#[test]
fn test_atomic_sequence_multiple_modifications() {
    let seq = AtomicSequenceOrdered::default();

    // Test multiple modifications in sequence
    seq.set(1);
    assert_eq!(seq.get(), 1);

    seq.set(5);
    assert_eq!(seq.get(), 5);

    assert!(seq.compare_and_swap(5, 10));
    assert_eq!(seq.get(), 10);

    assert!(!seq.compare_and_swap(5, 15));
    assert_eq!(seq.get(), 10);
}

#[test]
fn test_atomic_sequence_concurrent_reads() {
    let seq = Arc::new(AtomicSequenceOrdered::from(42));
    let mut handles = vec![];

    // Spawn multiple reader threads
    for _ in 0..5 {
        let seq_clone = Arc::clone(&seq);
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                assert_eq!(seq_clone.get(), 42);
            }
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_atomic_sequence_increment() {
    let seq = AtomicSequenceOrdered::default();

    // First increment should return 1 and set value to 1
    assert_eq!(seq.increment(), 1);
    assert_eq!(seq.get(), 1);

    // Second increment should return 2 and set value to 2
    assert_eq!(seq.increment(), 2);
    assert_eq!(seq.get(), 2);

    // Test multiple increments in sequence
    for i in 3..10 {
        assert_eq!(seq.increment(), i);
        assert_eq!(seq.get(), i);
    }
}

#[test]
fn test_atomic_sequence_concurrent_increment() {
    let seq = Arc::new(AtomicSequenceOrdered::default());
    let mut handles = vec![];
    let num_threads = 10;
    let increments_per_thread = 1000;

    // Spawn multiple threads that increment the sequence
    for _ in 0..num_threads {
        let seq_clone = Arc::clone(&seq);
        handles.push(thread::spawn(move || {
            for _ in 0..increments_per_thread {
                seq_clone.increment();
            }
        }));
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    // Verify final value
    assert_eq!(seq.get(), (num_threads * increments_per_thread) as Sequence);
}
