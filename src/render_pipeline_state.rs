use cgmath::{Zero, Rotation3, InnerSpace};
use wgpu::{RenderPipeline, Device, BindGroupLayout, SurfaceConfiguration, Buffer, util::DeviceExt};

pub const NUM_INSTANCES_PER_ROW: u32 = 3;
pub const NUM_INSTANCE_ROWS: u32 = 3;
const INSTANCES_OFFSET: cgmath::Vector3<f32> = cgmath::Vector3::new(
  NUM_INSTANCES_PER_ROW as f32 * 0.1,
  NUM_INSTANCE_ROWS as f32 * 0.1,
  NUM_INSTANCES_PER_ROW as f32 * 0.1,
);

pub struct RenderPipelineState {
  pub render_pipeline : RenderPipeline,
  pub instances       : Vec<Instance>,
  pub instance_buffer : Buffer,
  pub depth_texture   : Texture,
  pub instances_to_render_start: u32,
  pub instances_to_render_end: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
  pub position: [f32; 3], // [X, Y, Z]
  pub color   : [f32; 3], // [R, G, B]
}

/**
 * Defines the properties of different instances of objects/models
 */
pub struct Instance {
  pub position: cgmath::Vector3<f32>,
  pub rotation: cgmath::Quaternion<f32>,
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

pub struct Texture {
  pub texture : wgpu::Texture,
  pub view    : wgpu::TextureView,
  pub sampler : wgpu::Sampler,
}

impl RenderPipelineState {

  pub fn new(
    device: &Device,
    camera_bind_group_layout: &BindGroupLayout,
    config: &SurfaceConfiguration,
  ) -> Self {

    let render_pipeline = Self::configure_render_pipeline(
      device,
      camera_bind_group_layout,
      config,
    );

    let (
      instances,
      instance_buffer,
    ) = Self::configure_instances(device);

    let depth_texture = Texture::create_depth_texture(
      device,
      config,
      "depth_texture",
    );

    let instances_to_render_start = 0;
    let instances_to_render_end = NUM_INSTANCES_PER_ROW * NUM_INSTANCES_PER_ROW;

    return Self {
      render_pipeline,
      instances,
      instance_buffer,
      depth_texture,
      instances_to_render_start,
      instances_to_render_end,
    };
  }

  fn configure_render_pipeline(
    device: &Device,
    camera_bind_group_layout: &BindGroupLayout,
    config: &SurfaceConfiguration,
  ) -> RenderPipeline {
    let shader = device.create_shader_module(
      wgpu::include_wgsl!("shader.wgsl"),
    );
  
    let render_pipeline_layout = device.create_pipeline_layout(
      &wgpu::PipelineLayoutDescriptor {
        label               : Some("Render pipeline layout"),
        bind_group_layouts  : &[camera_bind_group_layout],
        push_constant_ranges: &[],
      },
    );
  
    let render_pipeline = device.create_render_pipeline(
      &wgpu::RenderPipelineDescriptor {
        label : Some("Render pipeline"),
        layout: Some(&render_pipeline_layout),
  
        vertex: wgpu::VertexState {
          module      : &shader,
          entry_point : "vs_main",
  
          // This tells `wgpu` what type of vertices to pass to the
          // vertex shader. Since we're specifying the vertices in the vertex
          // shader itself, empty is OK
          buffers: &[
            Vertex::desc(),
            InstanceRaw::desc(),
          ],
        },
  
        // Stores color data to the `surface`
        fragment: Some(wgpu::FragmentState {
          module      : &shader,
          entry_point : "fs_main",
  
          // Tells `wgpu` what color outputs it should set up. Currently,
          // we only need one for the surface.
          targets: &[Some(wgpu::ColorTargetState {
  
            // Uses the `surface` format so copying to it is easy
            format: config.format,
  
            // Tells the blending to replace old pixel data with new data
            blend: Some(wgpu::BlendState::REPLACE),
  
            // Tells `wgpu` to write to all colors: red, blue, green, alpha
            write_mask: wgpu::ColorWrites::ALL,
          })],
        }),
  
        primitive: wgpu::PrimitiveState {
  
          // Makes every three vertices correspond to one triangle
          topology          : wgpu::PrimitiveTopology::TriangleList,
          strip_index_format: None,
  
          // Defines the forward direction
          front_face: wgpu::FrontFace::Ccw,
  
          // Culls triangles which are not facing forward
          cull_mode: Some(wgpu::Face::Back),
  
          unclipped_depth   : false,
          polygon_mode      : wgpu::PolygonMode::Fill,
          conservative      : false,
        },
  
        depth_stencil : Some(wgpu::DepthStencilState {
          format              : Texture::DEPTH_FORMAT,
          depth_write_enabled : true,
          depth_compare       : wgpu::CompareFunction::Less,
          stencil             : wgpu::StencilState::default(),
          bias                : wgpu::DepthBiasState::default(),
        }),
  
        multisample: wgpu::MultisampleState {
          count : 1,
          mask  : !0,
          alpha_to_coverage_enabled: false,
        },
  
        multiview: None,
      },
    );
    
    return render_pipeline;
  }

  fn configure_instances(device: &Device) -> (
    Vec<Instance>,
    Buffer,
  ) {
    let instances =
      (0..NUM_INSTANCE_ROWS).flat_map(|y| {
        (0..NUM_INSTANCES_PER_ROW).flat_map(move |z| {
          (0..NUM_INSTANCES_PER_ROW).map(move |x| {
            let position = cgmath::Vector3 {
              // Individual instance position offsets
              x: x as f32 * 0.2,
              y: y as f32 * 0.2,
              z: z as f32 * 0.2,
            } - INSTANCES_OFFSET;
  
            let rotation = if position.is_zero() {
              // this is needed so an object at (0, 0, 0) won't get scaled to zero
              // as Quaternions can effect scale if they're not created correctly
              cgmath::Quaternion::from_axis_angle(
                cgmath::Vector3::unit_z(),
                cgmath::Deg(0.0)
              )
            } else {
              cgmath::Quaternion::from_axis_angle(
                position.normalize(),
                cgmath::Deg(0.0)
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
      &wgpu::util::BufferInitDescriptor {
        label   : Some("Instance buffer"),
        contents: bytemuck::cast_slice(&instance_data),
        usage   : wgpu::BufferUsages::VERTEX,
      },
    );
  
    return (
      instances,
      instance_buffer,
    );
  }
}

impl Vertex {

  const ATTRIBUTES: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
    0 => Float32x3,
    1 => Float32x3,
  ];

  fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
    wgpu::VertexBufferLayout {
      array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
      step_mode   : wgpu::VertexStepMode::Vertex,
      attributes  : &Self::ATTRIBUTES,
    }
  }
}

impl Instance {
  fn to_raw(&self) -> InstanceRaw {
    InstanceRaw {
      model: (
        cgmath::Matrix4::from_translation(self.position)
        * cgmath::Matrix4::from(self.rotation)
      ).into(),
    }
  }
}

impl InstanceRaw {
  fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
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

impl Texture {

  pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
    
  pub fn create_depth_texture(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
    label: &str
  ) -> Self {
    let size = wgpu::Extent3d {
      width: config.width,
      height: config.height,
      depth_or_array_layers: 1,
    };

    let desc = wgpu::TextureDescriptor {
      label: Some(label),
      size,
      mip_level_count: 1,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: Self::DEPTH_FORMAT,
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT
          | wgpu::TextureUsages::TEXTURE_BINDING,
      view_formats: Default::default(),
    };

    let texture = device.create_texture(&desc);
  
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
  
    let sampler = device.create_sampler(
      &wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Nearest,
        compare: Some(wgpu::CompareFunction::LessEqual),
        lod_min_clamp: 0.0,
        lod_max_clamp: 100.0,
        ..Default::default()
      }
    );
    
    return Self {
      texture,
      view,
      sampler,
    };
  }
}
