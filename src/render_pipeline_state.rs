use wgpu::{RenderPipeline, Device, BindGroupLayout, SurfaceConfiguration};

use crate::instance::{InstanceBuffer, InstanceRaw, NUM_INSTANCES_PER_ROW, NUM_INSTANCES_PER_COL};

pub struct RenderPipelineState {
  pub render_pipeline : RenderPipeline,
  pub instance_buffer : InstanceBuffer,
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

    let depth_texture = Texture::create_depth_texture(
      device,
      config,
      "depth_texture",
    );

    let instance_buffer = InstanceBuffer::new(&device);
    let instances_to_render_start = 0;
    let instances_to_render_end = NUM_INSTANCES_PER_ROW * NUM_INSTANCES_PER_COL;

    return Self {
      render_pipeline,
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
