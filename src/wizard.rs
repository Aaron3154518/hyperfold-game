use hyperfold_engine::ecs::components::Label;
use hyperfold_engine::ecs::entities::{Entity, NewEntity};
use hyperfold_engine::ecs::events::core;
use hyperfold_engine::framework::font::{FontData, TIMES};
use hyperfold_engine::framework::text::render_text;
use hyperfold_engine::framework::{event_system, physics, render_system};
use hyperfold_engine::sdl2::SDL_KeyCode::*;
use hyperfold_engine::utils::rect::{Align, PointF, Rect};

use crate::fireball::new_fireball;

#[hyperfold_engine::component]
struct Wizard;

#[hyperfold_engine::component]
struct Timer(i32);

#[hyperfold_engine::system(Init)]
fn init_wizard(
    entities: &mut dyn crate::_engine::AddComponent,
    rs: &mut render_system::RenderSystem,
    camera: &render_system::Camera,
) {
    let font = rs
        .am
        .get_font(FontData {
            w: Some(100),
            h: Some(50),
            sample: "World".to_string(),
            file: TIMES.to_string(),
        })
        .expect("Could not create font");

    let tex = render_text(
        rs.r.access(),
        "Hello\nWorld",
        &font,
        Rect {
            x: 0.0,
            y: 0.0,
            w: 100.0,
            h: 25.0,
        },
        Align::Center,
        Align::Center,
    );

    let e = Entity::new();
    hyperfold_engine::add_components!(
        entities,
        e,
        render_system::Elevation(2),
        render_system::Image::from(tex),
        // render_system::Image::from(rs.get_image("res/wizards/wizard.png")),
        physics::Position(Rect {
            x: camera.0.cx() - 50.0,
            y: camera.0.cy() - 150.0,
            w: 100.0,
            h: 100.0,
        }),
        physics::PhysicsData::new(),
        Timer(1000),
        Wizard
    );
}

#[hyperfold_engine::system]
fn track_wizard(
    _ev: &core::Update,
    pos: &physics::Position,
    camera: &mut render_system::Camera,
    _l: Label<Wizard>,
) {
    camera
        .0
        .set_pos(pos.0.cx(), pos.0.cy(), Align::Center, Align::Center)
}

#[hyperfold_engine::system]
fn move_keys(ev: &event_system::inputs::Key, pd: &mut physics::PhysicsData, _l: Label<Wizard>) {
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

#[hyperfold_engine::system]
fn update(
    ev: &core::Update,
    timer: &mut Timer,
    pos: &physics::Position,
    entities: &mut dyn crate::_engine::AddComponent,
    rs: &mut render_system::RenderSystem,
    screen: &render_system::Screen,
    _l: Label<Wizard>,
) {
    timer.0 -= ev.0 as i32;
    while timer.0 <= 0 {
        timer.0 += 1000;
        new_fireball(
            entities,
            rs,
            pos.0.center(),
            PointF {
                x: screen.0.w as f32 / 2.0,
                y: screen.0.h as f32 / 2.0,
            },
        );
    }
}
