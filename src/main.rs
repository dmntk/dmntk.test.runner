/*
 * DMNTK - Decision Model and Notation Toolkit
 *
 * MIT license
 *
 * Copyright (c) 2015-2023 Dariusz Depta, Engos Software
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 *
 * Apache license, Version 2.0
 *
 * Copyright (c) 2015-2023 Dariusz Depta, Engos Software
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

//! # Test runner for DMN™ Technology Compatibility Kit

use crate::context::{Context, TestResult};
use crate::dto::{InputNodeDto, OptionalValueDto, ResultDto, ValueDto};
use crate::model::{parse_test_file, Value};
use crate::params::EvaluateParams;
use regex::Regex;
use reqwest::blocking::Client;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::string::ToString;
use std::time::Instant;

mod config;
mod context;
mod dto;
mod model;
mod params;

pub const COLOR_RED: &str = "\u{1b}[31m";
pub const COLOR_GREEN: &str = "\u{1b}[32m";
pub const COLOR_BLUE: &str = "\u{1b}[34m";
pub const COLOR_YELLOW: &str = "\u{1b}[33m";
pub const COLOR_RESET: &str = "\u{1b}[0m";
pub const COLOR_BRIGHT_WHITE: &str = "\u{1b}[37;1m";
pub const GUTTER: usize = 250;
pub const GAP: &str = "...........................................................................................................................................................................................................";

/// Main entrypoint of the runner.
fn main() {
  // read configuration from file
  let config = config::get();
  // prepare the full directory path where test are stored
  let root_dir = Path::new(&config.test_cases_dir_path).canonicalize().expect("reading test directory failed");
  // create the testing context
  let mut ctx = Context::new(
    config.stop_on_failure,
    config.file_search_pattern,
    &config.report_file,
    &config.tck_report_file,
    root_dir.to_string_lossy().to_string(),
  );
  if root_dir.exists() && root_dir.is_dir() {
    print!("Starting DMN TCK runner...");
    let client = Client::new();
    println!("ok");
    println!("File search pattern: {}", ctx.file_search_pattern);
    print!("Searching DMN files in directory: {} ... ", root_dir.display());
    let mut files = BTreeMap::new();
    let pattern = Regex::new(&ctx.file_search_pattern).expect("parsing search pattern failed");
    search_files(&root_dir, &pattern, &mut files);
    println!("ok");
    for (dir_name, (files_dmn, files_xml)) in files {
      // retrieve model names and namespaces from DMN files
      for file_dmn in files_dmn {
        ctx.process_model_definitions(&root_dir, &dir_name, &file_dmn);
      }
      // execute all tests
      for file_xml in files_xml {
        let file_path = format!("{}/{}", dir_name, file_xml);
        execute_tests(&mut ctx, &file_path, &client, &config.evaluate_url);
      }
    }
    let success_count = ctx.success_count;
    let failure_count = ctx.failure_count;
    let total_count = success_count + failure_count;
    let total_execution_time = (ctx.execution_time / 1_000_000) as f64 / 1000.0;
    let requests_per_second = total_count as f64 / total_execution_time;
    let (success_perc, failure_perc) = if total_count > 0 {
      ((success_count * 100) as f64 / total_count as f64, (failure_count * 100) as f64 / total_count as f64)
    } else {
      (0.0, 0.0)
    };
    println!("\nTests:");
    println!("┌─────────┬───────┬─────────┐");
    println!("│   Total │ {total_count:>5} │         │");
    println!("├─────────┼───────┼─────────┤");
    println!("│ {1}Success{0} │ {1}{success_count:>5}{0} │{1}{success_perc:>7.2}%{0} │", COLOR_RESET, COLOR_GREEN);
    println!(
      "│ {1}Failure{0} │ {1}{failure_count:>5}{0} │{1}{failure_perc:>7.2}%{0} │",
      COLOR_RESET,
      if failure_count > 0 { COLOR_RED } else { COLOR_BRIGHT_WHITE }
    );
    println!("└─────────┴───────┴─────────┘");
    ctx.display_test_cases_report();
    println!("\nTimings:");
    println!("┌───────────────────────┬────────┐");
    println!("│ Average requests time │ {:>5.02}s │", (ctx.execution_time / 1_000_000) as f64 / 1000.0);
    println!("│   Requests per second │ {:>6.0} │", requests_per_second);
    println!("└───────────────────────┴────────┘");
  } else {
    usage();
  }
}

fn execute_tests(ctx: &mut Context, file_path: &str, client: &Client, evaluate_url: &str) {
  let text = format!("  Parsing test file: {}", file_path);
  print!("\n{} {} ", text, &GAP[..GUTTER - text.len()]);
  let test_cases = parse_test_file(file_path);
  println!("{1}ok{0}\n", COLOR_RESET, COLOR_GREEN);
  let empty_id = String::new();
  let model_file_name = test_cases.model_name.clone().expect("model name not specified in test case");
  let workspace_name = ctx.get_workspace_name(&model_file_name);
  let model_namespace = ctx.get_model_rdnn(&model_file_name);
  let model_name = ctx.get_model_name(&model_file_name);
  for test_case in &test_cases.test_cases {
    let test_case_id = test_case.id.as_ref().unwrap_or(&empty_id);
    let opt_invocable_name = test_case.invocable_name.as_ref().cloned();
    for (i, result_node) in test_case.result_nodes.iter().enumerate() {
      let test_id = if i > 0 { format!("{}:{}", test_case_id, i) } else { test_case_id.to_string() };
      let invocable_name = if let Some(invocable_name) = &opt_invocable_name {
        invocable_name.to_string()
      } else {
        result_node.name.clone()
      };
      let test_case_details = format!("Executing test case, id: {test_id}, model name: {model_name}, invocable name: {invocable_name}");
      let text = format!(
        "Executing test case, {1}id{0}: {2}{test_id}{0}, {1}model name{0}: {2}{model_name}{0}, {1}invocable name{0}: {2}{invocable_name}{0}",
        COLOR_RESET, COLOR_BRIGHT_WHITE, COLOR_BLUE
      );
      print!("{} {} ", text, &GAP[..GUTTER - test_case_details.len()]);
      let invocable_path = format!(
        "{}{}/{}",
        if workspace_name.is_empty() { "".to_string() } else { format!("{}/", workspace_name) },
        model_namespace,
        //model_name,
        invocable_name
      );
      let params = EvaluateParams {
        invocable_path,
        input_values: test_case.input_nodes.iter().map(InputNodeDto::from).collect(),
      };
      evaluate_test_case(ctx, file_path, client, evaluate_url, test_case_id, &test_id, &params, &result_node.expected);
    }
  }
}

#[allow(clippy::too_many_arguments)]
fn evaluate_test_case(
  ctx: &mut Context,
  file_path: &str,
  client: &Client,
  evaluate_url: &str,
  test_case_id: &str,
  test_id: &str,
  params: &EvaluateParams,
  opt_expected: &Option<Value>,
) {
  let execution_start_time = Instant::now();
  match client.post(evaluate_url).json(&params).send() {
    Ok(response) => {
      let execution_duration = execution_start_time.elapsed();
      ctx.execution_time += execution_duration.as_nanos();
      match response.json::<ResultDto<OptionalValueDto>>() {
        Ok(result) => {
          if let Some(data) = result.data {
            if let Some(result_dto) = data.value {
              if let Some(expected) = opt_expected {
                let expected_dto = ValueDto::from(expected);
                if result_dto == expected_dto {
                  ctx.write_line(file_path, test_case_id, test_id, TestResult::Success, &format!("{} µs", execution_duration.as_micros()));
                } else {
                  ctx.write_line(file_path, test_case_id, test_id, TestResult::Failure, "result differs from expected");
                  let result_json = serde_json::to_string(&result_dto).unwrap();
                  let expected_json = serde_json::to_string(&expected_dto).unwrap();
                  println!("    result: {1}{2}{0}", COLOR_RESET, COLOR_RED, result_json);
                  println!("  expected: {1}{2}{0}", COLOR_RESET, COLOR_GREEN, expected_json);
                  println!();
                  let mut result_chars = result_json.chars();
                  let mut expected_chars = expected_json.chars();
                  let mut index = 0;
                  while let Some((a, b)) = result_chars.next().zip(expected_chars.next()) {
                    if a != b {
                      if index > 30 {
                        index -= 30;
                      } else {
                        index = 0;
                      }
                      println!("    result [{3}..]: {1}{2}{0}", COLOR_RESET, COLOR_RED, &result_json[index..], index);
                      println!("  expected [{3}..]: {1}{2}{0}", COLOR_RESET, COLOR_GREEN, &expected_json[index..], index);
                      println!();
                      break;
                    } else {
                      index += 1;
                    }
                  }

                  let result_json_pretty = serde_json::to_string_pretty(&result_dto).unwrap();
                  let expected_json_pretty = serde_json::to_string_pretty(&expected_dto).unwrap();
                  let mut result_lines = result_json_pretty.lines();
                  let mut expected_lines = expected_json_pretty.lines();
                  let max_width = expected_json_pretty.lines().map(|line| line.len()).max().unwrap() + 5;
                  while let Some((a, b)) = result_lines.next().zip(expected_lines.next()) {
                    let color_red = if a != b { COLOR_RED } else { COLOR_RESET };
                    let color_green = if a != b { COLOR_GREEN } else { COLOR_RESET };
                    let marker = if a != b { "|" } else { " " };
                    println!("{3} {2}{5:6$}{0} {1}{4}{0}", COLOR_RESET, color_red, color_green, marker, a, b, max_width);
                  }
                }
              } else {
                ctx.write_line(file_path, test_case_id, test_id, TestResult::Failure, "no expected value");
              }
            } else {
              ctx.write_line(file_path, test_case_id, test_id, TestResult::Failure, "no actual value");
            }
          } else if result.errors.is_some() {
            ctx.write_line(file_path, test_case_id, test_id, TestResult::Failure, &result.to_string());
          } else {
            ctx.write_line(file_path, test_case_id, test_id, TestResult::Failure, format!("{:?}", result).as_str());
          }
        }
        Err(reason) => {
          ctx.write_line(file_path, test_case_id, test_id, TestResult::Failure, &reason.to_string());
        }
      }
    }
    Err(reason) => {
      let execution_duration = execution_start_time.elapsed();
      ctx.execution_time += execution_duration.as_nanos();
      ctx.write_line(file_path, test_case_id, test_id, TestResult::Failure, &reason.to_string());
    }
  }
}

fn search_files(path: &Path, pattern: &Regex, files: &mut BTreeMap<String, (Vec<String>, Vec<String>)>) {
  if let Ok(entries) = fs::read_dir(path) {
    for entry in entries.flatten() {
      let path = entry.path();
      if path.is_dir() {
        search_files(&path, pattern, files);
      } else if let Some(dir) = path.parent() {
        let dir_name = dir.canonicalize().unwrap().display().to_string();
        if let Some(exp) = path.extension() {
          if exp == "dmn" {
            let file_name = path.file_name().unwrap().to_string_lossy().to_string();
            let full_name = format!("{}/{}", dir_name, file_name);
            if pattern.is_match(&full_name) {
              let (files_dmn, _) = files.entry(dir_name.clone()).or_insert((vec![], vec![]));
              files_dmn.push(file_name);
            }
          }
          if exp == "xml" {
            let file_name = path.file_name().unwrap().to_string_lossy().to_string();
            let full_name = format!("{}/{}", dir_name, file_name);
            if pattern.is_match(&full_name) {
              let (_, files_xml) = files.entry(dir_name).or_insert((vec![], vec![]));
              files_xml.push(file_name);
            }
          }
        }
      }
    }
  }
}

/// Displays usage message.
fn usage() {
  println!("TBD")
}
