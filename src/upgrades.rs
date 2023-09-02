use hyperfold_engine::{
    add_components,
    ecs::entities::{Entity, NewEntity},
    framework::{
        physics::Position,
        render_system::{
            render_data::RenderTexture, RenderComponent, RenderOpts, Renderer, Screen, Texture,
        },
    },
    utils::{
        colors::GRAY,
        rect::{Align, Rect},
    },
};

use crate::{_engine::Components, utils::elevations::Elevations};

#[hyperfold_engine::component(Singleton)]
struct UpgradeBox;

#[hyperfold_engine::system(Init)]
pub fn init_upgrades(entities: &mut dyn Components, screen: &Screen, r: &Renderer) {
    let (w, h) = (screen.0.w as f32, screen.0.h as f32);
    let rect = Rect::from(
        w / 2.0,
        0.0,
        w / 2.0,
        h / 5.0,
        Align::Center,
        Align::TopLeft,
    );
    let tex = Texture::new(r, rect.w_i32() as u32, rect.h_i32() as u32, GRAY);

    let e = Entity::new();
    add_components!(
        entities,
        e,
        UpgradeBox,
        Position(rect),
        RenderOpts::new(Elevations::Upgrades as u8)
            .absolute()
            .with_visibility(false),
        RenderComponent::new(RenderTexture::new(Some(tex)))
    );
}
