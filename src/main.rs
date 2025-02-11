#![allow(dead_code, unused_variables)]

//! A simple, performant voxel sculpting tool.
//!
//! A sculpting tool built on sparse voxel octrees (SVO)
//! and the transvoxel algorithm.

use crate::app::App;
use winit::error::EventLoopError;
use winit::event_loop::{ControlFlow, EventLoop};

mod app;
mod wgpu_context;
mod editor;
mod document;
mod brush;

/// The entrypoint runs the event loop.
fn main() -> Result<(), EventLoopError> {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::default();
    event_loop.run_app(&mut app)
}
