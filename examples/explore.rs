/// Example: Exploring the infinite ASCII world
/// Demonstrates world generation and cat finding

use endless_utopia::World;

fn main() {
    println!("🌌 EndlessUtopia - Infinite ASCII World Explorer\n");
    
    let mut world = World::new();
    
    // Explore origin area
    println!("=== Region around origin (0,0) ===");
    let region = world.render_region(-10, -5, 40, 15);
    println!("{}", region);
    
    // Show a calm area
    println!("\n=== Calm space at (100, 100) ===");
    let calm = world.render_region(100, 100, 30, 10);
    println!("{}", calm);
    
    // Show a patterned area
    println!("\n=== Patterns at (500, 300) ===");
    let patterns = world.render_region(500, 300, 30, 10);
    println!("{}", patterns);
    
    // Show a glitch area
    println!("\n=== Glitch zone at (1000, -500) ===");
    let glitches = world.render_region(1000, -500, 30, 10);
    println!("{}", glitches);
    
    // Search for cats
    println!("\n=== Searching for wandering ASCII cats... ===");
    let cats = world.find_cat_nearby(0, 0, 100);
    
    if cats.is_empty() {
        println!("No cats found in a 100-unit radius around origin.");
        println!("Searching wider area (radius 500)...");
        let cats_wide = world.find_cat_nearby(0, 0, 500);
        
        if let Some(&(x, y)) = cats_wide.first() {
            println!("Found a cat at ({}, {})!", x, y);
            println!("\n=== Cat location ===");
            let cat_region = world.render_region(x - 5, y - 2, 20, 5);
            println!("{}", cat_region);
            
            // Visit the cat again to see its trace
            println!("\n=== Cat trace (after visiting) ===");
            let trace_region = world.render_region(x - 5, y - 2, 20, 5);
            println!("{}", trace_region);
        } else {
            println!("Cats are very rare! Try coordinates like (0, 0), (46, 0), or explore more.");
        }
    } else {
        for (x, y) in cats {
            println!("Found a cat at ({}, {})!", x, y);
        }
    }
    
    // Show determinism
    println!("\n=== Testing determinism ===");
    println!("Generating the same location twice:");
    let mut world2 = World::new();
    let test1 = world2.render_region(42, 17, 20, 5);
    let mut world3 = World::new();
    let test2 = world3.render_region(42, 17, 20, 5);
    println!("First generation:");
    println!("{}", test1);
    println!("Second generation (should be identical):");
    println!("{}", test2);
    println!("Match: {}", test1 == test2);
    
    println!("\n✨ The world is endless, quiet, and mysterious...");
}
