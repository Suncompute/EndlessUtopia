/// EndlessUtopia - A coordinate-based infinite ASCII world generator
/// Lightweight, deterministic, and WASM-compatible

pub mod world;

pub use world::{World, Tile, Biome};

#[cfg(target_arch = "wasm32")]
pub mod wasm;

#[cfg(target_arch = "wasm32")]
pub use wasm::*;
