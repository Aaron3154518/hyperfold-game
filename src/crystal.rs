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
            Asset, AssetManager, RenderComponent, Renderer, Texture,
        },
    },
    utils::{
        colors::{RED, TRANSPARENT, WHITE},
        rect::{Align, Point, Rect},
        util::AsType,
    },
};

// Crystal components
#[hyperfold_engine::component(Singleton)]
struct Crystal;

#[hyperfold_engine::component(Singleton)]
struct CrystalNumbers {
    pub magic: u32,
}

impl CrystalNumbers {
    pub fn new() -> Self {
        Self { magic: 0 }
    }
}

hyperfold_engine::components!(labels(Crystal), CrystalPos, pos: &'a physics::Position);

hyperfold_engine::components!(labels(Crystal), CrystalData, data: &'a mut CrystalNumbers);

// Crystal text components
#[hyperfold_engine::component(Singleton)]
struct CrystalText;

hyperfold_engine::components!(
    labels(CrystalText),
    CrystalTextData,
    pos: &'a mut physics::Position,
    text: &'a mut RenderComponent
);

// Crystal systems
#[hyperfold_engine::system(Init)]
fn init_crystal(
    entities: &mut dyn crate::_engine::AddComponent,
    r: &Renderer,
    am: &mut AssetManager,
    screen: &render_system::Screen,
) {
    // Crystal
    let cx = screen.0.w as f32 / 2.0;
    let cy = screen.0.h as f32 / 2.0;

    let rect = Rect::from(cx, cy, 100.0, 100.0, Align::Center, Align::Center);

    let e = Entity::new();
    hyperfold_engine::add_components!(
        entities,
        e,
        render_system::Elevation(1),
        RenderComponent::new(RenderAsset::new(
            Asset::File("res/wizards/crystal.png".to_string()),
            r,
            am
        )),
        physics::Position(rect),
        CrystalNumbers::new(),
        Crystal
    );

    // Magic text
    // let text_rect = Rect {
    //     x: 0.0,
    //     y: 0.0,
    //     w: rect.w,
    //     h: 50.0,
    // };
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
        render_system::Elevation(1),
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
                RenderAsset::from_file("res/wizards/catalyst.png".to_string(), r, am)
            ))])
            .with_background_color(TRANSPARENT)
            .with_dest_fit(Fit::fit_height())
            .with_dest_align(Align::Center, Align::BotRight)
        ),
        physics::Position(text_rect),
        CrystalText
    );

    // Boundary circle
    let w = screen.0.w.min(screen.0.h);
    let tex = Texture::new(r, w, w, TRANSPARENT);
    tex.draw(
        r,
        &mut Circle::new()
            .set_color(RED)
            .set_center(Point {
                x: w as i32 / 2,
                y: w as i32 / 2,
            })
            .border(w / 2, -3, false)
            .dashed(20),
    );

    let e = Entity::new();
    hyperfold_engine::add_components!(
        entities,
        e,
        render_system::Elevation(0),
        RenderComponent::new(RenderTexture::new(Some(tex))),
        physics::Position(
            rect.clone()
                .with_dim(w as f32, w as f32, Align::Center, Align::Center)
        )
    );
}

#[hyperfold_engine::system]
fn update_crystal_text(
    _: &Update,
    CrystalData {
        // pos: crys_pos,
        data,
        ..
    }: CrystalData,
    CrystalTextData { text, .. }: CrystalTextData,
) {
    // pos.0.set_pos(
    //     crys_pos.0.cx(),
    //     crys_pos.0.y,
    //     Align::Center,
    //     Align::BotRight,
    // );

    text.try_mut(|text: &mut RenderText| text.set_text(&format!("{}[i]", data.magic)));
}
