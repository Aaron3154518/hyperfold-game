use hyperfold_engine::{
    ecs::entities::{Entity, NewEntity},
    framework::{physics, render_system},
    utils::rect::Rect,
};

#[hyperfold_engine::system(Init)]
fn init_crystal(
    entities: &mut dyn crate::_engine::AddComponent,
    rs: &mut render_system::RenderSystem,
    camera: &render_system::Camera,
) {
    let e = Entity::new();
    hyperfold_engine::add_components!(
        entities,
        e,
        render_system::Elevation(0),
        render_system::Image(rs.get_image("res/wizards/crystal.png")),
        physics::Position(Rect {
            x: camera.0.cx() - 50.0,
            y: camera.0.cy() - 50.0,
            w: 100.0,
            h: 100.0,
        }),
    );
}
