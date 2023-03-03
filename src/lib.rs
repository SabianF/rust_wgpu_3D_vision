#![allow(non_snake_case)]

use wgpu::{util::DeviceExt, SurfaceConfiguration};
use winit::{
  event::{Event, WindowEvent, KeyboardInput, ElementState, VirtualKeyCode},
  event_loop::{ControlFlow, EventLoop},
  window::{Window},
};
use cgmath::prelude::*;

#[cfg(target_arch="wasm32")]
use wasm_bindgen::prelude::*;

// ============================================================
// Types
// ============================================================

struct Camera {
  eye   : cgmath::Point3<f32>,
  target: cgmath::Point3<f32>,
  up    : cgmath::Vector3<f32>,
  aspect: f32,
  fovy  : f32,
  znear : f32,
  zfar  : f32,
}

struct CameraController {
  speed: f32,
  is_forward_pressed: bool,
  is_backward_pressed: bool,
  is_left_pressed: bool,
  is_right_pressed: bool,
}

struct State {
  surface           : wgpu::Surface,
  device            : wgpu::Device,
  queue             : wgpu::Queue,
  config            : wgpu::SurfaceConfiguration,
  size              : winit::dpi::PhysicalSize<u32>,
  window            : Window,
  render_pipeline   : wgpu::RenderPipeline,
  cube_vertex_buffer: wgpu::Buffer,
  cube_index_buffer : wgpu::Buffer,
  cube_indices_count: u32,
  camera            : Camera,
  camera_uniform    : CameraUniform,
  camera_buffer     : wgpu::Buffer,
  camera_bind_group : wgpu::BindGroup,
  camera_controller : CameraController,
  instances         : Vec<Instance>,
  instance_buffer   : wgpu::Buffer,
  depth_texture     : Texture,
  instances_to_render_start: u32,
  instances_to_render_end: u32,
}

// This is needed to store data correctly for the shaders
#[repr(C)]
// This is needed to store data in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
  // We can't use cgmath with bytemuck directly so we first have
  // to convert the Matrix4 into a 4x4 f32 array
  view_proj: [[f32; 4]; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
  position: [f32; 3], // [X, Y, Z]
  color   : [f32; 3], // [R, G, B]
}

/**
 * Defines the properties of different instances of objects/models
 */
struct Instance {
  position: cgmath::Vector3<f32>,
  rotation: cgmath::Quaternion<f32>,
}

/**
 * A workaround to use quaternions in WGSL, since WGSL doesn't use quaternions
 * We're using this format to easily input rotation data into the Buffer
 */
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct InstanceRaw {
    model: [[f32; 4]; 4],
}

struct Texture {
  texture : wgpu::Texture,
  view    : wgpu::TextureView,
  sampler : wgpu::Sampler,
}

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

const CUBE_VERTICES: &[Vertex] = &[
  Vertex { position: [-0.5, -0.5, 0.5 ], color: [0.0, 0.0, 0.0] }, // A: 0
  Vertex { position: [0.5 , -0.5, 0.5 ], color: [0.0, 0.0, 0.5] }, // B: 1
  Vertex { position: [0.5 , 0.5 , 0.5 ], color: [0.0, 0.5, 0.0] }, // C: 2
  Vertex { position: [-0.5, 0.5 , 0.5 ], color: [0.0, 0.5, 0.5] }, // D: 3
  Vertex { position: [-0.5, -0.5, -0.5], color: [0.5, 0.0, 0.0] }, // E: 4
  Vertex { position: [0.5 , -0.5, -0.5], color: [0.5, 0.0, 0.5] }, // F: 5
  Vertex { position: [0.5 , 0.5 , -0.5], color: [0.5, 0.5, 0.0] }, // G: 6
  Vertex { position: [-0.5, 0.5 , -0.5], color: [0.5, 0.5, 0.5] }, // H: 7
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

const NUM_INSTANCES_PER_ROW: u32 = 3;
const NUM_INSTANCE_ROWS: u32 = 3;
const INSTANCE_SPACING: cgmath::Vector3<f32> = cgmath::Vector3::new(
  NUM_INSTANCES_PER_ROW as f32 * 0.5,
  NUM_INSTANCE_ROWS as f32 * 0.5,
  NUM_INSTANCES_PER_ROW as f32 * 0.5,
);

// ============================================================
// Functions
// ============================================================

impl CameraUniform {
  fn new() -> Self {
      Self {
          view_proj: cgmath::Matrix4::identity().into(),
      }
  }

  fn update_view_proj(&mut self, camera: &Camera) {
      self.view_proj = camera.build_view_projection_matrix().into();
  }
}

impl Camera {
  fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
      // 1.
      let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
      // 2.
      let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);

      // 3.
      return OPENGL_TO_WGPU_MATRIX * proj * view;
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

  fn process_events(&mut self, event: &WindowEvent) -> bool {
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

  fn update_camera(&self, camera: &mut Camera) {
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

impl State {

  async fn new(window: Window) -> Self {

    let (
      size,
      surface,
      device,
      queue,
      config,
    ) = configure_surface(&window).await;

    let (
      camera,
      camera_uniform,
      camera_buffer,
      camera_bind_group_layout,
      camera_bind_group,
      camera_controller,
    ) = configure_camera(&config, &device);

    let render_pipeline = configure_render_pipeline(
      &device,
      camera_bind_group_layout,
      &config,
    );

    let (
      cube_vertex_buffer,
      cube_index_buffer,
      cube_indices_count,
    ) = define_cube(&device);

    let (
      instances,
      instance_buffer,
    ) = configure_instances(&device);

    let depth_texture = Texture::create_depth_texture(
      &device,
      &config,
      "depth_texture",
    );

    let instances_to_render_start = 0;
    let instances_to_render_end = NUM_INSTANCES_PER_ROW * NUM_INSTANCES_PER_ROW;

    Self {
      window,
      surface,
      device,
      queue,
      config,
      size,
      render_pipeline,
      cube_vertex_buffer,
      cube_index_buffer,
      cube_indices_count,
      camera,
      camera_uniform,
      camera_buffer,
      camera_bind_group,
      camera_controller,
      instances,
      instance_buffer,
      depth_texture,
      instances_to_render_start,
      instances_to_render_end,
    }
  }

  pub fn window(&self) -> &Window {
    return &self.window;
  }

  fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {

    if new_size.width > 0 && new_size.height > 0 {
      self.size = new_size;
      self.config.width = new_size.width;
      self.config.height = new_size.height;
      self.surface.configure(&self.device, &self.config);
    }

    self.depth_texture = Texture::create_depth_texture(
      &self.device,
      &self.config,
      "depth_texture",
    );
  }

  fn input(&mut self, event: &WindowEvent) -> bool {
    self.camera_controller.process_events(event);
    return false;
  }

  fn iterate_volume_plane_instances_to_render(&mut self) {
    let range_increment_amount = NUM_INSTANCES_PER_ROW * NUM_INSTANCES_PER_ROW;
    let range_end_max = self.instances.len() as u32;
    let range_start_max = range_end_max - range_increment_amount;

    let range_end_min = range_increment_amount;
    let range_start_min = 0;

    if self.instances_to_render_start + range_increment_amount <= range_start_max {
      self.instances_to_render_start += range_increment_amount;
    } else {
      self.instances_to_render_start = range_start_min;
    }
    if self.instances_to_render_end + range_increment_amount <= range_end_max {
      self.instances_to_render_end += range_increment_amount;
    } else {
      self.instances_to_render_end = range_end_min;
    }
  }

  fn update(&mut self) {
    self.camera_controller.update_camera(&mut self.camera);
    self.camera_uniform.update_view_proj(&self.camera);
    self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform]));
  }

  fn render(&mut self) -> Result<(), wgpu::SurfaceError> {

    self.iterate_volume_plane_instances_to_render();

    let output = self.surface.get_current_texture()?;
    let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
      label: Some("Render Encoder"),
    });

    // Since `begin_render_pass()` borrows `encoder` mutably (aka &mut self),
    // we need to use a scoped block to release this mutable borrow, to call
    // encoder.finish()
    {
      let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Render Pass"),

        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
          view          : &view,
          resolve_target: None,

          ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(wgpu::Color {
              r: 0.1,
              g: 0.2,
              b: 0.3,
              a: 1.0,
            }),

            store: true,
          },
        })],

        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
          view: &self.depth_texture.view,

          depth_ops: Some(wgpu::Operations {
            load: wgpu::LoadOp::Clear(1.0),
            store: true,
          }),

          stencil_ops : None,
        }),
      });

      render_pass.set_pipeline(&self.render_pipeline);

      render_pass.set_bind_group(
        0,
        &self.camera_bind_group,
        &[],
      );

      render_pass.set_vertex_buffer(
        0,
        self.cube_vertex_buffer.slice(..),
      );
      render_pass.set_vertex_buffer(
        1,
        self.instance_buffer.slice(..),
      );
      render_pass.set_index_buffer(
        self.cube_index_buffer.slice(..),
        wgpu::IndexFormat::Uint16,
      );
      render_pass.draw_indexed(
        0..self.cube_indices_count,
        0,
        self.instances_to_render_start..self.instances_to_render_end,
      );
    }

    self.queue.submit(std::iter::once(encoder.finish()));
    output.present();
    return Ok(());
  }
}

pub async fn run() {
  env_logger::init();

  let event_loop = EventLoop::new();

  let window = Window::new(&event_loop).unwrap();
  window.set_title("2D Object Renderer");

  let mut state = State::new(window).await;

  event_loop.run(move |event, _, control_flow| {
    *control_flow = ControlFlow::Poll;
    match event {
      Event::WindowEvent {
        ref event,
        window_id,
      } if window_id == state.window().id() => {
        if !state.input(event) {
          match event {

            WindowEvent::CloseRequested => {
              *control_flow = ControlFlow::Exit;
            },

            WindowEvent::Resized(physical_size) => {
              state.resize(*physical_size);
            },
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
              state.resize(**new_inner_size);
            },

            _ => {}
          }
        }
      }

      // Rendering the surface
      Event::RedrawRequested(window_id) if window_id == state.window().id() => {
        state.update();
        match state.render() {
          Ok(_) => {}

          // Reconfigure the surface if lost
          Err(wgpu::SurfaceError::Lost) => state.resize(state.size),

          // Prevent excess memory usage
          Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,

          // Handle all other errors
          Err(e) => eprintln!("{:?}", e),
        }
        state.window().request_redraw();
      }

      Event::MainEventsCleared => {
        // Ensure RedrawRequested triggers only once, unless manually
        // requested.
        state.window().request_redraw();
      }

      _ => {}
    }
  });
}

fn configure_instances(device: &wgpu::Device) -> (
  Vec<Instance>,
  wgpu::Buffer,
) {
  let instances =
    (0..NUM_INSTANCE_ROWS).flat_map(|y| {
      (0..NUM_INSTANCES_PER_ROW).flat_map(move |z| {
        (0..NUM_INSTANCES_PER_ROW).map(move |x| {
          let position = cgmath::Vector3 {
            x: x as f32,
            y: y as f32,
            z: z as f32,
          } - INSTANCE_SPACING;

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

/**
 * Defines vertices & indices buffers for a cube
 */
fn define_cube(device: &wgpu::Device) -> (
  wgpu::Buffer,
  wgpu::Buffer,
  u32,
) {
  let cube_vertex_buffer = device.create_buffer_init(
    &wgpu::util::BufferInitDescriptor {
      label   : Some("Cube vertex buffer"),
      contents: bytemuck::cast_slice(CUBE_VERTICES),
      usage   : wgpu::BufferUsages::VERTEX,
    },
  );

  let cube_index_buffer = device.create_buffer_init(
    &wgpu::util::BufferInitDescriptor {
      label   : Some("Cube index buffer"),
      contents: bytemuck::cast_slice(CUBE_INDICES),
      usage   : wgpu::BufferUsages::INDEX,
    },
  );

  let cube_indices_count = CUBE_INDICES.len() as u32;

  return (
    cube_vertex_buffer,
    cube_index_buffer,
    cube_indices_count
  );
}

/**
 * Configures the shaders and render pipeline
 */
fn configure_render_pipeline(
  device: &wgpu::Device,
  camera_bind_group_layout: wgpu::BindGroupLayout,
  config: &SurfaceConfiguration,
) -> wgpu::RenderPipeline {
  let shader = device.create_shader_module(
    wgpu::include_wgsl!("shader.wgsl"),
  );

  let render_pipeline_layout = device.create_pipeline_layout(
    &wgpu::PipelineLayoutDescriptor {
      label               : Some("Render pipeline layout"),
      bind_group_layouts  : &[&camera_bind_group_layout],
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

/**
 * Configures the rendering window and surface
 */
async fn configure_surface(window: &Window) -> (
  winit::dpi::PhysicalSize<u32>,
  wgpu::Surface,
  wgpu::Device,
  wgpu::Queue,
  SurfaceConfiguration,
) {
  /*
    * The window size
    */
  let size = window.inner_size();

  /*
    * The handle to the GPU
    */
  let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
    backends: wgpu::Backends::all(),
    dx12_shader_compiler: Default::default(),
  });

  /*
    * This ensures the surface only lives as long as its parent window
    */
  let surface = unsafe { instance.create_surface(window) }.unwrap();

  let adapter = instance.request_adapter(
    &wgpu::RequestAdapterOptions {
      power_preference      : wgpu::PowerPreference::default(),
      compatible_surface    : Some(&surface),
      force_fallback_adapter: true,
    },
  ).await.unwrap();

  let (device, queue) = adapter.request_device(
    &wgpu::DeviceDescriptor {
      features: wgpu::Features::empty(),
      limits  : if cfg!(target_arch = "wasm32") {
        wgpu::Limits::downlevel_webgl2_defaults()
      } else {
        wgpu::Limits::default()
      },
      label: None,
    },
    None,
  ).await.unwrap();

  let surface_caps = surface.get_capabilities(&adapter);

  let surface_format = surface_caps.formats.iter()
    .copied()
    .filter(|f| f.describe().srgb)
    .next()
    .unwrap_or(surface_caps.formats[0]);

  let config = wgpu::SurfaceConfiguration {
    usage       : wgpu::TextureUsages::RENDER_ATTACHMENT,
    format      : surface_format,
    width       : size.width,
    height      : size.height,
    present_mode: surface_caps.present_modes[0],
    alpha_mode: surface_caps.alpha_modes[0],
    view_formats: vec![],
  };

  surface.configure(&device, &config);
  return (size, surface, device, queue, config);
}

/**
 * Configures the viewing camera angles and rotations
 */
fn configure_camera(
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
