use glam::Mat4;

use super::orbit_camera::OrbitCamera;

/// A camera is used for rendering specific parts of the scene.
pub trait Camera: Sized {
  fn build_view_projection_matrix(&self) -> Mat4;
}

/// The camera uniform contains the data linked to the camera that is passed to the shader.
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
  /// The eye position of the camera in homogenous coordinates.
  ///
  /// Homogenous coordinates are used to fullfill the 16 byte alignment requirement.
  pub view_position: [f32; 4],

  /// Contains the view projection matrix.
  pub view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
  /// Updates the view projection matrix of this [CameraUniform].
  ///
  /// Arguments:
  /// * `camera`: The [OrbitCamera] from which the matrix will be computed.
  pub fn update_view_proj(&mut self, camera: &OrbitCamera) {
      self.view_position = [camera.eye.x, camera.eye.y, camera.eye.z, 1.0];
      self.view_proj = camera.build_view_projection_matrix().to_cols_array_2d();
  }
}

impl Default for CameraUniform {
  /// Creates a default [CameraUniform].
  fn default() -> Self {
      Self {
          view_position: [0.0; 4],
          view_proj: Mat4::IDENTITY.to_cols_array_2d(),
      }
  }
}
