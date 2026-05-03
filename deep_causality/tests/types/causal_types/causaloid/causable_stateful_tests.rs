/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for [`StatefulMonadicCausable`] on `Causaloid` (singleton form).

use deep_causality::*;
use deep_causality_core::CausalityErrorEnum;
use deep_causality_haft::LogAddEntry;

#[derive(Debug, Default, Clone, PartialEq)]
struct CounterState {
    count: u64,
}

#[derive(Debug, Default, Clone, PartialEq)]
struct ConfigCtx {
    multiplier: u64,
}

fn stateful_increment(
    obs: EffectValue<u64>,
    mut state: CounterState,
    ctx: Option<ConfigCtx>,
) -> PropagatingProcess<u64, CounterState, ConfigCtx> {
    let val = match obs.into_value() {
        Some(v) => v,
        None => {
            return PropagatingProcess {
                value: EffectValue::None,
                state,
                context: ctx,
                error: Some(CausalityError::new(CausalityErrorEnum::Custom(
                    "stateful_increment: value is None".into(),
                ))),
                logs: EffectLog::new(),
            };
        }
    };

    let m = ctx.as_ref().map(|c| c.multiplier).unwrap_or(1);
    state.count += 1;

    let mut process: PropagatingProcess<u64, CounterState, ConfigCtx> = PropagatingProcess {
        value: EffectValue::Value(val * m),
        state,
        context: ctx,
        error: None,
        logs: EffectLog::new(),
    };
    process.logs.add_entry("stateful_increment: state advanced");
    process
}

fn stateful_failing(
    _obs: EffectValue<u64>,
    state: CounterState,
    ctx: Option<ConfigCtx>,
) -> PropagatingProcess<u64, CounterState, ConfigCtx> {
    let mut p: PropagatingProcess<u64, CounterState, ConfigCtx> = PropagatingProcess {
        value: EffectValue::None,
        state,
        context: ctx,
        error: Some(CausalityError::new(CausalityErrorEnum::Custom(
            "stateful_failing: deliberate failure".into(),
        ))),
        logs: EffectLog::new(),
    };
    p.logs
        .add_entry("stateful_failing: closure invoked, returning error");
    p
}

fn stateless_passthrough(input: u64) -> PropagatingEffect<u64> {
    PropagatingEffect::from_value(input)
}

fn build_incoming(
    state: CounterState,
    context: Option<ConfigCtx>,
    value: u64,
) -> PropagatingProcess<u64, CounterState, ConfigCtx> {
    PropagatingProcess {
        value: EffectValue::Value(value),
        state,
        context,
        error: None,
        logs: EffectLog::new(),
    }
}

#[test]
fn evaluate_stateful_threads_state_and_context_through_closure() {
    let causaloid: Causaloid<u64, u64, CounterState, ConfigCtx> = Causaloid::new_with_context(
        7,
        stateful_increment,
        ConfigCtx { multiplier: 3 },
        "stateful increment",
    );

    let initial_state = CounterState { count: 41 };
    let incoming = build_incoming(initial_state.clone(), Some(ConfigCtx { multiplier: 3 }), 5);

    let out = causaloid.evaluate_stateful(&incoming);

    assert!(
        out.error.is_none(),
        "expected no error, got {:?}",
        out.error
    );
    assert_eq!(out.state, CounterState { count: 42 });
    assert_ne!(out.state, CounterState::default());
    assert_eq!(out.value, EffectValue::Value(15));
    assert!(
        !out.logs.is_empty(),
        "expected log entries from input/output/closure logging"
    );
}

#[test]
fn evaluate_stateful_passes_state_through_when_closure_is_stateless() {
    let causaloid: Causaloid<u64, u64, CounterState, ConfigCtx> =
        Causaloid::new(11, stateless_passthrough, "stateless passthrough");

    let initial_state = CounterState { count: 99 };
    let initial_ctx = Some(ConfigCtx { multiplier: 7 });
    let incoming = build_incoming(initial_state.clone(), initial_ctx.clone(), 8);

    let out = causaloid.evaluate_stateful(&incoming);

    assert!(out.error.is_none());
    assert_eq!(
        out.state, initial_state,
        "stateless closure must not perturb caller state"
    );
    assert_eq!(out.context, initial_ctx);
    assert_eq!(out.value, EffectValue::Value(8));
}

#[test]
fn evaluate_stateful_short_circuits_with_state_preserved_on_error() {
    let causaloid: Causaloid<u64, u64, CounterState, ConfigCtx> = Causaloid::new_with_context(
        13,
        stateful_failing,
        ConfigCtx { multiplier: 1 },
        "failing causaloid",
    );

    let initial_state = CounterState { count: 21 };
    let incoming = build_incoming(initial_state.clone(), Some(ConfigCtx { multiplier: 1 }), 4);

    let out = causaloid.evaluate_stateful(&incoming);

    assert!(out.error.is_some());
    assert_eq!(
        out.state, initial_state,
        "state at moment of failure must be preserved (not defaulted)"
    );
    assert!(
        !out.logs.is_empty(),
        "logs accumulated up to and including failing step must be preserved"
    );
}

#[test]
fn stateless_evaluate_unchanged_for_existing_callers() {
    // Regression guard: build the same Causaloid and call the stateless
    // `MonadicCausable::evaluate` — its observable shape (value, error,
    // logs presence) must be unchanged after this change.
    let causaloid: Causaloid<u64, u64, CounterState, ConfigCtx> =
        Causaloid::new(17, stateless_passthrough, "stateless");

    let in_eff: PropagatingEffect<u64> = PropagatingEffect::from_value(123);
    let out_eff = causaloid.evaluate(&in_eff);

    assert!(out_eff.error.is_none());
    assert_eq!(out_eff.value, EffectValue::Value(123));
}

#[test]
fn evaluate_stateful_passes_through_existing_error_unchanged() {
    // Feed an already-errored process into a healthy causaloid. The closure
    // must not run; the original error and state must pass through.
    let causaloid: Causaloid<u64, u64, CounterState, ConfigCtx> = Causaloid::new_with_context(
        29,
        stateful_increment,
        ConfigCtx { multiplier: 4 },
        "should not run",
    );

    let initial_state = CounterState { count: 9 };
    let pre_existing_err =
        CausalityError::new(CausalityErrorEnum::Custom("upstream stage failed".into()));
    let errored_incoming: PropagatingProcess<u64, CounterState, ConfigCtx> = PropagatingProcess {
        value: EffectValue::None,
        state: initial_state.clone(),
        context: Some(ConfigCtx { multiplier: 4 }),
        error: Some(pre_existing_err.clone()),
        logs: EffectLog::new(),
    };

    let out = causaloid.evaluate_stateful(&errored_incoming);

    assert_eq!(
        out.error.as_ref().map(|e| format!("{:?}", e)),
        Some(format!("{:?}", pre_existing_err)),
        "incoming error must pass through unchanged"
    );
    assert_eq!(out.state, initial_state, "state must be preserved");
    // The closure should not have logged anything.
    let log_text = format!("{:?}", out.logs);
    assert!(
        !log_text.contains("Causaloid 29: Incoming"),
        "closure must not have run: {log_text}"
    );
}

#[test]
fn same_causaloid_evaluable_via_both_evaluate_and_evaluate_stateful() {
    // A Causaloid value built once via the existing `new_with_context`
    // can be evaluated either statelessly or statefully — no separate
    // constructor is required.
    let causaloid: Causaloid<u64, u64, CounterState, ConfigCtx> = Causaloid::new_with_context(
        19,
        stateful_increment,
        ConfigCtx { multiplier: 2 },
        "shared",
    );

    // Stateless evaluation: state and context are dropped to () by the
    // existing trait method.
    let in_eff: PropagatingEffect<u64> = PropagatingEffect::from_value(10);
    let out_eff = causaloid.evaluate(&in_eff);
    assert!(out_eff.error.is_none());

    // Stateful evaluation: the same causaloid threads state/context.
    let incoming = build_incoming(
        CounterState { count: 0 },
        Some(ConfigCtx { multiplier: 2 }),
        10,
    );
    let out_proc = causaloid.evaluate_stateful(&incoming);
    assert!(out_proc.error.is_none());
    assert_eq!(out_proc.state, CounterState { count: 1 });
}
