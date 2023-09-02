#![feature(trait_upcasting)]
#![feature(extract_if)]
#![feature(trait_alias)]
#![feature(array_methods)]

mod crystal;
mod enemies;
mod fireball;
pub mod param_dag;
mod text_wizard;
mod upgrades;
mod utils;
mod wizard;

hyperfold_engine::game_crate!();

fn main() {
    // hyperfold_engine::test::test();
    hyperfold_engine::run::<_engine::SFoo>();
}
