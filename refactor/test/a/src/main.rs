use b;
use c;

pub const T1: u8 = 0;
static T2: u8 = 0;
pub trait T3 {}
type T4 = u8;
union T5 {
    i: u8,
}
pub struct T6;
pub fn T7() {}
enum T8 {}

mod a1;
mod a2;

use a2 as a22;

fn main() {
    println!("Hello, world!");
}
