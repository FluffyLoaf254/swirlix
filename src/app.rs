use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

use crate::wgpu_context::WgpuContext;

#[derive(Default)]
pub struct App {
    window: Option<Arc<Window>>,
    wgpu_context: Option<WgpuContext>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let win_attr = Window::default_attributes().with_title("wgpu winit example");
            // use Arc.
            let window = Arc::new(
                event_loop
                    .create_window(win_attr)
                    .expect("create window err."),
            );
            self.window = Some(window.clone());
            let wgpu_context = WgpuContext::new(window.clone());
            self.wgpu_context = Some(wgpu_context);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                self.wgpu_context = None;
                // macOS err: https://github.com/rust-windowing/winit/issues/3668
                // This will be fixed as winit 0.30.1.
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                if let (Some(wgpu_context), Some(window)) =
                    (self.wgpu_context.as_mut(), self.window.as_ref())
                {
                    wgpu_context.resize((new_size.width, new_size.height));
                    window.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(wgpu_context) = self.wgpu_context.as_mut() {
                    wgpu_context.draw();
                }
            }
            _ => (),
        }
    }
}
