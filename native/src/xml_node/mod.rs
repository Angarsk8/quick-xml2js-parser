extern crate quick_xml;

use self::quick_xml::events::{attributes::Attributes as XmlAttributes, Event as XmlEvent};
use self::quick_xml::Reader as XmlReader;
use std::collections::hash_map::HashMap;
use std::str::{self, FromStr};

#[derive(Debug, Clone)]
pub enum Primitive {
  Null,
  Float(f64),
  Integer(i32),
  Boolean(bool),
  String(String),
}

impl Primitive {
  fn parse(string: &str) -> Self {
    if let Ok(value) = string.parse::<i32>() {
      Primitive::Integer(value)
    } else if let Ok(value) = string.parse::<f64>() {
      Primitive::Float(value)
    } else if let Ok(value) = string.parse::<bool>() {
      Primitive::Boolean(value)
    } else {
      Primitive::String(string.to_owned())
    }
  }
}

#[derive(Debug, Clone)]
pub enum ChildValue {
  Object(Node),
  Array(Vec<Node>),
}

#[derive(Debug, Clone)]
pub struct Node {
  pub name: String,
  pub attributes: Vec<(String, String)>,
  pub content: Primitive,
  pub children: HashMap<String, ChildValue>,
}

impl Node {
  fn new(name: String) -> Self {
    Self {
      name,
      attributes: Vec::new(),
      content: Primitive::Null,
      children: HashMap::new(),
    }
  }

  fn set_content(&mut self, content: &str) {
    self.content = Primitive::parse(&content);
  }

  fn set_attributes_from_reader(
    &mut self,
    reader: &XmlReader<&[u8]>,
    xml_attributes: XmlAttributes,
  ) {
    self.attributes =
      xml_attributes
        .into_iter()
        .fold(Vec::new(), |mut acc_attributes, attribute_result| {
          let attribute = attribute_result.unwrap();
          let key = str::from_utf8(attribute.key).unwrap();
          let value = attribute.unescape_and_decode_value(&reader).unwrap();
          acc_attributes.push((key.to_owned(), value));
          acc_attributes
        });
  }

  fn insert_child(&mut self, name: &str, value: Node) {
    match self.children.get_mut(name) {
      Some(thing) => match thing {
        ChildValue::Object(object) => {
          *thing = ChildValue::Array(vec![object.clone(), value]);
        }
        ChildValue::Array(array) => {
          array.push(value);
        }
      },
      None => {
        self
          .children
          .insert(name.to_owned(), ChildValue::Object(value));
      }
    };
  }
}

type ParseResult = Result<Node, String>;

impl FromStr for Node {
  type Err = String;

  fn from_str(xml_string: &str) -> ParseResult {
    let mut reader = XmlReader::from_str(xml_string);
    reader.trim_text(true);

    let mut buffer = Vec::new();
    let mut traversal_stack: Vec<Node> = Vec::new();
    let root = Node::new("root".to_owned());

    traversal_stack.push(root);

    loop {
      match reader.read_event(&mut buffer) {
        Ok(XmlEvent::Start(tag_result)) => {
          let name = str::from_utf8(&tag_result.name()).unwrap();
          let mut current_node = Node::new(name.to_owned());
          current_node.set_attributes_from_reader(&reader, tag_result.attributes());
          traversal_stack.push(current_node);
        }

        Ok(XmlEvent::Empty(tag_result)) => {
          let name = str::from_utf8(&tag_result.name()).unwrap();
          let current_parent = traversal_stack.last_mut().unwrap();
          let mut current_node = Node::new(name.to_owned());
          current_node.set_attributes_from_reader(&reader, tag_result.attributes());
          current_parent.insert_child(name, current_node);
        }

        Ok(XmlEvent::Text(content_result)) => {
          let content = content_result.unescape_and_decode(&reader).unwrap();
          let current_node = traversal_stack.last_mut().unwrap();
          current_node.set_content(&content);
        }

        Ok(XmlEvent::CData(cdata_result)) => {
          let current_node = traversal_stack.last_mut().unwrap();
          let cdata = match cdata_result.unescape_and_decode(&reader) {
            Ok(value) => value,
            Err(_) => reader
              .decode(cdata_result.escaped())
              .unwrap_or("")
              .to_owned(),
          };
          current_node.set_content(&cdata);
        }

        Ok(XmlEvent::End(tag_result)) => {
          let current_node = traversal_stack.last().unwrap();
          let name = str::from_utf8(&tag_result.name()).unwrap();

          if current_node.name != name {
            panic!(
              "Error at position {}. Expected {}, got {} instead",
              reader.buffer_position(),
              current_node.name,
              name
            );
          }

          let removed_node = traversal_stack.pop().unwrap();
          let current_last_node = traversal_stack.last_mut().unwrap();
          current_last_node.insert_child(name, removed_node);
        }

        Ok(XmlEvent::Eof) => {
          let stack_len = traversal_stack.len();
          let last_node = traversal_stack.pop().unwrap();

          if stack_len > 1 {
            panic!(
              "Unexpected EOF at {}. Expected closing tag </ {}>",
              reader.buffer_position(),
              last_node.name
            );
          }

          return Ok(last_node);
        }

        Err(reason) => panic!(
          "Error at position {}: {:?}",
          reader.buffer_position(),
          reason
        ),

        _ => (),
      }

      buffer.clear();
    }
  }
}
