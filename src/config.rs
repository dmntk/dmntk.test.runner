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
