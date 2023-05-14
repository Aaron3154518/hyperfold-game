pub struct A;
struct B;

#[engine::event]
struct X;
#[engine::event]
struct Y(u32);
#[engine::event]
struct Z {
    name: String,
}
