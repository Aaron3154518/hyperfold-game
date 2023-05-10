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
enum T8 {}

mod a1;
pub mod a2;

pub use a2 as a22;

pub use a2::a3::B::comp as component;
use engine::{Container, Label};

pub trait Tr {}

#[engine::system]
fn foo(t6: &T6, dc: &c::d::DC, sc: Label<T6>, vc: Container<(T6, c::d::DC)>, t: &mut dyn Tr) {}

fn main() {
    c::d::bar();

    println!("Hello, world!");
}
