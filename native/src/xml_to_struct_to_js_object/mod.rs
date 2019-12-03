use neon::prelude::*;
use std::collections::HashMap;
use std::str::FromStr;

use super::xml_node::{self, Node as XmlNode, Primitive};

fn parse_content_as_js<'a>(
  cx: &mut FunctionContext<'a>,
  content: &mut Primitive,
) -> Option<Handle<'a, JsValue>> {
  match content {
    Primitive::Boolean(boolean) => Some(cx.boolean(*boolean).upcast()),
    Primitive::Integer(number) => Some(cx.number(*number as f64).upcast()),
    Primitive::Float(number) => Some(cx.number(*number).upcast()),
    Primitive::String(ref string) => Some(cx.string(&string).upcast()),
    Primitive::Null => None,
  }
}

// TODO: This function cries for a refactor
fn collect_children_into(
  cx: &mut FunctionContext,
  children: &mut HashMap<String, xml_node::ChildValue>,
  js_object: &mut JsObject,
) {
  for (name, value) in children {
    match value {
      xml_node::ChildValue::Object(node_value) => {
        let content = parse_content_as_js(cx, &mut node_value.content);

        if node_value.attributes.is_empty() && node_value.children.is_empty() {
          let js_child_name = cx.string(&name);
          if let Some(js_content_value) = content {
            js_object.set(cx, js_child_name, js_content_value).unwrap();
          } else {
            let js_empty_string = cx.string("");
            js_object.set(cx, js_child_name, js_empty_string).unwrap();
          }
          continue;
        }

        let mut js_child_object = JsObject::new(cx);

        if !node_value.attributes.is_empty() {
          let js_attributes_obj = JsObject::new(cx);

          for (ref key, ref value) in &node_value.attributes {
            let js_key_string = cx.string(&key);
            let js_value_string = cx.string(&value);

            js_attributes_obj
              .set(cx, js_key_string, js_value_string)
              .unwrap();
          }

          js_child_object.set(cx, "$", js_attributes_obj).unwrap();
        }

        if let Some(js_content_value) = content {
          js_child_object.set(cx, "_", js_content_value).unwrap();
        }

        collect_children_into(cx, &mut node_value.children, &mut js_child_object);

        let js_child_name = cx.string(name);

        js_object.set(cx, js_child_name, js_child_object).unwrap();
      }

      xml_node::ChildValue::Array(vec_of_nodes) => {
        let js_child_values = JsArray::new(cx, vec_of_nodes.len() as u32);

        for (i, node_value) in vec_of_nodes.iter_mut().enumerate() {
          let mut js_entry_obj = JsObject::new(cx);

          let content = parse_content_as_js(cx, &mut node_value.content);

          if node_value.attributes.is_empty() && node_value.children.is_empty() {
            if let Some(js_content_value) = content {
              js_child_values.set(cx, i as u32, js_content_value).unwrap();
            } else {
              let js_empty_string = cx.string("");
              js_child_values.set(cx, i as u32, js_empty_string).unwrap();
            }
            continue;
          }

          if !node_value.attributes.is_empty() {
            let js_attributes_obj = JsObject::new(cx);

            for (ref key, ref value) in &node_value.attributes {
              let js_key_string = cx.string(&key);
              let js_value_string = cx.string(&value);

              js_attributes_obj
                .set(cx, js_key_string, js_value_string)
                .unwrap();
            }

            js_entry_obj.set(cx, "$", js_attributes_obj).unwrap();
          }

          if let Some(js_content_value) = content {
            js_entry_obj.set(cx, "_", js_content_value).unwrap();
          }

          collect_children_into(cx, &mut node_value.children, &mut js_entry_obj);

          js_child_values.set(cx, i as u32, js_entry_obj).unwrap();
        }

        let js_child_name = cx.string(&name);
        js_object.set(cx, js_child_name, js_child_values).unwrap();
      }
    }
  }
}

pub fn parse(mut cx: FunctionContext) -> JsResult<JsObject> {
  let xml_string = cx.argument::<JsString>(0)?.value();
  let mut root_node =
    XmlNode::from_str(&xml_string).expect("Failed to create XML Node from given string");
  let mut js_object = JsObject::new(&mut cx);
  collect_children_into(&mut cx, &mut root_node.children, &mut js_object);
  Ok(js_object)
}
