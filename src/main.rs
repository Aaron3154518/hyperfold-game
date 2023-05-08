use hyperfold_engine::ecs;
use hyperfold_engine::{sdl2, sdl2_image};

#[ecs::dependency]
use hyperfold_engine as hfe;
use hyperfold_engine;

mod crystal;
mod wizard;

ecs::component_manager!();

fn main() {
    println!("Hello, world!");

    hyperfold_engine::init_sdl();

    let mut f = SFoo::new();
    f.run();
    drop(f);

    hyperfold_engine::quit_sdl();
}
