#![allow(unused)]

use b as b2;
use c;

game_crate!();

pub const T1: u8 = 0;
static T2: u8 = 0;
pub trait T3 {}
type T4 = u8;
union T5 {
    i: u8,
}
#[engine::component]
pub struct T6;
pub fn T7() {}
// TODO: try toggling pub
pub mod t8 {
    #[engine::event]
    pub struct X;
    #[engine::event]
    pub struct Y(u8);
}

mod a1;
pub mod a2;

pub use a2 as a22;

pub use a2::a3::z::comp as component;
use engine::{
    ecs::components::{Container, Label, NorLabels},
    ecs::entities::Entity,
    game_crate,
};

#[engine::system]
fn comp(
    _e: &crate::t8::X,
    id: &Entity,
    t6: &T6,
    dc: &c::e::DC,
    sc: Label<T6>,
    re: NorLabels<(a2::a3::a::A, a2::a3::a::A)>,
    t: &dyn crate::_engine::Peepee,
) {
}

#[engine::system]
fn vec(
    _e: &crate::t8::Y,
    dc: &c::e::DC,
    sc: Label<T6>,
    re: NorLabels<(a2::a3::a::A, a2::a3::a::A)>,
    vc: Container<(&mut T6, &Entity)>,
    t: &dyn crate::_engine::Peepee,
) {
}

#[engine::system(Init)]
fn init(t: &dyn crate::_engine::Peepee) {}

fn main() {
    println!("Hello, world!");
}
