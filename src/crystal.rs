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
            render_data::{FitMode, RenderDataBuilderTrait, RenderTexture},
            render_text::RenderText,
            renderer::W,
            shapes::{Circle, Rectangle, ShapeTrait},
            RenderComponent, Renderer, Texture,
        },
    },
    utils::{
        colors::{BLUE, GREEN, MAGENTA, RED, TRANSPARENT, WHITE},
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
    screen: &render_system::Screen,
) {
    // Crystal
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

    let cx = screen.0.w as f32 / 2.0;
    let cy = screen.0.h as f32 / 2.0;

    let rect = Rect::from(cx, cy, 100.0, 100.0, Align::Center, Align::Center);

    let e = Entity::new();
    hyperfold_engine::add_components!(
        entities,
        e,
        render_system::Elevation(1),
        RenderComponent::new(RenderTexture::new(Some(tex))),
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
        50.0,
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
                w: Some(text_rect.w as u32),
                h: Some(text_rect.h as u32),
                sample: "9.99e999".to_string(),
                file: TIMES.to_string()
            })
            .with_text_align(Align::Center, Align::BotRight)
            .with_text_color(WHITE)
            .with_background_color(TRANSPARENT)
            .with_dest_fit(FitMode::FitWithin(Align::Center, Align::BotRight))
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
    CrystalTextData { pos, text, .. }: CrystalTextData,
) {
    // pos.0.set_pos(
    //     crys_pos.0.cx(),
    //     crys_pos.0.y,
    //     Align::Center,
    //     Align::BotRight,
    // );

    text.try_mut(|text: &mut RenderText| text.set_text(format!("{}", data.magic)));
}
