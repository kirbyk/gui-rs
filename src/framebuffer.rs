use std::iter::*;

use opengl::*;
use texture::*;

pub struct Framebuffer {
  pub id: GLuint,
  pub tex: Texture,
  pub depth_buffer: bool,
  pub depth_id: GLuint,
}

impl Framebuffer {
  pub fn new(width: i32, height: i32, internal_format: u32, depth_buffer: bool) -> Framebuffer {
    let mut id = 0;
    let mut depth_id = 0;
    unsafe {
      gl::GenFramebuffers(1, &mut id);
      gl::BindFramebuffer(gl::FRAMEBUFFER, id);
    }
    // TODO: allow users to set the min/mag filters
    let tex = Texture::texture2d_empty(width, height, internal_format, MinNearest, MagNearest);
    unsafe {
      gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, tex.id, 0);
      if depth_buffer {
        gl::GenRenderbuffers(1, &mut depth_id);
        gl::BindRenderbuffer(gl::RENDERBUFFER, depth_id);
        gl::RenderbufferStorage(gl::RENDERBUFFER, gl::DEPTH_COMPONENT32, width, height);
        gl::FramebufferRenderbuffer(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT,
            gl::RENDERBUFFER, depth_id);
      }
      assert!(gl::CheckFramebufferStatus(gl::FRAMEBUFFER) == gl::FRAMEBUFFER_COMPLETE, "Framebuffer not complete");
      gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
    }
    Framebuffer {id: id, tex: tex, depth_buffer: depth_buffer,
        depth_id: depth_id}
  }

  // NOTE: don't call this function from within a callback passed to this function; the previously bound framebuffer won't be restored properly.
  // TODO: fix it.
  pub fn render_to<F>(&self, f: F) where F: FnOnce() {
    let old_viewport = self.begin_render();
    f();
    self.end_render(old_viewport);
  }

  // Ugly hack to get around Rust's borrow checker
  pub fn begin_render(&self) -> Vec<i32> {
    unsafe {gl::BindFramebuffer(gl::FRAMEBUFFER, self.id);}
    let mut old_viewport: Vec<_> = repeat(0).take(4).collect();//Vec::from_elem(4, 0);
    unsafe {
      gl::GetIntegerv(gl::VIEWPORT, old_viewport.as_mut_ptr());
      gl::Viewport(0, 0, self.tex.size.x, self.tex.size.y);
    }
    old_viewport
  }

  pub fn end_render(&self, old_viewport: Vec<i32>) {
    unsafe {
      gl::Viewport(old_viewport[0], old_viewport[1], old_viewport[2], old_viewport[3]);
      gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
    }
  }
}

impl Drop for Framebuffer {
  fn drop(&mut self) {
    unsafe {
      if self.depth_buffer {
        gl::DeleteRenderbuffers(1, &self.depth_id);
      }
      gl::DeleteFramebuffers(1, &self.id);
    }
  }
}
