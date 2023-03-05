use game_loop::winit::event::{WindowEvent, KeyboardInput, ElementState, VirtualKeyCode};
use wgpu::{Buffer, BindGroupLayout, BindGroup, SurfaceConfiguration, util::DeviceExt};
use cgmath::prelude::*;

use crate::render_state::RenderState;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

pub struct CameraState {
  pub camera                  : Camera,
  pub camera_uniform          : CameraUniform,
  pub camera_buffer           : Buffer,
  pub camera_bind_group_layout: BindGroupLayout,
  pub camera_bind_group       : BindGroup,
  pub camera_controller       : CameraController,
}

pub struct Camera {
  eye   : cgmath::Point3<f32>,
  target: cgmath::Point3<f32>,
  up    : cgmath::Vector3<f32>,
  aspect: f32,
  fovy  : f32,
  znear : f32,
  zfar  : f32,
}

pub struct CameraController {
  speed: f32,
  is_forward_pressed: bool,
  is_backward_pressed: bool,
  is_left_pressed: bool,
  is_right_pressed: bool,
}

// This is needed to store data correctly for the shaders
#[repr(C)]
// This is needed to store data in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
  // We can't use cgmath with bytemuck directly so we first have
  // to convert the Matrix4 into a 4x4 f32 array
  view_proj: [[f32; 4]; 4],
}

impl CameraState {

  pub fn new(render_state: &RenderState) -> Self {

    let config = &render_state.config;
    let device = &render_state.device;

    let (
      camera,
      camera_uniform,
      camera_buffer,
      camera_bind_group_layout,
      camera_bind_group,
      camera_controller,
    ) = Camera::configure_camera(config, device);

    return Self {
      camera                  : camera,
      camera_uniform          : camera_uniform,
      camera_buffer           : camera_buffer,
      camera_bind_group_layout: camera_bind_group_layout,
      camera_bind_group       : camera_bind_group,
      camera_controller       : camera_controller,
    };
  }
}

impl CameraUniform {
  fn new() -> Self {
      Self {
          view_proj: cgmath::Matrix4::identity().into(),
      }
  }

  pub fn update_view_proj(&mut self, camera: &Camera) {
      self.view_proj = camera.build_view_projection_matrix().into();
  }
}

impl Camera {
  pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
      // 1.
      let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
      // 2.
      let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);

      // 3.
      return OPENGL_TO_WGPU_MATRIX * proj * view;
  }

  /**
   * Configures the viewing camera angles and rotations
   */
  pub fn configure_camera(
    config: &SurfaceConfiguration,
    device: &wgpu::Device,
  ) -> (
    Camera,
    CameraUniform,
    wgpu::Buffer,
    wgpu::BindGroupLayout,
    wgpu::BindGroup,
    CameraController,
  ) {
    let camera = Camera {
      // position the camera one unit up and 2 units back
      // +z is out of the screen
      eye: (0.0, 1.0, 2.0).into(),
      // have it look at the origin
      target: (0.0, 0.0, 0.0).into(),
      // which way is "up"
      up: cgmath::Vector3::unit_y(),
      aspect: config.width as f32 / config.height as f32,
      fovy: 45.0,
      znear: 0.1,
      zfar: 100.0,
    };

    let mut camera_uniform = CameraUniform::new();
    camera_uniform.update_view_proj(&camera);

    let camera_buffer = device.create_buffer_init(
      &wgpu::util::BufferInitDescriptor {
        label   : Some("Camera Buffer"),
        contents: bytemuck::cast_slice(&[camera_uniform]),
        usage   : wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
      },
    );

    let camera_bind_group_layout = device.create_bind_group_layout(
      &wgpu::BindGroupLayoutDescriptor {
        label: Some("camera_bind_group_layout"),

        entries: &[
          wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
              ty: wgpu::BufferBindingType::Uniform,
              has_dynamic_offset: false,
              min_binding_size: None,
            },
            count: None,
          }
        ],
      },
    );

    let camera_bind_group = device.create_bind_group(
      &wgpu::BindGroupDescriptor {
        layout: &camera_bind_group_layout,
        entries: &[
          wgpu::BindGroupEntry {
            binding: 0,
            resource: camera_buffer.as_entire_binding(),
          }
        ],
        label: Some("camera_bind_group"),
      },
    );

    let camera_controller = CameraController::new(0.2);

    return (
      camera,
      camera_uniform,
      camera_buffer,
      camera_bind_group_layout,
      camera_bind_group,
      camera_controller,
    )
  }
}

impl CameraController {
  fn new(speed: f32) -> Self {
    Self {
      speed,
      is_forward_pressed  : false,
      is_backward_pressed : false,
      is_left_pressed     : false,
      is_right_pressed    : false,
    }
  }

  pub fn process_events(&mut self, event: &WindowEvent) -> bool {
    match event {
      WindowEvent::KeyboardInput {
        input: KeyboardInput {
          state,
          virtual_keycode: Some(keycode),
          ..
        },
        ..
      } => {
        let is_pressed = *state == ElementState::Pressed;
        match keycode {
          VirtualKeyCode::W | VirtualKeyCode::Up => {
            self.is_forward_pressed = is_pressed;
            true
          }
          VirtualKeyCode::A | VirtualKeyCode::Left => {
            self.is_left_pressed = is_pressed;
            true
          }
          VirtualKeyCode::S | VirtualKeyCode::Down => {
            self.is_backward_pressed = is_pressed;
            true
          }
          VirtualKeyCode::D | VirtualKeyCode::Right => {
            self.is_right_pressed = is_pressed;
            true
          }
          _ => false,
        }
      }
      _ => false,
    }
  }

  pub fn update_camera(&self, camera: &mut Camera) {
    let forward = camera.target - camera.eye;
    let forward_norm = forward.normalize();
    let forward_mag = forward.magnitude();

    // Prevents glitching when camera gets too close to the
    // center of the scene.
    if self.is_forward_pressed && forward_mag > self.speed {
      camera.eye += forward_norm * self.speed;
    }
    if self.is_backward_pressed {
      camera.eye -= forward_norm * self.speed;
    }

    let right = forward_norm.cross(camera.up);

    // Redo radius calc in case the fowrard/backward is pressed.
    let forward = camera.target - camera.eye;
    let forward_mag = forward.magnitude();

    if self.is_right_pressed {
      // Rescale the distance between the target and eye so 
      // that it doesn't change. The eye therefore still 
      // lies on the circle made by the target and eye.
      camera.eye = camera.target - (forward + right * self.speed).normalize() * forward_mag;
    }
    if self.is_left_pressed {
      camera.eye = camera.target - (forward - right * self.speed).normalize() * forward_mag;
    }
  }
}
