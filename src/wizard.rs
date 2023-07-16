use hyperfold_engine::{
    ecs::{
        entities::{Entity, NewEntity},
        events::core,
    },
    framework::{
        event_system, physics,
        render_system::{
            self,
            render_data::{Animation, RenderAsset, RenderDataBuilderTrait},
            AssetManager, Renderer,
        },
    },
    sdl2::SDL_KeyCode::*,
    utils::rect::{Align, PointF, Rect},
};

use crate::fireball::CreateFireball;

#[hyperfold_engine::component(Singleton)]
struct Wizard;

#[hyperfold_engine::component]
struct Timer(i32);

#[hyperfold_engine::system(Init)]
fn init_wizard(
    entities: &mut dyn crate::_engine::AddComponent,
    r: &Renderer,
    am: &mut AssetManager,
    camera: &render_system::Camera,
) {
    let e = Entity::new();
    let rc = render_system::RenderComponent::new(
        RenderAsset::from_file("res/wizards/wizard_ss.png".to_string(), r, am).with_animation(
            entities,
            e,
            Animation::new(5, 150),
        ),
    );
    hyperfold_engine::add_components!(
        entities,
        e,
        render_system::Elevation(2),
        rc,
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

hyperfold_engine::components!(
    labels(Wizard),
    WizardData,
    pos: &'a physics::Position,
    pd: &'a mut physics::PhysicsData,
    timer: &'a mut Timer,
);

#[hyperfold_engine::system]
fn track_wizard(
    _ev: &core::Update,
    camera: &mut render_system::Camera,
    WizardData { pos, .. }: WizardData,
) {
    camera
        .0
        .set_pos(pos.0.cx(), pos.0.cy(), Align::Center, Align::Center)
}

#[hyperfold_engine::system]
fn move_keys(ev: &event_system::inputs::Key, WizardData { pd, .. }: WizardData) {
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
    events: &mut dyn crate::_engine::AddEvent,
    screen: &render_system::Screen,
    WizardData { timer, pos, .. }: WizardData,
) {
    timer.0 -= ev.0 as i32;
    while timer.0 <= 0 {
        timer.0 += 1000;
        events.new_event(CreateFireball {
            pos: pos.0.center(),
            target: PointF {
                x: screen.0.w as f32 / 2.0,
                y: screen.0.h as f32 / 2.0,
            },
        });
    }
}
