# Recipes behind the README's results map. Every measurement writes into out/ (or $OUT in the
# Docker image); the committed results under benchmarks/results/ are never overwritten.
# Measurements run serially because the analyzer shares SMT files between solver calls.

set shell := ["bash", "-euo", "pipefail", "-c"]
out := env_var_or_default("OUT", "out")

default:
    @just --list

# Build the analyzer and its tests in release mode.
build:
    cargo build --release -p korrekt --lib --tests

# The default test suite (Zcash backend); needs cvc5 on PATH.
test:
    cargo test --release -p korrekt -- --test-threads=1

# One bounded private-value recovery on the invoice example: a quick end-to-end check.
smoke: build
    cargo test --release -p korrekt --lib thesis_invoice_recovery_smoke -- --ignored --nocapture --test-threads=1

# Wall-clock time of the invoice recovery, 10 runs with hyperfine (benchmarks/results/invoice-base-benchmark.json).
bench-invoice runs="10": build
    mkdir -p {{out}}
    hyperfine --runs {{runs}} --export-json {{out}}/invoice-base-benchmark.json \
      'cargo test --release -p korrekt --lib thesis_invoice_recovery_smoke -- --ignored --nocapture --test-threads=1'

# Settlement audit measurement matrix (docs/settlement-audit-demo.md). case: small, chain, wide, unseeded or all.
bench-settlement case="all" repeats="3": build
    mkdir -p {{out}}
    UCP_SETTLEMENT_CASE={{case}} UCP_SETTLEMENT_REPEATS={{repeats}} \
      cargo test --release -p korrekt --lib settlement_audit_measurement_matrix -- --ignored --nocapture --test-threads=1 \
      | tee {{out}}/settlement-measurements.txt

# Private recovery vs. SMT-only reference (docs/private-recovery-measurements.md).
bench-privacy repetitions="5": build
    mkdir -p {{out}}
    PRIVACY_MEASUREMENTS_REPETITIONS={{repetitions}} \
    PRIVACY_MEASUREMENTS_OUTPUT={{absolute_path(out)}}/private-recovery-measurements.tsv \
    HALO2_ANALYZER_SMT_TIMEOUT_MS=2000 \
      cargo test --release -p korrekt --lib privacy_measurement_suite -- --ignored --nocapture --test-threads=1

# Publication hygiene: private links, local paths, paper sources, agent identities and files.
# `just hygiene --strict` also fails on history findings and must pass before a release.
hygiene *flags:
    python3 scripts/hygiene.py {{flags}}

# Every measurement in the results map. `just reproduce --smoke` runs one repetition of each.
reproduce *flags: build
    #!/usr/bin/env bash
    set -euo pipefail
    if [[ "{{flags}}" == *--smoke* ]]; then
      just smoke
      just bench-invoice 1
      just bench-settlement small 1
      just bench-privacy 1
    else
      just bench-invoice
      just bench-settlement
      just bench-privacy
    fi
    echo "regenerated into {{out}}/"
