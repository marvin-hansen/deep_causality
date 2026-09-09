/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! A CSS code as an abstraction from its physical qubits to its logical ones.
//!
//! `π` sends every logical qubit to the code's block of physical qubits; `τ` is the ideal recovery
//! and `E` the code-space isometry ([`IdealRecovery`]); the query map sends a logical gate to the
//! program Haruna's Table 1 emits for it. On the exact path the naturality square of a diagonal
//! gate is decided by reading the emitted program back as a [`GaugeFieldGate`] and comparing its
//! phase table with the gate's gauge-field expression `O_k`, and `H̄` by the Clifford tableau; the
//! Paulis are decided by the logical basis. No matrix is formed and the register width does not
//! enter. On the numeric path, for codes of at most ten qubits, the same squares are checked as
//! Choi operators through [`Abstraction::check_naturality`], which is the bridge between the two
//! paths.

use crate::QuantumError;
use crate::types::abstraction::abstraction::Abstraction;
use crate::types::abstraction::ideal_recovery::IdealRecovery;
use crate::types::abstraction::query::Query;
use crate::types::abstraction::type_alignment::{AlignmentSide, TypeAlignment};
use crate::types::circuit_model::{CircuitBox, CircuitModel, QcMorphism, SemanticsPath, WireType};
use crate::types::decision::{Check, CheckItem, CheckReport};
use crate::types::qcode::clifford_action::symplectic_dual_basis;
use crate::types::qcode::css_code::{CssCode, derive_code};
use crate::types::qcode::gauge_field_gate::GaugeFieldGate;
use crate::types::qcode::logical_equivalence::LogicalBasis;
use crate::types::qcode::logical_pauli::LogicalPauli;
use crate::types::qgates::gates_haruna::{
    logical_cz, logical_hadamard, logical_s, logical_t, logical_x, logical_z,
};
use crate::types::qgates::operator_linalg::identity_matrix;
use crate::types::qpu::circuit::GateOp;
use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use deep_causality_algebra::RealField;
use deep_causality_homology::{ChainComplex, Gf2Chain};
use deep_causality_num::{FromPrimitive, Gf2, NaturalNumber};

/// A logical gate of Table 1 on the code's logical qubits, by index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogicalGate {
    /// `Z̄(γᵢ)`.
    Z(usize),
    /// `X̄(γ̃ᵢ)`.
    X(usize),
    /// `S̄(γᵢ)`.
    S(usize),
    /// `T̄(γᵢ)`.
    T(usize),
    /// `H̄(γᵢ)` against the symplectic dual `γ̃ᵢ`.
    H(usize),
    /// `CZ̄(γᵢ, γⱼ)`.
    Cz(usize, usize),
}

impl LogicalGate {
    /// A short name for reports.
    pub fn name(&self) -> String {
        match self {
            Self::Z(i) => format!("Z̄({i})"),
            Self::X(i) => format!("X̄({i})"),
            Self::S(i) => format!("S̄({i})"),
            Self::T(i) => format!("T̄({i})"),
            Self::H(i) => format!("H̄({i})"),
            Self::Cz(i, j) => format!("CZ̄({i}, {j})"),
        }
    }

    /// The logical qubits the gate touches.
    pub fn qubits(&self) -> Vec<usize> {
        match self {
            Self::Z(i) | Self::X(i) | Self::S(i) | Self::T(i) | Self::H(i) => vec![*i],
            Self::Cz(i, j) => vec![*i, *j],
        }
    }
}

/// One gate's exact naturality verdict.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GateNaturality {
    /// The gate.
    pub gate: LogicalGate,
    /// Whether the square commutes.
    pub holds: bool,
    /// The emitted program's length.
    pub program_len: usize,
    /// What went wrong, when it did.
    pub witness: Option<String>,
}

/// The exact naturality check of a code abstraction over its gates.
#[derive(Debug, Clone, PartialEq)]
pub struct ExactNaturality<R> {
    /// One record per gate, `0` against `0` when it holds and `1` when it does not.
    pub report: CheckReport<R>,
    /// The per-gate verdicts with their witnesses.
    pub gates: Vec<GateNaturality>,
    /// Always `Exact`.
    pub path: SemanticsPath,
}

/// A CSS code as an abstraction. See the module documentation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeAbstraction<W> {
    basis: LogicalBasis<W>,
    code: CssCode<W>,
    duals: Vec<Gf2Chain<W>>,
    gates: Vec<LogicalGate>,
}

impl<W: NaturalNumber> CodeAbstraction<W> {
    /// The abstraction of the code a chain complex defines, over the named logical gates.
    ///
    /// # Errors
    ///
    /// [`QuantumError::DimensionMismatch`] if a gate names a logical qubit the code does not have;
    /// the errors of `LogicalBasis::from_complex`, `derive_code` and `symplectic_dual_basis`.
    pub fn new<K: ChainComplex + ?Sized>(
        complex: &K,
        gates: Vec<LogicalGate>,
    ) -> Result<Self, QuantumError> {
        let basis = LogicalBasis::<W>::from_complex(complex, 1)?;
        let code = derive_code::<W, K>(complex)?;
        let duals = symplectic_dual_basis(basis.homology(), basis.cohomology())?;
        let k = basis.num_logical_qubits();
        for g in &gates {
            if let Some(q) = g.qubits().into_iter().find(|&q| q >= k) {
                return Err(QuantumError::DimensionMismatch(format!(
                    "{} names logical qubit {q}, but the code encodes {k}",
                    g.name()
                )));
            }
            if let LogicalGate::Cz(i, j) = g
                && i == j
            {
                return Err(QuantumError::DimensionMismatch(format!(
                    "{} names one logical qubit twice",
                    g.name()
                )));
            }
        }
        Ok(Self {
            basis,
            code,
            duals,
            gates,
        })
    }

    /// Every Table 1 gate on every logical qubit, and `CZ̄` on each pair.
    pub fn table_one(k: usize) -> Vec<LogicalGate> {
        let mut gates = Vec::new();
        for i in 0..k {
            gates.extend([
                LogicalGate::Z(i),
                LogicalGate::X(i),
                LogicalGate::S(i),
                LogicalGate::T(i),
                LogicalGate::H(i),
            ]);
        }
        for i in 0..k {
            for j in i + 1..k {
                gates.push(LogicalGate::Cz(i, j));
            }
        }
        gates
    }

    /// The logical basis.
    pub fn basis(&self) -> &LogicalBasis<W> {
        &self.basis
    }

    /// The code.
    pub fn code(&self) -> &CssCode<W> {
        &self.code
    }

    /// The symplectic duals, one per logical qubit.
    pub fn duals(&self) -> &[Gf2Chain<W>] {
        &self.duals
    }

    /// The gates in the signature.
    pub fn gates(&self) -> &[LogicalGate] {
        &self.gates
    }

    /// The physical program Table 1 emits for a gate.
    ///
    /// # Errors
    ///
    /// The emitters' errors, including `logical_t`'s tuple cap.
    pub fn program<R>(&self, gate: &LogicalGate) -> Result<Vec<GateOp>, QuantumError>
    where
        R: RealField + FromPrimitive,
    {
        let gamma = |i: usize| &self.basis.homology()[i];
        Ok(match gate {
            LogicalGate::Z(i) => logical_z(gamma(*i)),
            LogicalGate::X(i) => logical_x(&self.duals[*i]),
            LogicalGate::S(i) => logical_s(gamma(*i)),
            LogicalGate::T(i) => logical_t(gamma(*i))?,
            LogicalGate::H(i) => logical_hadamard::<W, R>(gamma(*i), &self.duals[*i])?.0,
            LogicalGate::Cz(i, j) => logical_cz(gamma(*i), gamma(*j))?,
        })
    }

    /// The gauge-field expression a diagonal gate should implement, `O_k` on its blocks.
    pub(crate) fn expected_gauge(
        &self,
        gate: &LogicalGate,
    ) -> Option<Result<GaugeFieldGate<W>, QuantumError>> {
        let gamma = |i: usize| self.basis.homology()[i].clone();
        Some(match gate {
            LogicalGate::Z(i) => GaugeFieldGate::z(gamma(*i)),
            LogicalGate::S(i) => GaugeFieldGate::s(gamma(*i)),
            LogicalGate::T(i) => GaugeFieldGate::t(gamma(*i)),
            LogicalGate::Cz(i, j) => GaugeFieldGate::cz(gamma(*i), gamma(*j)),
            LogicalGate::X(_) | LogicalGate::H(_) => return None,
        })
    }

    /// One gate's exact verdict for a given physical program, which need not be the emitted one:
    /// a diagonal gate holds when the program read back as a [`GaugeFieldGate`] equals `O_k` on the
    /// gate's blocks; `X̄` holds when its Pauli lies in the normalizer, is not a stabilizer, and
    /// pairs with the logical `Z̄`s as the dual basis prescribes; `H̄` holds when the tableau swaps
    /// `Z̄(γ)` and `X̄(γ̃)` and fixes the other logical qubits.
    ///
    /// # Errors
    ///
    /// The basis's structural errors; a failing square is a verdict, not an error.
    pub fn check_gate_program(
        &self,
        gate: &LogicalGate,
        program: &[GateOp],
    ) -> Result<GateNaturality, QuantumError> {
        let n = self.basis.len();
        let (holds, witness) = match gate {
            LogicalGate::Z(_) | LogicalGate::S(_) | LogicalGate::T(_) | LogicalGate::Cz(_, _) => {
                let expected = self.expected_gauge(gate).expect("diagonal")?;
                match GaugeFieldGate::from_diagonal_program(n, program, expected.blocks().to_vec())
                {
                    Ok(read) if read.phases() == expected.phases() => (true, None),
                    Ok(read) => (
                        false,
                        Some(format!(
                            "the program's phase table {:?} differs from O_k's {:?}",
                            read.phases(),
                            expected.phases()
                        )),
                    ),
                    Err(e) => (false, Some(format!("{e}"))),
                }
            }
            LogicalGate::X(i) => {
                let zero = Gf2Chain::zeros(n, self.duals[*i].degree());
                let pauli = LogicalPauli::new(self.duals[*i].clone(), zero)?;
                match self.basis.is_logically_trivial(&pauli) {
                    Ok(true) => (false, Some("X̄ is a stabilizer".into())),
                    Ok(false) => {
                        let mut bad = None;
                        for (j, g) in self.basis.homology().iter().enumerate() {
                            let p = g
                                .inner(&self.duals[*i])
                                .map_err(|e| QuantumError::DimensionMismatch(format!("{e}")))?;
                            if p != Gf2::new(j == *i) {
                                bad = Some(format!("⟨γ_{j}, γ̃_{i}⟩ = {p}"));
                                break;
                            }
                        }
                        (bad.is_none(), bad)
                    }
                    Err(e) => (false, Some(format!("{e}"))),
                }
            }
            LogicalGate::H(i) => {
                match self
                    .basis
                    .check_clifford_action_on_qubit(program, *i, &self.duals)
                {
                    Ok(action) => (
                        action.holds,
                        (!action.holds).then(|| {
                            format!(
                                "Z̄ ↦ X̄: {}, X̄ ↦ Z̄: {}, others fixed: {}",
                                action.z_to_x, action.x_to_z, action.others_fixed
                            )
                        }),
                    ),
                    Err(e) => (false, Some(format!("{e}"))),
                }
            }
        };
        Ok(GateNaturality {
            gate: gate.clone(),
            holds,
            program_len: program.len(),
            witness,
        })
    }

    /// The exact naturality check over the signature, each gate against its emitted program.
    ///
    /// # Errors
    ///
    /// The emitters' and the basis's structural errors; a failing square is a record, not an error.
    pub fn check_naturality_exact<R>(&self) -> Result<ExactNaturality<R>, QuantumError>
    where
        R: RealField + FromPrimitive,
    {
        let mut checks = Vec::with_capacity(self.gates.len());
        let mut verdicts = Vec::with_capacity(self.gates.len());
        for (index, gate) in self.gates.iter().enumerate() {
            let program = self.program::<R>(gate)?;
            let verdict = self.check_gate_program(gate, &program)?;
            checks.push(Check::new(
                CheckItem::Index(index),
                if verdict.holds { R::zero() } else { R::one() },
                R::zero(),
            ));
            verdicts.push(verdict);
        }
        Ok(ExactNaturality {
            report: CheckReport::from_checks(checks),
            gates: verdicts,
            path: SemanticsPath::Exact,
        })
    }

    /// The numeric abstractions, one per gate, in the shape of Lorenz & Tull's Example 58: the
    /// low-level model encodes the logical input on its first `k` wires into the code, then runs
    /// the physical program, on `n` qubits; the high-level model runs the logical gate on `k`
    /// qubits. The input types align by the identity and the output types through the ideal
    /// recovery, so the square reads `τ ∘ U_phys ∘ W = U_log`. The query map sends `Io` to `Io`
    /// and the opening of the logical gate, `Open(S̄)`, to the opening of the physical program,
    /// `Open(π(S̄))`, whose fresh inputs align through the recovery. For codes of at most ten
    /// qubits.
    ///
    /// # Errors
    ///
    /// [`IdealRecovery::from_basis`]'s errors and the constructors'.
    #[allow(clippy::type_complexity)]
    pub fn numeric_abstractions<R>(
        &self,
    ) -> Result<
        Vec<(
            LogicalGate,
            Abstraction<R, CircuitModel<R>, CircuitModel<R>>,
        )>,
        QuantumError,
    >
    where
        R: RealField + FromPrimitive + Default + core::fmt::Debug,
    {
        let recovery = IdealRecovery::<R>::from_basis(&self.basis)?;
        let n = self.basis.len();
        let k = self.basis.num_logical_qubits();
        let all_low: Vec<usize> = (0..n).collect();
        let all_high: Vec<usize> = (0..k).collect();
        let identity = QcMorphism::from_kraus(&[identity_matrix::<R>(1 << k)])?;
        let mut out = Vec::with_capacity(self.gates.len());
        for gate in &self.gates {
            let program = self.program::<R>(gate)?;
            let low = CircuitModel::ungrouped(
                vec![WireType::qubit(); n],
                vec![
                    CircuitBox::Kraus {
                        wires: all_low.clone(),
                        kraus: recovery.encoder().kraus(),
                    },
                    CircuitBox::Unitary {
                        wires: all_low.clone(),
                        program,
                    },
                ],
                all_high.clone(),
                all_low.clone(),
            )?;
            let logical = match gate {
                LogicalGate::Z(i) => vec![GateOp::Z(*i)],
                LogicalGate::X(i) => vec![GateOp::X(*i)],
                LogicalGate::S(i) => vec![GateOp::S(*i)],
                LogicalGate::T(i) => vec![GateOp::T(*i)],
                LogicalGate::H(i) => vec![GateOp::H(*i)],
                LogicalGate::Cz(i, j) => vec![GateOp::Cz {
                    control: *i,
                    target: *j,
                }],
            };
            let high = CircuitModel::ungrouped(
                vec![WireType::qubit(); k],
                vec![CircuitBox::Unitary {
                    wires: all_high.clone(),
                    program: logical,
                }],
                all_high.clone(),
                all_high.clone(),
            )?;
            let alignment = TypeAlignment::new_sided(vec![
                (
                    AlignmentSide::Input,
                    (
                        all_high.clone(),
                        all_high.clone(),
                        identity.clone(),
                        identity.clone(),
                    ),
                ),
                (
                    AlignmentSide::Output,
                    (
                        all_high.clone(),
                        all_low.clone(),
                        recovery.recovery().clone(),
                        recovery.isometry().clone(),
                    ),
                ),
            ])?;
            out.push((
                gate.clone(),
                Abstraction::new(
                    low,
                    high,
                    alignment,
                    vec![
                        (Query::Io, Query::Io),
                        (Query::Open(vec![0]), Query::Open(vec![1])),
                    ],
                )?,
            ));
        }
        Ok(out)
    }
}
