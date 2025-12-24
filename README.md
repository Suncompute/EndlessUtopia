# EndlessUtopia

EndlessUtopia — A Coordinate‑Based ASCII Exploration Game

## Overview

**EndlessUtopia** is an infinite, procedurally-generated ASCII world where each coordinate `(x, y)` produces deterministic patterns, glitches, or calm empty spaces. The world is endless, quiet, and mysterious, featuring a rare wandering ASCII Cat that appears only at special coordinates and leaves subtle traces.

## Requirements

- **Rust 1.70+**: For building the library
- **Terminal/Browser Support**: Best experienced with Unicode-capable terminals or browsers
  - Most modern terminals (iTerm2, Windows Terminal, GNOME Terminal) support the full character set
  - Fallback ASCII characters are used for core gameplay elements (cat = @, C, c, o, O)
  - Block characters (░▒▓█) used for visual effects may vary by environment

## Features

- 🌌 **Infinite World**: Truly endless coordinate-based generation
- 🎲 **Deterministic**: Same coordinates always produce the same result
- 🐱 **Wandering Cat**: Rare ASCII cat appears at special coordinates
- 👣 **Cat Traces**: Subtle marks left where the cat has been visited
- 🎨 **Multiple Biomes**: Calm spaces, patterns, and glitch zones
- ⚡ **Lightweight**: Optimized for Rust + WASM ASCII rendering
- 🎯 **No Dependencies**: Pure Rust implementation (except wasm-bindgen for WASM targets)

## World Biomes

The world consists of several biome types:

- **Calm** (60%): Empty peaceful spaces with minimal decoration
- **Pattern** (25%): Various deterministic patterns including waves, checkerboards, stripes, and more
- **Glitch** (15%): Corrupted/glitchy areas with block characters
- **Cat Present**: Special coordinates where the wandering cat appears
- **Cat Trace**: Locations the cat has visited, marked with subtle dots

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
endless_utopia = "0.1.0"
```

## Usage

### Basic Example

```rust
use endless_utopia::World;

fn main() {
    let mut world = World::new();
    
    // Get a single tile
    let tile = world.get_tile(0, 0);
    println!("Character at origin: {}", tile.character);
    
    // Render a region
    let region = world.render_region(0, 0, 40, 20);
    println!("{}", region);
    
    // Find cats nearby
    let cats = world.find_cat_nearby(0, 0, 100);
    for (x, y) in cats {
        println!("Found cat at ({}, {})", x, y);
    }
}
```

### WASM Usage

```rust
use endless_utopia::WasmWorld;

let mut world = WasmWorld::new();

// Get character at coordinate
let ch = world.get_char(10, 20);

// Render a region
let ascii_art = world.render_region(0, 0, 80, 24);
console_log(&ascii_art);

// Find cats
let cat_coords = world.find_cats_near(0, 0, 200);
```

## Running Examples

Explore the infinite world:

```bash
cargo run --example explore
```

## Building for WASM

```bash
# Install wasm-pack if you haven't already
cargo install wasm-pack

# Build for web
wasm-pack build --target web

# Build for Node.js
wasm-pack build --target nodejs
```

## Performance

The generator is optimized for lightweight, real-time rendering:

- No heap allocations during tile generation
- Simple hash-based deterministic generation
- Optimized for size with `opt-level = "z"`
- Link-time optimization enabled for WASM builds

## The Wandering Cat

The ASCII cat is extremely rare and appears only at special coordinates determined by:
- Prime number-based hash matching
- Coordinate sum divisibility checks

When you visit a cat location, it leaves behind subtle traces (dots) that persist in your world instance, creating a unique exploration history.

Cat characters include: `@`, `C`, `c`, `o`, `O`

## Design Philosophy

EndlessUtopia follows these principles:

- **Minimal**: No unnecessary dependencies or complexity
- **Deterministic**: Exploration is reproducible and sharable via coordinates
- **Mysterious**: The world reveals itself gradually through exploration
- **Performant**: Suitable for real-time ASCII rendering in browsers

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Feel free to open issues or submit pull requests.

