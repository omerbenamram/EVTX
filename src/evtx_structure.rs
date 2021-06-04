use crate::binxml::name::BinXmlName;
use crate::binxml::value_variant::BinXmlValue;
use crate::err::{SerializationError, SerializationResult};
use crate::model::xml::{BinXmlPI, XmlAttribute, XmlElement};
use crate::xml_output::BinXmlOutput;
use chrono::prelude::*;
use std::borrow::Cow;
use std::mem;

use std::collections::HashMap;

#[derive(Debug)]
pub enum EvtxXmlContentType {
  Simple(String),
  Complex(Vec<EvtxXmlElement>),
  None,
}

#[derive(Debug)]
pub struct EvtxXmlElement {
  pub name: String,
  pub attributes: HashMap<String, String>,
  pub content: EvtxXmlContentType,
}

impl EvtxXmlElement {
  pub fn new(name: &str) -> Self {
    Self {
      name: name.to_owned(),
      attributes: HashMap::new(),
      content: EvtxXmlContentType::None,
    }
  }

  pub fn add_attribute(&mut self, name: &str, value: &str) {
    self.attributes.insert(name.to_owned(), value.to_owned());
  }

  pub fn add_simple_content(&mut self, value: &str) {
    match self.content {
      EvtxXmlContentType::None => self.content = EvtxXmlContentType::Simple(value.to_owned()),
      EvtxXmlContentType::Simple(ref mut s) => s.push_str(value),
      _ => {
        if !value.is_empty() {
          panic!(
            "this xml element has already a value assigned: {:?}, trying to add {:?}",
            self.content, value
          )
        }
      }
    }
  }

  pub fn add_child(&mut self, child: EvtxXmlElement) {
    match self.content {
      EvtxXmlContentType::Simple(_) => {
        panic!("this xml element is a text node and cannot contain child elements")
      }
      EvtxXmlContentType::None => self.content = EvtxXmlContentType::Complex(vec![child]),
      EvtxXmlContentType::Complex(ref mut v) => v.push(child),
    }
  }
}

pub struct EvtxStructure {
  event_record_id: u64,
  timestamp: DateTime<Utc>,
  content: EvtxXmlElement,
}

impl EvtxStructure {
  pub fn new(event_record_id: u64, timestamp: DateTime<Utc>) -> Self {
    Self {
      event_record_id,
      timestamp,
      content: EvtxXmlElement::new(""), // this will be overriden later
    }
  }

  pub fn new_empty() -> Self {
    Self {
      event_record_id: 0,
      timestamp: DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(0, 0), Utc),
      content: EvtxXmlElement::new(""),
    }
  }

  pub fn event_record_id(&self) -> u64 {
    self.event_record_id
  }

  pub fn timestamp(&self) -> &DateTime<Utc> {
    &self.timestamp
  }
}

pub struct StructureBuilder {
  result: EvtxStructure,
  node_stack: Vec<EvtxXmlElement>,
}

impl StructureBuilder {
  pub fn new(event_record_id: u64, timestamp: DateTime<Utc>) -> Self {
    Self {
      result: EvtxStructure::new(event_record_id, timestamp),
      node_stack: Vec::new(),
    }
  }

  /// consumes self and returns the generated structure
  pub fn get_structure(&mut self) -> EvtxStructure {
    let mut result = EvtxStructure::new_empty();
    mem::swap(&mut self.result, &mut result);
    return result;
  }

  pub fn enter_named_node(&mut self, name: &str, attributes: &Vec<XmlAttribute>) {
    let mut element = EvtxXmlElement::new(name);
    for a in attributes {
      element.add_attribute(a.name.as_ref().as_str(), &a.value.as_ref().as_cow_str());
    }
    self.node_stack.push(element);
  }

  pub fn leave_node(&mut self, _name: &str) {
    let my_node = self.node_stack.pop().expect("stack underflow");
    if self.node_stack.is_empty() {
      self.result.content = my_node;
    } else {
      self.node_stack.last_mut().unwrap().add_child(my_node);
    }
  }
}

impl BinXmlOutput for StructureBuilder {
  /// Called once when EOF is reached.
  fn visit_end_of_stream(&mut self) -> SerializationResult<()> {
    if !self.node_stack.is_empty() {
      return Err(SerializationError::StructureBuilderError {
        message: "node stack is not empty".to_owned(),
      });
    }
    Ok(())
  }

  /// Called on <Tag attr="value" another_attr="value">.
  fn visit_open_start_element(&mut self, element: &XmlElement) -> SerializationResult<()> {
    let name = element.name.as_ref().as_str();

    self.enter_named_node(name, &element.attributes);
    Ok(())
  }

  /// Called on </Tag>, implementor may want to keep a stack to properly close tags.
  fn visit_close_element(&mut self, element: &XmlElement) -> SerializationResult<()> {
    let name = element.name.as_ref().as_str();
    self.leave_node(&name);
    Ok(())
  }

  ///
  /// Called with value on xml text node,  (ex. <Computer>DESKTOP-0QT8017</Computer>)
  ///                                                     ~~~~~~~~~~~~~~~
  fn visit_characters(&mut self, value: &BinXmlValue) -> SerializationResult<()> {
    let cow: Cow<str> = value.as_cow_str();
    self.node_stack.last_mut().unwrap().add_simple_content(&cow);
    Ok(())
  }

  /// Unimplemented
  fn visit_cdata_section(&mut self) -> SerializationResult<()> {
    Ok(())
  }

  /// Emit the character "&" and the text.
  fn visit_entity_reference(&mut self, _: &BinXmlName) -> SerializationResult<()> {
    Ok(())
  }

  /// Emit the characters "&" and "#" and the decimal string representation of the value.
  fn visit_character_reference(&mut self, _: Cow<'_, str>) -> SerializationResult<()> {
    Ok(())
  }

  /// Unimplemented
  fn visit_processing_instruction(&mut self, _: &BinXmlPI) -> SerializationResult<()> {
    Ok(())
  }

  /// Called once on beginning of parsing.
  fn visit_start_of_stream(&mut self) -> SerializationResult<()> {
    if self.node_stack.len() != 0 {
      panic!("internal error: node stack is not empty");
    }
    Ok(())
  }
}
