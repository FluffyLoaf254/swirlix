use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::dpi::PhysicalPosition;
use winit::event::{MouseButton, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

use crate::editor::Editor;
use crate::wgpu_context::WgpuContext;

/// The application of winit for displaying windows.
#[derive(Default)]
pub struct App {
    window: Option<Arc<Window>>,
    context: Option<WgpuContext>,
    cursor_position: PhysicalPosition<f64>,
    editor: Editor,
    has_drawn: bool,
}

impl ApplicationHandler for App {
    /// Application has been resumed or started.
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let version = env!("CARGO_PKG_VERSION");
            let win_attr = Window::default_attributes().with_title(format!("Swirlix {version}"));
            // use Arc
            let window = Arc::new(
                event_loop
                    .create_window(win_attr)
                    .expect("Could not create the window."),
            );
            self.window = Some(window.clone());
            let context = WgpuContext::new(window.clone());
            self.context = Some(context);
        }
    }

    /// Handle events.
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
                if let Some(context) = self.context.as_mut() {
                    context.draw();
                }
            }
            WindowEvent::CursorMoved {
                device_id: _,
                position,
            } => {
                self.cursor_position = position;
            }
            WindowEvent::MouseInput {
                device_id: _,
                state: _,
                button,
            } => {
                if self.has_drawn {
                    return;
                }
                if button == MouseButton::Left {
                    let size = self.window.as_ref().unwrap().inner_size();
                    // remap x/y values from pixel to normalized device coordinates for now...
                    self.editor.add(((self.cursor_position.x / size.width as f64) * 2.0 - 1.0) as f32, ((self.cursor_position.y / size.height as f64) * 2.0 - 1.0) as f32);
                    self.has_drawn = true;
                }
                if button == MouseButton::Right {
                    let size = self.window.as_ref().unwrap().inner_size();
                    // remap x/y values from pixel to normalized device coordinates for now...
                    self.editor.remove(((self.cursor_position.x / size.width as f64) * 2.0 - 1.0) as f32, ((self.cursor_position.y / size.height as f64) * 2.0 - 1.0) as f32);
                }
            }
            _ => (),
        }
    }
}
