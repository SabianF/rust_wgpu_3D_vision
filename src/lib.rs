
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
  surface : wgpu::Surface,
  device  : wgpu::Device,
  queue   : wgpu::Queue,
  config  : wgpu::SurfaceConfiguration,
  size    : winit::dpi::PhysicalSize<u32>,
  window  : Window,
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
    let instance =wgpu::Instance::new(wgpu::InstanceDescriptor {
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
        force_fallback_adapter: false,
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

    Self {
      window,
      surface,
      device,
      queue,
      config,
      size,
    }
  }

  pub fn window(&self) -> &Window {
      &self.window
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
      todo!()
  }

  fn update(&mut self) {
      todo!()
  }

  fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
      todo!()
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
              event: WindowEvent::CloseRequested,
              ..
          } => *control_flow = ControlFlow::Exit,
          _ => {}
      }
  });
}
