use hyperfold_engine::{
    ecs::{
        entities::{Entity, NewEntity},
        events::core,
    },
    framework::{
        event_system::{
            self,
            inputs::{self, Click},
            Drag, DragTrigger,
        },
        physics,
        render_system::{
            self,
            render_data::{Animation, RenderAsset, RenderDataBuilderTrait},
            AssetManager, Renderer,
        },
    },
    sdl2::SDL_KeyCode::*,
    utils::rect::{Align, PointF, Rect},
};

use crate::{
    crystal::{Crystal, CrystalData},
    fireball::CreateFireball,
};

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
        Drag::new(DragTrigger::DelayMs(500)),
        Timer(1000),
        Wizard
    );
}

hyperfold_engine::components!(
    labels(Wizard),
    WizardDragData,
    pos: &'a mut physics::Position,
);

#[hyperfold_engine::system]
fn drag_wizard(drag: &inputs::Drag, WizardDragData { pos, .. }: WizardDragData) {
    pos.0.set_pos(
        drag.mouse_x as f32,
        drag.mouse_y as f32,
        Align::Center,
        Align::Center,
    );
}

#[hyperfold_engine::system]
fn click_wizard(m: &Click, WizardDragData { eid, .. }: WizardDragData) {
    if m.is_me(eid) {
        eprintln!("Clicked :#)")
    }
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
    update: &core::Update,
    camera: &mut render_system::Camera,
    WizardData { pos: wiz_pos, .. }: WizardData,
    CrystalData { pos: crys_pos, .. }: CrystalData,
) {
    let (wiz_pos, crys_pos) = (wiz_pos.0.center(), crys_pos.0.center());
    let pos = match wiz_pos.dist(crys_pos) < camera.0.w.min(camera.0.h) / 2.0 {
        true => crys_pos,
        false => wiz_pos,
    };
    let cam_pos = camera.0.center();
    let mut delta = pos - cam_pos;
    if delta.mag() > 1.0 {
        delta = delta * (update.0 as f32 / 500.0).min(1.0);
    }
    camera.0.set_pos(
        cam_pos.x + delta.x,
        cam_pos.y + delta.y,
        Align::Center,
        Align::Center,
    )
}

#[hyperfold_engine::system]
fn move_keys(ev: &event_system::inputs::Key, WizardData { pd, .. }: WizardData) {
    const V: f32 = 150.0;
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
