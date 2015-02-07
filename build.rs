#![feature(path, io, os)]

extern crate gl_generator;
extern crate khronos_api;

use std::os;
use std::old_io::File;

fn main() {
  let dest = Path::new(os::getenv("OUT_DIR").unwrap());
  let mut file = File::create(&dest.join("gl_bindings.rs")).unwrap();

  gl_generator::generate_bindings(
    gl_generator::GlobalGenerator,
    gl_generator::registry::Ns::Gl,
    khronos_api::GL_XML,
    vec![
      "GL_EXT_texture_filter_anisotropic".to_string(), "GL_EXT_texture_array".to_string(),
      "ARB_texture_cube_map".to_string(), "GL_ARB_framebuffer_object".to_string()
    ],
    "3.0", "core",
    &mut file).unwrap();

/*gl_generator::generate_bindings(gl_generator::GlobalGenerator,
  gl_generator::registry::Ns::Gl,
  khronos_api::GL_XML, vec![], "4.5", "core",
  &mut file).unwrap();*/

}
