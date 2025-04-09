//! # Configuration data

use serde::{Deserialize, Serialize};

/// Runner configuration parameters.
#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigurationParams {
  /// Path to directory containing test cases.
  pub test_cases_dir_path: String,
  /// Pattern for matching test file names.
  /// Only files whose name matches the pattern will be processed.
  pub file_search_pattern: String,
  /// URL to service where model definitions will be evaluated.
  pub evaluate_url: String,
  /// Path to report file.
  pub report_file: String,
  /// Path to report file for TCK.
  pub tck_report_file: String,
  /// Flag indicating if testing should immediately stop on failure.
  pub stop_on_failure: bool,
}

pub fn get() -> ConfigurationParams {
  let args: Vec<String> = std::env::args().collect();
  let cfg_file_name = if args.len() == 2 { args[1].as_str() } else { "config.yml" };
  let err_read = format!("reading configuration file '{}' failed", cfg_file_name);
  let file_content = std::fs::read_to_string(cfg_file_name).expect(&err_read);
  let err_parse = format!("parsing configuration file '{}' failed", cfg_file_name);
  serde_yaml::from_str(&file_content).expect(&err_parse)
}
