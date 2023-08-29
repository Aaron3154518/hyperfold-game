use hyperfold_engine::{
    ecs::{
        entities::{Entity, NewEntity},
        events::core::Update,
    },
    framework::{
        physics,
        render_system::{
            self,
            drawable::Canvas,
            font::{FontData, TIMES},
            render_data::{Fit, RenderAsset, RenderDataBuilderTrait, RenderTexture},
            render_text::{RenderText, TextImage},
            shapes::{Circle, ShapeTrait},
            Asset, AssetManager, Camera, RenderComponent, Renderer, Texture,
        },
    },
    utils::{
        colors::{RED, TRANSPARENT, WHITE},
        number::Number,
        rect::{Align, Point, Rect},
        util::AsType,
    },
};

use crate::{
    observers,
    param_dag::{NodeTrait, NumDag, Observe, Observer, Root},
    parameters,
    utils::elevations::Elevations,
};

pub fn crystal_radius(camera: &Camera) -> f32 {
    camera.0.w.min(camera.0.h) / 2.0
}

// Crystal components
#[hyperfold_engine::component(Singleton)]
struct Crystal;

// Parameters
parameters!(CrystalNumbers(Magic = 0));

#[hyperfold_engine::component(Dummy)]
struct CrystalTextObservers;

observers!(
    CrystalTextObservers<Number> {
        magic = CrystalNumbers::Magic
    }
);

hyperfold_engine::components!(labels(Crystal), CrystalPos, pos: &'a physics::Position);

// Crystal text components
#[hyperfold_engine::component(Singleton)]
struct CrystalText;

hyperfold_engine::components!(
    labels(CrystalText),
    CrystalTextData,
    pos: &'a mut physics::Position,
    text: &'a mut RenderComponent,
    observers: &'a mut CrystalTextObservers
);

// Crystal systems
#[hyperfold_engine::system(Init)]
fn init_crystal(
    entities: &mut dyn crate::_engine::Components,
    r: &Renderer,
    am: &mut AssetManager,
    screen: &render_system::Screen,
    camera: &Camera,
) {
    // Crystal
    let cx = screen.0.w as f32 / 2.0;
    let cy = screen.0.h as f32 / 2.0;

    let rect = Rect::from(cx, cy, 100.0, 100.0, Align::Center, Align::Center);

    let e = Entity::new();
    hyperfold_engine::add_components!(
        entities,
        e,
        Crystal,
        render_system::Elevation(Elevations::Crystal as u8),
        RenderComponent::new(RenderAsset::new(
            Asset::File("res/wizards/crystal.png".to_string()),
            r,
            am
        )),
        physics::Position(rect),
    );

    // Magic text
    let text_rect = Rect::from(
        rect.cx(),
        rect.y,
        rect.w,
        30.0,
        Align::Center,
        Align::BotRight,
    );

    let e = Entity::new();
    hyperfold_engine::add_components!(
        entities,
        e,
        render_system::Elevation(Elevations::Crystal as u8),
        RenderComponent::new(
            RenderText::new(FontData {
                w: None,
                h: Some(text_rect.h as u32),
                sample: "9.99e999".to_string(),
                file: TIMES.to_string()
            })
            .with_text_align(Align::Center, Align::BotRight)
            .with_text_color(WHITE)
            .with_text("[i]")
            .with_images(vec![TextImage::Render(RenderComponent::new(
                RenderAsset::from_file("res/wizards/catalyst.png", r, am)
            ))])
            .with_background_color(TRANSPARENT)
            .with_dest_fit(Fit::fit_height())
            .with_dest_align(Align::Center, Align::BotRight)
        ),
        physics::Position(text_rect),
        CrystalText,
        CrystalTextObservers::new(),
    );

    // Boundary circle
    let rad = crystal_radius(camera);
    let diam = rad + rad;
    let tex = Texture::new(r, diam as u32, diam as u32, TRANSPARENT);
    tex.draw(
        r,
        &mut Circle::new()
            .set_color(RED)
            .set_center(Point {
                x: rad as i32,
                y: rad as i32,
            })
            .border(rad as u32, -3, false)
            .dashed(20),
    );

    let e = Entity::new();
    hyperfold_engine::add_components!(
        entities,
        e,
        render_system::Elevation(Elevations::Background as u8),
        RenderComponent::new(RenderTexture::new(Some(tex))),
        physics::Position(
            rect.clone()
                .with_dim(diam, diam, Align::Center, Align::Center)
        )
    );
}

#[hyperfold_engine::system]
fn update_crystal_text(
    _: &Update,
    CrystalTextData {
        text, observers, ..
    }: CrystalTextData,
    dag: &mut NumDag,
) {
    // pos.0.set_pos(
    //     crys_pos.0.cx(),
    //     crys_pos.0.y,
    //     Align::Center,
    //     Align::BotRight,
    // );

    observers.magic.check(&mut dag.0, |m| {
        text.try_as_mut(|text: &mut RenderText| text.set_text(&format!("{m}[i]")));
    });
}
