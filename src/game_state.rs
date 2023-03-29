use game_loop::winit::dpi::PhysicalPosition;
use game_loop::winit::dpi::PhysicalSize;
use game_loop::winit::event::DeviceEvent;
use game_loop::winit::event::ElementState;
use game_loop::winit::event::KeyboardInput;
use game_loop::winit::event::MouseScrollDelta;
use game_loop::winit::event::VirtualKeyCode;
use wgpu::LoadOp;
use wgpu::Operations;
use wgpu::SurfaceError;
use wgpu::TextureViewDescriptor;

use crate::Event;
use crate::WindowEvent;
use crate::Window;
use crate::camera::camera_state::CameraState;
use crate::cube_model::CubeModel;
use crate::instance::NUM_INSTANCES_PER_COL;
use crate::instance::NUM_INSTANCES_PER_ROW;
use crate::render_pipeline_state::RenderPipelineState;
use crate::render_pipeline_state::Texture;
use crate::render_state::RenderState;

pub struct GameState {
  render_state: RenderState,
  camera_state: CameraState,
  render_pipeline_state: RenderPipelineState,
  cube_model: CubeModel,
  pub volumes_refreshed: u32,
  enable_voxel_flicker: bool,
  mouse_left_pressed: bool,
}

impl GameState {

  pub async fn new(window: &Window) -> Self {
    let render_state = RenderState::new(&window).await;
    let camera_state = CameraState::new(&render_state.device, &window);

    let render_pipeline_state = RenderPipelineState::new(
      &render_state.device,
      &camera_state.camera_bind_group_layout,
      &render_state.config,
    );

    let cube_model = CubeModel::new(&render_state.device);

    let counter = 0;
    let enable_voxel_flicker = false;
    let mouse_left_pressed = false;

    return Self {
      render_state,
      camera_state,
      render_pipeline_state,
      cube_model,
      volumes_refreshed: counter,
      enable_voxel_flicker,
      mouse_left_pressed,
    }
  }

  fn resize(&mut self, new_size: PhysicalSize<u32>) {

    if new_size.width > 0 && new_size.height > 0 {
      self.render_state.size = new_size;
      self.render_state.config.width = new_size.width;
      self.render_state.config.height = new_size.height;

      self.render_state.surface.configure(
        &self.render_state.device,
        &self.render_state.config
      );
    }

    self.render_pipeline_state.depth_texture = Texture::create_depth_texture(
      &self.render_state.device,
      &self.render_state.config,
      "depth_texture",
    );
  }

  pub fn input(
    &mut self,
    event: &Event<()>,
    window: &Window,
  ) -> bool {
    self.camera_state.camera_controller.process_events(
      event,
      window,
      &mut self.camera_state.camera,
    );

    match event {
      Event::WindowEvent {
        event: window_event,
        ..
      } => {
        match window_event {
          WindowEvent::KeyboardInput {
            input: KeyboardInput {
              state: ElementState::Pressed,
              virtual_keycode: Some(VirtualKeyCode::Key0),
              ..
            },
            ..
          } => {
            self.enable_voxel_flicker = !self.enable_voxel_flicker;
            return true;
          },

          _ => return false
        }
      },

      Event::DeviceEvent {
        event: device_event,
        ..
      } => {
        match device_event {
          DeviceEvent::Button {
            // The Left Mouse Button on MacOS.
            #[cfg(target_os = "macos")]
            button: 0,
            // The Left Mouse Button on all other platforms.
            #[cfg(not(target_os = "macos"))]
            button: 1,

            state: mb_state,
          } => {
            self.mouse_left_pressed = *mb_state == ElementState::Pressed;
            return true;
          },

          DeviceEvent::MouseMotion {
            delta,
          } => if self.mouse_left_pressed {
            self.camera_state.camera.add_yaw(
              -delta.0 as f32 * self.camera_state.camera_controller.rotate_speed
            );
            self.camera_state.camera.add_pitch(
              delta.1 as f32 * self.camera_state.camera_controller.rotate_speed
            );
            return true;

          } else {
            return false;
          },

          DeviceEvent::MouseWheel {
            delta,
          } => {
            let scroll_amount = -match delta {
              MouseScrollDelta::LineDelta(_, scroll) => {
                scroll * 1.0
              },
              MouseScrollDelta::PixelDelta(PhysicalPosition { y: scroll, .. }) => {
                *scroll as f32
              },
            };

            self.camera_state.camera.add_distance(
              scroll_amount * self.camera_state.camera_controller.zoom_speed
            );
            return true;
          },

          _ => return false
        }
      },

      _ => return false
    }
  }

  pub fn update(&mut self) {
    self.camera_state.camera.update();

    self.camera_state.camera_uniform
      .update_view_proj(&self.camera_state.camera);

    if self.enable_voxel_flicker {
      self.iterate_volume_plane_instances_to_render();
    }

    self.render_state.queue.write_buffer(
      &self.camera_state.camera_buffer,
      0,
      bytemuck::cast_slice(&[self.camera_state.camera_uniform])
    );

    self.volumes_refreshed += 1;
  }

  /**
   * Returns false on unrecoverable error
   */
  pub fn render (&mut self) -> bool {
    match self.prerender() {

      // Reconfigure the surface if lost
      Err(wgpu::SurfaceError::Lost) => {
        self.resize(self.render_state.size);
      },

      // Prevent excess memory usage
      Err(wgpu::SurfaceError::OutOfMemory) => {
        return false;
      },

      // Handle all other errors
      Err(e) => {
        eprintln!("{:?}", e);
      },
      
      Ok(_) => {},
    }

    return true;
  }

  fn prerender (&mut self) -> Result<(), SurfaceError> {
    let output = self.render_state.surface.get_current_texture()?;
    let view = output.texture.create_view(&TextureViewDescriptor::default());

    let mut encoder = self.render_state.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
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
          view: &self.render_pipeline_state.depth_texture.view,

          depth_ops: Some(Operations {
            load: LoadOp::Clear(1.0),
            store: true,
          }),

          stencil_ops : None,
        }),
      });

      render_pass.set_pipeline(&self.render_pipeline_state.render_pipeline);

      render_pass.set_bind_group(
        0,
        &self.camera_state.camera_bind_group,
        &[],
      );

      render_pass.set_vertex_buffer(
        0,
        self.cube_model.cube_vertex_buffer.slice(..),
      );
      render_pass.set_vertex_buffer(
        1,
        self.render_pipeline_state.instance_buffer.buffer.slice(..),
      );
      render_pass.set_index_buffer(
        self.cube_model.cube_index_buffer.slice(..),
        wgpu::IndexFormat::Uint16,
      );
      render_pass.draw_indexed(
        0..self.cube_model.cube_indices_count,
        0,
        self.render_pipeline_state.instances_to_render_start..self.render_pipeline_state.instances_to_render_end,
      );
    }

    self.render_state.queue.submit(std::iter::once(encoder.finish()));
    output.present();
    return Ok(());
  }

  pub fn handle_events (
    &mut self,
    event: &Event<()>,
    window: &Window,
  ) -> bool {
    let event_was_input = self.input(event, window);
    if event_was_input {
      return true;
    }

    match event {
      Event::WindowEvent {
          event,
          ..
      } => {
        match event {
          WindowEvent::CloseRequested => {
            return false;
          },

          WindowEvent::Resized(physical_size) => {
            self.resize(*physical_size);
            return true;
          },
          WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
            self.resize(**new_inner_size);
            return true;
          },

          _ => return true,
        };
      },

      _ => return true,
    };
  }

  fn iterate_volume_plane_instances_to_render(&mut self) {
    let range_increment_amount = NUM_INSTANCES_PER_ROW * NUM_INSTANCES_PER_COL;
    let range_end_max = self.render_pipeline_state.instance_buffer.instances.len() as u32;
    let range_start_max = range_end_max - range_increment_amount;

    let range_end_min = range_increment_amount;
    let range_start_min = 0;

    if self.render_pipeline_state.instances_to_render_start + range_increment_amount <= range_start_max {
      self.render_pipeline_state.instances_to_render_start += range_increment_amount;
    } else {
      self.render_pipeline_state.instances_to_render_start = range_start_min;
    }
    if self.render_pipeline_state.instances_to_render_end + range_increment_amount <= range_end_max {
      self.render_pipeline_state.instances_to_render_end += range_increment_amount;
    } else {
      self.render_pipeline_state.instances_to_render_end = range_end_min;
    }
  }
}