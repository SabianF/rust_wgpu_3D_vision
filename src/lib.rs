
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
    todo!()
  }

  pub fn window(&self) -> &Window {
      &self.window
  }

  fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
      todo!()
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

pub fn run() {
  env_logger::init();

  let event_loop = EventLoop::new();
  let window = Window::new(&event_loop).unwrap();
  window.set_title("2D Object Renderer");

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
