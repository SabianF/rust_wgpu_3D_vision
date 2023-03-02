#![allow(non_snake_case)]

use rust_wgpu_3D_vision::run;

fn main() {
  pollster::block_on(run());
}
