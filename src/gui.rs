/*
  Major differences from conrod:
    We allow users to define their own widgets; they use a trait, not an enum
    We use GLFW rather than SDL, meaning it's easier to get working on more platforms
    We have a layout system and don't have to specify the locations of widgets manually
  TODO:
    Handle click events on specific widgets
      Should their position be relative to the position of the widget?
    When holding a mouse button down, it should send a repeat event
*/

/*
  Each GLProgram object has zero or more associated uniforms, of various types. Each of these uniforms must be provided at each draw call. We shouldn't update OpenGL state unnecessarily - we need to keep track of the current values for the uniforms.
  Additionally, each GLProgram can only accept meshes with a certain configuration of values.
*/


extern crate glfw;
extern crate freetype;
extern crate image;

use std::rc::Rc;
use std::cmp;
use std::collections::hash_map::*;
use std::num::Float;
use std::num::FromPrimitive;

pub use self::Layout::*;
pub use self::LayoutMinSize::*;
pub use self::Alignment::*;

use glfw::MouseButton;
use glfw::WindowHint;
use glfw::Glfw;
use glfw::Window;
use glfw::Context;
use std::sync::mpsc::Receiver;

use vecmat::*;
use renderer::*;
use text::*;
use texture::*;
use opengl::*;
use gl_program::*;
use mesh::*;
use new_gl_program::*;
use util::*;
use color::*;

pub trait Widget {
  // Each widget must have a unique ID, generated from the global next_id() function
  // TODO: create a more thread-safe mechanism for generating Ids
  fn id(&self) -> Id;

  fn draw(&mut self, pos: Vec2<i32>, size: Vec2<i32>, window: &mut GUIWindow);

  fn min_size(&self, window: &mut GUIWindow) -> Vec2<i32> {Vec2(0,0)}
  fn handle_event(&mut self, event: Event, window: &mut GUIWindow) {}
}

impl<'a> PartialEq for Widget + 'a {
  fn eq(&self, other: &Widget) -> bool {
    self.id() == other.id()
  }
}

impl<'a> Eq for Widget + 'a {}







pub enum Alignment {Leading, Center, Trailing}

// TODO: implement grids
pub enum Layout<'a> {
  VPanel(Alignment, Vec<(Layout<'a>, f64)>),
  HPanel(Alignment, Vec<(Layout<'a>, f64)>),
  // TODO: allow alignment of children
  OverlapPanel(Vec<Layout<'a>>),
  //OverlapPanel(Vec<Layout<'a>),  // TODO
  LWidget(&'a mut (Widget + 'a)),
}

impl<'a> Layout<'a> {
  // Also adjusts the flexes to sum to 1
  fn into_layout_min_size(self, window: &mut GUIWindow) -> LayoutMinSize<'a> {
    match self {
      LWidget(widget) => {
        let min_size = widget.min_size(window);
        LWidget_(widget, min_size)
      }
      VPanel(alignment, children) => {
        let mut min_size = Vec2::zero();
        let mut new_children = Vec::new();
        let mut total_flex = 0.0;
        for &(_, flex) in children.iter() {
          total_flex += flex;
        }
        if total_flex == 0.0 {
          total_flex = 1.0;
        }
        for (layout, flex) in children.into_iter() {
          let layout = layout.into_layout_min_size(window);
          let layout_min_size = layout.min_size();
          min_size.x = cmp::max(min_size.x, layout_min_size.x);
          min_size.y += layout_min_size.y;
          new_children.push((layout, flex/total_flex));
        }
        VPanel_(alignment, new_children, min_size)
      }
      HPanel(alignment, children) => {
        let mut min_size = Vec2::zero();
        let mut new_children = Vec::new();
        let mut total_flex = 0.0;
        for &(_, flex) in children.iter() {
          total_flex += flex;
        }
        if total_flex == 0.0 {
          total_flex = 1.0;
        }
        for (layout, flex) in children.into_iter() {
          let layout = layout.into_layout_min_size(window);
          let layout_min_size = layout.min_size();
          min_size.y = cmp::max(min_size.y, layout_min_size.y);
          min_size.x += layout_min_size.x;
          new_children.push((layout, flex/total_flex));
        }
        HPanel_(alignment, new_children, min_size)
      }
      OverlapPanel(children) => {
        let mut min_size = Vec2::zero();
        let mut new_children = Vec::new();
        for layout in children.into_iter() {
          let layout = layout.into_layout_min_size(window);
          min_size = min_size.component_max(layout.min_size());
          new_children.push(layout);
        }
        OverlapPanel_(new_children, min_size)
      }
    }
  }
}

enum LayoutMinSize<'a> {
  VPanel_(Alignment, Vec<(LayoutMinSize<'a>, f64)>, Vec2<i32>),
  HPanel_(Alignment, Vec<(LayoutMinSize<'a>, f64)>, Vec2<i32>),
  OverlapPanel_(Vec<LayoutMinSize<'a>>, Vec2<i32>),
  LWidget_(&'a mut (Widget + 'a), Vec2<i32>),
}

impl<'a> LayoutMinSize<'a> {
  // This is very fast because it's computed in advance
  fn min_size(&self) -> Vec2<i32> {
    match *self {
      VPanel_(_, _, min_size) => min_size,
      HPanel_(_, _, min_size) => min_size,
      OverlapPanel_(_, min_size) => min_size,
      LWidget_(_, min_size) => min_size,
    }
  }

  fn collect_widgets(self, widgets: &mut Vec<&'a mut (Widget + 'a)>) {
    match self {
      LWidget_(widget, _) => widgets.push(widget),
      VPanel_(_, children, _) => {
        for (child,_) in children.into_iter() {
          child.collect_widgets(widgets);
        }
      }
      HPanel_(_, children, _) => {
        for (child,_) in children.into_iter() {
          child.collect_widgets(widgets);
        }
      }
      OverlapPanel_(children, _) => {
        for child in children.into_iter() {
          child.collect_widgets(widgets);
        }
      }
    }
  }

  // The 'pos' parameter is the position of the top-level layout widget; it must calculate the positions of its children and call calc_pos for each of them
  // TODO: this should probably set a Rect/AABB or something instead of setting the position and size separately
  fn calc_pos_size(&self, pos: Vec2<i32>, size: Vec2<i32>, widget_poses: &mut HashMap<Id, Vec2<i32>>, widget_sizes: &mut HashMap<Id, Vec2<i32>>) {
    match *self {
      LWidget_(ref widget, ref min_size) => {
        let real_size = min_size.component_max(size);
        if real_size != size {
          println!("Warning: widget is larger than allocated size; its contents may overlap adjacent widgets.");
        }
        widget_sizes.insert(widget.id(), real_size);
        widget_poses.insert(widget.id(), pos);
      }
      VPanel_(ref align, ref children, ref min_size) => {
        let extra_space = size.y - min_size.y;
        let mut child_sizes = Vec::new();
        for &(ref child_layout, flex) in children.iter() {
          child_sizes.push(Vec2(size.x, child_layout.min_size().y + (extra_space as f64*flex).floor() as i32));
        }

        let mut pos = pos;
        for i in range(0, children.len()) {
          let (ref child_layout, _) = children[i];
          let child_size = child_sizes[i];
          let wiggle_room = size.x - child_size.x;
          let x_pos = match *align {
            Leading => pos.x,
            Center => pos.x + wiggle_room/2,
            Trailing => pos.x + wiggle_room,
          };
          child_layout.calc_pos_size(Vec2(x_pos, pos.y), child_size, widget_poses, widget_sizes);
          pos.y += child_size.y;
        }
      }
      HPanel_(ref align, ref children, ref min_size) => {
        let extra_space = size.x - min_size.x;
        let mut child_sizes = Vec::new();
        for &(ref child_layout, flex) in children.iter() {
          child_sizes.push(Vec2(child_layout.min_size().x + (extra_space as f64*flex).floor() as i32, size.y));
        }

        let mut pos = pos;
        for i in range(0, children.len()) {
          let (ref child_layout, _) = children[i];
          let child_size = child_sizes[i];
          let wiggle_room = size.y - child_size.y;
          let y_pos = match *align {
            Leading => pos.y,
            Center => pos.y + wiggle_room/2,
            Trailing => pos.y + wiggle_room,
          };
          child_layout.calc_pos_size(Vec2(pos.x, y_pos), child_size, widget_poses, widget_sizes);
          pos.x += child_size.x;
        }
      }
      OverlapPanel_(ref children, ref min_size) => {
        for child in children.iter() {
          child.calc_pos_size(pos, size, widget_poses, widget_sizes);
        }
      }
    }
  }

  fn draw(&mut self, widget_sizes: &HashMap<Id, Vec2<i32>>, widget_poses: &HashMap<Id, Vec2<i32>>, window: &mut GUIWindow) {
    match *self {
      LWidget_(ref mut widget, _) => {
        let pos = *widget_poses.get(&widget.id()).unwrap();
        let size = *widget_sizes.get(&widget.id()).unwrap();
        widget.draw(pos, size, window);
      }
      VPanel_(_, ref mut children, _) => for &mut (ref mut layout,_) in children.iter_mut() {
        layout.draw(widget_sizes, widget_poses, window);
      },
      HPanel_(_, ref mut children, _) => for &mut (ref mut layout,_) in children.iter_mut() {
        layout.draw(widget_sizes, widget_poses, window);
      },
      OverlapPanel_(ref mut children, _) => for &mut ref mut layout in children.iter_mut() {
        layout.draw(widget_sizes, widget_poses, window);
      },
    }
  }
}








// TODO: get rid of lifetime by switching to String?
pub enum GUIWindowMode<'a> {
  Fullscreen,
  Windowed{title: &'a str, min_size: Vec2<i32>},
  FixedWindowed{title: &'a str, size: Vec2<i32>},
}

impl<'a> GUIWindowMode<'a> {
  pub fn fixed_size(&self) -> bool {
    match *self {
      GUIWindowMode::Fullscreen | GUIWindowMode::FixedWindowed{title: _, size: _} => true,
      GUIWindowMode::Windowed{title: _, min_size: _} => false,
    }
  }
}

pub fn init_glfw() -> Glfw {
  glfw::init(glfw::FAIL_ON_ERRORS).unwrap()
}










pub struct UnlitVertex {
  pub pos: Vec2<f32>,
  pub texcoord: Vec2<f32>,
}
pub struct UnlitUniforms<'a> {
  pub model_view_matrix: Mat4<f32>,
  pub proj_matrix: Mat4<f32>,
  pub tex: &'a Texture,
}

pub struct UnlitProgram {
  inner: Rc<GenericGLProgram>,
  model_view: NewMat4Uniform,
  proj: NewMat4Uniform,
}

impl UnlitProgram {
  pub fn new() -> Rc<UnlitProgram> {
    let inner = GenericGLProgram::new(
      include_str!("../shaders/unlit_vert_shader.glsl"),
      include_str!("../shaders/unlit_frag_shader.glsl"),
      "out_color", vec![("position",2),("texcoord",2)]);
    let model_view = NewMat4Uniform::new("modelViewMatrix", inner.clone());
    let proj = NewMat4Uniform::new("projMatrix", inner.clone());
    Rc::new(UnlitProgram{inner: inner, model_view: model_view, proj: proj})
  }
}

impl<'a> NewGLProgram for UnlitProgram {
  type Vertex = UnlitVertex;
  type Uniforms = UnlitUniforms<'a>;

  fn inner(&self) -> &GenericGLProgram {&self.inner}
  fn set_uniforms(&self, uniforms: UnlitUniforms) {
    self.model_view.set(uniforms.model_view_matrix);
    self.proj.set(uniforms.proj_matrix);
    uniforms.tex.bind(0);
  }
  fn add_vertex(&self, mesh: &mut NewMesh<Self>, vertex: UnlitVertex) {
    mesh.vertex_data(&[vertex.pos.x, vertex.pos.y,
      vertex.texcoord.x, vertex.texcoord.y]);
  }
}



pub struct GUIWindow<'a> {
  pub id: Id,
  mode: GUIWindowMode<'a>,
  pub glfw_window: Window,
  glfw_events: Receiver<(f64, glfw::WindowEvent)>,
  events: Vec<Event>,
  pub window_size: Vec2<i32>,
  // Used in get_events
  widget_poses: HashMap<Id, Vec2<i32>>,
  widget_sizes: HashMap<Id, Vec2<i32>>,
  // Programs and other stuff specific to this window
  // TODO: these shouldn't be public - add a better API

  // TODO: can we get rid of this Rc?
  pub unlit_program: Rc<UnlitProgram>,

  pub untextured_program: Rc<GLProgram>,
  pub text_program: Rc<GLProgram>,
  pub text_program_2: Rc<GLProgram>,
  pub untextured_model_view_matrix_uni: Mat4Uniform,
  pub untextured_proj_matrix_uni: Mat4Uniform,
  pub untextured_color_uni: ColorUniform,
  font_loader: FontLoader,
  // TODO: get rid of this hack
  pub focused: Option<Id>,
  pub focusable: Vec<Id>,
}


impl<'a> GUIWindow<'a> {
  pub fn new(glfw: &mut Glfw, mode: GUIWindowMode<'a>, resource_path: &Path) -> GUIWindow<'a> {
    let resizable = match mode {
      GUIWindowMode::Windowed{title: _, min_size: _} => true,
      _ => false,
    };
    glfw.window_hint(WindowHint::Resizable(resizable));
    glfw.window_hint(WindowHint::Samples(4));
    let (mut window, events) = match mode {
      GUIWindowMode::Fullscreen => create_fullscreen(glfw),
      // TODO: in this case, don't show the window until it's been resized by the GUI
      GUIWindowMode::Windowed{ref title, ref min_size} => create_windowed(glfw, min_size.x as u32, min_size.y as u32, title.as_slice()),
      GUIWindowMode::FixedWindowed{ref title, ref size} => create_windowed(glfw, size.x as u32, size.y as u32, title.as_slice()),
    };

    let current_window_size = get_window_size(&window);

    window.set_key_polling(true);
    window.set_char_polling(true);
    window.set_mouse_button_polling(true);
    window.set_cursor_pos_polling(true);
    glfw.set_swap_interval(0);

    Blend.enable();
    unsafe {gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);}
    Multisample.enable();
    // Stops textures with widths not divisible by 4 from being all wonky
    // (very important for text)
    unsafe {gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);}






    let vs_text       = Shader::new(include_str!("../shaders/text_vert_shader.glsl"), "", VertexShader);
    let fs_text       = Shader::new(include_str!("../shaders/text_frag_shader.glsl"), "", FragmentShader);
    let vs_text_2     = Shader::new(include_str!("../shaders/text_vert_shader_2.glsl"), "", VertexShader);
    let fs_text_2     = Shader::new(include_str!("../shaders/text_frag_shader_2.glsl"), "", FragmentShader);
    let vs_untextured = Shader::new(include_str!("../shaders/untextured_vert_shader.glsl"), "", VertexShader);
    let fs_untextured = Shader::new(include_str!("../shaders/untextured_frag_shader.glsl"), "", FragmentShader);
    let text_program = GLProgram::new(vs_text, fs_text, "out_color",
        vec![("position",2),("texcoord",2),("color",4)]);
    let text_program_2 = GLProgram::new(vs_text_2, fs_text_2, "out_color",
        vec![("position",2),("texcoord",2)]);
    let untextured_program = GLProgram::new(vs_untextured, fs_untextured, "out_color",
        vec![("position",2), ("color",4)]);

    let unlit_program = UnlitProgram::new();

    let untextured_model_view_matrix_uni = Mat4Uniform::new("modelViewMatrix", untextured_program.clone());
    let untextured_proj_matrix_uni = Mat4Uniform::new("projMatrix", untextured_program.clone());
    let untextured_color_uni = ColorUniform::new("color", untextured_program.clone());

    let font_loader = FontLoader::new();

    let gui_window = GUIWindow {id: next_id(), mode: mode, glfw_window: window, glfw_events: events, events: Vec::new(),
      window_size: current_window_size,
      widget_poses: HashMap::new(), widget_sizes: HashMap::new(),
      unlit_program: unlit_program, untextured_program: untextured_program,
      text_program: text_program, text_program_2: text_program_2,
      untextured_model_view_matrix_uni: untextured_model_view_matrix_uni,
      untextured_proj_matrix_uni: untextured_proj_matrix_uni,
      untextured_color_uni: untextured_color_uni,
      font_loader: font_loader,
      focused: None,
      focusable: vec![],
    };
    gui_window
  }

  pub fn focus_next(&mut self) {
    //biggest hack I've ever made - just a prototype
    if self.focused.is_none() {
      self.focused = Some(self.focusable[0]);
    } else {
      if self.focused == Some(self.focusable[0]) {
        self.focused = Some(self.focusable[1]);
      } else if self.focused == Some(self.focusable[1]) {
        self.focused = Some(self.focusable[2]);
      } else if self.focused == Some(self.focusable[2]) {
        self.focused = Some(self.focusable[0]);
      }
    }
  }

  pub fn load_font(&mut self, path: &Path, size: i32) -> Font {
    self.glfw_window.make_current(); // is this needed?

    Font::new(&self.font_loader, path, size, self.text_program_2.clone(), self.text_program.clone())
  }

  pub fn draw_gui(&mut self, layout: Layout, glfw: &mut Glfw, background_color: Color<f32>) {
    self.draw_gui_with_extra(layout, glfw, background_color, |_| ());
  }

  // Draws the GUI, with some extra drawing done before swapping buffers
  pub fn draw_gui_with_extra<F: FnMut(&mut GUIWindow)>(&mut self, layout: Layout, glfw: &mut Glfw, background_color: Color<f32>, mut extra_drawing: F) {
    self.glfw_window.make_current();

    check_gl_error("draw_gui");

    unsafe {gl::ClearColor(background_color.r, background_color.g, background_color.b, 1.0)};
    unsafe {gl::Clear(gl::COLOR_BUFFER_BIT);}


    let window_size = get_window_size(&self.glfw_window);
    // TODO: the following two lines should be disabled by default
    /*self.unlit_proj_matrix_uni.set(Mat4::ortho_flip(window_size.x as f32, window_size.y as f32));
    self.unlit_model_view_matrix_uni.set(Mat4::id());*/

    let mut layout = layout.into_layout_min_size(self);
    let min_size = layout.min_size();
    let current_window_size = get_window_size(&self.glfw_window);
    let min_window_size = match self.mode {
      GUIWindowMode::Fullscreen => current_window_size,
      GUIWindowMode::FixedWindowed{ref title, ref size} => *size,
      GUIWindowMode::Windowed{ref title, min_size: ref window_min_size} => min_size.component_max(*window_min_size),
    };

    let real_size = current_window_size.component_max(min_window_size);

    let mut widget_sizes = HashMap::new();
    let mut widget_poses = HashMap::new();

    layout.calc_pos_size(Vec2::zero(), real_size, &mut widget_poses, &mut widget_sizes);

    let desired_window_size = if self.mode.fixed_size() {current_window_size} else {real_size};
    let new_window_size = current_window_size.component_max(desired_window_size);
    self.window_size = new_window_size;
    if new_window_size != current_window_size {
      self.glfw_window.set_size(new_window_size.x, new_window_size.y);
      // TODO: move window to middle of screen?
    }
    if !self.glfw_window.is_visible() {
      self.glfw_window.show();
    }

    // TODO: do we really need SIX different variables for window size?
    // println!("{} {} {} {} {} {}", min_size, current_window_size, min_window_size, real_size, desired_window_size, new_window_size);

    unsafe {gl::Viewport(0, 0, new_window_size.x, new_window_size.y);}

    // We draw this before the rest of the GUI so widgets can overlay it
    extra_drawing(self);

    layout.draw(&widget_sizes, &widget_poses, self);


    self.glfw_window.swap_buffers();


    let mut all_widgets = Vec::new();
    layout.collect_widgets(&mut all_widgets);


    self.events = Vec::new();
    glfw.poll_events();
    for (_, event) in glfw::flush_messages(&self.glfw_events) {
      self.events.push(Event::from_glfw(event, &self.glfw_window));
    }

    self.widget_poses = widget_poses;
    self.widget_sizes = widget_sizes;

    for event in self.events.clone().iter() {
      match event.position() {
        Some(pos) => for widget in all_widgets.iter_mut() {
          let widget_pos = *self.widget_poses.get(&widget.id()).unwrap();
          let widget_size = *self.widget_sizes.get(&widget.id()).unwrap();
          let widget_aabb = AABB2::from_pos_size(widget_pos, widget_size);
          if widget_aabb.contains_vec(pos) {
            widget.handle_event(event.for_widget(widget_pos), self);
          }
        },
        None => (),
      }
    }
  }

  pub fn get_events(&self) -> Vec<Event> {
    self.events.clone()
  }

  /// You usually shouldn't use this directly.
  pub fn get_widget_events(&self, widget: &Widget) -> Vec<Event> {
    let mut events = Vec::new();
    for event in self.events.iter() {
      match event.position() {
        Some(pos) => {
          let widget_pos = *self.widget_poses.get(&widget.id()).unwrap();
          let widget_size = *self.widget_sizes.get(&widget.id()).unwrap();
          let widget_aabb = AABB2::from_pos_size(widget_pos, widget_size);
          if widget_aabb.contains_vec(pos) {
            events.push(event.for_widget(widget_pos));
          }
        },
        None => (),
      }
    }
    events
  }
}


// Repeat is treated the same as Press
#[derive(Debug, Copy, Clone)]
pub enum Action {Press, Release}

impl Action {
  fn from_glfw(action: glfw::Action) -> Action {
    match action {
      glfw::Action::Press | glfw::Action::Repeat => Action::Press,
      glfw::Action::Release => Action::Release,
    }
  }
}

// TODO: more events
#[derive(Debug, Clone)]
pub enum Event {
  // These are derived from raw GLFW events
  MouseButton(glfw::MouseButton, Action, glfw::Modifiers, Vec2<i32>),
  MouseMove(Vec2<i32>, Vec<glfw::MouseButton>),
  Key(glfw::Key, glfw::Scancode, Action, glfw::Modifiers),
  Char(char),
  Unknown,
  // These are higher-level events sent from widgets
  // Activate(Id),
}

impl Event {
  // Note: for some events this looks at the window's current cursor position
  fn from_glfw(event: glfw::WindowEvent, window: &glfw::Window) -> Event {
    match event {
      glfw::WindowEvent::MouseButton(button, action, mods) => {
        let (cursor_x, cursor_y) = window.get_cursor_pos();
        let cursor_pos = Vec2(cursor_x as i32, cursor_y as i32);
        Event::MouseButton(button, Action::from_glfw(action), mods, cursor_pos)
      },
      glfw::WindowEvent::CursorPos(cursor_x, cursor_y) => {
        let cursor_pos = Vec2(cursor_x as i32, cursor_y as i32);
        let mut buttons = Vec::new();
        // TODO: this isn't great
        for i in range(0, 8) {
          let button = FromPrimitive::from_u8(i).unwrap();
          if window.get_mouse_button(button) != glfw::Action::Release {
            buttons.push(button);
          }
        }
        Event::MouseMove(cursor_pos, buttons)
      },
      glfw::WindowEvent::Key(key, scancode, action, mods) =>
        Event::Key(key, scancode, Action::from_glfw(action), mods),
      glfw::WindowEvent::Char(char) =>
        Event::Char(char),
      _ => Event::Unknown,
    }
  }

  fn position(&self) -> Option<Vec2<i32>> {
    match *self {
      Event::MouseButton(_, _, _, pos) => Some(pos),
      Event::MouseMove(pos, _) => Some(pos),
      _ => None,
    }
  }

  fn for_widget(&self, widget_pos: Vec2<i32>) -> Event {
    match self {
      &Event::MouseButton(button, action, mods, pos) =>
        Event::MouseButton(button, action, mods, pos-widget_pos),
      &Event::MouseMove(pos, ref buttons) =>
        Event::MouseMove(pos-widget_pos, buttons.clone()),
      x => x.clone(),
    }
  }
}

type EventHandler<'a> = Fn(Event) -> bool + 'a;
