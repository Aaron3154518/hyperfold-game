use hyperfold_engine::ecs;
use hyperfold_engine::ecs::entities;
use hyperfold_engine::framework::{physics, render_system};
use hyperfold_engine::includes::*;
use hyperfold_engine::utils::rect::Rect;

#[ecs::system(Init)]
fn init_crystal(
    entities: &mut dyn crate::CFooT,
    rs: &mut render_system::RenderSystem,
    camera: &render_system::Camera,
) {
    let e = entities::new();
    entities.add_component(e, render_system::Elevation(0));
    entities.add_component(
        e,
        render_system::Image(rs.get_image("res/wizards/crystal.png")),
    );
    entities.add_component(
        e,
        physics::Position(Rect {
            x: camera.0.cx() - 50.0,
            y: camera.0.cy() - 50.0,
            w: 100.0,
            h: 100.0,
        }),
    );
}
