//! # Data transfer objects for input and output values

use crate::model::{Component, InputNode, List, Simple, Value};
use serde::{Deserialize, Serialize};

/// Data transfer object for an error.
#[derive(Debug, Deserialize)]
pub struct ErrorDto {
  /// Error details.
  #[serde(rename = "detail")]
  pub detail: String,
}

/// Data transfer object for a result.
#[derive(Debug, Deserialize)]
pub struct ResultDto<T> {
  /// Result containing data.
  #[serde(rename = "data")]
  pub data: Option<T>,
  /// Result containing errors.
  #[serde(rename = "errors")]
  pub errors: Option<Vec<ErrorDto>>,
}

impl<T> ToString for ResultDto<T> {
  /// Converts results to string.
  fn to_string(&self) -> String {
    self
      .errors
      .as_ref()
      .map(|v| v.iter().map(|e| e.detail.clone()).collect::<Vec<String>>().join(", "))
      .unwrap_or_default()
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InputNodeDto {
  #[serde(rename = "name")]
  pub name: String,
  #[serde(rename = "value")]
  pub value: Option<ValueDto>,
}

#[derive(Debug, Deserialize)]
pub struct OptionalValueDto {
  #[serde(rename = "value")]
  pub value: Option<ValueDto>,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct ValueDto {
  #[serde(rename = "simple", skip_serializing_if = "Option::is_none")]
  pub simple: Option<SimpleDto>,
  #[serde(rename = "components", skip_serializing_if = "Option::is_none")]
  pub components: Option<Vec<ComponentDto>>,
  #[serde(rename = "list", skip_serializing_if = "Option::is_none")]
  pub list: Option<ListDto>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SimpleDto {
  #[serde(rename = "type")]
  pub typ: Option<String>,
  #[serde(rename = "text")]
  pub text: Option<String>,
  #[serde(rename = "isNil")]
  pub nil: bool,
}

impl PartialEq for SimpleDto {
  fn eq(&self, rhs: &Self) -> bool {
    // if self.typ.is_some()
    //   && rhs.typ.is_some()
    //   && (self.typ.as_ref().unwrap() == "xsd:decimal" || self.typ.as_ref().unwrap() == "xsd:double")
    //   && (rhs.typ.as_ref().unwrap() == "xsd:decimal" || rhs.typ.as_ref().unwrap() == "xsd:double")
    //   && self.nil == rhs.nil
    // {
    //   return compare_decimals(self.text.clone(), rhs.text.clone());
    // }
    self.typ == rhs.typ && self.text == rhs.text && self.nil == rhs.nil
  }
}

// ///
// fn compare_decimals(actual: Option<String>, expected: Option<String>) -> bool {
//   if let Some((actual_text, expected_text)) = actual.zip(expected.as_ref()) {
//     if actual_text.starts_with(expected_text) {
//       return true;
//     }
//     if actual_text.starts_with(&expected_text[..expected_text.len() - 1]) {
//       //TODO report warning 1
//       return true;
//     }
//     if actual_text.starts_with(&expected_text[..expected_text.len() - 2]) {
//       //TODO report warning 2
//       return true;
//     }
//     if actual_text.starts_with(&expected_text[..expected_text.len() - 3]) {
//       //TODO report warning 3
//       return true;
//     }
//   }
//   false
// }

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ComponentDto {
  #[serde(rename = "name")]
  pub name: Option<String>,
  #[serde(rename = "value")]
  pub value: Option<ValueDto>,
  #[serde(rename = "isNil")]
  pub nil: bool,
}

impl From<&Component> for ComponentDto {
  fn from(component: &Component) -> Self {
    Self {
      name: component.name.clone(),
      value: component.value.as_ref().map(|value| value.into()),
      nil: component.nil,
    }
  }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ListDto {
  #[serde(rename = "items")]
  pub items: Vec<ValueDto>,
  #[serde(rename = "isNil")]
  pub nil: bool,
}

impl From<&List> for ListDto {
  fn from(list: &List) -> Self {
    Self {
      items: list.items.iter().map(ValueDto::from).collect(),
      nil: list.nil,
    }
  }
}

impl From<&InputNode> for InputNodeDto {
  fn from(input_node: &InputNode) -> Self {
    Self {
      name: input_node.name.clone(),
      value: input_node.value.as_ref().map(|value| value.into()),
    }
  }
}

impl From<&Simple> for SimpleDto {
  fn from(simple: &Simple) -> Self {
    Self {
      typ: simple.typ.clone(),
      text: simple.text.clone(),
      nil: simple.nil,
    }
  }
}

impl From<&Value> for ValueDto {
  fn from(value: &Value) -> Self {
    match &value {
      Value::Simple(simple) => Self {
        simple: Some(simple.into()),
        ..Default::default()
      },
      Value::Components(components) => Self {
        components: Some(components.iter().map(ComponentDto::from).collect()),
        ..Default::default()
      },
      Value::List(list) => Self {
        list: Some(ListDto::from(list)),
        ..Default::default()
      },
    }
  }
}
