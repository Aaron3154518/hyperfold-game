use engine::game_crate;

game_crate!();

#[engine::global]
pub struct DC;

#[engine::system(Init)]
pub fn bar(dc: &mut DC) {}
