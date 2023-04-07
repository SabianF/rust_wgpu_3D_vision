use game_loop::winit::window::Window;
use glam::Vec3;
use wgpu::{Buffer, BindGroupLayout, Device, util::DeviceExt, BindGroup};

use super::{camera::{CameraUniform}, orbit_camera::OrbitCamera, camera_controller::CameraController};

pub struct CameraState {
  pub camera                  : OrbitCamera,
  pub camera_controller       : CameraController,
  pub camera_uniform          : CameraUniform,
  pub camera_buffer           : Buffer,
  pub camera_bind_group_layout: BindGroupLayout,
  pub camera_bind_group       : BindGroup,
}

impl CameraState {
  pub fn new(device: &Device, window: &Window) -> Self {

    let size = window.inner_size();

    let mut camera = OrbitCamera::new(
      2.0,
      1.5,
      1.25,
      Vec3::new(0.0, 0.0, 0.0),
      size.width as f32 / size.height as f32,
    );
    camera.bounds.min_distance = Some(1.1);

    let camera_controller = CameraController::new(
      0.005,
      0.10,
    );

    let mut camera_uniform = CameraUniform::default();
    camera_uniform.update_view_proj(&camera);

    let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("Camera Buffer"),
      contents: bytemuck::cast_slice(&[camera_uniform]),
      usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let camera_bind_group_layout =
      device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[wgpu::BindGroupLayoutEntry {
          binding: 0,
          visibility: wgpu::ShaderStages::VERTEX,
          ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
          },
          count: None,
        }],
        label: Some("camera_bind_group_layout"),
      });

    let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
      layout: &camera_bind_group_layout,
      entries: &[wgpu::BindGroupEntry {
        binding: 0,
        resource: camera_buffer.as_entire_binding(),
      }],
      label: Some("camera_bind_group"),
    });

    return Self {
      camera                  ,
      camera_controller       ,
      camera_uniform          ,
      camera_buffer           ,
      camera_bind_group_layout,
      camera_bind_group       ,
    };
  }
}
