/// WASM bindings for EndlessUtopia
/// Provides a simple interface for web-based rendering

use wasm_bindgen::prelude::*;
use crate::world::World;

/// WASM-compatible world generator
#[wasm_bindgen]
pub struct WasmWorld {
    world: World,
}

#[wasm_bindgen]
impl WasmWorld {
    /// Create a new world instance
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        WasmWorld {
            world: World::new(),
        }
    }

    /// Get a single character at the given coordinates
    #[wasm_bindgen]
    pub fn get_char(&mut self, x: i32, y: i32) -> String {
        let tile = self.world.get_tile(x, y);
        tile.character.to_string()
    }

    /// Render a rectangular region as a string
    #[wasm_bindgen]
    pub fn render_region(&mut self, x: i32, y: i32, width: usize, height: usize) -> String {
        self.world.render_region(x, y, width, height)
    }

    /// Find cat locations near a coordinate
    #[wasm_bindgen]
    pub fn find_cats_near(&self, x: i32, y: i32, radius: i32) -> Vec<i32> {
        let locations = self.world.find_cat_nearby(x, y, radius);
        let mut result = Vec::with_capacity(locations.len() * 2);
        
        for (cat_x, cat_y) in locations {
            result.push(cat_x);
            result.push(cat_y);
        }
        
        result
    }

    /// Check if a location has a cat
    #[wasm_bindgen]
    pub fn has_cat(&self, x: i32, y: i32) -> bool {
        self.world.is_cat_location(x, y)
    }
}

impl Default for WasmWorld {
    fn default() -> Self {
        Self::new()
    }
}
