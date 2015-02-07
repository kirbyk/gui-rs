/*#[phase(plugin)]
extern crate gl_generator;

pub use self::gl::types::*;

pub mod gl {
  generate_gl_bindings!(
    api: "gl",
    profile: "core",
    version: "3.0",
    generator: "global",
    extensions: [
      "GL_EXT_texture_filter_anisotropic", "GL_EXT_texture_array",
      "ARB_texture_cube_map", "GL_ARB_framebuffer_object"
    ]
  );
}
*/

pub use self::gl::types::*;

pub mod gl {
  include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}
