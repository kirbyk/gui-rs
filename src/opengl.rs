pub use self::gl::types::*;

pub mod gl {
  include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}
