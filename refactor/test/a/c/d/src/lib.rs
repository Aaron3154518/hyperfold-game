use engine::game_crate;

game_crate!();

#[engine::global]
pub struct DC;

impl DC {
    pub fn new() -> Self {
        Self
    }
}

#[engine::system(Init)]
pub fn bar(dc: &mut DC) {}
