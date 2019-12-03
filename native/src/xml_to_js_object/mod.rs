extern crate quick_xml;

use self::quick_xml::events::{attributes::Attributes as XmlAttributes, Event as XmlEvent};
use self::quick_xml::Reader as XmlReader;
use neon::prelude::*;
use std::str;

#[derive(Debug)]
struct NodeFlags {
  has_attributes: bool,
  has_content: bool,
  has_children: bool,
}

impl NodeFlags {
  fn new() -> Self {
    NodeFlags {
      has_attributes: false,
      has_content: false,
      has_children: false,
    }
  }

  fn is_empty(&self) -> bool {
    !self.has_attributes && !self.has_content && !self.has_children
  }

  fn is_just_content(&self) -> bool {
    !self.has_attributes && !self.has_children
  }
}

fn insert_value_into_js_object(
  cx: &mut FunctionContext,
  key: &str,
  js_value: Handle<JsValue>,
  js_object: &mut JsObject,
) {
  let previous_js_entry = js_object.get(cx, key).unwrap();

  if previous_js_entry.is_a::<JsArray>() {
    let array = previous_js_entry.downcast::<JsArray>().unwrap();
    array.set(cx, array.len(), js_value).unwrap();
    return;
  }

  if previous_js_entry.is_a::<JsObject>() || previous_js_entry.is_a::<JsString>() {
    let new_js_array = JsArray::new(cx, 2);

    new_js_array.set(cx, 0, previous_js_entry).unwrap();
    new_js_array.set(cx, 1, js_value).unwrap();

    js_object.set(cx, key, new_js_array).unwrap();
    return;
  }

  js_object.set(cx, key, js_value).unwrap();
}

fn set_attributes_into_js_object<'a>(
  cx: &mut FunctionContext<'a>,
  reader: &XmlReader<&[u8]>,
  xml_attributes: XmlAttributes,
  js_object: &mut JsObject,
) -> JsResult<'a, JsBoolean> {
  let mut count = 0u8;
  let attributes_js_object = JsObject::new(cx);

  for attribute_result in xml_attributes {
    if attribute_result.is_err() {
      return cx.throw_error::<_, Handle<'a, JsBoolean>>(format!(
        "Failed to parse attribute at position {}: {}",
        reader.buffer_position(),
        attribute_result.unwrap_err()
      ));
    }

    let attribute = attribute_result.unwrap();
    let key = str::from_utf8(attribute.key);

    if key.is_err() {
      return cx.throw_error::<_, Handle<'a, JsBoolean>>(format!(
        "Failed to parse attribute key at position {}: {}",
        reader.buffer_position(),
        key.unwrap_err()
      ));
    }

    let value_result = attribute.unescape_and_decode_value(&reader);
    if value_result.is_err() {
      return cx.throw_error::<_, Handle<'a, JsBoolean>>(format!(
        "Failed to parse attribute value at position {}: {}",
        reader.buffer_position(),
        value_result.unwrap_err()
      ));
    }

    let value = value_result.unwrap();
    let value_as_js_string = cx.string(&value);
    attributes_js_object
      .set(cx, key.unwrap(), value_as_js_string)
      .unwrap();

    count += 1;
  }

  if count > 0 {
    js_object.set(cx, "$", attributes_js_object).unwrap();
    return Ok(cx.boolean(true));
  }

  return Ok(cx.boolean(false));
}

pub fn parse(mut cx: FunctionContext) -> JsResult<JsObject> {
  let xml_string = cx.argument::<JsString>(0)?.value();

  let mut reader = XmlReader::from_str(&xml_string);
  let mut buffer = Vec::new();

  reader.trim_text(true);

  let mut traversal_stack: Vec<(String, Handle<'_, JsObject>, NodeFlags)> = Vec::new();
  let root_js_object = JsObject::new(&mut cx);

  traversal_stack.push(("root".to_owned(), root_js_object, NodeFlags::new()));

  loop {
    match reader.read_event(&mut buffer) {
      Ok(XmlEvent::Start(tag_result)) => {
        let name = str::from_utf8(&tag_result.name()).unwrap();
        let mut current_js_object = JsObject::new(&mut cx);
        let was_set = set_attributes_into_js_object(
          &mut cx,
          &reader,
          tag_result.attributes(),
          &mut current_js_object,
        )?
        .value();

        let mut current_node_flags = NodeFlags::new();
        current_node_flags.has_attributes = was_set;
        traversal_stack.push((name.to_owned(), current_js_object, current_node_flags));
      }

      Ok(XmlEvent::Empty(tag_result)) => {
        let name = str::from_utf8(&tag_result.name()).unwrap();
        let (_, parent_js_object, parent_node_flags) = traversal_stack.last_mut().unwrap();
        let mut current_js_object = JsObject::new(&mut cx);
        let was_set = set_attributes_into_js_object(
          &mut cx,
          &reader,
          tag_result.attributes(),
          &mut current_js_object,
        )?
        .value();

        parent_node_flags.has_children = true;

        if was_set {
          parent_node_flags.has_attributes = true;
          insert_value_into_js_object(&mut cx, name, current_js_object.upcast(), parent_js_object);
        } else {
          let empty = cx.string("").upcast();
          insert_value_into_js_object(&mut cx, name, empty, parent_js_object);
        };
      }

      Ok(XmlEvent::Text(content_result)) => {
        let content = content_result.unescape_and_decode(&reader).unwrap();
        let (_, current_js_object, node_node_flags) = traversal_stack.last_mut().unwrap();
        let content_as_js_value = if let Ok(value) = content.parse::<i32>() {
          cx.number(value).upcast::<JsValue>()
        } else if let Ok(value) = content.parse::<f64>() {
          cx.number(value).upcast::<JsValue>()
        } else if let Ok(value) = content.parse::<bool>() {
          cx.boolean(value).upcast::<JsValue>()
        } else {
          cx.string(content).upcast::<JsValue>()
        };

        node_node_flags.has_content = true;
        current_js_object.set(&mut cx, "_", content_as_js_value)?;
      }

      Ok(XmlEvent::CData(cdata_result)) => {
        let parsed_cdata_result = &cdata_result.unescape_and_decode(&reader);
        let (_, current_js_object, current_node_flags) = traversal_stack.last_mut().unwrap();
        let cdata = match parsed_cdata_result {
          Ok(value) => value,
          Err(_) => reader.decode(cdata_result.escaped()).unwrap_or(""),
        };
        let cdata_as_js_string = cx.string(cdata);

        current_node_flags.has_content = true;
        current_js_object.set(&mut cx, "_", cdata_as_js_string)?;
      }

      Ok(XmlEvent::End(tag_result)) => {
        let name = str::from_utf8(&tag_result.name()).unwrap();
        let (removed_node_name, removed_js_object, removed_node_flags) =
          traversal_stack.pop().unwrap();

        if removed_node_name != name {
          return cx.throw_error(format!(
            "Error at position {}. Expected {}, got {} instead",
            reader.buffer_position(),
            removed_node_name,
            name
          ));
        }

        let (_, new_current_js_object, new_current_node_flags) =
          traversal_stack.last_mut().unwrap();

        new_current_node_flags.has_children = true;

        if removed_node_flags.is_empty() {
          let empty = cx.string("").upcast();
          insert_value_into_js_object(&mut cx, name, empty, new_current_js_object);
        } else if removed_node_flags.is_just_content() {
          let content_as_js_value = removed_js_object.get(&mut cx, "_")?;
          insert_value_into_js_object(&mut cx, name, content_as_js_value, new_current_js_object);
        } else {
          insert_value_into_js_object(
            &mut cx,
            name,
            removed_js_object.upcast(),
            new_current_js_object,
          );
        }
      }

      Ok(XmlEvent::Eof) => {
        let stack_len = traversal_stack.len();
        let (last_node_name, last_js_object, _) = traversal_stack.pop().unwrap();

        if stack_len > 1 {
          return cx.throw_error(format!(
            "Unexpected EOF at {}. Expected closing tag </ {}>",
            reader.buffer_position(),
            last_node_name
          ));
        }

        return Ok(last_js_object);
      }

      Err(reason) => {
        return cx.throw_error(format!(
          "Error at position {}: {:?}",
          reader.buffer_position(),
          reason
        ));
      }

      _ => (),
    }

    buffer.clear();
  }
}
