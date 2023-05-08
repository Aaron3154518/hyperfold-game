use hyperfold_engine::ecs;
use hyperfold_engine::ecs::components::Label;
use hyperfold_engine::ecs::events::CoreEvent;
use hyperfold_engine::framework::{event_system, physics, render_system};
use hyperfold_engine::includes::*;
use hyperfold_engine::sdl2::SDL_KeyCode::*;
use hyperfold_engine::utils::rect::{Align, Rect};

#[ecs::component]
struct Wizard;

#[ecs::system(Init)]
fn init_wizard(
    entities: &mut dyn crate::CFooT,
    rs: &mut render_system::RenderSystem,
    camera: &render_system::Camera,
) {
    let e = ecs::create_entity!(
        entities,
        render_system::Elevation(1),
        render_system::Image(rs.get_image("res/wizards/wizard.png")),
        physics::Position(Rect {
            x: camera.0.cx() - 50.0,
            y: camera.0.cy() - 150.0,
            w: 100.0,
            h: 100.0,
        }),
        physics::PhysicsData::new(),
        Wizard
    );
}

#[ecs::system]
fn track_wizard(
    _ev: &CoreEvent::Update,
    pos: &physics::Position,
    camera: &mut render_system::Camera,
    _l: Label<Wizard>,
) {
    camera
        .0
        .set_pos(pos.0.cx(), pos.0.cy(), Align::Center, Align::Center)
}

#[ecs::system]
fn move_keys(ev: &event_system::Events::Key, pd: &mut physics::PhysicsData, _l: Label<Wizard>) {
    const V: f32 = 250.0;
    if let Some((val, amnt)) = match ev.0.key {
        SDLK_a => Some((&mut pd.v.x, -V)),
        SDLK_d => Some((&mut pd.v.x, V)),
        SDLK_w => Some((&mut pd.v.y, -V)),
        SDLK_s => Some((&mut pd.v.y, V)),
        _ => None,
    } {
        if ev.0.down() {
            *val += amnt;
        }
        if ev.0.up() {
            *val -= amnt;
        }
    }
}
