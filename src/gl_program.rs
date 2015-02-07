extern crate libc;

use std::rc::Rc;
use std::str;
use std::ptr;
use std::iter::*;

use color::*;
use vecmat::*;
use opengl::*;
use util::*;

pub use self::ShaderType::*;

pub struct Shader {
  pub id: GLuint,
}

impl !Send for Shader {}
impl !Sync for Shader {}

pub struct GLProgram {
  pub id: GLuint,
  pub vertex: Rc<Shader>,
  pub fragment: Rc<Shader>,
  pub frag_data_location: &'static str,
  pub attributes: Vec<(&'static str, i32)>,
}

impl Drop for Shader {
  fn drop(&mut self) {
    // println!("Dropping shader");
    unsafe {gl::DeleteShader(self.id);}
  }
}

impl Drop for GLProgram {
  fn drop(&mut self) {
    // println!("Dropping program");
    unsafe {gl::DeleteProgram(self.id);}
  }
}

pub enum ShaderType {VertexShader, FragmentShader}

impl ShaderType {
  fn as_gl(&self) -> GLenum {
    match *self {
      VertexShader => gl::VERTEX_SHADER,
      FragmentShader => gl::FRAGMENT_SHADER
    }
  }
}

// Shaders use reference counting because there may be multiple programs
// that use one shader.
impl Shader {
  // program_source is the location where the program was aquired, e.g. the filename
  pub fn new(source_code: &str, program_source: &str, typ: ShaderType) -> Rc<Shader> {
    let shader = unsafe {gl::CreateShader(typ.as_gl())};
    unsafe {
      gl::ShaderSource(shader, 1, &c_str_from_slice(source_code), ptr::null());
      gl::CompileShader(shader);

      let mut status = false as GLint;
      gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

      if status != true as GLint {
        let mut len = 0;
        gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
        let mut buf: Vec<_> = repeat(0).take(len as usize - 1).collect();
        gl::GetShaderInfoLog(shader, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
        panic!("Error in program '{}': {}", program_source, str::from_utf8(buf.as_slice()).ok().expect("GetShaderInfoLog not valid UTF8"));
      }
    }
    Rc::new(Shader {id: shader})
  }

  pub fn from_file(path: &Path, typ: ShaderType) -> Rc<Shader> {
    let contents = read_file(path);
    Shader::new(contents.as_slice(), path.as_str().unwrap(), typ)
  }
}

impl GLProgram {
  pub fn new(vertex_shader: Rc<Shader>, fragment_shader: Rc<Shader>,
        frag_data_location: &'static str,
        attributes: Vec<(&'static str, i32)>) -> Rc<GLProgram> {
    let program = unsafe {gl::CreateProgram()};
    unsafe {
      gl::AttachShader(program, vertex_shader.id);
      gl::AttachShader(program, fragment_shader.id);
      gl::LinkProgram(program);

      let mut status = false as GLint;
      gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

      if status != true as GLint {
        let mut len: GLint = 0;
        gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
        let mut buf: Vec<_> = repeat(0).take(len as usize - 1).collect();
        gl::GetProgramInfoLog(program, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
        panic!("{}", str::from_utf8(buf.as_slice()).ok().expect("GetProgramInfoLog not valid UTF8"));
      }
    }

    Rc::new(GLProgram {id: program, vertex: vertex_shader, fragment: fragment_shader, frag_data_location: frag_data_location, attributes: attributes})
  }

  pub fn bind(&self) {
    unsafe {gl::UseProgram(self.id);}
  }

  pub fn init(&self) {
    self.bind();
    unsafe {
      gl::BindFragDataLocation(self.id, 0, c_str_from_slice(self.frag_data_location));
    }

    let stride = self.attributes.iter().map(|&(_,size)| size).fold(0, |sum, x| sum+x);
    let mut offsets = vec![];
    let mut offset = 0;
    for &(_, size) in self.attributes.iter() {
      offsets.push(offset);
      offset += size;
    }

    unsafe {
      for i in range(0, self.attributes.len()) {
        let (attr, size) = self.attributes[i];
        let offset = offsets[i];
        let gl_attr = gl::GetAttribLocation(self.id, c_str_from_slice(attr));
        gl::EnableVertexAttribArray(gl_attr as GLuint);
        gl::VertexAttribPointer(gl_attr as GLuint, size as i32, gl::FLOAT, false as GLboolean, stride as i32*4, (offset*4) as (*const libc::types::common::c95::c_void));
      }
    }
  }
}


pub struct Mat4Uniform {
  pub id: GLint,
  pub program: Rc<GLProgram>,
}

impl Mat4Uniform {
  pub fn new(name: &str, program: Rc<GLProgram>) -> Mat4Uniform {
    program.bind(); // IDK if this is needed
    let id = unsafe {
      gl::GetUniformLocation(program.id, c_str_from_slice(name))
    };
    Mat4Uniform {id: id, program: program}
  }

  pub fn set(&self, mat: Mat4<f32>) {
    self.program.bind();
    unsafe {
      gl::UniformMatrix4fv(self.id, 1, false as GLboolean, &mat.m00 as *const f32);
    }
  }
}


pub struct ColorUniform {
  pub id: GLint,
  pub program: Rc<GLProgram>,
}

impl ColorUniform {
  pub fn new(name: &str, program: Rc<GLProgram>) -> ColorUniform {
    program.bind(); // IDK if this is needed
    let id = unsafe {
      gl::GetUniformLocation(program.id, c_str_from_slice(name))
    };
    ColorUniform {id: id, program: program}
  }

  pub fn set(&self, color: Color<f32>) {
    self.program.bind();
    unsafe {gl::Uniform4f(self.id, color.r, color.g, color.b, color.a);}
  }
}


pub struct FloatUniform {
  pub id: GLint,
  pub program: Rc<GLProgram>,
}

impl FloatUniform {
  pub fn new(name: &str, program: Rc<GLProgram>) -> FloatUniform {
    program.bind(); // IDK if this is needed
    let id = unsafe {
      gl::GetUniformLocation(program.id, c_str_from_slice(name))
    };
    FloatUniform {id: id, program: program}
  }

  pub fn set(&self, val: f32) {
    self.program.bind();
    unsafe {gl::Uniform1f(self.id, val);}
  }
}

pub struct UIntUniform {
  pub id: GLint,
  pub program: Rc<GLProgram>,
}

impl UIntUniform {
  pub fn new(name: &str, program: Rc<GLProgram>) -> UIntUniform {
    program.bind(); // IDK if this is needed
    let id = unsafe {
      gl::GetUniformLocation(program.id, c_str_from_slice(name))
    };
    UIntUniform {id: id, program: program}
  }

  pub fn set(&self, val: u32) {
    self.program.bind();
    unsafe {gl::Uniform1ui(self.id, val);}
  }
}


pub struct Vec2Uniform {
  pub id: GLint,
  pub program: Rc<GLProgram>,
}

impl Vec2Uniform {
  pub fn new(name: &str, program: Rc<GLProgram>) -> Vec2Uniform {
    program.bind(); // IDK if this is needed
    let id = unsafe {
      gl::GetUniformLocation(program.id, c_str_from_slice(name))
    };
    Vec2Uniform {id: id, program: program}
  }

  pub fn set(&self, vec:Vec2<f32>) {
    self.program.bind();
    unsafe {gl::Uniform2f(self.id, vec.x, vec.y);}
  }
}


pub struct Vec3Uniform {
  pub id: GLint,
  pub program: Rc<GLProgram>,
}

impl Vec3Uniform {
  pub fn new(name: &str, program: Rc<GLProgram>) -> Vec3Uniform {
    program.bind(); // IDK if this is needed
    let id = unsafe {
      gl::GetUniformLocation(program.id, c_str_from_slice(name))
    };
    Vec3Uniform {id: id, program: program}
  }

  pub fn set(&self, vec:Vec3<f32>) {
    self.program.bind();
    unsafe {gl::Uniform3f(self.id, vec.x, vec.y, vec.z);}
  }
}


pub struct Vec4Uniform {
  pub id: GLint,
  pub program: Rc<GLProgram>,
}

impl Vec4Uniform {
  pub fn new(name: &str, program: Rc<GLProgram>) -> Vec4Uniform {
    program.bind(); // IDK if this is needed
    let id = unsafe {
      gl::GetUniformLocation(program.id, c_str_from_slice(name))
    };
    Vec4Uniform {id: id, program: program}
  }

  pub fn set(&self, vec:Vec4<f32>) {
    self.program.bind();
    unsafe {gl::Uniform4f(self.id, vec.x, vec.y, vec.z, vec.w);}
  }
}
