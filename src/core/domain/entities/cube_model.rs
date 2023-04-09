use wgpu::{Buffer, Device, util::{DeviceExt, self}, BufferUsages};

use crate::core::presentation::states::render_pipeline_state::Vertex;

const CUBE_VERTICES: &[Vertex] = &[
  Vertex { position: [-0.1, -0.1, 0.1 ], color: [0.0, 0.0, 0.0] }, // A: 0
  Vertex { position: [0.1 , -0.1, 0.1 ], color: [0.0, 0.0, 0.5] }, // B: 1
  Vertex { position: [0.1 , 0.1 , 0.1 ], color: [0.0, 0.5, 0.0] }, // C: 2
  Vertex { position: [-0.1, 0.1 , 0.1 ], color: [0.0, 0.5, 0.5] }, // D: 3
  Vertex { position: [-0.1, -0.1, -0.1], color: [0.5, 0.0, 0.0] }, // E: 4
  Vertex { position: [0.1 , -0.1, -0.1], color: [0.5, 0.0, 0.5] }, // F: 5
  Vertex { position: [0.1 , 0.1 , -0.1], color: [0.5, 0.5, 0.0] }, // G: 6
  Vertex { position: [-0.1, 0.1 , -0.1], color: [0.5, 0.5, 0.5] }, // H: 7
];

const CUBE_INDICES: &[u16] = &[
  // Front
  0, 1, 2,
  0, 2, 3,

  // Top
  0, 4, 1,
  1, 4, 5,

  // Left
  1, 5, 2,
  2, 5, 6,

  // Bottom
  2, 6, 7,
  2, 7, 3,

  // Right
  3, 4, 0,
  3, 7, 4,

  // Back
  6, 5, 4,
  4, 7, 6,
];

pub struct CubeModel {
  pub cube_vertex_buffer: Buffer,
  pub cube_index_buffer : Buffer,
  pub cube_indices_count: u32,
}

impl CubeModel {

  pub fn new(device: &Device) -> Self {

    let (
      cube_vertex_buffer,
      cube_index_buffer,
      cube_indices_count,
    ) = Self::define_cube(device);

    return Self {
      cube_vertex_buffer,
      cube_index_buffer,
      cube_indices_count,
    };
  }

  /**
   * Defines vertices & indices buffers for a cube
   */
  fn define_cube(device: &Device) -> (
    Buffer,
    Buffer,
    u32,
  ) {
    let cube_vertex_buffer = device.create_buffer_init(
      &util::BufferInitDescriptor {
        label   : Some("Cube vertex buffer"),
        contents: bytemuck::cast_slice(CUBE_VERTICES),
        usage   : BufferUsages::VERTEX,
      },
    );
  
    let cube_index_buffer = device.create_buffer_init(
      &util::BufferInitDescriptor {
        label   : Some("Cube index buffer"),
        contents: bytemuck::cast_slice(CUBE_INDICES),
        usage   : BufferUsages::INDEX,
      },
    );
  
    let cube_indices_count = CUBE_INDICES.len() as u32;
  
    return (
      cube_vertex_buffer,
      cube_index_buffer,
      cube_indices_count,
    );
  }
}
