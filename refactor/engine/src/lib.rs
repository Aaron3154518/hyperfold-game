#![feature(hash_drain_filter)]

pub use macros::{component, event, game_crate, global, system};
pub mod ecs;
pub mod framework;
pub mod intersect;
pub mod utils;

game_crate!();
