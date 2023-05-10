pub struct A;
struct B;

#[engine::event]
enum Event {
    A,
    B(u32),
    C { name: String },
}
