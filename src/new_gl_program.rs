extern crate libc;

use std::rc::Rc;
use std::mem;
use std::str;
use std::ptr;
use std::iter::*;

use color::*;
use vecmat::*;
use opengl::*;
use util::*;
use gl_program::*;
use mesh::*;


pub trait NewGLProgram {
  type Vertex;
  type Uniforms;

  fn inner(&self) -> &GenericGLProgram;
  fn set_uniforms(&self, uniforms: Self::Uniforms);
  fn add_vertex(&self, mesh: &mut NewMesh<Self>, vertex: Self::Vertex);

  // These don't need to be implemented
  fn bind(&self) {
    self.inner().bind();
  }

  fn init(&self) {
    self.inner().init();
  }

  fn id(&self) -> GLuint {
    self.inner().id
  }
}



pub struct GenericGLProgram {
  id: GLuint,
  vertex: Rc<Shader>,
  fragment: Rc<Shader>,
  frag_data_location: &'static str,
  attributes: Vec<(&'static str, i32)>,
}


impl GenericGLProgram {
  pub fn id(&self) -> GLuint {self.id}

  // TODO: get rid of Shader struct
  pub fn new(vert_shader: &str, frag_shader: &str,
        frag_data_location: &'static str,
        attributes: Vec<(&'static str, i32)>) -> Rc<GenericGLProgram> {
    let vertex_shader = Shader::new(vert_shader, "", VertexShader);
    let fragment_shader = Shader::new(frag_shader, "", FragmentShader);
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

    Rc::new(GenericGLProgram {id: program, vertex: vertex_shader, fragment: fragment_shader, frag_data_location: frag_data_location, attributes: attributes})
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



pub struct NewMat4Uniform {
  pub id: GLint,
  pub program: Rc<GenericGLProgram>,
}

impl NewMat4Uniform {
  pub fn new(name: &str, program: Rc<GenericGLProgram>) -> NewMat4Uniform {
    program.bind(); // IDK if this is needed
    let id = unsafe {
      gl::GetUniformLocation(program.id(), c_str_from_slice(name))
    };
    NewMat4Uniform {id: id, program: program}
  }

  pub fn set(&self, mat: Mat4<f32>) {
    self.program.bind();
    unsafe {
      gl::UniformMatrix4fv(self.id, 1, false as GLboolean, &mat.m00 as *const f32);
    }
  }
}


pub struct NewColorUniform {
  pub id: GLint,
  pub program: Rc<GenericGLProgram>,
}

impl NewColorUniform {
  pub fn new(name: &str, program: Rc<GenericGLProgram>) -> NewColorUniform {
    program.bind(); // IDK if this is needed
    let id = unsafe {
      gl::GetUniformLocation(program.id(), c_str_from_slice(name))
    };
    NewColorUniform {id: id, program: program}
  }

  pub fn set(&self, color: Color<f32>) {
    self.program.bind();
    unsafe {gl::Uniform4f(self.id, color.r, color.g, color.b, color.a);}
  }
}


pub struct NewFloatUniform {
  pub id: GLint,
  pub program: Rc<GenericGLProgram>,
}

impl NewFloatUniform {
  pub fn new(name: &str, program: Rc<GenericGLProgram>) -> NewFloatUniform {
    program.bind(); // IDK if this is needed
    let id = unsafe {
      gl::GetUniformLocation(program.id(), c_str_from_slice(name))
    };
    NewFloatUniform {id: id, program: program}
  }

  pub fn set(&self, val: f32) {
    self.program.bind();
    unsafe {gl::Uniform1f(self.id, val);}
  }
}

pub struct NewUIntUniform {
  pub id: GLint,
  pub program: Rc<GenericGLProgram>,
}

impl NewUIntUniform {
  pub fn new(name: &str, program: Rc<GenericGLProgram>) -> NewUIntUniform {
    program.bind(); // IDK if this is needed
    let id = unsafe {
      gl::GetUniformLocation(program.id(), c_str_from_slice(name))
    };
    NewUIntUniform {id: id, program: program}
  }

  pub fn set(&self, val: u32) {
    self.program.bind();
    unsafe {gl::Uniform1ui(self.id, val);}
  }
}


pub struct NewVec2Uniform {
  pub id: GLint,
  pub program: Rc<GenericGLProgram>,
}

impl NewVec2Uniform {
  pub fn new(name: &str, program: Rc<GenericGLProgram>) -> NewVec2Uniform {
    program.bind(); // IDK if this is needed
    let id = unsafe {
      gl::GetUniformLocation(program.id(), c_str_from_slice(name))
    };
    NewVec2Uniform {id: id, program: program}
  }

  pub fn set(&self, vec:Vec2<f32>) {
    self.program.bind();
    unsafe {gl::Uniform2f(self.id, vec.x, vec.y);}
  }
}


pub struct NewVec3Uniform {
  pub id: GLint,
  pub program: Rc<GenericGLProgram>,
}

impl NewVec3Uniform {
  pub fn new(name: &str, program: Rc<GenericGLProgram>) -> NewVec3Uniform {
    program.bind(); // IDK if this is needed
    let id = unsafe {
      gl::GetUniformLocation(program.id(), c_str_from_slice(name))
    };
    NewVec3Uniform {id: id, program: program}
  }

  pub fn set(&self, vec:Vec3<f32>) {
    self.program.bind();
    unsafe {gl::Uniform3f(self.id, vec.x, vec.y, vec.z);}
  }
}


pub struct NewVec4Uniform {
  pub id: GLint,
  pub program: Rc<GenericGLProgram>,
}

impl NewVec4Uniform {
  pub fn new(name: &str, program: Rc<GenericGLProgram>) -> NewVec4Uniform {
    program.bind(); // IDK if this is needed
    let id = unsafe {
      gl::GetUniformLocation(program.id(), c_str_from_slice(name))
    };
    NewVec4Uniform {id: id, program: program}
  }

  pub fn set(&self, vec:Vec4<f32>) {
    self.program.bind();
    unsafe {gl::Uniform4f(self.id, vec.x, vec.y, vec.z, vec.w);}
  }
}



pub struct NewMesh<P: NewGLProgram> {
  vao: GLuint,
  vbo: GLuint,
  ibo: GLuint,
  vertices: Vec<f32>,
  indices: Vec<MeshIndex>,
  primitive: GLuint,
  usage: GLuint,
  updated: bool,
  program: Rc<P>,
  cur_index: MeshIndex,
  num_indices: i32,
}


impl<P: NewGLProgram> NewMesh<P> {
  pub fn new(program: Rc<P>, primitive: Primitive, usage: MeshUsage) -> NewMesh<P> {
    let mut vao = 0;
    let mut vbo = 0;
    let mut ibo = 0;
    unsafe {
      gl::GenVertexArrays(1, &mut vao);
      gl::BindVertexArray(vao);
      gl::GenBuffers(1, &mut vbo);
      gl::GenBuffers(1, &mut ibo);
    }

    NewMesh {vao: vao, vbo: vbo, ibo: ibo, vertices: vec![], indices: vec![], primitive: primitive.as_gl(), usage: usage.as_gl(), updated: false, program: program, cur_index: 0, num_indices: 0}
  }

  pub fn print(&self) {
    for vert in self.vertices.iter() {
      println!("Vert: {}", vert);
    }
    for index in self.indices.iter() {
      println!("Index: {}", index);
    }
  }

  fn bind(&self) {
    unsafe {gl::BindVertexArray(self.vao);}
  }

  pub fn clear(&mut self) {
    self.cur_index = 0;
    self.num_indices = 0;
    self.updated = false;

    self.vertices.clear();
    // self.vertices.shrink_to_fit();
    self.indices.clear();
    // self.indices.shrink_to_fit();

  }

  pub fn update(&mut self) {
    if !self.updated {
      self.updated = true;
      if self.indices.is_empty() {
        return;
      }

      self.bind();
      unsafe {
        gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
        gl::BufferData(gl::ARRAY_BUFFER,
            (self.vertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
            mem::transmute(&self.vertices[0]), self.usage);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ibo);
        gl::BufferData(gl::ELEMENT_ARRAY_BUFFER,
            (self.indices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
            mem::transmute(&self.indices[0]), self.usage);
        self.vertices.clear();
        // self.vertices.shrink_to_fit();
        self.indices.clear();
        // self.indices.shrink_to_fit();

        self.program.init(); // TODO: does this need to be called every time the mesh is updated?
      }
    }
  }

  #[inline]
  pub fn vertex_data(&mut self, xs: &[f32]) {
    self.vertices.push_all(xs);
  }

  #[inline]
  pub fn index(&mut self, a: MeshIndex) {
    self.indices.push(a);
    self.num_indices += 1;
    self.updated = false;
  }

  pub fn triangle(&mut self, a: MeshIndex, b: MeshIndex, c: MeshIndex) {
    self.indices.push(a);
    self.indices.push(b);
    self.indices.push(c);

    self.num_indices += 3;

    self.updated = false;
  }

  pub fn draw(&mut self, uniforms: P::Uniforms) {
    if self.num_indices == 0 {
      return;
    }
    self.program.set_uniforms(uniforms);
    self.program.bind();
    self.update();
    self.bind();
    unsafe {
      gl::DrawElements(self.primitive, self.num_indices, gl::UNSIGNED_INT, 0 as *const libc::types::common::c95::c_void);
    }
  }

  pub fn add_vertex(&mut self, vertex: P::Vertex) -> MeshIndex {
    let index = self.cur_index;
    self.cur_index += 1;
    // TODO: can we avoid this clone?
    self.program.clone().add_vertex(self, vertex);
    index
  }
}
  
#[unsafe_destructor]
impl<P> Drop for NewMesh<P> {
  fn drop(&mut self) {
    unsafe {
      // println!("Dropping mesh");
      gl::DeleteBuffers(1, &self.vbo);
      gl::DeleteBuffers(1, &self.ibo);
      gl::DeleteVertexArrays(1, &self.vao);
    }
  }
}
