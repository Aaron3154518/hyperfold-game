#[engine2::system]
fn sys(_ev: &L::O::L::Lol) {}

#[engine2::component]
struct S;

pub mod L {
    pub mod O {
        pub mod L {
            #[engine2::event]
            pub struct Lol;

            #[engine2::event]
            struct Cat;
        }
    }
}
