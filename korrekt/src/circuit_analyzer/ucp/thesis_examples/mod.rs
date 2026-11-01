//! A dolgozat összefüggő példáit és azok külön referencia-ellenőrzését tartalmazza.
//! A példák kizárólag tesztfordításkor kapcsolódnak az elemzőhöz.

mod invoice_approval_tests;
mod invoice_circuit;
mod invoice_reference;
mod invoice_tests;

use super::privacy::PrivacyAnalysisOptions;

// A számla minden változatának ugyanazt a teljes keresési konfigurációt adja, a circuit vizsgálata nélkül.
pub(super) fn recovery_options() -> PrivacyAnalysisOptions {
    fn env_value<T: std::str::FromStr>(name: &str, fallback: T) -> T {
        std::env::var(name)
            .map(|value| value.parse().unwrap_or_else(|_| panic!("Invalid {name}")))
            .unwrap_or(fallback)
    }
    let mut options = PrivacyAnalysisOptions::default();
    options.solver_timeout_ms = Some(env_value("UCP_INVOICE_TIMEOUT_MS", 3_000));
    options.initial_sat_timeout_ms = Some(env_value("UCP_INVOICE_INITIAL_SAT_TIMEOUT_MS", 60_000));
    options.max_semantic_queries =
        env_value("UCP_INVOICE_SEMANTIC_QUERIES", options.max_semantic_queries);
    options.domain_refinement.max_solver_checks = env_value(
        "UCP_INVOICE_REFINEMENT_CHECKS",
        options.domain_refinement.max_solver_checks,
    );
    options.domain_discovery.max_solver_checks = env_value(
        "UCP_INVOICE_DISCOVERY_CHECKS",
        options.domain_discovery.max_solver_checks,
    );
    options.domain_discovery.max_values_per_cell = env_value(
        "UCP_INVOICE_DISCOVERY_VALUES",
        options.domain_discovery.max_values_per_cell,
    );
    // Az összehasonlítható mérésekben az újrapróbálkozás sem kap rejtett hosszabb időkorlátot.
    options.search.retry_solver_timeout_ms = None;
    assert!(options.solver_timeout_ms.unwrap() > 0);
    assert!(options.initial_sat_timeout_ms.unwrap() > 0);
    assert!(options.max_semantic_queries > 0);
    assert!(options.domain_refinement.max_solver_checks > 0);
    assert!(options.domain_discovery.max_solver_checks > 0);
    options
}

#[test]
fn thesis_examples_share_enabled_standard_search_budgets() {
    let options = recovery_options();
    assert!(options.max_semantic_queries > 0);
    assert!(options.domain_refinement.max_solver_checks > 0);
    assert!(options.domain_discovery.max_solver_checks > 0);
}
