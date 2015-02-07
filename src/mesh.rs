extern crate libc;

use std::mem;
use std::rc::Rc;

use opengl::*;
use gl_program::*;
use color::*;
use vecmat::*;

use self::Primitive::*;
use self::MeshUsage::*;

// TODO: switch to u16 to save memory; mesh should panic if trying to add too many elements
pub type MeshIndex = u32;

pub enum MeshUsage {
  StaticDraw, DynamicDraw, StreamDraw
}

impl MeshUsage {
  fn as_gl(&self) -> GLuint {
    match *self {
      StaticDraw => gl::STATIC_DRAW,
      DynamicDraw => gl::DYNAMIC_DRAW,
      StreamDraw => gl::STREAM_DRAW,
    }
  }
}



pub enum Primitive {
  Triangles, Lines, Points
}

impl Primitive {
  fn as_gl(&self) -> GLuint {
    match *self {
      Triangles => gl::TRIANGLES,
      Lines => gl::LINES,
      Points => gl::POINTS,
    }
  }
}


/*
trait Vertex {
  fn write_to(mesh: &mut Mesh2<Self>);
}

impl Vertex for Vec2<f32> {

}


type Vertex = Vec2<f32>;
  fn add_vertex(&mut self, vertex: Vec2<f32>) -> MeshIndex {
    let index = self.inner.cur_index;
    self.inner.cur_index += 1;
    self.inner.vertex_data(vertex.x);
    self.inner.vertex_data(vertex.y);
    index
  }



struct GenericMesh {
  pub vao: GLuint,
  pub vbo: GLuint,
  pub ibo: GLuint,
  pub vertices: Vec<f32>,
  pub indices: Vec<MeshIndex>,
  primitive: GLuint,
  usage: GLuint,
  pub updated: bool,
  pub program: Rc<GLProgram>,
  pub cur_index: MeshIndex, // TODO: this shouldn't be updated by main
  pub num_indices: i32,
}

impl Drop for GenericMesh {
  fn drop(&mut self) {
    unsafe {
      // println!("Dropping mesh");
      gl::DeleteBuffers(1, &self.vbo);
      gl::DeleteBuffers(1, &self.ibo);
      gl::DeleteVertexArrays(1, &self.vao);
    }
  }
}



struct Mesh2<T: Vertex> {
  inner: GenericMesh,
}
*/


/*impl Mesh<V: Vertex, U: Uniforms> {
  fn add_vertex(vertex: V) {...}

  fn draw(&mut self, uniforms: U) {...}
}

*/
/*
trait Mesh2 {
  fn inner(&mut self) -> &mut GenericMesh;

  fn print(&mut self) {self.inner().print()}
  fn clear(&mut self) {self.inner().clear()}
  fn update(&mut self) {self.inner().update()}
  fn index(&mut self, a: MeshIndex) {self.inner().index(a)}
  fn triangle(&mut self, a: MeshIndex, b: MeshIndex, c: MeshIndex) {
    self.inner().triangle(a, b, c);
  }
  fn draw(&mut self) {self.inner().draw()}

  type Vertex;
  fn add_vertex(&mut self, vertex: Self::Vertex) -> MeshIndex;
}


struct UntexturedMesh {
  inner: GenericMesh,
}

impl UntexturedMesh {
  pub fn new(program: Rc<GLProgram>, primitive: Primitive, usage: MeshUsage) -> UntexturedMesh {
    UntexturedMesh{inner: GenericMesh::new(program, primitive, usage)}
  }
}

impl Mesh2 for UntexturedMesh {
  fn inner(&mut self) -> &mut GenericMesh {&mut self.inner}

  type Vertex = Vec2<f32>;
  fn add_vertex(&mut self, vertex: Vec2<f32>) -> MeshIndex {
    let index = self.inner.cur_index;
    self.inner.cur_index += 1;
    self.inner.vertex_data(vertex.x);
    self.inner.vertex_data(vertex.y);
    index
  }
}
*/













pub struct Mesh {
  pub vao: GLuint,
  pub vbo: GLuint,
  pub ibo: GLuint,
  pub vertices: Vec<f32>,
  pub indices: Vec<MeshIndex>,
  primitive: GLuint,
  usage: GLuint,
  pub updated: bool,
  pub program: Rc<GLProgram>,
  pub cur_index: MeshIndex, // TODO: this shouldn't be updated by main
  pub num_indices: i32,
}

impl Mesh {
  pub fn new(program: Rc<GLProgram>, primitive: Primitive, usage: MeshUsage) -> Mesh {
    let mut vao = 0;
    let mut vbo = 0;
    let mut ibo = 0;
    unsafe {
      gl::GenVertexArrays(1, &mut vao);
      gl::BindVertexArray(vao);
      gl::GenBuffers(1, &mut vbo);
      gl::GenBuffers(1, &mut ibo);
    }

    Mesh {vao: vao, vbo: vbo, ibo: ibo, vertices: vec![], indices: vec![], primitive: primitive.as_gl(), usage: usage.as_gl(), updated: false, program: program, cur_index: 0, num_indices: 0}
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
            mem::transmute(&self.vertices[0]), self.usage); //TODO: what does transmute do?
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
  pub fn vertex_data(&mut self, x: f32) {
    self.vertices.push(x);
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

  pub fn draw(&mut self) {
    if self.num_indices == 0 {
      return;
    }
    self.program.bind();
    self.update();
    self.bind();
    unsafe {
      gl::DrawElements(self.primitive, self.num_indices, gl::UNSIGNED_INT, 0 as *const libc::types::common::c95::c_void);
    }
  }
}

impl Drop for Mesh {
  fn drop(&mut self) {
    unsafe {
      // println!("Dropping mesh");
      gl::DeleteBuffers(1, &self.vbo);
      gl::DeleteBuffers(1, &self.ibo);
      gl::DeleteVertexArrays(1, &self.vao);
    }
  }
}


pub fn add_vertex_unlit_2d(mesh: &mut Mesh, x: f32, y: f32, tex_x: f32, tex_y: f32) {
  mesh.cur_index += 1;
  mesh.vertex_data(x);
  mesh.vertex_data(y);
  mesh.vertex_data(tex_x);
  mesh.vertex_data(tex_y);
}
pub fn add_vertex_text(mesh: &mut Mesh, x: f32, y: f32, tex_x: f32, tex_y: f32, color:Color<f32>) {
  mesh.cur_index += 1;
  mesh.vertex_data(x);
  mesh.vertex_data(y);
  mesh.vertex_data(tex_x);
  mesh.vertex_data(tex_y);
  mesh.vertex_data(color.r);
  mesh.vertex_data(color.g);
  mesh.vertex_data(color.b);
  mesh.vertex_data(color.a);
}
pub fn add_vertex_text_2(mesh: &mut Mesh, x: f32, y: f32, tex_x: f32, tex_y: f32) {
  mesh.cur_index += 1;
  mesh.vertex_data(x);
  mesh.vertex_data(y);
  mesh.vertex_data(tex_x);
  mesh.vertex_data(tex_y);
}

#[inline]
pub fn add_vertex_untextured(mesh: &mut Mesh, pos: Vec2<f32>, color: Color<f32>) -> MeshIndex {
  let index = mesh.cur_index;
  mesh.cur_index += 1;
  mesh.vertex_data(pos.x);
  mesh.vertex_data(pos.y);
  mesh.vertex_data(color.r);
  mesh.vertex_data(color.g);
  mesh.vertex_data(color.b);
  mesh.vertex_data(color.a);
  index
}





/*pub fn add_quad_unlit(mesh: &mut Mesh, pos: Vec2<f32>) {
  let start_index = mesh.cur_index;
  add_vertex_unlit(mesh, pos.x-block_size_half_f32, pos.y-block_size_half_f32, pos.z+block_size_half_f32, 1.0, 1.0);
  add_vertex_unlit(mesh, pos.x+block_size_half_f32, pos.y-block_size_half_f32, pos.z+block_size_half_f32, 0.0, 1.0);
  add_vertex_unlit(mesh, pos.x+block_size_half_f32, pos.y+block_size_half_f32, pos.z+block_size_half_f32, 0.0, 0.0);
  add_vertex_unlit(mesh, pos.x-block_size_half_f32, pos.y+block_size_half_f32, pos.z+block_size_half_f32, 1.0, 0.0);

  mesh.triangle(start_index+0, start_index+1, start_index+2);
  mesh.triangle(start_index+2, start_index+3, start_index+0);
}
*/
