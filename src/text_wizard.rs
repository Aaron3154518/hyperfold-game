use std::f32::consts::{PI, TAU};

use hyperfold_engine::{
    ecs::{
        components::Label,
        entities::{Entity, NewEntity},
        events::core::Update,
    },
    framework::{
        physics::Position,
        render_system::{
            self,
            font::{FontData, TIMES},
            text::render_text,
            AssetManager, Renderer,
        },
    },
    utils::rect::{Align, PointF, Rect},
};

#[hyperfold_engine::component]
struct Rotation {
    r: f32,
    theta: f32,
}

#[hyperfold_engine::component]
struct TextWizard;

#[hyperfold_engine::system(Init)]
fn init_text_wizard(
    entities: &mut dyn crate::_engine::AddComponent,
    r: &Renderer,
    am: &mut AssetManager,
) {
    let tex = render_text(
        r,
        am,
        "Hello\nWorld",
        FontData {
            w: Some(100),
            h: Some(50),
            sample: "World".to_string(),
            file: TIMES.to_string(),
        },
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
        render_system::RenderComponent::from_texture(tex),
        Position(Rect {
            x: 0.0,
            y: 0.0,
            w: 100.0,
            h: 100.0,
        }),
        Rotation {
            r: 300.0,
            theta: 0.0
        },
        TextWizard
    );
}

#[hyperfold_engine::system]
fn rotate_text_wizard(
    update: &Update,
    pos: &mut Position,
    screen: &render_system::Screen,
    rot: &mut Rotation,
    _l: Label<TextWizard>,
) {
    rot.theta = (rot.theta + update.0 as f32 * PI / 3000.0) % TAU;
    let c = PointF {
        x: screen.0.w as f32 / 2.0,
        y: screen.0.h as f32 / 2.0,
    };
    pos.0.set_pos(
        rot.r * rot.theta.cos() + c.x,
        rot.r * rot.theta.sin() + c.y,
        Align::Center,
        Align::Center,
    );
    // let d = PointF {
    //     x: screen.0.w as f32 / 2.0,
    //     y: screen.0.h as f32 / 2.0,
    // } - pos.0.center();
    // let mag = d.mag();
    // if mag < F32_ERR {
    //     pd.a = PointF { x: 0.0, y: 0.0 };
    // } else {
    //     let v_mag = pd.v.mag();
    //     let frac = v_mag.powf(2.0) / mag + 5.0;
    //     pd.a = PointF {
    //         x: d.x * frac / mag,
    //         y: d.y * frac / mag,
    //     }
    // }
}
