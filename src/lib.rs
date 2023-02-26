
use winit::{
  event::{Event, WindowEvent},
  event_loop::{ControlFlow, EventLoop},
  window::{Window},
};

#[cfg(target_arch="wasm32")]
use wasm_bindgen::prelude::*;

// ============================================================
// Types
// ============================================================


struct State {
  surface         : wgpu::Surface,
  device          : wgpu::Device,
  queue           : wgpu::Queue,
  config          : wgpu::SurfaceConfiguration,
  size            : winit::dpi::PhysicalSize<u32>,
  window          : Window,
  render_pipeline : wgpu::RenderPipeline,
}

// ============================================================
// Functions
// ============================================================

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
          buffers     : &[],
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

    Self {
      window,
      surface,
      device,
      queue,
      config,
      size,
      render_pipeline,
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
    return false;
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
      render_pass.draw(0..3, 0..1);
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
