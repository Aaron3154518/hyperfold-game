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
    utils::{
        colors::{BLUE, GREEN, MAGENTA},
        rect::{Point, Rect},
    },
};

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

    let e = Entity::new();
    hyperfold_engine::add_components!(
        entities,
        e,
        render_system::Elevation(1),
        render_system::RenderComponent::new(RenderTexture::new(Some(tex))),
        // render_system::Image(rs.get_image("res/wizards/crystal.png")),
        physics::Position(Rect {
            x: screen.0.w as f32 / 2.0 - 50.0,
            y: screen.0.h as f32 / 2.0 - 50.0,
            w: 100.0,
            h: 100.0,
        }),
    );
}
