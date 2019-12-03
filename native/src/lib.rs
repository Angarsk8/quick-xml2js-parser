extern crate neon;

use neon::prelude::*;

mod xml_node;
mod xml_to_js_object;
mod xml_to_struct_to_js_object;

register_module!(mut module, {
  module.export_function("indirectParse", xml_to_struct_to_js_object::parse)?;
  module.export_function("directParse", xml_to_js_object::parse)?;
  Ok(())
});
