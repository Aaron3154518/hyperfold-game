use b as b2;
use c;

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
pub mod T8 {
    #[engine::event]
    pub struct X;
    #[engine::event]
    pub struct Y(u8);
}

mod a1;
pub mod a2;

pub use a2 as a22;

pub use a2::a3::B::comp as component;
use engine::{Container, Label, NorLabels};

pub trait EFoo {}

#[engine::system]
fn comp(
    _e: &crate::T8::X,
    t6: &T6,
    dc: &c::e::DC,
    sc: Label<T6>,
    re: NorLabels<(a2::a3::A::A, a2::a3::A::A)>,
    t: &dyn EFoo,
) {
}

#[engine::system]
fn vec(
    _e: &crate::T8::Y,
    dc: &c::e::DC,
    sc: Label<T6>,
    re: NorLabels<(a2::a3::A::A, a2::a3::A::A)>,
    vc: Container<(&mut T6, &engine::Entity)>,
    t: &dyn EFoo,
) {
}

#[engine::system(Init)]
fn init(t: &dyn EFoo) {}

fn main() {
    println!("Hello, world!");
}
