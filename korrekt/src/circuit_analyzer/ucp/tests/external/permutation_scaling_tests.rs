//! Scalability sweep over LibeccioLabs' Halo2-Playground `PermutationCircuit`.
//! The upstream crate is imported unchanged from a pinned revision; the workspace patch
//! redirects its Halo2 dependency to the analyzer backend. Only the five-element
//! instance appears in the paper's walkthrough; this harness instantiates it for
//! N = 3..=8 under the same Standard configuration as the thesis examples.
//!
//! One run per process: `PERM_SCALING_N` selects the size, the result is printed as a
//! single `PERM_SCALING_RESULT {json}` line and, if `PERM_SCALING_OUT` is set, appended
//! to that file. The solver writes to `src/output/` relative to the working directory,
//! so concurrent runs need distinct working directories.

use crate::{
    circuit_analyzer::{
        analyzable::Analyzable,
        halo2_proofs_libs::*,
        ucp::{
            cell::CellId,
            extractor::extract_ucp_problem,
            privacy::{
                analyze_private_recoverability_with_witness, PrivacyAnalysisProfile,
                PrivacyAnalysisReport, PrivacyAnalysisStatus, PrivateCellReport,
                PrivateCellUniqueness, RecoveryLimitation, RecoveryPhase,
            },
        },
    },
    io::analyzer_io_type::{AnalyzerInput, LookupMethod, VerificationInput, VerificationMethod},
    smt_solver::process::invocation_count,
};
use std::{io::Write, time::Instant};
use zk_halo2_playground::PermutationCircuit;

/// Rows needed by the bubble-sort swap schedule grow quadratically; K = 6 covers N <= 8.
const K: u32 = 6;

/// Witness: inputs (N-1, ..., 0) and the reversal permutation, so the public output is
/// (0, 1, ..., N-1). Every input order is valid under this output; the witness is only
/// the native satisfiability evidence, never a recovery premise.
fn witness<const N: usize>() -> ([Value<Fr>; N], [usize; N]) {
    let items = core::array::from_fn(|i| Value::known(Fr::from((N - 1 - i) as u64)));
    let reversal = core::array::from_fn(|i| N - 1 - i);
    (items, reversal)
}

fn circuit<const N: usize>() -> PermutationCircuit<Fr, N> {
    let (items, permutation) = witness::<N>();
    PermutationCircuit::new_unchecked(items, permutation)
}

fn public_output<const N: usize>() -> Vec<u64> {
    (0..N as u64).collect()
}

fn observations<const N: usize>() -> AnalyzerInput {
    AnalyzerInput {
        verification_method: VerificationMethod::Specific,
        verification_input: VerificationInput::new(
            public_output::<N>()
                .into_iter()
                .enumerate()
                .map(|(row, value)| (format!("I-0-{row}"), value as i64)),
            1,
        ),
        lookup_method: LookupMethod::InlineConstraints,
    }
}

/// The N upstream input cells: the single-row region named "input values".
fn targets<const N: usize>(analyzable: &Analyzable<Fr>) -> Vec<CellId> {
    let regions: Vec<_> = analyzable
        .regions
        .iter()
        .filter(|region| region.name == "input values")
        .collect();
    assert_eq!(regions.len(), 1, "Upstream input layout changed");
    let cells: Vec<_> = regions[0]
        .cells
        .iter()
        .map(|(column, row)| {
            assert_eq!(column.column_type(), &Any::Advice);
            CellId::advice(column.index(), i32::try_from(*row).unwrap())
        })
        .collect();
    assert_eq!(cells.len(), N, "Expected exactly the N upstream inputs");
    cells
}

fn mock_check<const N: usize>() {
    let public = public_output::<N>().into_iter().map(Fr::from).collect();
    zcash_halo2_proofs::dev::MockProver::run(K, &circuit::<N>(), vec![public])
        .unwrap()
        .assert_satisfied();
}

struct Size {
    swaps: usize,
    equations: usize,
    advice_cells: usize,
}

fn size<const N: usize>(analyzable: &Analyzable<Fr>) -> Size {
    let problem = extract_ucp_problem(analyzable);
    Size {
        swaps: N * (N - 1) / 2,
        equations: problem.expressions.len(),
        advice_cells: problem.witness_cells.len(),
    }
}

fn result_line(
    n: usize,
    size: &Size,
    report: &PrivacyAnalysisReport,
    calls: usize,
    ms: f64,
) -> String {
    let (mut smt_phases, mut smt_ms, mut ucp_ms, mut timeouts) = (0, 0.0, 0.0, 0);
    for phase in &report.phases {
        match phase {
            RecoveryPhase::Ucp { elapsed, .. } => ucp_ms += elapsed.as_secs_f64() * 1000.0,
            RecoveryPhase::Smt { elapsed, steps, .. } => {
                smt_phases += 1;
                smt_ms += elapsed.as_secs_f64() * 1000.0;
                timeouts += steps
                    .iter()
                    .filter(|s| matches!(s.limitation, Some(RecoveryLimitation::SolverTimeout)))
                    .count();
            }
        }
    }
    let count =
        |f: &dyn Fn(&PrivateCellReport) -> bool| report.cells.iter().filter(|c| f(c)).count();
    format!(
        concat!(
            "{{\"n\":{},\"swaps\":{},\"equations\":{},\"advice_cells\":{},\"targets\":{},",
            "\"status\":\"{:?}\",\"smt_calls\":{},\"elapsed_ms\":{:.3},\"smt_ms\":{:.3},",
            "\"ucp_ms\":{:.3},\"smt_phases\":{},\"unique\":{},\"non_unique\":{},",
            "\"unknown_uniqueness\":{},\"complete_domains\":{},\"timeout_steps\":{},",
            "\"semantic_queries\":{},\"refinement_checks\":{},\"discovery_checks\":{}}}"
        ),
        n,
        size.swaps,
        size.equations,
        size.advice_cells,
        report.cells.len(),
        report.status,
        calls,
        ms,
        smt_ms,
        ucp_ms,
        smt_phases,
        count(&|c| c.uniqueness == PrivateCellUniqueness::Unique),
        count(&|c| c.uniqueness == PrivateCellUniqueness::NonUnique),
        count(&|c| c.uniqueness == PrivateCellUniqueness::Unknown),
        count(&|c| c.domain_is_complete),
        timeouts,
        report.semantic_queries,
        report.domain_refinement_statistics.solver_checks,
        report.domain_discovery_statistics.solver_checks,
    )
}

fn run<const N: usize>() {
    mock_check::<N>();
    let circuit = circuit::<N>();
    let analyzable = Analyzable::<Fr>::config_and_synthesize(&circuit, K).unwrap();
    let size = size::<N>(&analyzable);
    let targets = targets::<N>(&analyzable);
    let options =
        super::thesis_examples::recovery_options().with_profile(PrivacyAnalysisProfile::Standard);
    let before = invocation_count();
    let started = Instant::now();
    let report = analyze_private_recoverability_with_witness::<Fr, _, _>(
        &circuit,
        K,
        &observations::<N>(),
        targets,
        options,
    )
    .unwrap();
    let ms = started.elapsed().as_secs_f64() * 1000.0;
    let calls = invocation_count() - before;
    assert_eq!(calls, report.solver_invocations);
    assert_ne!(report.status, PrivacyAnalysisStatus::Inconsistent);
    // Every input order is valid, so no input may be reported unique (N >= 2).
    assert!(report
        .cells
        .iter()
        .all(|c| c.uniqueness != PrivateCellUniqueness::Unique));
    let line = result_line(N, &size, &report, calls, ms);
    println!("PERM_SCALING_RESULT {line}");
    if std::env::var("PERM_SCALING_VERBOSE").is_ok_and(|v| v == "1") {
        println!("PERM_SCALING_PUBLIC {:?}", report.public_values);
        println!("{}", report.phase_report_with_names(|_| None));
    }
    if let Ok(path) = std::env::var("PERM_SCALING_OUT") {
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .unwrap();
        writeln!(file, "{line}").unwrap();
    }
}

#[test]
fn permutation_scaling_witnesses_satisfy_mock_prover() {
    mock_check::<3>();
    mock_check::<4>();
    mock_check::<5>();
    mock_check::<6>();
    mock_check::<7>();
    mock_check::<8>();
}

#[test]
#[ignore = "One bounded recovery per process; set PERM_SCALING_N and run with --ignored --nocapture --test-threads=1"]
fn permutation_scaling_run() {
    let n: usize = std::env::var("PERM_SCALING_N")
        .expect("Set PERM_SCALING_N to 3..=8")
        .parse()
        .expect("Invalid PERM_SCALING_N");
    match n {
        3 => run::<3>(),
        4 => run::<4>(),
        5 => run::<5>(),
        6 => run::<6>(),
        7 => run::<7>(),
        8 => run::<8>(),
        _ => panic!("PERM_SCALING_N must be in 3..=8"),
    }
}
