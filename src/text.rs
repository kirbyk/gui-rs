extern crate freetype;

use std::rc::Rc;
use std::collections::hash_map::*;
use std::cell::RefCell;

use vecmat::*;
use opengl::*;
use texture::*;
use framebuffer::*;
use mesh::*;
use gl_program::*;
use color::*;

/// Note: only create one of these or it might behave strangely
pub struct FontLoader {
  freetype: freetype::Library,
}

impl FontLoader {
  pub fn new() -> FontLoader {
    FontLoader {freetype: freetype::Library::init().unwrap()}
  }
}

// TODO: Font should manage its own programs
impl Font_ {
  // TODO: should this be a method on FontLoader?
  fn new(loader: &FontLoader, path: &Path, size: i32, cache_program: Rc<GLProgram>, render_program: Rc<GLProgram>) -> Font_ {
    let mut face = match loader.freetype.new_face(path, 0) {
      Ok(face) => face,
      Err(err) => panic!("Unable to load font at {:?} because {:?}", path, err),
    };
    face.set_pixel_sizes(0, size as u32).unwrap();

    let size_metrics = unsafe {(*face.raw().size).metrics};
    let vert_advance = size_metrics.height / 64;
    let descender = size_metrics.descender / 64;

    let framebuffer = Framebuffer::new(1024, 1024, gl::RED, false);
    let cache_mesh = Mesh::new(cache_program.clone(), Primitive::Triangles, MeshUsage::StreamDraw);
    let render_mesh = Mesh::new(render_program.clone(), Primitive::Triangles, MeshUsage::StreamDraw);
    let cache_program_matrix_uni = Mat4Uniform::new("matrix", cache_program.clone());
    let render_program_matrix_uni = Mat4Uniform::new("matrix", render_program.clone());
    // TODO: I probably don't need to store the matrix uniforms
    cache_program_matrix_uni.set(Mat4::ortho(framebuffer.tex.size.x as f32, framebuffer.tex.size.y as f32));
    Font_ {face: face, size: size, vert_advance: vert_advance,
        descender: descender, framebuffer: framebuffer,
        glyphs: HashMap::new(), kerning: HashMap::new(), cur_x: 0, cur_y: 0, cache_mesh: cache_mesh, cache_program: cache_program, cache_program_matrix_uni: cache_program_matrix_uni, render_mesh: render_mesh, render_program: render_program, render_program_matrix_uni: render_program_matrix_uni}
  }

  fn load_glyph(&mut self, c: char) -> Glyph {
    self.face.load_char(c as usize, freetype::face::RENDER).unwrap();
    let glyph = self.face.glyph();
    let left = glyph.bitmap_left();
    let top = glyph.bitmap_top();
    let metrics = glyph.metrics();
    let advance_x = (metrics.horiAdvance/64) as i32;
    let bitmap = glyph.bitmap();

    let texture = Texture::texture2d_from_data(bitmap.buffer(), bitmap.width(), bitmap.rows(), gl::RED, gl::RED, MinNearest, MagNearest);

    Glyph {tex: texture, left: left, top: top, advance_x: advance_x}
  }

  fn get_kerning(&mut self, a: char, b: char) -> i32 {
    *match self.kerning.entry((a,b)) {
      Entry::Vacant(entry) => {
        let a_index = self.face.get_char_index(a as usize);
        let b_index = self.face.get_char_index(b as usize);
        let kerning = self.face.get_kerning(a_index, b_index, freetype::face::KerningMode::KerningDefault).unwrap().x / 64;
        entry.insert(kerning)
      }
      Entry::Occupied(entry) => entry.into_mut()
    }
  }

  fn cache_glyph(&mut self, c: char) {
    if !self.glyphs.contains_key(&c) {
      let glyph = self.load_glyph(c);
      let (x,y) = (glyph.tex.size.x, glyph.tex.size.y);
      let next_line = self.cur_x + glyph.tex.size.x >= self.framebuffer.tex.size.x;
      let x = if next_line {0} else {self.cur_x};
      let y = if next_line {self.cur_y + self.vert_advance} else {self.cur_y};
      if y >= self.framebuffer.tex.size.y {
        panic!("Font texture atlas full.");
      }

      unsafe {gl::BlendFunc(gl::ONE, gl::ONE);}
      let old_viewport = self.framebuffer.begin_render();

      self.cache_mesh.clear();
      add_vertex_text_2(&mut self.cache_mesh, x as f32, y as f32, 0.0, 0.0);
      add_vertex_text_2(&mut self.cache_mesh, (x+glyph.tex.size.x) as f32, y as f32, 1.0, 0.0);
      add_vertex_text_2(&mut self.cache_mesh, (x+glyph.tex.size.x) as f32, (y+glyph.tex.size.y) as f32, 1.0, 1.0);
      add_vertex_text_2(&mut self.cache_mesh, x as f32, (y+glyph.tex.size.y) as f32, 0.0, 1.0);
      self.cache_mesh.triangle(0, 2, 1);
      self.cache_mesh.triangle(2, 0, 3);

      glyph.tex.bind(0);
      // TODO: this won't work for GUI Code
      /*DepthTest.disable();
      CullFace.disable();*/
      self.cache_mesh.draw();
      /*DepthTest.enable();
      CullFace.enable();*/

      // TODO: store old blendfunc and restore it?

      self.framebuffer.end_render(old_viewport);
      unsafe {gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);}

      self.cur_x = x + glyph.tex.size.x;
      self.cur_y  = y;
      let size = glyph.tex.size;
      let cached_glyph = CachedGlyph {loc: Vec2(x,y), size: size, left: glyph.left, top: glyph.top, advance_x: glyph.advance_x};
      self.glyphs.insert(c, cached_glyph);
    }
  }

  fn get_cached_glyph(&self, c: char) -> CachedGlyph {
    match self.glyphs.get(&c) {
      None => panic!("Glyph not in cache"),
      Some(glyph) => *glyph
    }
  }

  // TODO: add support for background colors
  // TODO: get rid of window_size parameter
  fn draw_string(&mut self, str: &str, loc: Vec2<i32>, color: Color<f32>, window_size: Vec2<i32>) {
    for c in str.chars() {self.cache_glyph(c);}
    self.framebuffer.tex.bind(0);

    // TODO: this won't work for GUI Code
    /*DepthTest.disable();
    CullFace.disable();*/
    self.render_program_matrix_uni.set(Mat4::ortho(window_size.x as f32, window_size.y as f32));
    let mut iterator_a = str.chars();
    let mut iterator_b = str.chars().skip(1);
    let mut advance = 0;
    loop {
      match iterator_a.next() {
        None => break,
        Some(a) => {
          let glyph = self.get_cached_glyph(a);

          let loc = Vec2(loc.x + 1, -loc.y + window_size.y - self.vert_advance + 2);

          let tex_start = glyph.loc;
          let tex_end = tex_start + glyph.size;
          let tex_start_x = (tex_start.x as f32) / self.framebuffer.tex.size.x as f32;
          let tex_end_y = (tex_start.y as f32) / self.framebuffer.tex.size.y as f32;
          let tex_end_x = (tex_end.x as f32) / self.framebuffer.tex.size.x as f32;
          let tex_start_y = (tex_end.y as f32) / self.framebuffer.tex.size.y as f32;

          let size = glyph.size;

          self.render_mesh.clear();
          add_vertex_text(&mut self.render_mesh, (loc.x+advance+glyph.left) as f32, (loc.y-size.y+glyph.top) as f32, tex_start_x, tex_start_y, color);
          add_vertex_text(&mut self.render_mesh, (loc.x+advance+size.x+glyph.left) as f32, (loc.y-size.y+glyph.top) as f32, tex_end_x, tex_start_y, color);
          add_vertex_text(&mut self.render_mesh, (loc.x+advance+size.x+glyph.left) as f32, (loc.y+glyph.top) as f32, tex_end_x, tex_end_y, color);
          add_vertex_text(&mut self.render_mesh, (loc.x+advance+glyph.left) as f32, (loc.y+glyph.top) as f32, tex_start_x, tex_end_y, color);
          self.render_mesh.triangle(0, 2, 1);
          self.render_mesh.triangle(2, 0, 3);
          self.render_mesh.draw();

          match iterator_b.next() {
            None => (),
            Some(b) => advance += self.horiz_advance_between(a, b)
          }
        }
      }
    }
    /*DepthTest.enable();
    CullFace.enable();*/
  }

  fn horiz_advance_between(&mut self, a: char, b: char) -> i32 {
    let kerning = self.get_kerning(a, b);
    let glyph = self.get_cached_glyph(a);
    glyph.advance_x + kerning
  }

  fn horiz_advance_after(&mut self, a: char) -> i32 {
    let glyph = self.get_cached_glyph(a);
    glyph.advance_x
  }

  fn string_width(&mut self, str: &str) -> i32 {
    for c in str.chars() {self.cache_glyph(c);}
    if str.is_empty() {return 0}

    let mut width = 0;
    // AFAIK, there's no easier way to do what I want to do here.
    let mut iterator_a = str.chars();
    let mut iterator_b = str.chars().skip(1);
    loop {
      match iterator_b.next() {
        None => {
          width += self.horiz_advance_after(iterator_a.next().unwrap());
          break;
        }
        Some(b) => {
          let a = iterator_a.next().unwrap();
          width += self.horiz_advance_between(a, b);
        }
      }
    }
    /*for i in range(0, str.len()) {
      if i == str.len() - 1 {width += self.horiz_advance_after(str[i]);}
      else {width += self.horiz_advance_between(str[i], str[i+1]);}
    }*/
    width
  }

  fn string_size(&mut self, str: &str) -> Vec2<i32> {
    Vec2(self.string_width(str), self.vert_advance)
  }
}

struct Font_ {
  face: freetype::Face,
  size: i32,
  vert_advance: i32,
  descender: i32,
  framebuffer: Framebuffer,
  glyphs: HashMap<char, CachedGlyph>,
  kerning: HashMap<(char,char), i32>,
  cur_x: i32,
  cur_y: i32,
  cache_mesh: Mesh,
  cache_program: Rc<GLProgram>,
  cache_program_matrix_uni: Mat4Uniform,
  render_mesh: Mesh,
  render_program: Rc<GLProgram>,
  render_program_matrix_uni: Mat4Uniform,
}

#[derive(Debug, Copy)]
struct CachedGlyph {
  loc: Vec2<i32>,
  size: Vec2<i32>,
  left: i32,
  top: i32,
  advance_x: i32,
}

struct Glyph {
  tex: Texture,
  left: i32,
  top: i32,
  advance_x: i32,
}

#[derive(Clone)]
pub struct Font {
  inner: Rc<RefCell<Font_>>,
}

// A wrapper that exposes a public interface without &mut self
impl Font {
  pub fn new(loader: &FontLoader, path: &Path, size: i32, cache_program: Rc<
    GLProgram>, render_program: Rc<GLProgram>) -> Font {
    Font{inner: Rc::new(RefCell::new(Font_::new(loader, path, size, cache_program, render_program)))}
  }

  pub fn draw_string(&self, str: &str, loc: Vec2<i32>, color: Color<f32>, window_size: Vec2<i32>) {
    self.inner.borrow_mut().draw_string(str, loc, color, window_size);
  }
  pub fn horiz_advance_between(&self, a: char, b: char) -> i32 {
    self.inner.borrow_mut().horiz_advance_between(a, b)
  }
  pub fn horiz_advance_after(&self, a: char) -> i32 {
    self.inner.borrow_mut().horiz_advance_after(a)
  }
  pub fn string_width(&self, str: &str) -> i32 {
    self.inner.borrow_mut().string_width(str)
  }
  pub fn string_size(&self, str: &str) -> Vec2<i32> {
    self.inner.borrow_mut().string_size(str)
  }
  /// The recommended vertical distance between lines of text
  pub fn vert_advance(&self) -> i32 {
    self.inner.borrow().vert_advance
  }
  /// The distance from the baseline to the lowest point on the lowest char in the font; probably negative
  pub fn descender(&self) -> i32 {
    self.inner.borrow().descender
  }
}
