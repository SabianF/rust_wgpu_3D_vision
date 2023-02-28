
use wgpu::util::DeviceExt;
use winit::{
  event::{Event, WindowEvent, KeyboardInput, ElementState, VirtualKeyCode},
  event_loop::{ControlFlow, EventLoop},
  window::{Window},
};

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

struct State {
  surface               : wgpu::Surface,
  device                : wgpu::Device,
  queue                 : wgpu::Queue,
  config                : wgpu::SurfaceConfiguration,
  size                  : winit::dpi::PhysicalSize<u32>,
  window                : Window,
  render_pipeline       : wgpu::RenderPipeline,
  vertex_buffer         : wgpu::Buffer,
  vertices_count        : u32,
  cube_vertex_buffer: wgpu::Buffer,
  cube_index_buffer : wgpu::Buffer,
  cube_indices_count: u32,
  object_selection      : u32, // 0=triangle, 1=cube
  camera                : Camera,
  camera_buffer         : wgpu::Buffer,
  camera_bind_group     : wgpu::BindGroup,
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

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

const VERTICES: &[Vertex] = &[
  // Defining vertices in CCW order as a std, because in render_pipeline
  // we specified CCW as the front face, and to cull the back face
  Vertex { position: [0.0 , 0.5 , 0.0], color: [1.0, 0.0, 0.0], },
  Vertex { position: [-0.5, -0.5, 0.0], color: [0.0, 1.0, 0.0], },
  Vertex { position: [0.5 , -0.5, 0.0], color: [0.0, 0.0, 1.0], },
];

const CUBE_VERTICES: &[Vertex] = &[
  Vertex { position: [-0.5, -0.5, 0.5 ], color: [0.5, 0.0, 0.5] }, // A: 0
  Vertex { position: [0.5 , -0.5, 0.5 ], color: [0.5, 0.0, 0.5] }, // B: 1
  Vertex { position: [0.5 , 0.5 , 0.5 ], color: [0.5, 0.0, 0.5] }, // C: 2
  Vertex { position: [-0.5, 0.5 , 0.5 ], color: [0.5, 0.0, 0.5] }, // D: 3
  Vertex { position: [-0.5, -0.5, -0.5], color: [0.5, 0.0, 0.5] }, // E: 4
  Vertex { position: [0.5 , -0.5, -0.5], color: [0.5, 0.0, 0.5] }, // F: 5
  Vertex { position: [0.5 , 0.5 , -0.5], color: [0.5, 0.0, 0.5] }, // G: 6
  Vertex { position: [-0.5, 0.5 , -0.5], color: [0.5, 0.0, 0.5] }, // H: 7
];

const CUBE_INDICES: &[u16] = &[
  // Front
  0, 1, 2,
  2, 3, 0,

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
  0, 4, 3,
  4, 7, 3,

  // Back
  4, 5, 6,
  6, 7, 4,
];

// ============================================================
// Functions
// ============================================================

impl CameraUniform {
  fn new() -> Self {
      use cgmath::SquareMatrix;
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

impl State {

  async fn new(window: Window) -> Self {

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
    let surface = unsafe { instance.create_surface(&window) }.unwrap();

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

    let shader = device.create_shader_module(
      wgpu::include_wgsl!("shader.wgsl"),
    );

    let render_pipeline_layout = device.create_pipeline_layout(
      &wgpu::PipelineLayoutDescriptor {
        label               : Some("Render pipeline layout"),
        bind_group_layouts  : &[],
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
          buffers     : &[ Vertex::desc(), ],
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

        depth_stencil : None,

        multisample: wgpu::MultisampleState {
          count : 1,
          mask  : !0,
          alpha_to_coverage_enabled: false,
        },

        multiview: None,
      },
    );

    let vertex_buffer = device.create_buffer_init(
      &wgpu::util::BufferInitDescriptor {
        label   : Some("Vertex buffer"),
        contents: bytemuck::cast_slice(VERTICES),
        usage   : wgpu::BufferUsages::VERTEX,
      },
    );

    let vertices_count = VERTICES.len() as u32;

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

    let object_selection = 0;

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
        label: Some("camera_bind_group_layout"),
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

    let render_pipeline_layout = device.create_pipeline_layout(
      &wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[
          &camera_bind_group_layout,
        ],
        push_constant_ranges: &[],
      },
    );

    Self {
      window,
      surface,
      device,
      queue,
      config,
      size,
      render_pipeline,
      vertex_buffer,
      vertices_count,
      cube_vertex_buffer,
      cube_index_buffer,
      cube_indices_count,
      object_selection,
      camera,
      camera_buffer,
      camera_bind_group,
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
  }

  fn input(&mut self, event: &WindowEvent) -> bool {
    match event {
      WindowEvent::KeyboardInput {
        input: KeyboardInput {
            state: ElementState::Pressed,
            virtual_keycode,
            ..
        },
        ..
      } => if virtual_keycode == &Some(VirtualKeyCode::Key0) {
        self.object_selection = 0;
        return true;

      } else if virtual_keycode == &Some(VirtualKeyCode::Key1) {
        self.object_selection = 1;
        return true;

      } else {
        return false;
      },

      _ => return false,
    };
  }

  fn update(&mut self) {}

  fn render(&mut self) -> Result<(), wgpu::SurfaceError> {

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

        depth_stencil_attachment: None,
      });

      render_pass.set_pipeline(&self.render_pipeline);

      match self.object_selection {
        0 => {
          render_pass.set_vertex_buffer(
            0,
            self.vertex_buffer.slice(..),
          );
          render_pass.draw(
            0..self.vertices_count,
            0..1,
          );
        },

        1 => {
          render_pass.set_vertex_buffer(
            0,
            self.cube_vertex_buffer.slice(..),
          );
          render_pass.set_index_buffer(
            self.cube_index_buffer.slice(..),
            wgpu::IndexFormat::Uint16,
          );
          render_pass.draw_indexed(
            0..self.cube_indices_count,
            0,
            0..1,
          );
        },

        _ => {},
      }
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
    *control_flow = ControlFlow::Wait;
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
