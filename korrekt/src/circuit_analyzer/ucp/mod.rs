//! Beköti az UCP működési moduljait és a feltételesen forduló tesztmodulokat.

pub mod all_but_one;
pub mod base_conv;
pub mod bigint_mul;
pub mod cell;
pub mod choose_var;
pub mod engine;
pub mod expr;
pub mod extractor;
pub mod facts;
pub mod lookup;
pub mod pipeline;
pub mod plonkish;
pub mod profiling;
pub mod privacy;
pub mod privacy_discovery;
pub mod privacy_domain;
pub mod privacy_observation;
pub(crate) mod privacy_search;
pub(crate) mod privacy_subset;
pub mod privacy_validation;
pub mod rules;
pub mod value;

#[cfg(test)]
#[path = "tests/demos/demo_tests.rs"]
mod demo_tests;

#[cfg(all(test, feature = "use_zcash_halo2_proofs"))]
#[path = "tests/demos/settlement_audit_demo_tests.rs"]
mod settlement_audit_demo_tests;

#[cfg(all(test, feature = "use_zcash_halo2_proofs"))]
mod thesis_examples;

#[cfg(test)]
#[path = "tests/relational_propagation_tests.rs"]
mod relational_propagation_tests;

#[cfg(all(test, feature = "use_zcash_halo2_proofs"))]
#[path = "tests/measurements/pipeline_comparison_tests.rs"]
mod pipeline_comparison_tests;

#[cfg(all(test, feature = "use_zcash_halo2_proofs"))]
#[path = "tests/demos/sample_circuit_demo_tests.rs"]
mod sample_circuit_demo_tests;

#[cfg(all(test, feature = "use_zcash_halo2_proofs"))]
#[path = "tests/measurements/privacy_measurement_tests.rs"]
mod privacy_measurement_tests;

#[cfg(all(test, feature = "use_zcash_halo2_proofs"))]
#[path = "tests/measurements/privacy_profile_measurement_tests.rs"]
mod privacy_profile_measurement_tests;

#[cfg(all(test, feature = "use_zcash_halo2_proofs"))]
#[path = "tests/privacy_domain_tests.rs"]
mod privacy_domain_tests;

#[cfg(all(test, feature = "use_zcash_halo2_proofs"))]
#[path = "tests/measurements/privacy_search_measurement_tests.rs"]
mod privacy_search_measurement_tests;

#[cfg(all(test, feature = "use_zcash_halo2_proofs"))]
#[path = "tests/privacy_discovery_tests.rs"]
mod privacy_discovery_tests;

#[cfg(all(test, feature = "use_zcash_halo2_proofs"))]
#[path = "tests/privacy_observation_tests.rs"]
mod privacy_observation_tests;

#[cfg(all(test, feature = "use_zcash_halo2_proofs"))]
#[path = "tests/external/external_circuit_tests.rs"]
mod external_circuit_tests;

#[cfg(all(test, feature = "use_zcash_halo2_proofs"))]
#[path = "tests/external/external_range_tests.rs"]
mod external_range_tests;

#[cfg(all(test, feature = "use_zcash_halo2_proofs"))]
#[path = "tests/soundness_regression_tests.rs"]
mod soundness_regression_tests;

#[cfg(all(test, feature = "use_zcash_halo2_proofs"))]
#[path = "tests/wide_value_tests.rs"]
mod wide_value_tests;

#[cfg(all(test, feature = "use_zcash_halo2_proofs"))]
#[path = "tests/external/external_poseidon_tests.rs"]
mod external_poseidon_tests;

#[cfg(all(test, feature = "use_zcash_halo2_proofs"))]
#[path = "tests/external/permutation_scaling_tests.rs"]
mod permutation_scaling_tests;
