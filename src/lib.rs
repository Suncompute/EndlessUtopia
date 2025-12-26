/// EndlessUtopia - A coordinate-based infinite ASCII world generator
/// Lightweight, deterministic, and WASM-compatible

pub mod world;

pub use world::{World, Tile, Biome};

#[cfg(target_arch = "wasm32")]
pub mod app;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    // Initialize app
    app::App::new()?;
    
    Ok(())
}
