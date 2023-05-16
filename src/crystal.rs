use hyperfold_engine::{
    ecs::entities::{Entity, NewEntity},
    framework::{
        physics, render_system,
        shapes::{Rectangle, ShapeTrait},
        texture_builder::TextureBuilder,
    },
    utils::{
        colors::{BLUE, MAGENTA},
        rect::Rect,
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

    let e = Entity::new();
    hyperfold_engine::add_components!(
        entities,
        e,
        render_system::Elevation(0),
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
