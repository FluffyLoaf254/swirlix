//! A simple, performant voxel sculpting tool.
//!
//! A sculpting tool built using sparse voxel octrees
//! and ray marching.

use std::error::Error;
use sbrush::App;

/// The entrypoint runs the event loop.
fn main() -> Result<(), Box<dyn Error>> {
    App::run()?;

    Ok(())
}
