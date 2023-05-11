#[engine2::system]
fn sys() {}

#[engine2::component]
struct S;

#[engine2::event]
enum Ev {}

pub mod L {
    pub mod O {
        #[engine2::event]
        enum L {
            Lol,
            Cat,
        }
    }
}
