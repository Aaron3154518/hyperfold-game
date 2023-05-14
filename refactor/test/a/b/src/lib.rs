engine2::game_crate!();

#[engine2::system]
fn sys(_ev: &l::o::l::Lol) {}

#[engine2::component]
struct S;

pub mod l {
    pub mod o {
        pub mod l {
            #[engine2::event]
            pub struct Lol;

            #[engine2::event]
            struct Cat;
        }
    }
}
