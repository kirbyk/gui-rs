extern crate glfw;
extern crate freetype;
extern crate image;

use std::rc::Rc;
use std::collections::hash_map::*;
use std::num::Float;

use image::DynamicImage;
use image::GenericImage;

use vecmat::*;
use text::*;
use texture::*;
use gl_program::*;
use util::*;
use color::*;
use mesh::*;
use gui::*;
use new_gl_program::*;

// TODO: background color
pub struct ButtonWidget {
  font: Font,
  text: String,
  text_color: Color<f32>,
  id: Id,
  was_pressed: bool,
}

impl ButtonWidget {
  pub fn new(font: Font, text: &str, text_color: Color<f32>) -> ButtonWidget {
    ButtonWidget{font: font, text: text.to_string(), text_color: text_color, id: next_id(), was_pressed: false}
  }

  pub fn text(&self) -> &str {self.text.as_slice()}
  pub fn set_text(&mut self, text: &str) {self.text = text.to_string();}

  pub fn was_pressed(&self) -> bool {self.was_pressed}
}

impl Widget for ButtonWidget {
  fn id(&self) -> Id {self.id}

  fn draw(&mut self, pos: Vec2<i32>, size: Vec2<i32>, window: &mut GUIWindow) {
    let window_size = window.window_size;
    self.font.draw_string(self.text.as_slice(), pos, self.text_color, window_size);

    // TODO: move this somewhere else
    self.was_pressed = false;
  }

  fn min_size(&self, window: &mut GUIWindow) -> Vec2<i32> {
    self.font.string_size(self.text.as_slice())
  }

  fn handle_event(&mut self, event: Event, window: &mut GUIWindow) {
    match event {
      Event::MouseButton(glfw::MouseButtonLeft, Action::Press, _, _) => self.was_pressed = true,
      _ => ()
    }
  }
}

// TODO: background color
pub struct LabelWidget {
  font: Font,
  text: String,
  text_color: Color<f32>,
  id: Id,
}

// TODO: more getters/setters, also for buttons
impl LabelWidget {
  pub fn new(font: Font, text: &str, text_color: Color<f32>) -> LabelWidget {
    LabelWidget{font: font, text: text.to_string(), text_color: text_color, id: next_id()}
  }

  pub fn text(&self) -> &str {self.text.as_slice()}
  pub fn set_text(&mut self, text: &str) {self.text = text.to_string();}
}

impl Widget for LabelWidget {
  fn id(&self) -> Id {self.id}

  fn draw(&mut self, pos: Vec2<i32>, size: Vec2<i32>, window: &mut GUIWindow) {
    let window_size = window.window_size;
    self.font.draw_string(self.text.as_slice(), pos, self.text_color, window_size);
  }

  fn min_size(&self, window: &mut GUIWindow) -> Vec2<i32> {
    self.font.string_size(self.text.as_slice())
  }
}

pub struct EmptyWidget {
  min_size: Vec2<i32>,
  id: Id,
}

impl EmptyWidget {
  pub fn new(min_size: Vec2<i32>) -> EmptyWidget {
    EmptyWidget{min_size: min_size, id: next_id()}
  }
}
impl Widget for EmptyWidget {
  fn id(&self) -> Id {self.id}

  fn draw(&mut self, pos: Vec2<i32>, size: Vec2<i32>, window: &mut GUIWindow) {}

  fn min_size(&self, window: &mut GUIWindow) -> Vec2<i32> {self.min_size}
}

#[derive(Debug)]
pub enum ImageResizeMode {
  /// Fills the entire container, even changing the aspect ratio
  FillContainer,
  /// Fills as much of the container as possible while maintaining the
  /// correct aspect ratio
  /// The parameter is whether to allow resizing images to larger than their native size
  KeepAspect{resize_smaller: bool},
  /// Always draws the image at its native size. If the widget is larger
  /// than necessary, puts it in the center.
  NativeSize,
}

pub struct ImageWidget {
  image_fixture: ImageFixture,
  pub resize_mode: ImageResizeMode,
  id: Id,
}

impl ImageWidget {
  pub fn new(image: DynamicImage, resize_mode: ImageResizeMode) -> ImageWidget {
    let size = image_size(&image);
    let image_fixture = ImageFixture::new(image, Rect(Vec2(0, 0), Vec2(0, 0)), Rect(Vec2(0, 0), size));
    ImageWidget{image_fixture: image_fixture, resize_mode: resize_mode, id: next_id()}
  }

  pub fn set_image(&mut self, image: DynamicImage) {
    let size = image_size(&image);
    self.image_fixture.set_image(image);
    self.image_fixture.set_source_rect(Rect(Vec2(0, 0), size));
  }
}

impl Widget for ImageWidget {
  fn id(&self) -> Id {self.id}

  fn draw(&mut self, pos: Vec2<i32>, size: Vec2<i32>, window: &mut GUIWindow) {
    let img_size = image_size(self.image_fixture.image());
    match self.resize_mode {
      ImageResizeMode::FillContainer => {
        self.image_fixture.set_rect(Rect(pos, pos+size));
      },
      ImageResizeMode::KeepAspect{resize_smaller} => {
        let w_ratio = size.x as f32 / img_size.x as f32;
        let h_ratio = size.y as f32 / img_size.y as f32;
        let ratio_ = w_ratio.min(h_ratio);
        let ratio = if resize_smaller || ratio_ < 1.0 {ratio_} else {1.0};
        let center = pos.cvt::<Vec2<f32>>() + size.cvt::<Vec2<f32>>() * 0.5;
        let display_size = img_size.cvt::<Vec2<f32>>() * ratio;
        self.image_fixture.set_rect(Rect((center - display_size * 0.5).cvt::<Vec2<i32>>(), (center + display_size * 0.5).cvt::<Vec2<i32>>()));
      },
      ImageResizeMode::NativeSize => {
        let excess_size = size - img_size;
        let offset = excess_size / 2;
        self.image_fixture.set_rect(Rect(pos + offset, pos + offset + img_size));
      },
    }
    self.image_fixture.draw(window);
  }

  fn min_size(&self, window: &mut GUIWindow) -> Vec2<i32> {
    match self.resize_mode {
      ImageResizeMode::NativeSize => image_size(self.image_fixture.image()),
      _ => Vec2::zero()
    }
  }
}




pub struct BorderWidget<T: Widget> {
  pub inner: T,
  pub border_color: Color<f32>,
  border_width: i32,
}

impl<T: Widget> BorderWidget<T> {
  pub fn new(inner: T, border_color: Color<f32>, border_width: i32) -> BorderWidget<T> {
    BorderWidget{inner: inner, border_color: border_color, border_width: border_width}
  }
}

impl<T: Widget> Widget for BorderWidget<T> {
  fn id(&self) -> Id {self.inner.id()}

  fn draw(&mut self, pos: Vec2<i32>, size: Vec2<i32>, window: &mut GUIWindow) {
    self.inner.draw(pos + Vec2(self.border_width,self.border_width), size - Vec2(self.border_width, self.border_width)*2, window);
    // TODO
  }

  fn min_size(&self, window: &mut GUIWindow) -> Vec2<i32> {
    self.inner.min_size(window) + Vec2(self.border_width, self.border_width)*2
  }

  fn handle_event(&mut self, event: Event, window: &mut GUIWindow) {
    self.inner.handle_event(event, window);
  }
}



pub struct FocusTestWidget {
  id: Id,
  font: Font,
}

impl FocusTestWidget {
  pub fn new(font: Font) -> FocusTestWidget {
    FocusTestWidget{font: font, id: next_id()}
  }
}

impl Widget for FocusTestWidget {
  fn id(&self) -> Id {self.id}

  fn draw(&mut self, pos: Vec2<i32>, size: Vec2<i32>, window: &mut GUIWindow) {
    let window_size = window.window_size;
    let focused = window.focused == Some(self.id);
    if focused {
      self.font.draw_string("Focused!", pos, Color::red(), window_size);
    } else {
      self.font.draw_string("Not focused!", pos, Color::blue(), window_size);
    }
  }

  fn min_size(&self, window: &mut GUIWindow) -> Vec2<i32> {
    Vec2(100, 100)
  }

  fn handle_event(&mut self, event: Event, window: &mut GUIWindow) {
    match event {
      Event::MouseButton(glfw::MouseButtonLeft, Action::Press, _, _) => window.focused = Some(self.id),
      _ => ()
    }
  }
}









pub struct ImageFixture {
  image: DynamicImage,
  rect: Rect<i32>,
  source_rect: Rect<i32>,
  // TODO: do these need to be regenerated when the OpenGL context changes?
  // TODO: remove textures, meshes, etc for windows that have been closed
  entries: HashMap<Id, ImageFixtureEntry>
}

struct ImageFixtureEntry {
  texture: Option<Texture>,
  mesh: NewMesh<UnlitProgram>,
  mesh_dirty: bool,
  program: Rc<UnlitProgram>,
}

impl ImageFixture {
  pub fn new(image: DynamicImage, rect: Rect<i32>, source_rect: Rect<i32>) -> ImageFixture {
    ImageFixture{image: image, rect: rect, source_rect: source_rect, entries: HashMap::new()}
  }

  /*pub fn new(image: DynamicImage, rect: Rect<i32>, source_rect: Rect<i32>, program: Rc<GLProgram>) -> ImageFixture {
    let mesh = Mesh::new(program.clone(), Triangles, DynamicDraw);
    ImageFixture {image: image, rect: rect, source_rect: source_rect, texture: None, mesh: mesh, mesh_dirty: true, program: program}
  }*/

  pub fn image(&self) -> &DynamicImage {&self.image}
  pub fn rect(&self) -> Rect<i32> {self.rect}
  pub fn source_rect(&self) -> Rect<i32> {self.source_rect}
  pub fn set_image(&mut self, image: DynamicImage) {
    self.image = image;
    for (_,entry) in self.entries.iter_mut() {
      entry.texture = None;
    }
  }
  pub fn set_rect(&mut self, rect: Rect<i32>) {
    if rect != self.rect {
      self.rect = rect;
      for (_,entry) in self.entries.iter_mut() {
        entry.mesh_dirty = true;
      }
    }
  }
  pub fn set_source_rect(&mut self, source_rect: Rect<i32>) {
    if source_rect != self.source_rect {
      self.source_rect = source_rect;
      for (_,entry) in self.entries.iter_mut() {
        entry.mesh_dirty = true;
      }
    }
  }

  pub fn draw(&mut self, window: &mut GUIWindow) {
    let entry = match self.entries.entry(window.id) {
      Entry::Vacant(entry) => {
        let mesh = NewMesh::new(window.unlit_program.clone(), Primitive::Triangles, MeshUsage::DynamicDraw);
        entry.insert(ImageFixtureEntry{texture: None, mesh: mesh, mesh_dirty: true, program: window.unlit_program.clone()})
      }
      Entry::Occupied(entry) => entry.into_mut(),
    };

    if entry.texture.is_none() {
      entry.texture = Some(Texture::texture2d_from_image_nonsrgb(&self.image, MinLinear, MagLinear));
    }
    if entry.mesh_dirty {
      entry.mesh_dirty = false;
      entry.mesh.clear();
      let image_size = image_size(&self.image);
      let source_rect_scale = Vec2(1.0 / image_size.x as f32, 1.0 / image_size.y as f32);

      let start: Vec2<f32> = self.rect.start.cvt();
      let end: Vec2<f32> = self.rect.end.cvt();
      let scale = source_rect_scale;
      entry.mesh.add_vertex(UnlitVertex{
        pos: start,
        texcoord: start.component_mul(scale),
      });
      entry.mesh.add_vertex(UnlitVertex{
        pos: Vec2(end.x, start.y),
        texcoord: Vec2(end.x*scale.x, start.y*scale.y),
      });
      entry.mesh.add_vertex(UnlitVertex{
        pos: end,
        texcoord: end.component_mul(scale),
      });
      entry.mesh.add_vertex(UnlitVertex{
        pos: Vec2(start.x, end.y),
        texcoord: Vec2(start.x*scale.x, end.y*scale.y),
      });
      entry.mesh.triangle(0, 2, 1);
      entry.mesh.triangle(2, 0, 3);
    }

    match entry.texture {
      Some(ref texture) => {
        // texture.bind(0);
        entry.mesh.draw(UnlitUniforms{
          model_view_matrix: Mat4::ortho_flip(
            window.window_size.x as f32, window.window_size.y as f32),
          proj_matrix: Mat4::id(),
          tex: &texture
        });
      },
      None => panic!("")
    }
  }
}

// TODO: find a better alternative to this
pub fn image_size(image: &DynamicImage) -> Vec2<i32> {
  let (w, h) = image.dimensions();
  Vec2(w as i32, h as i32)
}
