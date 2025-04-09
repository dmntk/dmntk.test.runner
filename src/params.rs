//! # Endpoint parameters

use crate::dto::InputNodeDto;
use serde::Serialize;

/// Parameters for evaluating an invocable.
#[derive(Serialize)]
pub struct EvaluateParams {
  /// Path to invocable to be evaluated.
  #[serde(rename = "invocable")]
  pub invocable_path: String,
  /// Input values.
  #[serde(rename = "input")]
  pub input_values: Vec<InputNodeDto>,
}
