use crate::editor::Editor;
use crate::renderer::Renderer;

use std::sync::Arc;

use winit::error::EventLoopError;
use winit::event_loop::{EventLoop, ControlFlow, ActiveEventLoop};
use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::keyboard::KeyCode;
use winit::window::{Window, WindowId};

/// The main application class.
///
/// A winit application. Manages the window and owns all other resources.
#[derive(Default)]
pub struct App {
    window: Option<Arc<Window>>,
    context: Option<Renderer>,
    cursor_position: PhysicalPosition<f64>,
    editor: Editor,
}

impl App {
    /// Run the main event loop.
    pub fn run() -> Result<(), EventLoopError> {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);
        let mut app = App::default();
        event_loop.run_app(&mut app)
    }
}

impl ApplicationHandler for App {
    /// Start or resume the application.
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let version = env!("CARGO_PKG_VERSION");
            let win_attr = Window::default_attributes()
                .with_title(format!("Swirlix {version}"))
                .with_inner_size(PhysicalSize {
                    width: 1024,
                    height: 1024,
                });
            // use Arc
            let window = Arc::new(
                event_loop
                    .create_window(win_attr)
                    .expect("Could not create the window."),
            );
            self.window = Some(window.clone());
            let context = Renderer::new(window.clone(), self.editor.get_sculpt_resolution());
            self.context = Some(context);
        }
    }

    /// Handle window events.
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                // drop the context to avoid segfault at close
                self.context = None;
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                if let (Some(context), Some(window)) =
                    (self.context.as_mut(), self.window.as_ref())
                {
                    context.resize((new_size.width, new_size.height));
                    window.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                if let (Some(context), Some(window)) = (self.context.as_mut(), self.window.as_ref()) {
                    context.draw();
                }
            }
            WindowEvent::CursorMoved {
                device_id: _,
                position,
            } => {
                self.cursor_position = position;
            }
            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } => {
                if event.physical_key == KeyCode::KeyR {
                    self.editor.set_brush(0);
                }
                if event.physical_key == KeyCode::KeyS {
                    self.editor.set_brush(1);
                }
            }
            WindowEvent::MouseInput {
                device_id: _,
                state,
                button,
            } => {
                // left click = add
                if state == ElementState::Pressed && button == MouseButton::Left {
                    let size = self.window.as_ref().unwrap().inner_size();
                    // remap x/y values from pixel to 0-1 for now...
                    self.editor.add((self.cursor_position.x / size.width as f64) as f32, (self.cursor_position.y / size.height as f64) as f32);
                    self.context.as_mut().unwrap().set_material_buffer(self.editor.get_material_buffer());
                    self.context.as_mut().unwrap().set_voxel_buffer(self.editor.get_voxel_buffer());
                    if let Some(window) = self.window.as_ref() {
                        window.request_redraw();
                    }
                }
                // right click = remove
                if state == ElementState::Pressed && button == MouseButton::Right {
                    let size = self.window.as_ref().unwrap().inner_size();
                    // remap x/y values from pixel to 0-1 for now...
                    self.editor.remove((self.cursor_position.x / size.width as f64) as f32, (self.cursor_position.y / size.height as f64) as f32);
                    self.context.as_mut().unwrap().set_material_buffer(self.editor.get_material_buffer());
                    self.context.as_mut().unwrap().set_voxel_buffer(self.editor.get_voxel_buffer());
                    if let Some(window) = self.window.as_ref() {
                        window.request_redraw();
                    }
                }
            }
            _ => (),
        }
    }
}
