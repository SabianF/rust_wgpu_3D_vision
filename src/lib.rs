#![allow(non_snake_case)]

use game_loop::{winit::{event_loop::EventLoop, window::Window, event::{Event, WindowEvent, KeyboardInput, ElementState, VirtualKeyCode}}, game_loop};

mod game_state;
mod render_state;
mod camera;
mod render_pipeline_state;
mod cube_model;
mod instance;
mod extras;

use game_state::GameState;

static TARGET_FPS: u32 = 60;

pub async fn run() {
  env_logger::init();

  let (
    event_loop,
    window,
    game_state,
  ) = game_init().await;

  game_loop(
    event_loop,
    window,
    game_state,
    TARGET_FPS,
    0.1,
    |g| {
      g.game.update();

      // We are rendering here, because this block updates at the speed of
      // TARGET_FPS, which is the speed we want to render layers of voxels
      // in order to utilize consistent flicker fusion for 3D volumes
      let render_error = g.game.render() == false;
      if render_error {
        g.exit();
      };
    },
    |_g| {
      // This block updates faster than TARGET_FPS, which is not suitable
      // for our current implementation of rendering voxel layers
    },
    |g, event| {
      detect_change_framerate(g, event);
      detect_exit_request(g, event);
    },
  );
}

async fn game_init() -> (
  EventLoop<()>,
  Window,
  GameState,
) {
  let event_loop = EventLoop::new();

  let window = Window::new(&event_loop)
  .unwrap();
  window.set_title("3D Vision Renderer");

  let game_state = GameState::new(&window).await;
  
  return (
    event_loop,
    window,
    game_state,
  );
}

fn detect_exit_request(
  g: &mut game_loop::GameLoop<GameState,
  game_loop::Time, Window>, event: &Event<()>,
) {
  if g.game.handle_events(event, &g.window) == false {
    g.exit();
  };
}

fn detect_change_framerate(
  g: &mut game_loop::GameLoop<GameState,
  game_loop::Time, Window>, event: &Event<()>,
) {
  match event {
    Event::WindowEvent {
      event: WindowEvent::KeyboardInput {
        input: KeyboardInput {
          state: ElementState::Pressed,
          virtual_keycode,
          ..
        },
        ..
      },
      ..
    } => {
      if virtual_keycode == &Some(VirtualKeyCode::Key1) {
        g.set_updates_per_second(&g.updates_per_second - 10);
      }

      if virtual_keycode == &Some(VirtualKeyCode::Key2) {
        g.set_updates_per_second(&g.updates_per_second + 10);
      }
    }

    _ => {}
  }
}
