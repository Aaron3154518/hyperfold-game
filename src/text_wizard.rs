use std::f32::consts::{PI, TAU};

use hyperfold_engine::{
    ecs::{
        entities::{Entity, NewEntity},
        events::core::Update,
    },
    framework::{
        physics::Position,
        render_system::{
            self,
            drawable::Canvas,
            font::{FontData, TIMES},
            render_data::{Animation, Fit, RenderAsset, RenderDataBuilderTrait, RenderTexture},
            render_text::{RenderText, TextImage},
            shapes::{Circle, Rectangle, ShapeTrait},
            AssetManager, RenderComponent, Renderer, Texture,
        },
    },
    utils::{
        colors::{BLUE, GREEN, MAGENTA, TRANSPARENT, WHITE},
        rect::{Align, Point, PointF, Rect},
    },
};

use crate::utils::elevations::Elevations;

#[hyperfold_engine::component]
struct Rotation {
    r: f32,
    theta: f32,
}

#[hyperfold_engine::component]
struct TextWizard;

#[hyperfold_engine::system(Init)]
fn init_text_wizard(
    entities: &mut dyn crate::_engine::Components,
    r: &Renderer,
    am: &mut AssetManager,
) {
    let anim_e = Entity::new();
    let anim = Animation::new(8, 100);
    let rc = RenderComponent::new(
        RenderAsset::from_file("res/wizards/power_wizard_ss.png", r, am).with_animation(anim),
    );
    hyperfold_engine::add_components!(entities, anim_e, anim, rc);

    // Draw shapes
    let tex = Texture::new(r, 100, 100, MAGENTA);
    tex.draw(
        r,
        &mut Rectangle::new()
            .set_color(BLUE)
            .set_boundary(Rect {
                x: 5.0,
                y: 5.0,
                w: 95.0,
                h: 95.0,
            })
            .fill(Rect {
                x: 0.0,
                y: 0.0,
                w: 50.0,
                h: 50.0,
            })
            .except(Rect {
                x: 40.0,
                y: 40.0,
                w: 40.0,
                h: 40.0,
            }),
    );
    tex.draw(
        r,
        &mut Circle::new()
            .set_color(GREEN)
            .set_boundary(Rect {
                x: 0.0,
                y: 5.0,
                w: 95.0,
                h: 95.0,
            })
            .set_center(Point { x: 85, y: 15 })
            .border(13, 3, true)
            .dashed(20)
            .set_angle_deg(180.0, 45.0),
    );

    let e = Entity::new();
    hyperfold_engine::add_components!(
        entities,
        e,
        render_system::RenderOpts::new(Elevations::Wizards as u8),
        RenderComponent::new(
            RenderText::new(FontData {
                w: Some(100),
                h: Some(30),
                sample: "World".to_string(),
                file: TIMES.to_string(),
            })
            .with_text("Hello[b][i][i][b]World\n[i]")
            .with_text_align(Align::Center, Align::Center)
            .with_dest_fit(Fit::fit_width())
            .with_images(vec![
                TextImage::Render(RenderComponent::new(RenderAsset::from_file(
                    "res/projectiles/fireball2.png",
                    r,
                    am
                ))),
                TextImage::Reference(anim_e),
                TextImage::Render(RenderComponent::new(RenderTexture::new(Some(tex))))
            ])
            .with_text_color(WHITE)
            .with_background_color(TRANSPARENT)
        ),
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

hyperfold_engine::components!(
    labels(TextWizard),
    RotateTextWizard,
    pos: &'a mut Position,
    rot: &'a mut Rotation,
);

#[hyperfold_engine::system]
fn rotate_text_wizard(
    update: &Update,
    screen: &render_system::Screen,
    entities: Vec<RotateTextWizard>,
) {
    for RotateTextWizard { pos, rot, .. } in entities {
        rot.theta = (rot.theta + update.0 as f32 * PI / 5000.0) % TAU;
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
}
