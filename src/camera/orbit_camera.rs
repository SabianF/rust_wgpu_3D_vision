
use glam::{Mat4, Vec3};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::{extras::math::vector3::Vector3};

use super::camera::Camera;

/// An [OrbitCamera] only permits rotation of the eye on a spherical shell around a target.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Debug, Clone, Copy)]
pub struct OrbitCamera {
  /// The distance of the eye from the target.
  pub distance: f32,

  /// The pitch angle in radians.
  pub pitch: f32,

  /// The yaw angle in radians.
  pub yaw: f32,

  /// The eye of the camera in cartesian coordinates.
  pub(crate) eye: Vector3,

  /// The target of the orbit camera.
  pub target: Vector3,

  /// The cameras' up vector.
  pub up: Vector3,

  /// The bounds within which the camera can be moved.
  pub bounds: OrbitCameraBounds,

  /// The aspect ratio of the camera.
  pub aspect: f32,

  /// The field of view of the camera.
  pub fovy: f32,

  /// The near clipping plane of the camera.
  pub znear: f32,

  /// The far clipping plane of the camera.
  pub zfar: f32,
}

impl Camera for OrbitCamera {
  fn build_view_projection_matrix(&self) -> Mat4 {
    let view = Mat4::look_at_rh(self.eye.to_vec3(), self.target.to_vec3(), self.up.to_vec3());
    let proj = Mat4::perspective_rh(self.fovy, self.aspect, self.znear, self.zfar);
    proj * view
  }
}

impl OrbitCamera {
  /// Creates a new [OrbitCamera].
  ///
  /// Arguments:
  ///
  /// * `distance`: The distance of the eye to the `target`.
  /// * `pitch`: The pitch angle in radians.
  /// * `yaw`: The yaw angle in radians.
  /// * `target`: The point around which the camera rotates.
  /// * `aspect`: The aspect ratio of the camera.
  pub fn new(distance: f32, pitch: f32, yaw: f32, target: Vec3, aspect: f32) -> Self {
    let mut camera = Self {
      distance,
      pitch,
      yaw,
      eye: Vector3::from_vec3(Vec3::ZERO), // Will be auto-calculted in `update()` nevertheless.
      target: Vector3::from_vec3(target),
      up: Vector3::from_vec3(Vec3::Y),
      bounds: OrbitCameraBounds::default(),
      aspect,
      fovy: std::f32::consts::PI / 2.0,
      znear: 0.1,
      zfar: 1000.0,
    };
    camera.update();
    camera
  }

  /// Sets the distance of the [OrbitCamera] from the target.
  ///
  /// Arguments:
  ///
  /// * `distance`: The euclidean distance between the cameras' eye and the target.
  pub fn set_distance(&mut self, distance: f32) {
    self.distance = distance.clamp(
      self.bounds.min_distance.unwrap_or(f32::EPSILON),
      self.bounds.max_distance.unwrap_or(f32::MAX),
    );
    self.update();
  }

  /// Incrementally changes the distance of the [OrbitCamera] from the target.
  ///
  /// Arguments:
  ///
  /// `delta`: The amount by which the distance will be changed.
  pub fn add_distance(&mut self, delta: f32) {
    self.set_distance(self.distance + delta);
  }

  /// Sets the pitch of the [OrbitCamera].
  ///
  /// Arguments:
  ///
  /// * `pitch`: The new pitch angle in radians.
  pub fn set_pitch(&mut self, pitch: f32) {
    self.pitch = pitch.clamp(self.bounds.min_pitch, self.bounds.max_pitch);
    self.update();
  }

  /// Incrementally changes the pitch of the [OrbitCamera].
  ///
  /// Arguments:
  ///
  /// `delta`: The amount by which the pitch will be changed.
  pub fn add_pitch(&mut self, delta: f32) {
    self.set_pitch(self.pitch + delta);
  }

  /// Sets the yaw of the [OrbitCamera].
  ///
  /// Arguments:
  ///
  /// * `yaw`: The new yaw angle in radians.
  pub fn set_yaw(&mut self, yaw: f32) {
    let mut bounded_yaw = yaw;
    if let Some(min_yaw) = self.bounds.min_yaw {
      bounded_yaw = bounded_yaw.clamp(min_yaw, f32::MAX);
    }
    if let Some(max_yaw) = self.bounds.max_yaw {
      bounded_yaw = bounded_yaw.clamp(f32::MIN, max_yaw);
    }
    self.yaw = bounded_yaw;
    self.update();
  }

  /// Incrementally changes the yaw of the [OrbitCamera].
  ///
  /// Arguments:
  ///
  /// `delta`: The amount by which the yaw will be changed.
  pub fn add_yaw(&mut self, delta: f32) {
    self.set_yaw(self.yaw + delta);
  }

  /// Updates the camera after changing `distance`, `pitch` or `yaw`.
  pub fn update(&mut self) {
    self.eye = Vector3::from_vec3(calculate_cartesian_eye_position(
      self.pitch,
      self.yaw,
      self.distance,
    ));
  }
}

/// The boundaries for how an [OrbitCamera] can be rotated.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Debug, Clone, Copy)]
pub struct OrbitCameraBounds {
  /// The minimum distance between the eye and the target.
  /// This should not be negative. In order to ensure this the minimum distance
  /// should never be smaller than [f32::EPSILON].
  pub min_distance: Option<f32>,

  /// If set it is not possible to move the camera further from the target
  /// than the specified amount.
  pub max_distance: Option<f32>,

  /// The `min_pitch` can only be between `]-PI / 2, 0]` due to mathematical reasons.
  pub min_pitch: f32,

  /// The `min_pitch` can only be between `]0, PI / 2]` due to mathematical reasons.
  pub max_pitch: f32,

  /// If set the yaw angle will be constrained. The constrain should be in the
  /// interval `[-PI, 0]`.
  pub min_yaw: Option<f32>,

  /// If set the yaw angle will be constrained. The constrain should be in the
  /// interval `[0, PI]`.
  pub max_yaw: Option<f32>,
}

impl Default for OrbitCameraBounds {
  fn default() -> Self {
    Self {
      min_distance: None,
      max_distance: None,
      min_pitch: -std::f32::consts::PI / 2.0 + f32::EPSILON,
      max_pitch: std::f32::consts::PI / 2.0 - f32::EPSILON,
      min_yaw: None,
      max_yaw: None,
    }
  }
}

/// Calulcates the eye position in cartesian coordinates from spherical coordinates.
///
/// Arguments:
///
/// * `pitch`: The pitch angle in radians.
/// * `yaw`: The yaw angle in radians.
/// * `distance`: The euclidean distance to the target.
fn calculate_cartesian_eye_position(pitch: f32, yaw: f32, distance: f32) -> Vec3 {
  Vec3::new(
    distance * yaw.sin() * pitch.cos(),
    distance * pitch.sin(),
    distance * yaw.cos() * pitch.cos(),
  )
}
