use std::hash;

pub mod a {
    pub use super::z::a as A1;
    pub use A as A2;
    #[super::mac::component]
    pub struct A;
    impl A {
        pub fn foo(&self) {
            println!("A")
        }
    }
}

use engine as mac;

pub mod z {
    pub use super::a;
    use super::hash::Hash;
    pub use super::mac::component as comp;
    pub use a::A1 as A2;
    struct B;
    impl B {
        pub fn foo(&self) {
            println!("B")
        }
    }
}

pub use z::a as A3;
