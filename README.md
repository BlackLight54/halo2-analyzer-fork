# Artifact for: Overconstrainedness

[![DOI](https://zenodo.org/badge/DOI/10.5281/zenodo.TODO.svg)](https://doi.org/10.5281/zenodo.TODO)

Code and measurement scripts for the TDK paper *Overconstrainedness* (Gábor Koncz, BME VIK, 2026).
It extends the Halo2 analyzer Korrekt to measure how far the public data of a Halo2 circuit narrows
down its private inputs, and to recover those inputs where it can.

## Reproduce everything

```sh
docker build -t halo2-analyzer-fork .
docker run --rm -v "$PWD/out:/out" halo2-analyzer-fork reproduce
```

reruns every measurement in the results map into `out/`. TODO once the release exists: replace the
build line with the published image, `ghcr.io/gaborkoncz73/halo2-analyzer-fork:v1.0.0`.

With the repository cloned and the dependencies below installed, `just reproduce` does the same,
and `just reproduce --smoke` runs one repetition of each measurement. `just --list` names every
recipe; any of them also works as the `docker run` command.

Dependencies, pinned in the `Dockerfile`: Rust `nightly-2026-08-28`, crate versions from the
committed `Cargo.lock`, cvc5 built with finite-field support from commit
[`1e05d08b9`](https://github.com/cvc5/cvc5/commit/1e05d08b9f73281473dd06285bcafbeca0ae1b9f) (the version
the measurements in `docs/` report), hyperfine and just.

## Results map

No figure or table of the paper uses these measurements yet; the first column is filled in when the
evaluation chapter is written.

| Paper | Recipe | Output | Committed result | Time |
|---|---|---|---|---|
| TODO | `just bench-invoice` | `out/invoice-base-benchmark.json` | [`benchmarks/results/invoice-base-benchmark.json`](benchmarks/results/invoice-base-benchmark.json): 10 runs, mean 1.76 s | about 20 s |
| TODO | `just bench-settlement` | `out/settlement-measurements.txt` | [`benchmarks/results/settlement-benchmark.json`](benchmarks/results/settlement-benchmark.json) is empty | TODO |
| TODO | `just bench-privacy` | `out/private-recovery-measurements.tsv`, `.summary.tsv`, `.incomplete.tsv` | discussed in [`docs/private-recovery-measurements.md`](docs/private-recovery-measurements.md) | TODO |

Hardware: TODO (cores, memory, OS) of the machine that produced the committed results.

`just reproduce --smoke` took 225 s in the Docker image on a 32-core x86_64 Linux machine with
93 GB of memory, almost all of it in `bench-settlement small 1` (203 s). The full run takes longer;
its time is TODO.

## What does not reproduce

- The SMT checks run under per-call wall-clock limits (`HALO2_ANALYZER_SMT_TIMEOUT_MS`, 2000 ms in
  `bench-privacy`). On a slower machine a check that finished in time can report `unknown`, so
  recovery counts as well as times can differ. The harnesses report such cases as incomplete
  instead of dropping them.
- Times depend on the machine; only the relative comparisons within one run are meaningful.
- The invocation counts in [`docs/measurements/`](docs/measurements/) come from a harness at
  baseline commit `a1c4976` that is not part of this tree; no recipe regenerates them.
- Other Halo2 backends (PSE, Axiom, Scroll) were not measured; the recipes use the default Zcash backend.

## Results tag

The numbers in the paper come from tag `v1.0.0`, Zenodo version TODO. Later versions change the
packaging, never the results.

## Upstream and what this fork adds

This repository is a fork of [quantstamp/halo2-analyzer](https://github.com/quantstamp/halo2-analyzer)
(Korrekt), Quantstamp's analyzer with contributions by Fatemeh Heidari, Jeffrey Kam, Jan Gorzny,
mr-ma, Mohammad Jahanara and sorawee. It branches at upstream commit
[`c201ed3`](https://github.com/quantstamp/halo2-analyzer/commit/c201ed37b9daeab1c2289b57da13d097c009908f)
(2026-01-28); everything after it is Gábor Koncz's work:

- `korrekt/src/circuit_analyzer/ucp/`: uniqueness constraint propagation (UCP) over an expression-tree view
  of the circuit, value and domain inference, bounded private-value recovery with SMT refinement,
  domain discovery and independent validation, the paper's invoice and settlement examples
  (`thesis_examples/`), and the measurement harnesses.
- `korrekt/src/smt_solver/process.rs`: cvc5 runs as a supervised process with a wall-clock limit.
- `korrekt/src/sample_circuits/zcash/simple/decoder_circuits.rs`: underconstrained decoder examples.
- `docs/`: one note per feature, most of them in Hungarian, linked from
  [Private Value Recovery In This Fork](#private-value-recovery-in-this-fork) below.
- Test fixtures copied from `zcash/halo2` keep their own MIT notice in
  `korrekt/src/circuit_analyzer/ucp/tests/external/fixtures/`.

The rest of this README is the upstream documentation of the analyzer, with the fork's additions
at its end.

## Cite

TODO BibTeX of the paper and of this artifact (the concept DOI). `CITATION.cff` and `.zenodo.json`
carry the artifact metadata. Please also credit the upstream analyzer.

## License

MIT, see [`LICENSE`](LICENSE): the upstream Quantstamp copyright plus Gábor Koncz's for the changes.
The Docker image builds cvc5 with CoCoA, which makes that cvc5 binary GPL-licensed; the image is
for reproduction and is not part of the MIT-licensed source.

---

# Korrekt, the upstream analyzer

## Prerequisites

Before you begin, ensure you have installed CVC5 for finite fields. The `Dockerfile` in this repository builds it on Ubuntu; the steps below are upstream's macOS notes.

## Installing CVC5 FOR Finite Field

These are the steps to install CVC5 on MacOS (probably transferable to Linux).
Instructions taken from: <https://github.com/cvc5/cvc5/blob/main/INSTALL.rst>

1. Clone CVC5 from official repo

```
git clone https://github.com/cvc5/cvc5
```

You can also download the source code from <https://github.com/cvc5/cvc5/releases/tag/cvc5-1.0.5> but using the repo is recommended.
2. Install all dependencies and set proper ENV variables. See [dependencies](#dependencies) section below
3. Run

```
./configure.sh —cocoa —auto-download
```

4. The do:

```
cd build
make -j4
make check
sudo make install
```

## Dependencies

- Python 3.9 (you might need to install pip or update it):

```
brew install python@3.9
python3.9 -m pip install toml 
python3.9 -m pip install pyparsing
```

- Java. Go to the following link and install the latest Java
<https://www.oracle.com/java/technologies/downloads/>

- Necessary for cocoa

```
brew install gmp
brew install libpgm
```

- Now that you have `gmp` installed you need to tell CVC5 where to find it. The easiest way is through an env variable. If you install gap via homebrew, the path will look something like the following

```
export GMP_LIB=/opt/homebrew/Cellar/gmp/6.2.1_1/lib/libgmp.a
```

- If you’re having trouble finding it you can search for it with:

```
find /usr /opt -name "libgmp.a"
```

## Feature Flags and Halo2 Versions

This project leverages feature flags to enable support of different versions of the Halo2. Each feature flag corresponds to a specific version of Halo2, allowing for flexible and targeted compilation.

### Using Feature Flags

Feature flags can be managed directly via the `Cargo.toml` file or through command line options when building or testing.

#### Default Feature Flags

- **Default Configuration:**
  By default, the `use_zcash_halo2_proofs` feature is enabled in the `Cargo.toml`.

- **Changing the Default Feature Flag:**
  There are two ways for changing the default feature flag related to the Halo2 version:

    1. **Modifying the `Cargo.toml` File:**
       To change the default feature flag used in builds and tests, you can modify the `Cargo.toml` file. Update the `[features]` section by replacing `use_zcash_halo2_proofs` with the feature flag of your choice. This will set a new default feature for the project, affecting all builds and tests unless overridden in the command line.

    2. **Disabling Default Features via Command Line:**
       You can disable default features in command line or in cargo toml.
       For disabling the default in command line and specifying your desired flag:

       ```bash
       cargo build --no-default-features --features your_custom_feature
       cargo test --no-default-features --features your_custom_feature
       ```

       By removing the default flag from `crgo.toml` the commands will be:

       ```bash
       cargo build --features your_custom_feature
       cargo test --features your_custom_feature
       ```

        Replace `your_custom_feature` with the appropriate feature flag. This ensures that only the specified feature is active during the build or test process.

### Managing Halo2 Patches

For certain versions, multiple patches may be available. It's important to ensure that only the relevant patch is enabled to avoid conflicts. This is managed by commenting out or uncommenting the appropriate lines in the Cargo.toml file under the [patch] section.

## How to run

1. Go to "korrekt"

2. Run `cargo run` for the default feature flag (`use_zcash_halo2_proofs`).

    ```bash
    cargo run -- [OPTIONS]
    ```

    For other versions of halo2 run `cargo run` with relevant halo2 version feature flag:

    ```bash
    cargo run --no-default-features --features use_pse_halo2_proofs -- [OPTIONS]
    cargo run --no-default-features --features use_axiom_halo2_proofs -- [OPTIONS]
    cargo run --no-default-features --features use_scroll_halo2_proofs -- [OPTIONS]
    cargo run --no-default-features --features use_pse_v1_halo2_proofs -- [OPTIONS]
    ```

Replace `[OPTIONS]` with the desired options as described below.

### Options

- `-p, --profile <PROFILE>`: Select a predefined configuration profile. Available profiles include:
  - `specific_inline`
  - `random_inline`
  - `random_uninterpreted`
  - `random_interpreted`

- `-t, --type <TYPE>`: Specify the type of analysis to be performed. Available types include:
  - `unused_gates` (`ug`)
  - `unused_columns` (`uc`)
  - `unconstrained_cells` (`ucc`)
  - `underconstrained_circuit` (`undcc`)

- `-l, --lookup <METHOD>`: Set the lookup method. This option is required for the `underconstrained_circuit` type. Available methods include:
  - `uninterpreted` (`u`)
  - `interpreted` (`i`)
  - `inline` (`in`)

- `-v, --verification <METHOD>`: Specify the verification method. This option is required for the `underconstrained_circuit` type. Available methods include:
  - `specific` (`s`)
  - `random` (`r`)

- `-i, --iterations <COUNT>`: Specify the number of iterations for random input verification. This option is only needed if the verification method is set to `random`.

## How to test

1. Go to "korrekt"

2. Run `cargo test` for the default feature flag (`use_zcash_halo2_proofs`).

    ```bash
    cargo test -- --test-threads=1
    ```

   For other versions of halo2 run `cargo test` with relevant halo2 version feature flag:

    ```bash
    cargo test --no-default-features --features use_pse_halo2_proofs -- --test-threads=1
    cargo test --no-default-features --features use_axiom_halo2_proofs -- --test-threads=1
    cargo test --no-default-features --features use_scroll_halo2_proofs -- --test-threads=1
    cargo test --no-default-features --features use_pse_v1_halo2_proofs -- --test-threads=1
    ```

## For linting

1. Go to "korrekt"
2. `cargo clippy --all-targets --all-features -- -D warnings`

## For Formatting

1. Go to "korrekt"
2. `cargo fmt --`

## For Benchmark

Part of our benchmarking involves comparing the performance of analyzer version 2 with version 1. To enable this comparison, version 2 supports the exact version of Halo2 that version 1 supports, activated by the `use_pse_v1_halo2_proofs` feature flag.

### Setup Instructions

To run the benchmarks successfully, follow these steps:

1. **Update Dependency Features:** Open the `Cargo.toml` file in the `benchmarks` directory, and update the `features` for `korrekt_V2`:

    ```bash
    korrekt_V2 = { path = "../korrekt", package = "korrekt", features = ["use_pse_v1_halo2_proofs"] }
    ```

2. **Remove Default Feature:** Ensure that there are no default feature flag set for halo2 dependency in the `Cargo.toml` file of the `korrekt`.

    ```bash
    [features]
    default = []
    use_zcash_halo2_proofs = ["zcash_halo2_proofs"]
    use_pse_halo2_proofs = ["pse_halo2_proofs"]
    use_axiom_halo2_proofs = ["axiom_halo2_proofs"]
    use_scroll_halo2_proofs = ["scroll_halo2_proofs"]
    use_pse_v1_halo2_proofs = ["pse_v1_halo2_proofs"]
    ```

3. **Activate Relevant Patch:** In the `Cargo.toml` file of the `korrekt`, ensure the relevant patch is active.

    ```bash
    [patch."https://github.com/privacy-scaling-explorations/halo2.git"]
    pse_v1_halo2_proofs = { git = "https://github.com/Analyzable-Halo2/v1-halo2.git", package = "halo2_proofs"}
    ```

### Running Benchmarks

Once you've made these updates, navigate to the benchmarks directory and for all available benchmarks run:

```bash
cargo bench
```

Multiple benchmarks are available:

- benchmark_bit_decomp
- benchmark_fibo_v1
- benchmark_fibo_v2

For running a specific benchmark run:

```bash
cargo bench --bench NAME_OF_BENCHMARK
```

Replace `NAME_OF_BENCHMARK` with one of the available benchmarks.

## How it works

### Intro to Halo2

When constructing a Halo2 relation, it consists of:

- Gates: a "small" number of (parameterized) constraints between cells.
- Regions: a logical collection of cells with a set of enabled gates (as indicated by the "selectors").

More on this below.

#### Halo2 "Gates"

Calling the Halo2 "Gates" is kind of a misnomer, for multiple reasons:

##### They are very powerful

Halo2 "gates" are very powerful: they can enforce any number of arbitrary non-deterministic relations.
For instance, the following can be enforced by a single Halo2 gate:

```
a * b * c != 0
```

Using the polynomial:

```
f(a,b,c,d) := (a * b * c) * d - 1
```

Which has an assignment for `d` st. `f(a,b,c,d) = 0` iff `a * b * c != 0`.
Note that the "gate" above has no "output wires".
Note that `d` is not really "an input": it is part of the witness, but in the example above, it cannot be the output of another gate or part of the statement: it's a "gate hint".

##### The circuit topology is restricted

When describing a "circuit" you can usually wire any output to any input.
This is also possible in Halo2, indeed, the original PlonK paper describes a "universal gate" and a permutation check which enables this (in Halo2 this corresponds to "enabling equality" on the 3 input/output columns).
However, often in Plonkish arithmetization the layout is intentionally restricted: allowing some subset of the wires (columns) to have arbitrary wiring
and restricting the rest to accessing other cells defined only locally.
This is done for efficiency.
An example of when this is useful is the repeated application of a function:

```
y = f(f(f(x))
```

Which can be described as:

```
z1 = f(x)
z2 = f(z1)
y  = f(z2)
```

Note that every application of "f" only accesses the output from the previous application.
Defining a "gate" implementing "f" taking as "input" the "output register" of the previous row,
enables computing repeated applications of "f" without wiring checks.
When "f" is, for instance, the round function of a hash function, a single application of "f" is never used, instead large numbers of consecutive rows will implement the full call to the compression function.

#### How to mess it up

As you can probably see there is ample opportunity to screw this up, e.g.

- Accidentally choosing the wrong offset when defining/using a gate.
- Forgetting to enable a selector, or enabling the wrong selector.

These pit-falls will lead to soundness (not completeness) issues: the proof will still generate and will still be valid.
This problem is much more pronounced with Halo than previous systems e.g. bellman, since witness generation (completeness) is completely independent of constraints (soundness).

### Extracting a description of the relation from Halo2

As outlined above, Halo2 first defines a number of "gates" (which are enabled by selectors), each such "gate" consists of one/more polynomials which can be enforced over regions. Korrekt works by collecting all the gates and regions using a custom implementation (`AnalyticLayouter` of the `Layouter` trait, which normally is responsible for "place-and-route" (similar to the role of PnR in HDLs is responsible for mapping the regions to rows to minimize the number of required rows to enforce the constraints: essentially it solves a packing problem).

In our case, the `Layouter` will simply record the details of every region.

Since we never need to actually generate a proof the actual PnR is not needed.

### Abstractly Evaluating Gates across Regions

At this point we have for every region:

- The set of enabled selectors.
- The assigned cells (but with dummy values)

And the gates:

- A set of polynomials for each gate.

If the "circuit" is satisfied, then every polynomial of every gate should vanish over every region: for gates which are not "enabled" the polynomial is `0 * g(<witness>)`, since the "selector" variable is set to 0, i.e. the gate polynomial is:

```
f(<witness>, selector) = selector * g(<witness>)
```

However, since we do not have a witness we can't evaluate the gate polynomials (also, doing so would always yield 0).

What we do have is the selectors and constants which enables us to *abstractly evaluate* every polynomial of every gate in every region for *concrete* values of selector and constant free variables.

An example is in order:

Suppose we have the following polynomial:

```
f(s,a,b,C) := s * (C * a - b)
```

The variables have the following interpretations:

- `s` is the "selector" (determining if the "gate" is enabled), if it is 0 then the polynomial is always 0.
- `a` is the "input" of the gate.
- `b` is the "output" of the gate.
- `C` is a parameter used to configure the gate: a constant in the circuit.

The "gate" "computes" the following relation: `b = C * a` i.e. it is a "multiplication by constant" gate.

After running synthesis with the `AnalyticalLayouer` we have assignments for `s` and `C` (but not for `a` and `b`),
as such we cannot fully evaluate `f` but we can determine if it is identically zero, i.e. zero for *every* `a` and `b`.
We could so so by enumerating assignments for `a` and `b`, which would be slow and accurate, by using a SAT solver, which would be much faster and accurate, or as presently done using abstract interpretation, which is blazingly fast at the cost of accuracy i.e. may yields *false negatives*:
may return that `f` is not identically zero, when in fact it is.

We introduce the following type:

```rust
pub enum AbsResult {
    Variable,
    NonZero,
    Zero,
}
```

Which represents a polynomial which is either:

1. Something (probably depending on the witness).
2. Definitely *not zero* (for any witness).
3. Definitely *zero* (for any witness).

To see how to generate this consider the simples possible case of a constant:

```rust
pub fn eval_abstract<F: FieldExt>(expr: &Expression<F>, selectors: &HashSet<Selector>) -> AbsResult {
    match expr {
        Expression::Constant(v) => if v.is_zero().into() { AbsResult::Zero } else {  AbsResult::NonZero },
```

The function of the code is pretty self-explanatory. We can do a similar thing for the selectors:

```rust
Expression::Selector(selector) => {
    match selectors.contains(selector) {
        true => AbsResult::NonZero,
        false => AbsResult::Zero
    }
}
```

However for the "advice" (wire assignments), we have to throw up our hands and output "Variable" (meaning it could be anything):

```rust
Expression::Advice { .. } => AbsResult::Variable,
```

We can also calculate with these, e.g. a negation does not affect the zero/non-zero status:

```rust
Expression::Negated(expr)  => {
    let res = eval_abstract(expr, selectors);
    return res
}
```

While a sum operates as follows:

```rust
Expression::Sum(left, right) => {
    let res1 = eval_abstract(left, selectors);
    let res2 = eval_abstract(right, selectors);
    match (res1, res2) {
        (AbsResult::Variable, _) => AbsResult::Variable, // could be anything
        (_, AbsResult::Variable) => AbsResult::Variable, // could be anything
        (AbsResult::NonZero, AbsResult::NonZero) => AbsResult::Variable, // could be zero or non-zero
        (AbsResult::Zero, AbsResult::Zero) => AbsResult::Zero,
        (AbsResult::Zero, AbsResult::NonZero) => AbsResult::NonZero,
        (AbsResult::NonZero, AbsResult::Zero) => AbsResult::NonZero,
    }
}
```

And a multiplication operates as follows:

```rust
Expression::Product(left, right) => {
    let res1 = eval_abstract(left, selectors);
    let res2 = eval_abstract(right, selectors);
    match (res1, res2) {
        (AbsResult::Zero, _) => AbsResult::Zero,
        (_, AbsResult::Zero) => AbsResult::Zero,
        (AbsResult::NonZero, AbsResult::NonZero) => AbsResult::NonZero,
        _ => AbsResult::Variable,
    }
}
 ```

After this process, we have a result indicating whether a polynomial is: definitely zero (trivially satisfied), definitely non-zero (impossible to satisfy) or variable (depending on the witness).
With this simple tool we then proceed to implement a bunch of checks for individual regions described next.

### Checks

#### Unused Gate

Check that for every gate there exists a region in which it is not identically zero.

#### Unconstrained Cell

Check that for every assigned cell in the region, it occurs in a polynomial which is not identically zero over this region.
This means that part of the witness is not constrained -- this is almost certainly a bug.

#### Unused Column

Check that every column occurs in some polynomial.

### Private Value Recovery In This Fork

The UCP module can analyze explicitly selected private advice cells under concrete
public observations. An optional, fresh SMT validation pass checks recovered exact
values and finite candidate supersets without assuming inferred facts. See
[Private Recovery Validation](docs/private-recovery-validation.md) for the API,
checked property, demos, and limitations.

Repeated same-target comparisons with an SMT-only reference, separate validation
timings, and TSV exports are described in
[Private Recovery Measurements](docs/private-recovery-measurements.md).

[Optional Recovery Profile Measurements](docs/private-recovery-profile-measurements.md)
compare baseline, finite refinement, full discovery, and targeted discovery with
separate validation/public-observation control timings, including partial attempts
and per-cell evidence.

[Compositional Masking Demo](docs/compositional-masking-demo.md) shows how two
public observations reveal a secret when a linear mask is reused, with
single-publication and independent-mask controls. Demos print compact summaries
by default; set `UCP_DEMO_VERBOSE=1` for full reports.

[External Halo2 Book Evaluation](docs/external-halo2-book-evaluation.md) analyzes
an unchanged, pinned upstream circuit and a separately identified local control
that exposes additional intermediate values.

[Bounded Private Recovery](docs/bounded-private-recovery.md) documents explicit
solver-check and supervised process budgets, per-cell recovery categories, and structured reasons
for incomplete or unsupported analyses.

[Adaptive Private Recovery](docs/adaptive-private-recovery.md) reserves first
target attempts, bounds bridge/retry work, and renews the stagnation budget on
actual progress without resetting total query counts.

[Finite-Domain SMT Refinement](docs/finite-domain-refinement.md) optionally checks
small candidate sets against an immutable original model, distinguishes partial
supersets from complete feasible projections, and feeds refinements back into UCP.

[SMT Domain Discovery](docs/smt-domain-discovery.md) optionally enumerates private
cell values without a pre-existing UCP candidate set. Only proven exhaustion
certifies a complete projection; partial samples never become recovered domains.
Fair check rounds reuse UCP priority and preserve samples across continuation;
explicit larger-timeout retries never reset the hard total allowance.

[Targeted SMT Recovery](docs/targeted-smt-recovery.md) optionally proves discovery
exhaustion from small literal original subsets. Feasible values still come from
the full original circuit; subset SAT/unknown requires a separately counted full fallback.

[Public Observation Control](docs/public-observation-control.md) independently
checks whether a reported domain requires the concrete public values or already
follows from the original circuit alone. It separates publication-dependent
narrowing from stronger SMT reasoning that simply surpassed the UCP baseline.

[Full-Width Private Recovery](docs/full-field-recovery.md) demonstrates 254-bit
exact values, signed field representatives, and propagation without sampling
private witness assignments. Public observations now accept BigInt through
`VerificationInput::new`; domain cardinality and raw-input/value bit limits remain explicit.

[Imported Range-Gadget Evaluation](docs/imported-range-gadget-evaluation.md)
compares polynomial and lookup-based running sums imported from `halo2_gadgets`:
an aggregate can expose both bounded private inputs at a range boundary.

[Relational Delta Propagation](docs/relational-delta-propagation.md) explains
affine Root inference, retained lookup memberships, and fixed-constant seeding:
range domains can be recovered before asking SMT about individual private cells.

[Selective SMT Domain Export](docs/selective-smt-domain-export.md) keeps all
reported domains while avoiding redundant structural finite-set assertions in
privacy queries; repeated measurements document the lookup regression fix.

[Soundness Regression Checks](docs/soundness-hardening.md) documents modular
assignment, zero-factor handling, selector-independent constraint coverage and
the field/Halo2 counterexamples used to test them.

[SAT Hint Acceleration](docs/sat-hint-acceleration.md) explains the scoped,
positive-only initial satisfiability shortcut and the permutation-copy fix.

[Original-Subset Domain Validation](docs/subset-domain-validation.md) proves
exclusion using literal original constraints, with full-SMT fallback and separate
nonvacuity checking; the Poseidon publication control now passes fresh validation.

[External Poseidon Evaluation](docs/external-poseidon-evaluation.md) adds the full
upstream hash circuit, a local publication control, and bounded incomplete outcomes.
It also documents successful control recovery, local linear-subsystem inference,
and the remaining original-SAT bottleneck in the SMT-only performance reference.
