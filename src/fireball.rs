use hyperfold_engine::{
    ecs::{
        components::Label,
        entities::{Entity, EntityTrash, NewEntity},
        events,
    },
    framework::{physics, render_system},
    utils::rect::{PointF, Rect},
};

#[hyperfold_engine::component]
struct Fireball;

pub fn new_fireball(
    entities: &mut dyn crate::_engine::AddComponent,
    rs: &mut render_system::RenderSystem,
    pos: PointF,
    target: PointF,
) {
    let e = Entity::new();
    let (mut dx, mut dy) = (target.x - pos.x, target.y - pos.y);
    let mag = (dx * dx + dy * dy).sqrt();
    if mag > 0.0 {
        dx *= 150.0 / mag;
        dy *= 150.0 / mag;
    }
    hyperfold_engine::add_components!(
        entities,
        e,
        render_system::Elevation(0),
        render_system::Image(rs.get_image("res/projectiles/fireball.png")),
        physics::Position(Rect {
            x: pos.x - 25.0,
            y: pos.y - 25.0,
            w: 50.0,
            h: 50.0,
        }),
        physics::PhysicsData {
            v: PointF { x: dx, y: dy },
            a: PointF::new(),
            boundary: None
        },
        Fireball
    );
}

#[hyperfold_engine::system]
fn update_fireball(
    _e: &events::core::Update,
    eid: &Entity,
    pos: &physics::Position,
    camera: &render_system::Camera,
    trash: &mut EntityTrash,
    _l: Label<Fireball>,
) {
    if !pos.0.intersects(&camera.0) {
        trash.0.push(*eid)
    }
}
