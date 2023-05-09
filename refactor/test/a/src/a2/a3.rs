use std::hash;

pub mod A {
    pub use super::B::A as A1;
    pub use A as A2;
    pub struct A;
    impl A {
        pub fn foo(&self) {
            println!("A")
        }
    }
}

use macros as mac;

pub mod B {
    use super::hash::Hash;
    pub use super::mac::component as comp;
    pub use super::A;
    pub use A::A1 as A2;
    struct B;
    impl B {
        pub fn foo(&self) {
            println!("B")
        }
    }
}

pub use B::A as A3;
