
use game_loop::winit::{dpi::PhysicalSize};
use wgpu::{SurfaceConfiguration, Surface, Device, Queue};

use crate::Window;

pub struct RenderState {
  pub size    : PhysicalSize<u32>,
  pub surface : Surface,
  pub device  : Device,
  pub queue   : Queue,
  pub config  : SurfaceConfiguration,
}

impl RenderState {

  pub async fn new(window: &Window) -> Self {
    let (
      size,
      surface,
      device,
      queue,
      config,
    ) = Self::configure_surface(window).await;

    return Self {
      size,
      surface,
      device,
      queue,
      config,
    };
  }

  async fn configure_surface(window: &Window) -> (
    PhysicalSize<u32>,
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
}
