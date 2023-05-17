use hyperfold_engine::{
    ecs::entities::{Entity, NewEntity},
    framework::{
        physics, render_system,
        shapes::{Circle, Rectangle, ShapeTrait},
        texture_builder::TextureBuilder,
    },
    utils::{
        colors::{BLUE, GREEN, MAGENTA},
        rect::{Point, Rect},
    },
};

#[hyperfold_engine::system(Init)]
fn init_crystal(
    entities: &mut dyn crate::_engine::AddComponent,
    rs: &mut render_system::RenderSystem,
    camera: &render_system::Camera,
) {
    let (tb, tex) = TextureBuilder::new(rs.r.access(), 100, 100, MAGENTA);
    tb.draw(
        Rectangle::new()
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
    tb.draw(
        Circle::new()
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
        render_system::Image::from(tex),
        // render_system::Image(rs.get_image("res/wizards/crystal.png")),
        physics::Position(Rect {
            x: camera.0.cx() - 50.0,
            y: camera.0.cy() - 50.0,
            w: 100.0,
            h: 100.0,
        }),
    );
}
