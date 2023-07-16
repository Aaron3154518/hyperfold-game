use hyperfold_engine::{
    ecs::{
        entities::{Entity, EntityTrash, NewEntity},
        events,
        systems::Entities,
    },
    framework::{
        physics,
        render_system::{self, render_data::RenderAsset, AssetManager, Renderer},
    },
    utils::rect::{PointF, Rect},
};

#[hyperfold_engine::component]
struct Fireball;

#[hyperfold_engine::event]
struct CreateFireball {
    pub pos: PointF,
    pub target: PointF,
}

#[hyperfold_engine::system]
pub fn new_fireball(
    data: &CreateFireball,
    entities: &mut dyn crate::_engine::AddComponent,
    r: &Renderer,
    am: &mut AssetManager,
) {
    let e = Entity::new();
    let (mut dx, mut dy) = (data.target.x - data.pos.x, data.target.y - data.pos.y);
    let mag = (dx * dx + dy * dy).sqrt();
    if mag > 0.0 {
        dx *= 150.0 / mag;
        dy *= 150.0 / mag;
    }
    hyperfold_engine::add_components!(
        entities,
        e,
        render_system::Elevation(0),
        render_system::RenderComponent::new(RenderAsset::from_file(
            "res/projectiles/fireball.png".to_string(),
            r,
            am
        )),
        physics::Position(Rect {
            x: data.pos.x - 25.0,
            y: data.pos.y - 25.0,
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

hyperfold_engine::components!(labels(Fireball), UpdateFireball, pos: &'a physics::Position,);

#[hyperfold_engine::system]
fn update_fireball(
    _e: &events::core::Update,
    camera: &render_system::Camera,
    trash: &mut EntityTrash,
    fballs: Entities<UpdateFireball>,
) {
    for UpdateFireball { eid, pos } in fballs {
        if !pos.0.intersects(&camera.0) {
            trash.0.push(*eid)
        }
    }
}
