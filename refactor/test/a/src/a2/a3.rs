use std::hash;

pub mod A {
    pub use super::B::A as A1;
    use A as A2;
    pub struct A;
    impl A {
        pub fn foo(&self) {
            println!("A")
        }
    }
}

mod B {
    use super::hash::Hash;
    pub use super::A;
    pub use super::A::A1 as A2;
    struct B;
    impl B {
        pub fn foo(&self) {
            println!("B")
        }
    }
}

pub use B::A as A3;
