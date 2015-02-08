use std::ops::*;
use std::num::*;

use vecmat::*;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Color<T: Float> {
  pub r: T, pub g: T, pub b: T, pub a: T
}

// TODO: allow u8 components?
// TODO: get rid of type param and require it to be f32?
impl<T: Float + From<f32>> Color<T> {
  pub fn rgba(r: T, g: T, b: T, a: T) -> Color<T> { Color {r: r, g: g, b: b, a: a} }
  pub fn rgb(r: T, g: T, b: T) -> Color<T> { Color {r: r, g: g, b: b, a: 1.0.cvt()} }

  /// Blends this color with another color. `self_amount` is
  /// the amount of the current color to keep; the rest
  /// is replaced with a contribution from the other color.
  pub fn blend(self, other: Color<T>, self_amount: T) -> Color<T> {
    let one: T = 1.0.cvt();
    self*self_amount + other*(one-self_amount)
  }

  pub fn black() -> Color<T> {
    let zero: T = 0.0.cvt();
    Color::rgb(zero, zero, zero)
  }
  pub fn white() -> Color<T> {
    let one: T = 1.0.cvt();
    Color::rgb(one, one, one)
  }
  pub fn red() -> Color<T> {
    let zero: T = 0.0.cvt();
    let one: T = 1.0.cvt();
    Color::rgb(one, zero, zero)
  }
  pub fn green() -> Color<T> {
    let zero: T = 0.0.cvt();
    let one: T = 1.0.cvt();
    Color::rgb(zero, one, zero)
  }
  pub fn blue() -> Color<T> {
    let zero: T = 0.0.cvt();
    let one: T = 1.0.cvt();
    Color::rgb(zero, zero, one)
  }
  pub fn cyan() -> Color<T> {
    let zero: T = 0.0.cvt();
    let one: T = 1.0.cvt();
    Color::rgb(zero, one, one)
  }
  pub fn magenta() -> Color<T> {
    let zero: T = 0.0.cvt();
    let one: T = 1.0.cvt();
    Color::rgb(one, zero, one)
  }
  pub fn yellow() -> Color<T> {
    let zero: T = 0.0.cvt();
    let one: T = 1.0.cvt();
    Color::rgb(one, one, zero)
  }

  pub fn apply_gamma(&self, gamma: T) -> Color<T> {
    Color::rgba(self.r.powf(gamma), self.g.powf(gamma), self.b.powf(gamma), self.a)
  }
}

// TODO: does the multiplication of each component by that color's
// alpha make sense?
impl<T: Float + From<f32>> Add<Color<T>> for Color<T> {
  type Output = Color<T>;
  fn add(self, rhs: Color<T>) -> Color<T> {
    let half: T = 0.5.cvt();
    Color{r: self.r*self.a + rhs.r*rhs.a, g: self.g*self.a + rhs.g*rhs.a, b: self.b*self.a + rhs.b*rhs.a, a: (self.a+rhs.a)*half}
  }
}

impl<T: Float> Mul<T> for Color<T> {
  type Output = Color<T>;
  fn mul(self, rhs: T) -> Color<T> {
    Color{r: self.r*rhs, g: self.g*rhs, b: self.b*rhs, a: self.a}
  }
}
