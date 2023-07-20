use hyperfold_engine::{
    ecs::entities::{Entity, NewEntity},
    framework::{
        physics,
        render_system::{
            self,
            drawable::Canvas,
            render_data::RenderTexture,
            shapes::{Circle, Rectangle, ShapeTrait},
            Renderer, Texture,
        },
    },
    sdl2::SDL_Color,
    utils::{
        colors::{BLUE, GREEN, MAGENTA, RED, TRANSPARENT},
        rect::{Align, Point, Rect},
    },
};

#[hyperfold_engine::component(Singleton)]
struct Crystal;

hyperfold_engine::components!(labels(Crystal), CrystalData, pos: &'a physics::Position,);

#[hyperfold_engine::system(Init)]
fn init_crystal(
    entities: &mut dyn crate::_engine::AddComponent,
    r: &Renderer,
    screen: &render_system::Screen,
) {
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

    let e = Entity::new();
    hyperfold_engine::add_components!(
        entities,
        e,
        render_system::Elevation(1),
        render_system::RenderComponent::new(RenderTexture::new(Some(tex))),
        // render_system::Image(rs.get_image("res/wizards/crystal.png")),
        physics::Position(Rect::from(
            cx,
            cy,
            100.0,
            100.0,
            Align::Center,
            Align::Center
        )),
        Crystal
    );

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
        render_system::RenderComponent::new(RenderTexture::new(Some(tex))),
        physics::Position(Rect::from(
            cx,
            cy,
            w as f32,
            w as f32,
            Align::Center,
            Align::Center
        ))
    );
}
