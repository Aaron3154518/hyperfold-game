use hyperfold_engine;
use hyperfold_engine::ecs;
use hyperfold_engine::{sdl2, sdl2_image};

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
