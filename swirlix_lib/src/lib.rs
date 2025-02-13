#![allow(dead_code, unused_variables)]

//! A simple, performant voxel sculpting tool library.
//!
//! The library crate for a sculpting tool built on sparse 
//! voxel octrees (SVO) and the transvoxel algorithm.

mod util;
mod app;
mod editor;
mod renderer;
mod sculpt;
mod brush;
mod material;

pub use app::App;
