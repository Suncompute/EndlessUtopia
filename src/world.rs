/// Core world generation module for EndlessUtopia
/// Generates deterministic ASCII patterns based on coordinates

use std::collections::HashSet;

/// Represents a tile in the ASCII world
#[derive(Debug, Clone, PartialEq)]
pub struct Tile {
    pub character: char,
    pub biome: Biome,
}

/// Different biomes/pattern types in the world
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Biome {
    Calm,        // Empty peaceful spaces
    Pattern,     // Regular patterns
    Glitch,      // Glitchy/corrupted areas
    CatTrace,    // Where the cat has been
    CatPresent,  // Current cat location
}

/// The infinite world generator
pub struct World {
    cat_visited: HashSet<(i32, i32)>,
}

impl World {
    pub fn new() -> Self {
        World {
            cat_visited: HashSet::new(),
        }
    }

    /// Generate a tile at the given coordinates
    pub fn get_tile(&mut self, x: i32, y: i32) -> Tile {
        // Check if cat has visited this location (check first!)
        if self.cat_visited.contains(&(x, y)) {
            return Tile {
                character: self.get_trace_char(x, y),
                biome: Biome::CatTrace,
            };
        }

        // Check if cat is present at this coordinate
        if self.is_cat_location(x, y) {
            self.cat_visited.insert((x, y));
            return Tile {
                character: self.get_cat_char(x, y),
                biome: Biome::CatPresent,
            };
        }

        // Determine biome based on coordinates
        let hash = self.coord_hash(x, y);
        let biome_selector = hash % 100;

        if biome_selector < 60 {
            // 60% calm empty spaces
            Tile {
                character: ' ',
                biome: Biome::Calm,
            }
        } else if biome_selector < 85 {
            // 25% patterns
            Tile {
                character: self.get_pattern_char(x, y, hash),
                biome: Biome::Pattern,
            }
        } else {
            // 15% glitch areas
            Tile {
                character: self.get_glitch_char(x, y, hash),
                biome: Biome::Glitch,
            }
        }
    }

    /// Deterministic hash function for coordinates
    fn coord_hash(&self, x: i32, y: i32) -> u64 {
        // Simple but effective hash mixing
        let mut h = 0x517cc1b727220a95u64;
        h = h.wrapping_mul(0x6c62272e07bb0142u64);
        h ^= x as u64;
        h = h.wrapping_mul(0x6c62272e07bb0142u64);
        h ^= y as u64;
        h = h.wrapping_mul(0x6c62272e07bb0142u64);
        h
    }

    /// Check if the cat appears at this special coordinate
    pub fn is_cat_location(&self, x: i32, y: i32) -> bool {
        // Only one true Ascicat in the world!
        let (cat_x, cat_y) = Self::ascicat_position();
        x == cat_x && y == cat_y
    }

    /// Returns the one true Ascicat position (deterministic, but schwer zu erraten)
    pub fn ascicat_position() -> (i32, i32) {
        // Use a hash of a fixed string to generate unique but stable coordinates
        let mut h = 0x517cc1b727220a95u64;
        for b in b"ascicat" {
            h = h.wrapping_mul(0x6c62272e07bb0142u64);
            h ^= *b as u64;
        }
        let x = ((h >> 16) & 0xFFFF_FFFF) as i32 - 50_000; // Bereich -50_000..+50_000
        let y = (h & 0xFFFF_FFFF) as i32 - 50_000;
        (x, y)
    }

    /// Get the cat character based on position (different poses)
    fn get_cat_char(&self, x: i32, y: i32) -> char {
        let hash = self.coord_hash(x, y);
        // Use basic ASCII for better WASM compatibility
        let basic_poses = ['@', 'C', 'c', 'o', 'O'];
        basic_poses[(hash % basic_poses.len() as u64) as usize]
    }

    /// Get trace character where cat has walked
    fn get_trace_char(&self, x: i32, y: i32) -> char {
        let hash = self.coord_hash(x, y);
        let traces = ['.', '·', '˙', '∙', '•'];
        traces[(hash % traces.len() as u64) as usize]
    }

    /// Generate pattern characters
    fn get_pattern_char(&self, x: i32, y: i32, hash: u64) -> char {
        let pattern_type = (hash / 100) % 10;
        
        match pattern_type {
            0 | 1 => {
                // Checkerboard-like patterns
                if (x + y) % 2 == 0 { '·' } else { ' ' }
            }
            2 | 3 => {
                // Wave patterns
                let wave = ((x as f64 * 0.5).sin() + (y as f64 * 0.3).cos()) * 3.0;
                if wave.abs() < 1.0 { '~' } else { ' ' }
            }
            4 => {
                // Diagonal stripes
                if (x - y) % 3 == 0 { '/' } else { ' ' }
            }
            5 => {
                // Sparse dots
                if (x * 7 + y * 11) % 13 == 0 { '•' } else { ' ' }
            }
            6 => {
                // Cross patterns
                if x % 5 == 0 || y % 5 == 0 { '+' } else { ' ' }
            }
            7 => {
                // Concentric patterns
                let dist = ((x * x + y * y) as f64).sqrt() as i32;
                if dist % 10 == 0 { 'o' } else { ' ' }
            }
            8 => {
                // Random-looking sparse characters
                let chars = ['*', '·', '˙', ' ', ' ', ' '];
                chars[(hash % chars.len() as u64) as usize]
            }
            _ => {
                // Minimalist single dots
                if hash % 20 == 0 { '.' } else { ' ' }
            }
        }
    }

    /// Generate glitch characters
    fn get_glitch_char(&self, _x: i32, _y: i32, hash: u64) -> char {
        let glitch_intensity = hash % 10;
        
        if glitch_intensity < 3 {
            // Light glitches
            let chars = ['▓', '▒', '░', '█'];
            chars[(hash % chars.len() as u64) as usize]
        } else if glitch_intensity < 6 {
            // Medium glitches with symbols
            let chars = ['#', '$', '%', '&', '@', '¤'];
            chars[(hash % chars.len() as u64) as usize]
        } else if glitch_intensity < 8 {
            // Heavy glitches
            let chars = ['█', '▓', '▒', '░', '▪', '▫'];
            chars[(hash % chars.len() as u64) as usize]
        } else {
            // Rare intense glitches
            let chars = ['▀', '▄', '▌', '▐', '█', '▓'];
            chars[(hash % chars.len() as u64) as usize]
        }
    }

    /// Get a rectangular region of the world
    pub fn get_region(&mut self, x_start: i32, y_start: i32, width: usize, height: usize) -> Vec<Vec<Tile>> {
        let mut region = Vec::with_capacity(height);
        
        for y in 0..height {
            let mut row = Vec::with_capacity(width);
            for x in 0..width {
                let tile = self.get_tile(x_start + x as i32, y_start + y as i32);
                row.push(tile);
            }
            region.push(row);
        }
        
        region
    }

    /// Render a region to ASCII string
    pub fn render_region(&mut self, x_start: i32, y_start: i32, width: usize, height: usize) -> String {
        let region = self.get_region(x_start, y_start, width, height);
        let mut output = String::with_capacity(width * height + height);
        
        for row in region {
            for tile in row {
                output.push(tile.character);
            }
            output.push('\n');
        }
        
        output
    }

    /// Find nearby cat locations (for exploration)
    pub fn find_cat_nearby(&self, x_center: i32, y_center: i32, radius: i32) -> Vec<(i32, i32)> {
        let mut cat_locations = Vec::new();
        
        for y in (y_center - radius)..=(y_center + radius) {
            for x in (x_center - radius)..=(x_center + radius) {
                if self.is_cat_location(x, y) {
                    cat_locations.push((x, y));
                }
            }
        }
        
        cat_locations
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_generation() {
        let mut world1 = World::new();
        let mut world2 = World::new();
        
        let tile1 = world1.get_tile(42, 17);
        let tile2 = world2.get_tile(42, 17);
        
        assert_eq!(tile1.character, tile2.character);
        assert_eq!(tile1.biome, tile2.biome);
    }

    #[test]
    fn test_coord_hash_uniqueness() {
        let world = World::new();
        let hash1 = world.coord_hash(0, 0);
        let hash2 = world.coord_hash(1, 0);
        let hash3 = world.coord_hash(0, 1);
        
        assert_ne!(hash1, hash2);
        assert_ne!(hash1, hash3);
        assert_ne!(hash2, hash3);
    }

    #[test]
    fn test_cat_rarity() {
        let world = World::new();
        let mut cat_count = 0;
        let sample_size = 10000;
        
        for y in 0..100 {
            for x in 0..100 {
                if world.is_cat_location(x, y) {
                    cat_count += 1;
                }
            }
        }
        
        // Cats should be very rare (much less than 1%)
        assert!(cat_count < sample_size / 100);
    }

    #[test]
    fn test_region_generation() {
        let mut world = World::new();
        let region = world.get_region(0, 0, 10, 10);
        
        assert_eq!(region.len(), 10);
        assert_eq!(region[0].len(), 10);
    }

    #[test]
    fn test_cat_trace_persistence() {
        let mut world = World::new();
        
        // Find a cat location
        let cats = world.find_cat_nearby(0, 0, 1000);
        if let Some(&(x, y)) = cats.first() {
            // Visit the cat
            let tile1 = world.get_tile(x, y);
            assert_eq!(tile1.biome, Biome::CatPresent);
            
            // Visit again - should leave a trace
            let tile2 = world.get_tile(x, y);
            assert_eq!(tile2.biome, Biome::CatTrace);
        }
    }
}
