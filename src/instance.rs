use cgmath::{Vector3, Quaternion, InnerSpace, Zero, Rotation3, Deg, Matrix4};
use wgpu::{Device, Buffer, util::{DeviceExt, BufferInitDescriptor}, BufferUsages};

pub const NUM_INSTANCES_PER_ROW : u32 = 5;
pub const NUM_INSTANCES_PER_COL : u32 = 5;
pub const NUM_INSTANCE_PLANES   : u32 = 2;

const INSTANCES_OFFSET: cgmath::Vector3<f32> = cgmath::Vector3::new(
  NUM_INSTANCES_PER_ROW as f32 * 0.1,
  NUM_INSTANCE_PLANES as f32 * 0.1,
  NUM_INSTANCES_PER_ROW as f32 * 0.1,
);

/**
 * Defines the properties of different instances of objects/models
 */
pub struct Instance {
  pub position: Vector3<f32>,
  pub rotation: Quaternion<f32>,
}

/**
 * A workaround to use quaternions in WGSL, since WGSL doesn't use quaternions
 * We're using this format to easily input rotation data into the Buffer
 */
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
  pub model: [[f32; 4]; 4],
}

pub struct InstanceBuffer {
  pub instances : Vec<Instance>,
  pub buffer    : Buffer,
}

impl Instance {
  fn to_raw(&self) -> InstanceRaw {
    InstanceRaw {
      model: (
        Matrix4::from_translation(self.position)
        * Matrix4::from(self.rotation)
      ).into(),
    }
  }
}

impl InstanceRaw {
  pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
    use std::mem;
    wgpu::VertexBufferLayout {
      array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
      // We need to switch from using a step mode of Vertex to Instance
      // This means that our shaders will only change to use the next
      // instance when the shader starts processing a new instance
      step_mode: wgpu::VertexStepMode::Instance,
      attributes: &[
        wgpu::VertexAttribute {
          offset: 0,
          // While our vertex shader only uses locations 0, and 1 now, in later tutorials we'll
          // be using 2, 3, and 4, for Vertex. We'll start at slot 5 not conflict with them later
          shader_location: 5,
          format: wgpu::VertexFormat::Float32x4,
        },
        // A mat4 takes up 4 vertex slots as it is technically 4 vec4s. We need to define a slot
        // for each vec4. We'll have to reassemble the mat4 in
        // the shader.
        wgpu::VertexAttribute {
          offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
          shader_location: 6,
          format: wgpu::VertexFormat::Float32x4,
        },
        wgpu::VertexAttribute {
          offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
          shader_location: 7,
          format: wgpu::VertexFormat::Float32x4,
        },
        wgpu::VertexAttribute {
          offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
          shader_location: 8,
          format: wgpu::VertexFormat::Float32x4,
        },
      ],
    }
  }
}

impl InstanceBuffer {
  pub fn new(device: &Device) -> Self {
    let (
      instances,
      buffer,
    ) = Self::configure_instances(device);

    return Self {
      instances,
      buffer,
    };
  }

  fn configure_instances(device: &Device) -> (
    Vec<Instance>,
    Buffer,
  ) {
    let instances =
      (0..NUM_INSTANCE_PLANES).flat_map(|y| {
        (0..NUM_INSTANCES_PER_COL).flat_map(move |z| {
          (0..NUM_INSTANCES_PER_ROW).map(move |x| {
            let position = Vector3 {
              // Individual instance position offsets
              x: x as f32 * 0.2,
              y: y as f32 * 0.2,
              z: z as f32 * 0.2,
            } - INSTANCES_OFFSET;
  
            let rotation = if position.is_zero() {
              // this is needed so an object at (0, 0, 0) won't get scaled to zero
              // as Quaternions can effect scale if they're not created correctly
              Quaternion::from_axis_angle(
                Vector3::unit_z(),
                Deg(0.0)
              )
            } else {
              Quaternion::from_axis_angle(
                position.normalize(),
                Deg(0.0)
              )
            };
  
            return Instance {
              position,
              rotation,
            };
          })
        })
      })
      .collect::<Vec<_>>();
  
    let instance_data = instances
      .iter()
      .map(Instance::to_raw)
      .collect::<Vec<_>>();
  
    let instance_buffer = device.create_buffer_init(
      &BufferInitDescriptor {
        label   : Some("Instance buffer"),
        contents: bytemuck::cast_slice(&instance_data),
        usage   : BufferUsages::VERTEX,
      },
    );
  
    return (
      instances,
      instance_buffer,
    );
  }
}