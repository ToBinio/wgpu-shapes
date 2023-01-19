//! WGPU SHAPES is a simple "renderer" which makes it easer to render basic shapes with wgpu.
//!
//! best used with [wgpu_noboiler](https://crates.io/crates/wgpu-noboiler)


pub mod shape_renderer;
pub mod rect;
pub mod oval;
pub mod shapes;
mod vertex;
mod instance;
mod depth_buffer;
pub mod texture;
