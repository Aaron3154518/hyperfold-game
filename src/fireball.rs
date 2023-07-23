use hyperfold_engine::{
    ecs::{
        entities::{Entity, EntityTrash, NewEntity},
        events,
    },
    framework::{
        physics,
        render_system::{self, render_data::RenderAsset, AssetManager, Renderer},
    },
    utils::rect::{PointF, Rect},
};

use crate::crystal::{CrystalData, CrystalPos};

#[hyperfold_engine::component]
struct Fireball;

#[hyperfold_engine::event]
struct CreateFireball {
    pub pos: PointF,
}

#[hyperfold_engine::system]
pub fn new_fireball(
    data: &CreateFireball,
    entities: &mut dyn crate::_engine::AddComponent,
    r: &Renderer,
    am: &mut AssetManager,
) {
    let e = Entity::new();
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
            v: PointF::new(),
            a: PointF::new(),
            boundary: None
        },
        Fireball
    );
}

hyperfold_engine::components!(
    labels(Fireball),
    UpdateFireball,
    pos: &'a physics::Position,
    pd: &'a mut physics::PhysicsData
);

#[hyperfold_engine::system]
fn update_fireball(
    _e: &events::core::Update,
    trash: &mut EntityTrash,
    fballs: Vec<UpdateFireball>,
    crystal: CrystalPos,
    CrystalData { data, .. }: CrystalData,
) {
    for UpdateFireball { eid, pos, pd } in fballs {
        let target = crystal.pos.0.center();
        let (dx, dy) = (target.x - pos.0.cx(), target.y - pos.0.cy());
        let mag = (dx * dx + dy * dy).sqrt();
        if mag <= 1.0 {
            trash.0.push(*eid);
            data.magic += 100;
        } else {
            pd.v.x = dx * 150.0 / mag;
            pd.v.y = dy * 150.0 / mag;
        }
    }
}
