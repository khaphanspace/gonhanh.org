//! Single integration-test binary: one link step instead of 29.
//! Run one suite with: cargo test --test suite <module>::

mod auto_capitalize_test;
mod auto_restore_dynamic_test;
mod bug_reports_test;
mod checked_tone_stop_final_test;
mod common;
mod compose_diff;
mod dictionary_sources;
mod disabled_shortcut_test;
mod dynamic_test;
#[cfg(feature = "engine_v2")]
mod engine_fuzz;
mod engine_test;
mod english_100k_test;
mod english_auto_restore_test;
mod english_auto_restore_toggle_test;
mod english_telex_patterns_test;
mod english_words_test;
mod foreign_consonants_test;
mod golden_digest;
mod integration_test;
mod issue_387_backspace_correction_test;
#[cfg(feature = "engine_v2")]
mod issue_regressions;
mod issue_uo_horn_test;
mod issue_w_path_test;
mod paragraph_test;
mod permutation_test;
mod phonology_audit;
mod restore_toggle_after_backspace_test;
mod revert_auto_restore_test;
mod test_lissa_larra;
mod typing_order_consistency_test;
mod typing_order_permutation_test;
mod typing_test;
mod unit_test;
mod vietnamese_22k_test;
mod vietnamese_dict_test;
