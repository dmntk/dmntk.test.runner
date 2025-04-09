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

//! # XML model for test cases

use roxmltree::Node;
use std::fs::read_to_string;

const XSI: &str = "http://www.w3.org/2001/XMLSchema-instance";

const NODE_COMPONENT: &str = "component";
const NODE_COMPUTED: &str = "computed";
const NODE_DESCRIPTION: &str = "description";
const NODE_EXPECTED: &str = "expected";
const NODE_INPUT_NODE: &str = "inputNode";
const NODE_ITEM: &str = "item";
const NODE_LABELS: &str = "labels";
const NODE_LABEL: &str = "label";
const NODE_LIST: &str = "list";
const NODE_MODEL_NAME: &str = "modelName";
const NODE_RESULT_NODE: &str = "resultNode";
const NODE_TEST_CASE: &str = "testCase";
const NODE_TEST_CASES: &str = "testCases";
const NODE_VALUE: &str = "value";

const ATTR_CAST: &str = "cast";
const ATTR_ERROR_RESULT: &str = "errorResult";
const ATTR_ID: &str = "id";
const ATTR_INVOCABLE_NAME: &str = "invocableName";
const ATTR_NAME: &str = "name";
const ATTR_NIL: &str = "nil";
const ATTR_TYPE: &str = "type";

/// Test cases.
#[derive(Debug)]
pub struct TestCases {
  pub model_name: Option<String>,
  pub labels: Vec<String>,
  pub test_cases: Vec<TestCase>,
}

/// Type of the test case.
#[derive(Debug, PartialEq, Eq)]
pub enum TestCaseType {
  Decision,
  BusinessKnowledgeModel,
  DecisionService,
}

impl From<String> for TestCaseType {
  fn from(value: String) -> Self {
    match value.to_lowercase().trim() {
      "bkm" => Self::BusinessKnowledgeModel,
      "decisionservice" => Self::DecisionService,
      _ => Self::Decision,
    }
  }
}

impl From<Option<String>> for TestCaseType {
  fn from(value: Option<String>) -> Self {
    if let Some(s) = value {
      return Self::from(s);
    }
    TestCaseType::Decision
  }
}

impl ToString for TestCaseType {
  fn to_string(&self) -> String {
    match self {
      TestCaseType::Decision => "decision",
      TestCaseType::BusinessKnowledgeModel => "bkm",
      TestCaseType::DecisionService => "decisionService",
    }
    .to_string()
  }
}

/// Single test case.
#[derive(Debug)]
pub struct TestCase {
  /// Optional identifier of this [TestCase].
  pub id: Option<String>,
  /// Optional name of this [TestCase].
  pub name: Option<String>,
  /// Type of this [TestCase] with default value `Decision`.
  pub typ: TestCaseType,
  /// Optional description for this [TestCase].
  pub description: Option<String>,
  /// Optional invocable name.
  pub invocable_name: Option<String>,
  /// Collection of input nodes.
  pub input_nodes: Vec<InputNode>,
  /// Collection of result nodes.
  pub result_nodes: Vec<ResultNode>,
}

/// Input node defined for test case.
#[derive(Debug)]
pub struct InputNode {
  /// Required name of this [InputNode].
  pub name: String,
  /// Optional value of this [InputNode].
  pub value: Option<Value>,
}

/// Result node defined for the test case.
#[derive(Debug)]
pub struct ResultNode {
  pub name: String,
  pub error_result: bool,
  pub typ: TestCaseType,
  pub cast: Option<String>,
  pub expected: Option<Value>,
  pub computed: Option<Value>,
}

/// Types of values.
/// [Value] may be a simple (single) value,
/// collection of components or a list.
#[derive(Debug)]
pub enum Value {
  Simple(Simple),
  Components(Vec<Component>),
  List(List),
}

/// Value representing simple result of the test case.
#[derive(Debug)]
pub struct Simple {
  /// Type of the value in namespace-prefixed form.
  pub typ: Option<String>,
  /// Optional text representing the value.
  pub text: Option<String>,
  /// Flag indicating if this [Value] is nil, like `xsi:nil="true"`.
  pub nil: bool,
}

/// Value representing complex result of a test case.
#[derive(Debug)]
pub struct Component {
  /// Optional name of this component.
  pub name: Option<String>,
  /// Optional value contained in this [Component].
  pub value: Option<Value>,
  /// Flag indicating if this [Component] is nil, like `xsi:nil="true"`.
  pub nil: bool,
}

/// Value representing a list.
#[derive(Debug)]
pub struct List {
  /// Vector of list items (values), may be empty.
  pub items: Vec<Value>,
  /// Flag indicating if this [List] is nil, like `xsi:nil="true"`.
  pub nil: bool,
}

impl Default for List {
  /// [List] is empty and nil by default.
  fn default() -> Self {
    Self { items: vec![], nil: true }
  }
}

/// Parses the XML file containing test cases.
pub fn parse_test_file(file_name: &str) -> TestCases {
  let content = read_to_string(file_name).expect("reading test file failed");
  let document = roxmltree::Document::parse(&content).expect("parsing test file failed");
  let test_cases_node = document.root_element();
  if test_cases_node.tag_name().name() != NODE_TEST_CASES {
    panic!("Expected mandatory node: {}", NODE_TEST_CASES);
  } else {
    parse_root_node(&test_cases_node)
  }
}

/// Parses `testCases` node being the root element of the document.
fn parse_root_node(node: &Node) -> TestCases {
  TestCases {
    model_name: optional_child_required_content(node, NODE_MODEL_NAME),
    labels: parse_labels(node),
    test_cases: parse_test_cases(node),
  }
}

/// Parses all labels.
fn parse_labels(node: &Node) -> Vec<String> {
  let mut items = vec![];
  if let Some(labels_node) = node.children().find(|n| n.tag_name().name() == NODE_LABELS) {
    for ref label_node in labels_node.children().filter(|n| n.tag_name().name() == NODE_LABEL) {
      items.push(required_content(label_node))
    }
  }
  items
}

/// Parses all test cases.
fn parse_test_cases(node: &Node) -> Vec<TestCase> {
  let mut items = vec![];
  for ref test_case_node in node.children().filter(|n| n.tag_name().name() == NODE_TEST_CASE) {
    items.push(TestCase {
      id: optional_attribute(test_case_node, ATTR_ID),
      name: optional_attribute(test_case_node, ATTR_NAME),
      typ: parse_test_case_type(test_case_node),
      description: optional_child_required_content(test_case_node, NODE_DESCRIPTION),
      invocable_name: optional_attribute(test_case_node, ATTR_INVOCABLE_NAME),
      input_nodes: parse_input_nodes(test_case_node),
      result_nodes: parse_result_nodes(test_case_node),
    })
  }
  items
}

/// Parses test case type. The default value is [TestCaseType#Decision].
fn parse_test_case_type(node: &Node) -> TestCaseType {
  match optional_attribute(node, ATTR_TYPE) {
    Some(t) if t == "bkm" => TestCaseType::BusinessKnowledgeModel,
    Some(t) if t == "decisionService" => TestCaseType::DecisionService,
    _ => TestCaseType::Decision,
  }
}

/// Parses input nodes defined for test case.
fn parse_input_nodes(node: &Node) -> Vec<InputNode> {
  let mut items = vec![];
  for ref input_node in node.children().filter(|n| n.tag_name().name() == NODE_INPUT_NODE) {
    items.push(InputNode {
      name: required_attribute(input_node, ATTR_NAME),
      value: parse_value_type(input_node),
    })
  }
  items
}

/// Parses result nodes expected by test case.
fn parse_result_nodes(node: &Node) -> Vec<ResultNode> {
  let mut items = vec![];
  for ref result_node in node.children().filter(|n| n.tag_name().name() == NODE_RESULT_NODE) {
    items.push(ResultNode {
      name: required_attribute(result_node, ATTR_NAME),
      error_result: optional_attribute(result_node, ATTR_ERROR_RESULT).is_some_and( |v| v == "true"),
      typ: optional_attribute(result_node, ATTR_TYPE).into(),
      cast: optional_attribute(result_node, ATTR_CAST),
      expected: parse_child_value_type(result_node, NODE_EXPECTED),
      computed: parse_child_value_type(result_node, NODE_COMPUTED),
    })
  }
  items
}

/// Parses value type.
fn parse_value_type(node: &Node) -> Option<Value> {
  if let Some(v) = parse_simple_value(node) {
    return Some(Value::Simple(v));
  }
  if let Some(c) = parse_value_components(node) {
    return Some(Value::Components(c));
  }
  if let Some(l) = parse_value_list(node) {
    return Some(Value::List(l));
  }
  None
}

/// Parses value type from child node.
fn parse_child_value_type(node: &Node, child_name: &str) -> Option<Value> {
  if let Some(ref child_node) = node.children().find(|n| n.tag_name().name() == child_name) {
    parse_value_type(child_node)
  } else {
    None
  }
}

/// Parses simple value.
fn parse_simple_value(node: &Node) -> Option<Simple> {
  if let Some(ref value_node) = node.children().find(|n| n.tag_name().name() == NODE_VALUE) {
    let typ = optional_xsi_type_attribute(value_node);
    let text = optional_content(value_node);
    let nil = optional_nil_attribute(value_node);
    return Some(match (typ.is_some(), text.is_some(), nil) {
      (true, false, false) => Simple {
        typ,
        text: Some("".to_string()),
        nil,
      },
      _ => Simple { typ, text, nil },
    });
  }
  None
}

/// Parses a collection of component values.
fn parse_value_components(node: &Node) -> Option<Vec<Component>> {
  let mut items = vec![];
  for ref component_node in node.children().filter(|n| n.tag_name().name() == NODE_COMPONENT) {
    items.push(Component {
      name: optional_attribute(component_node, ATTR_NAME),
      value: parse_value_type(component_node),
      nil: optional_nil_attribute(component_node),
    })
  }
  if !items.is_empty() {
    items.sort_by(|a, b| a.name.cmp(&b.name));
    return Some(items);
  }
  None
}

/// Parses a list of values.
fn parse_value_list(node: &Node) -> Option<List> {
  let mut items = vec![];
  if let Some(ref list_node) = node.children().find(|n| n.tag_name().name() == NODE_LIST) {
    if optional_nil_attribute(list_node) {
      return Some(List::default());
    }
    for ref item_node in list_node.children().filter(|n| n.tag_name().name() == NODE_ITEM) {
      if let Some(value_type) = parse_value_type(item_node) {
        items.push(value_type)
      }
    }
    return Some(List { items, nil: false });
  }
  None
}

/// XML utility function that returns the value of the required attribute or an error.
fn required_attribute(node: &Node, attr_name: &str) -> String {
  node
    .attribute(attr_name)
    .unwrap_or_else(|| panic!("No mandatory attribute '{}' in node '{}'", attr_name, node.tag_name().name()))
    .to_string()
}

/// XML utility function that returns the value of the optional attribute.
fn optional_attribute(node: &Node, attr_name: &str) -> Option<String> {
  node.attribute(attr_name).map(|attr_value| attr_value.to_owned())
}

/// XML utility function that returns the value of the optional `xsi:type` attribute.
fn optional_xsi_type_attribute(node: &Node) -> Option<String> {
  node.attribute((XSI, ATTR_TYPE)).map(|attr_value| attr_value.to_owned())
}

/// XML utility function that returns `true` when `xsi:nil="true"` attribute is specified.
fn optional_nil_attribute(node: &Node) -> bool {
  node.attribute((XSI, ATTR_NIL))== Some("true")
}

/// XML utility function that returns required textual content from the specified node.
fn required_content(node: &Node) -> String {
  node
    .text()
    .unwrap_or_else(|| panic!("No mandatory text content in node: '{}'", node.tag_name().name()))
    .to_string()
}

/// XML utility function that returns optional textual content of the node.
fn optional_content(node: &Node) -> Option<String> {
  node.text().map(|text| text.to_owned())
}

/// XML utility function that returns the required textual content from the optional child node.
fn optional_child_required_content(node: &Node, child_name: &str) -> Option<String> {
  node.children().find(|n| n.tag_name().name() == child_name).map(|child_node| required_content(&child_node))
}
