extern crate glfw;

use std::sync::mpsc::Receiver;

use glfw::*;

use vecmat::*;

use opengl::*;

pub use self::GLFlag::*;

pub fn create_windowed(glfw: &mut Glfw, width:u32, height:u32, title: &str) -> (Window, Receiver<(f64, WindowEvent)>) {
  glfw.window_hint(glfw::WindowHint::Visible(false));
  glfw.with_primary_monitor(|glfw, m| {
    let monitor = m.expect("Failed to find monitor.");
    let mode = monitor.get_video_mode().expect("Failed to get video mode.");
    let (posx, posy) = ((mode.width-width)/2, (mode.height-height)/2);
    let (mut window, events) = glfw.create_window(width, height, title, glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");
    window.set_pos(posx as i32, posy as i32);

    // window.show(); // TODO: this should NOT be here for GUI code
    window.make_current();

    // Load the OpenGL function pointers
    gl::load_with(|s| window.get_proc_address(s));

    (window, events)
  })
}

pub fn create_fullscreen(glfw: &mut Glfw) -> (Window, Receiver<(f64, WindowEvent)>) {
  glfw.with_primary_monitor(|glfw, m| {
    let monitor = m.expect("Failed to find monitor.");
    let mode = monitor.get_video_mode().expect("Failed to get video mode.");
    let (mut window, events) = glfw.create_window(mode.width, mode.height, "", glfw::WindowMode::FullScreen(monitor))
        .expect("Failed to create GLFW window.");

    window.make_current();

    // Load the OpenGL function pointers
    gl::load_with(|s| window.get_proc_address(s));

    (window, events)
  })
}



pub fn check_gl_error(loc: &str) {
  let err = unsafe {gl::GetError()};
  if err != 0 {
    println!("OpenGL error {} in {}", err, loc);
  }
}



//A small selection of OpenGL flags - this only includes the ones I use
#[derive(PartialEq, Copy, Clone)]
pub enum GLFlag {
  DepthTest,
  Blend,
  Multisample,
  CullFace,
}

impl GLFlag {
  fn as_gl(&self) -> GLuint {
    match *self {
      DepthTest => gl::DEPTH_TEST,
      Blend => gl::BLEND,
      Multisample => gl::MULTISAMPLE,
      CullFace => gl::CULL_FACE,
    }
  }

  pub fn enable(&self) {
    unsafe {gl::Enable(self.as_gl())};
  }

  pub fn disable(&self) {
    unsafe {gl::Disable(self.as_gl())};
  }
}

pub fn get_window_size(window: &Window) -> Vec2<i32> {
  let (w,h) = window.get_framebuffer_size();
  Vec2(w, h)
}
