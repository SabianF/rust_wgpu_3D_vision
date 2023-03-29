
use glam::Vec3;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// A three-dimensional vector mainly used to pass data via `wasm-bindgen`.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Debug, Clone, Copy)]
pub struct Vector3 {
  pub x: f32,
  pub y: f32,
  pub z: f32,
}

impl Vector3 {
  /// Creates a new [Vector3] from a [Vec3].
  ///
  /// Arguments:
  ///
  /// * `vec3`: The [Vec3] that will be converted.
  pub fn from_vec3(vec3: Vec3) -> Self {
    Self {
      x: vec3.x,
      y: vec3.y,
      z: vec3.z,
    }
  }

  /// Creates a new [Vec3] from this [Vector3]
  pub fn to_vec3(&self) -> Vec3 {
    Vec3::new(self.x, self.y, self.z)
  }
}