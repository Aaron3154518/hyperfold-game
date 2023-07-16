#![feature(trait_upcasting)]
#![feature(drain_filter)]

mod crystal;
mod fireball;
mod text_wizard;
mod wizard;

hyperfold_engine::game_crate!();

// #[hyperfold_engine::component(Singleton)]
// struct SingleLabel;

// #[hyperfold_engine::component]
// struct NotLabel;

// #[hyperfold_engine::component]
// struct NotIn;

// #[hyperfold_engine::global]
// struct Glob;

// hyperfold_engine::components!(
//     labels(NotLabel && ((NotLabel) || !NotLabel || !SingleLabel) && SingleLabel),
//     Thing,
//     lab: &'a NotLabel,
//     not_in: &'a NotIn,
// );

// #[hyperfold_engine::system]
// fn foo(
//     ev: &hyperfold_engine::ecs::events::core::Update,
//     thing: Thing,
//     thing2: Thing,
//     thing3: Thing,
// ) {
// }

// #[hyperfold_engine::component]
// struct MutMe;

// hyperfold_engine::components!(
//     labels(NotIn && NotLabel && !MutMe),
//     NoIn,
//     not: &'a NotIn,
//     not2: &'a NotLabel,
//     mut_me: &'a mut MutMe
// );

// #[hyperfold_engine::system]
// fn bar(
//     ev: &hyperfold_engine::ecs::events::core::Update,
//     not: hyperfold_engine::ecs::systems::Entities<NoIn>,
//     thing: Thing,
// ) {
// }

// #[hyperfold_engine::system(Init)]
// fn init(r: &Renderer) {}

fn main() {
    // hyperfold_engine::test::test();
    hyperfold_engine::init_sdl();

    let mut f = _engine::SFoo::new();
    f.run();
    drop(f);

    hyperfold_engine::quit_sdl();
}
