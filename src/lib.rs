use winit::{
  event::{Event, WindowEvent},
  event_loop::{ControlFlow, EventLoop},
  window::{Window},
};

#[cfg(target_arch="wasm32")]
use wasm_bindgen::prelude::*;

#[cfg_attr(target_arch="wasm32", wasm_bindgen(start))]
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
