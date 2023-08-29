use hyperfold_engine::{
    ecs::{
        entities::{Entity, NewEntity},
        events::core,
    },
    framework::{
        event_system::{
            components::DragTrigger,
            events::{Click, Drag, Key},
        },
        physics,
        render_system::{
            self,
            render_data::{Animation, RenderAsset, RenderDataBuilderTrait},
            AssetManager, Renderer,
        },
    },
    sdl2::SDL_KeyCode::*,
    utils::{
        number::Number,
        rect::{Align, Rect},
        timer::{Timer, TimerTrait},
    },
};

use crate::{
    crystal::{crystal_radius, CrystalNumbers, CrystalPos},
    equation,
    fireball::CreateFireball,
    param_dag::{Node, NodeDefault, NodeTrait, NumDag},
    parameters,
    utils::elevations::Elevations,
};

parameters!(WizardNumbers(Power));

#[hyperfold_engine::system(Init)]
fn init_wizard_numbers(dag: &mut NumDag) {
    dag.0.add_node(
        WizardNumbers::Power,
        equation!(|(m: CrystalNumbers::Magic,)| (*m + 1.into()).log10() + 1.into()),
    );
}

#[hyperfold_engine::component(Singleton)]
struct Wizard;

#[hyperfold_engine::system(Init)]
fn init_wizard(
    entities: &mut dyn crate::_engine::Components,
    r: &Renderer,
    am: &mut AssetManager,
    camera: &render_system::Camera,
) {
    let e = Entity::new();
    let anim = Animation::new(5, 150);
    let rc = render_system::RenderComponent::new(
        RenderAsset::from_file("res/wizards/wizard_ss.png", r, am).with_animation(anim),
    );
    hyperfold_engine::add_components!(
        entities,
        e,
        Wizard,
        render_system::Elevation(Elevations::Wizards as u8),
        rc,
        anim,
        physics::Position(Rect {
            x: camera.0.cx() - 50.0,
            y: camera.0.cy() - 150.0,
            w: 100.0,
            h: 100.0,
        }),
        physics::PhysicsData::new(),
        DragTrigger::OnMove,
        Timer::new(1000),
    );
}

hyperfold_engine::components!(labels(Wizard), WizardPos, pos: &'a physics::Position,);
hyperfold_engine::components!(labels(Wizard), WizardPosMut, pos: &'a mut physics::Position,);

#[hyperfold_engine::system]
fn drag_wizard(drag: &Drag, WizardPosMut { pos, .. }: WizardPosMut) {
    pos.0.set_pos(
        drag.mouse_x as f32,
        drag.mouse_y as f32,
        Align::Center,
        Align::Center,
    );
}

#[hyperfold_engine::system]
fn click_wizard(m: &Click, WizardPos { eid, .. }: WizardPos) {
    if m.is_me(eid) {
        eprintln!("Clicked :#)")
    }
}

#[hyperfold_engine::system]
fn track_wizard(
    update: &core::Update,
    camera: &mut render_system::Camera,
    WizardPos { pos: wiz_pos, .. }: WizardPos,
    CrystalPos { pos: crys_pos, .. }: CrystalPos,
) {
    let (wiz_pos, crys_pos) = (wiz_pos.0.center(), crys_pos.0.center());
    let pos = match wiz_pos.dist(crys_pos) < crystal_radius(camera) {
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

hyperfold_engine::components!(
    labels(Wizard),
    WizardData,
    pos: &'a physics::Position,
    pd: &'a mut physics::PhysicsData,
    timer: &'a mut Timer,
);

#[hyperfold_engine::system]
fn move_keys(ev: &Key, WizardData { pd, .. }: WizardData) {
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
    dt: &core::Update,
    events: &mut dyn crate::_engine::Events,
    WizardData { timer, pos, .. }: WizardData,
    dag: &mut NumDag,
) {
    for _ in 0..timer.add_time(dt.0) {
        events.new_event(CreateFireball {
            pos: pos.0.center(),
            value: *dag.0.get(WizardNumbers::Power),
        });
    }
}
