#![feature(trait_upcasting)]

mod crystal;
mod fireball;
mod text_wizard;
mod wizard;

hyperfold_engine::game_crate!();

fn main() {
    // hyperfold_engine::test::test();
    hyperfold_engine::init_sdl();

    let mut f = _engine::SFoo::new();
    f.run();
    drop(f);

    hyperfold_engine::quit_sdl();
}
