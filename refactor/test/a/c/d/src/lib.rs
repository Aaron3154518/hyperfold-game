#[engine::global]
pub struct DC;

#[engine::system(Init)]
pub fn bar(dc: &mut DC) {}
