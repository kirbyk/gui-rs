extern crate libc;
extern crate image;

use std::ptr;

use image::*;

use vecmat::*;
use opengl::*;

pub use self::MinFilter::*;
pub use self::MagFilter::*;


impl !Send for Texture {}
impl !Sync for Texture {}

impl Drop for Texture {
  fn drop(&mut self) {
    unsafe {
      gl::DeleteTextures(1, &self.id);
    }
  }
}

pub enum MinFilter {
  MinNearest, MinLinear,
  MinNearestMipmapNearest, MinNearestMipmapLinear, MinLinearMipmapNearest, MinLinearMipmapLinear,
}

pub enum MagFilter {
  MagNearest, MagLinear,
}

impl MinFilter {
  fn as_gl(&self) -> GLuint {
    match *self {
      MinNearest => gl::NEAREST,
      MinLinear => gl::LINEAR,
      MinNearestMipmapNearest => gl::NEAREST_MIPMAP_NEAREST,
      MinNearestMipmapLinear => gl::NEAREST_MIPMAP_LINEAR,
      MinLinearMipmapNearest => gl::LINEAR_MIPMAP_NEAREST,
      MinLinearMipmapLinear => gl::LINEAR_MIPMAP_LINEAR,
    }
  }

  fn has_mipmap(&self) -> bool {
    match *self {
      MinNearest => false,
      MinLinear => false,
      MinNearestMipmapNearest => true,
      MinNearestMipmapLinear => true,
      MinLinearMipmapNearest => true,
      MinLinearMipmapLinear => true,
    }
  }
}

impl MagFilter {
  fn as_gl(&self) -> GLuint {
    match *self {
      MagNearest => gl::NEAREST,
      MagLinear => gl::LINEAR,
    }
  }
}

// For array textures and cubemaps, `size` is the size of a 2D layer
pub struct Texture {
  pub id: GLuint,
  pub tex_type: GLuint,
  pub size: Vec2<i32>,
}

impl Texture {
  pub fn update_texture2d_from_pixels(&mut self, pixels: &Vec<u8>) {
    /*let (w,h) = image.dimensions();
    assert!(w as i32 == self.size.x && h as i32 == self.size.y);*/

    unsafe {
      gl::BindTexture(gl::TEXTURE_2D, self.id);

      gl::TexSubImage2D(gl::TEXTURE_2D, 0, 0, 0, self.size.x as i32, self.size.y as i32, gl::RGB, gl::UNSIGNED_BYTE, pixels.as_slice().as_ptr() as *const libc::types::common::c95::c_void);

      /*match *image {
        ImageRgb8(_) => gl::TexSubImage2D(gl::TEXTURE_2D, 0, 0, 0, w as i32, h as i32, gl::RGB, gl::UNSIGNED_BYTE, image.raw_pixels().as_ptr() as *const libc::types::common::c95::c_void),
/*        ImageRgba8(_) => gl::TexImage2D(gl::TEXTURE_2D, 0, gl::SRGB8_ALPHA8 as i32, w as i32, h as i32, 0, gl::RGBA, gl::UNSIGNED_BYTE, image.raw_pixels().as_ptr() as *const libc::types::common::c95::c_void),
        ImageLuma8(_) => gl::TexImage2D(gl::TEXTURE_2D, 0, gl::SLUMINANCE8 as i32, w as i32, h as i32, 0, gl::LUMINANCE, gl::UNSIGNED_BYTE, image.raw_pixels().as_ptr() as *const libc::types::common::c95::c_void),
        ImageLumaA8(_) => gl::TexImage2D(gl::TEXTURE_2D, 0, gl::SLUMINANCE8_ALPHA8 as i32, w as i32, h as i32, 0, gl::LUMINANCE_ALPHA, gl::UNSIGNED_BYTE, image.raw_pixels().as_ptr() as *const libc::types::common::c95::c_void),*/
        _ => panic!("")
      }*/
    }
  }














  pub fn texture2d_empty(width: i32, height: i32, internal_format: u32, min_filter: MinFilter, mag_filter: MagFilter) -> Texture {
    assert!(!min_filter.has_mipmap());

    let mut texture = 0;
    unsafe {
      gl::GenTextures(1, &mut texture);
      gl::BindTexture(gl::TEXTURE_2D, texture);

      // We use gl::RGBA just because we have to put *something* there;
      // it doesn't matter what because it's not used
      gl::TexImage2D(gl::TEXTURE_2D, 0, internal_format as i32, width, height, 0, gl::RGBA, gl::UNSIGNED_BYTE, ptr::null());

      gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
      gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
      gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, min_filter.as_gl() as i32);
      gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, mag_filter.as_gl() as i32);
    }
    Texture {id: texture, tex_type: gl::TEXTURE_2D, size: Vec2(width, height)}
  }

  pub fn texture2d_from_data(data: &[u8], width: i32, height: i32, format: u32, internal_format: u32, min_filter: MinFilter, mag_filter: MagFilter) -> Texture {
    let mut texture = 0;
    unsafe {
      gl::GenTextures(1, &mut texture);
      gl::BindTexture(gl::TEXTURE_2D, texture);

      gl::TexImage2D(gl::TEXTURE_2D, 0, internal_format as i32, width, height, 0, format, gl::UNSIGNED_BYTE, data.as_ptr() as *const libc::types::common::c95::c_void);

      gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
      gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
      gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, min_filter.as_gl() as i32);
      gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, mag_filter.as_gl() as i32);
      let mut max_aniso = 0.0;
      unsafe {
        gl::GetFloatv(gl::MAX_TEXTURE_MAX_ANISOTROPY_EXT, &mut max_aniso);
      }
      if min_filter.has_mipmap() {
        gl::GenerateMipmap(gl::TEXTURE_2D);
        gl::TexParameterf(gl::TEXTURE_2D, gl::TEXTURE_MAX_ANISOTROPY_EXT, max_aniso);
      }
    }
    Texture {id: texture, tex_type: gl::TEXTURE_2D, size: Vec2(width, height)}
  }

  pub fn texture2d(path: &str, min_filter: MinFilter, mag_filter: MagFilter) -> Texture {
    Texture::texture2d_from_image(&image::open(&Path::new(path)).unwrap(), min_filter, mag_filter)
  }

  pub fn texture2d_from_image(image: &DynamicImage, min_filter: MinFilter, mag_filter: MagFilter) -> Texture {
    let (w,h) = image.dimensions();
    let mut texture = 0;
    unsafe {
      gl::GenTextures(1, &mut texture);
      gl::BindTexture(gl::TEXTURE_2D, texture);

      match *image {
        ImageRgb8(_) => gl::TexImage2D(gl::TEXTURE_2D, 0, gl::SRGB8 as i32, w as i32, h as i32, 0, gl::RGB, gl::UNSIGNED_BYTE, image.raw_pixels().as_ptr() as *const libc::types::common::c95::c_void),
        ImageRgba8(_) => gl::TexImage2D(gl::TEXTURE_2D, 0, gl::SRGB8_ALPHA8 as i32, w as i32, h as i32, 0, gl::RGBA, gl::UNSIGNED_BYTE, image.raw_pixels().as_ptr() as *const libc::types::common::c95::c_void),
        ImageLuma8(_) => gl::TexImage2D(gl::TEXTURE_2D, 0, gl::SLUMINANCE8 as i32, w as i32, h as i32, 0, gl::LUMINANCE, gl::UNSIGNED_BYTE, image.raw_pixels().as_ptr() as *const libc::types::common::c95::c_void),
        ImageLumaA8(_) => gl::TexImage2D(gl::TEXTURE_2D, 0, gl::SLUMINANCE8_ALPHA8 as i32, w as i32, h as i32, 0, gl::LUMINANCE_ALPHA, gl::UNSIGNED_BYTE, image.raw_pixels().as_ptr() as *const libc::types::common::c95::c_void),
      }

      gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
      gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
      gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, min_filter.as_gl() as i32);
      gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, mag_filter.as_gl() as i32);
      let mut max_aniso = 0.0;
      unsafe {
        gl::GetFloatv(gl::MAX_TEXTURE_MAX_ANISOTROPY_EXT, &mut max_aniso);
      }
      if min_filter.has_mipmap() {
        gl::GenerateMipmap(gl::TEXTURE_2D);
        gl::TexParameterf(gl::TEXTURE_2D, gl::TEXTURE_MAX_ANISOTROPY_EXT, max_aniso);
      }
    }
    Texture {id: texture, tex_type: gl::TEXTURE_2D, size: Vec2(w as i32, h as i32)}
  }


  pub fn texture2d_from_image_nonsrgb(image: &DynamicImage, min_filter: MinFilter, mag_filter: MagFilter) -> Texture {
    let (w,h) = image.dimensions();
    let mut texture = 0;
    unsafe {
      gl::GenTextures(1, &mut texture);
      gl::BindTexture(gl::TEXTURE_2D, texture);

      match *image {
        ImageRgb8(_) => gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB as i32, w as i32, h as i32, 0, gl::RGB, gl::UNSIGNED_BYTE, image.raw_pixels().as_ptr() as *const libc::types::common::c95::c_void),
        ImageRgba8(_) => gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as i32, w as i32, h as i32, 0, gl::RGBA, gl::UNSIGNED_BYTE, image.raw_pixels().as_ptr() as *const libc::types::common::c95::c_void),
        ImageLuma8(_) => gl::TexImage2D(gl::TEXTURE_2D, 0, gl::LUMINANCE as i32, w as i32, h as i32, 0, gl::LUMINANCE, gl::UNSIGNED_BYTE, image.raw_pixels().as_ptr() as *const libc::types::common::c95::c_void),
        ImageLumaA8(_) => gl::TexImage2D(gl::TEXTURE_2D, 0, gl::LUMINANCE_ALPHA as i32, w as i32, h as i32, 0, gl::LUMINANCE_ALPHA, gl::UNSIGNED_BYTE, image.raw_pixels().as_ptr() as *const libc::types::common::c95::c_void),
      }

      gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
      gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
      gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, min_filter.as_gl() as i32);
      gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, mag_filter.as_gl() as i32);
      let mut max_aniso = 0.0;
      unsafe {
        gl::GetFloatv(gl::MAX_TEXTURE_MAX_ANISOTROPY_EXT, &mut max_aniso);
      }
      if min_filter.has_mipmap() {
        gl::GenerateMipmap(gl::TEXTURE_2D);
        gl::TexParameterf(gl::TEXTURE_2D, gl::TEXTURE_MAX_ANISOTROPY_EXT, max_aniso);
      }
    }
    Texture {id: texture, tex_type: gl::TEXTURE_2D, size: Vec2(w as i32, h as i32)}
  }
  // Doesn't verify that all the images are the same size.
  pub fn cubemap(bottom: &str, top: &str, left: &str, right: &str, back: &str, front: &str, min_filter: MinFilter, mag_filter: MagFilter) -> Texture {
    let mut texture = 0;
    let img = image::open(&Path::new(bottom)).unwrap();
    let (w,h) = img.dimensions();
    unsafe {
      gl::GenTextures(1, &mut texture);
      gl::BindTexture(gl::TEXTURE_CUBE_MAP, texture);
      match img {
        ImageRgb8(_) => gl::TexImage2D(gl::TEXTURE_CUBE_MAP_NEGATIVE_Y, 0, gl::SRGB8 as i32, w as i32, h as i32, 0, gl::RGB, gl::UNSIGNED_BYTE, img.raw_pixels().as_ptr() as *const libc::types::common::c95::c_void),
        ImageRgba8(_) => gl::TexImage2D(gl::TEXTURE_CUBE_MAP_NEGATIVE_Y, 0, gl::SRGB8_ALPHA8 as i32, w as i32, h as i32, 0, gl::RGBA, gl::UNSIGNED_BYTE, img.raw_pixels().as_ptr() as *const libc::types::common::c95::c_void),
        _ => panic!("Unsupported image type."),
      }

      let img = image::open(&Path::new(top)).unwrap();
      let (w,h) = img.dimensions();
      match img {
        ImageRgb8(_) => gl::TexImage2D(gl::TEXTURE_CUBE_MAP_POSITIVE_Y, 0, gl::SRGB8 as i32, w as i32, h as i32, 0, gl::RGB, gl::UNSIGNED_BYTE, img.raw_pixels().as_ptr() as *const libc::types::common::c95::c_void),
        ImageRgba8(_) => gl::TexImage2D(gl::TEXTURE_CUBE_MAP_POSITIVE_Y, 0, gl::SRGB8_ALPHA8 as i32, w as i32, h as i32, 0, gl::RGBA, gl::UNSIGNED_BYTE, img.raw_pixels().as_ptr() as *const libc::types::common::c95::c_void),
        _ => panic!("Unsupported image type."),
      }

      let img = image::open(&Path::new(left)).unwrap();
      let (w,h) = img.dimensions();
      match img {
        ImageRgb8(_) => gl::TexImage2D(gl::TEXTURE_CUBE_MAP_NEGATIVE_X, 0, gl::SRGB8 as i32, w as i32, h as i32, 0, gl::RGB, gl::UNSIGNED_BYTE, img.raw_pixels().as_ptr() as *const libc::types::common::c95::c_void),
        ImageRgba8(_) => gl::TexImage2D(gl::TEXTURE_CUBE_MAP_NEGATIVE_X, 0, gl::SRGB8_ALPHA8 as i32, w as i32, h as i32, 0, gl::RGBA, gl::UNSIGNED_BYTE, img.raw_pixels().as_ptr() as *const libc::types::common::c95::c_void),
        _ => panic!("Unsupported image type."),
      }

      let img = image::open(&Path::new(right)).unwrap();
      let (w,h) = img.dimensions();
      match img {
        ImageRgb8(_) => gl::TexImage2D(gl::TEXTURE_CUBE_MAP_POSITIVE_X, 0, gl::SRGB8 as i32, w as i32, h as i32, 0, gl::RGB, gl::UNSIGNED_BYTE, img.raw_pixels().as_ptr() as *const libc::types::common::c95::c_void),
        ImageRgba8(_) => gl::TexImage2D(gl::TEXTURE_CUBE_MAP_POSITIVE_X, 0, gl::SRGB8_ALPHA8 as i32, w as i32, h as i32, 0, gl::RGBA, gl::UNSIGNED_BYTE, img.raw_pixels().as_ptr() as *const libc::types::common::c95::c_void),
        _ => panic!("Unsupported image type."),
      }

      let img = image::open(&Path::new(back)).unwrap();
      let (w,h) = img.dimensions();
      match img {
        ImageRgb8(_) => gl::TexImage2D(gl::TEXTURE_CUBE_MAP_NEGATIVE_Z, 0, gl::SRGB8 as i32, w as i32, h as i32, 0, gl::RGB, gl::UNSIGNED_BYTE, img.raw_pixels().as_ptr() as *const libc::types::common::c95::c_void),
        ImageRgba8(_) => gl::TexImage2D(gl::TEXTURE_CUBE_MAP_NEGATIVE_Z, 0, gl::SRGB8_ALPHA8 as i32, w as i32, h as i32, 0, gl::RGBA, gl::UNSIGNED_BYTE, img.raw_pixels().as_ptr() as *const libc::types::common::c95::c_void),
        _ => panic!("Unsupported image type."),
      }

      let img = image::open(&Path::new(front)).unwrap();
      let (w,h) = img.dimensions();
      match img {
        ImageRgb8(_) => gl::TexImage2D(gl::TEXTURE_CUBE_MAP_POSITIVE_Z, 0, gl::SRGB8 as i32, w as i32, h as i32, 0, gl::RGB, gl::UNSIGNED_BYTE, img.raw_pixels().as_ptr() as *const libc::types::common::c95::c_void),
        ImageRgba8(_) => gl::TexImage2D(gl::TEXTURE_CUBE_MAP_POSITIVE_Z, 0, gl::SRGB8_ALPHA8 as i32, w as i32, h as i32, 0, gl::RGBA, gl::UNSIGNED_BYTE, img.raw_pixels().as_ptr() as *const libc::types::common::c95::c_void),
        _ => panic!("Unsupported image type."),
      }
    }
    unsafe {
      gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
      gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
      gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MIN_FILTER, min_filter.as_gl() as i32);
      gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MAG_FILTER, mag_filter.as_gl() as i32);
      let mut max_aniso = 0.0;
      gl::GetFloatv(gl::MAX_TEXTURE_MAX_ANISOTROPY_EXT, &mut max_aniso);
      if min_filter.has_mipmap() {
        gl::GenerateMipmap(gl::TEXTURE_CUBE_MAP);
        gl::TexParameterf(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MAX_ANISOTROPY_EXT, max_aniso);
      }
    }
    Texture {id: texture, tex_type: gl::TEXTURE_CUBE_MAP, size: Vec2(w as i32, h as i32)}
  }

  pub fn array_texture(paths: Vec<String>, width: i32, height: i32, min_filter: MinFilter, mag_filter: MagFilter) -> Texture {
    let mut texture = 0;
    unsafe {
      gl::GenTextures(1, &mut texture);
      gl::BindTexture(gl::TEXTURE_2D_ARRAY, texture);

      gl::TexImage3D(gl::TEXTURE_2D_ARRAY, 0, gl::SRGB8_ALPHA8 as i32, width, height, paths.len() as i32, 0, gl::RGBA, gl::UNSIGNED_BYTE, ptr::null());

      for i in range(0, paths.len()) {
        let img = image::open(&Path::new(paths[i].as_slice())).unwrap();
        match img {
          ImageRgb8(_) => gl::TexSubImage3D(gl::TEXTURE_2D_ARRAY, 0, 0, 0, i as i32, width as i32, height as i32, 1, gl::RGB, gl::UNSIGNED_BYTE, img.raw_pixels().as_ptr() as *const libc::types::common::c95::c_void),
          ImageRgba8(_) => gl::TexSubImage3D(gl::TEXTURE_2D_ARRAY, 0, 0, 0, i as i32, width as i32, height as i32, 1, gl::RGBA, gl::UNSIGNED_BYTE, img.raw_pixels().as_ptr() as *const libc::types::common::c95::c_void),
          _ => panic!("Unsupported image type."),
        }
      }

      gl::TexParameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
      gl::TexParameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
      gl::TexParameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_MIN_FILTER, min_filter.as_gl() as i32);
      gl::TexParameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_MAG_FILTER, mag_filter.as_gl() as i32);

      let mut max_aniso = 0.0;
      unsafe {
        gl::GetFloatv(gl::MAX_TEXTURE_MAX_ANISOTROPY_EXT, &mut max_aniso);
      }
      if min_filter.has_mipmap() {
        gl::GenerateMipmap(gl::TEXTURE_2D_ARRAY);
        gl::TexParameterf(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_MAX_ANISOTROPY_EXT, max_aniso);
      }
    }
    Texture {id: texture, tex_type: gl::TEXTURE_2D_ARRAY, size: Vec2(width, height)}
  }

  pub fn bind(&self, texture_unit: u32) {
    unsafe {
      gl::ActiveTexture(gl::TEXTURE0 + texture_unit as GLuint);
      gl::BindTexture(self.tex_type, self.id);
    }
  }
}
