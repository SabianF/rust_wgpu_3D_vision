#![allow(non_snake_case)]

use game_loop::{winit::{window::Window, event::{Event, WindowEvent}}};

pub mod game_state;
pub mod render_state;
pub mod camera;
pub mod render_pipeline_state;
pub mod cube_model;
pub mod instance;
pub mod extras;
