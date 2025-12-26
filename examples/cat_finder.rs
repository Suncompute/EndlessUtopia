/// Example: Interactive cat finder
/// Helps locate rare cat coordinates

use endless_utopia::World;

fn main() {
    println!("🐱 EndlessUtopia - Cat Finder");
    println!("================================\n");
    
    let world = World::new();
    
    // Search a large area systematically
    println!("Searching for cats in a 2000x2000 area around origin...\n");
    
    let mut cat_locations = Vec::new();
    let search_radius = 1000;
    
    // Sample strategically instead of checking every coordinate
    for y in (-search_radius..=search_radius).step_by(13) {
        for x in (-search_radius..=search_radius).step_by(13) {
            if world.is_cat_location(x, y) {
                cat_locations.push((x, y));
            }
        }
    }
    
    println!("Found {} cats in sampled area!", cat_locations.len());
    println!("\nCat Coordinates:");
    println!("================");
    
    for (i, (x, y)) in cat_locations.iter().take(20).enumerate() {
        println!("{:2}. ({:5}, {:5})", i + 1, x, y);
    }
    
    if cat_locations.len() > 20 {
        println!("... and {} more!", cat_locations.len() - 20);
    }
    
    // Show one cat in detail
    if let Some(&(x, y)) = cat_locations.first() {
        println!("\n\nViewing first cat at ({}, {}):", x, y);
        println!("================================");
        let mut world_mut = World::new();
        let view = world_mut.render_region(x - 15, y - 5, 30, 10);
        println!("{}", view);
        
        println!("\nAfter visiting (showing trace):");
        let view_with_trace = world_mut.render_region(x - 15, y - 5, 30, 10);
        println!("{}", view_with_trace);
    }
    
    println!("\n💡 Tip: Copy any coordinate above and explore it in your own code!");
}
