use hyperfold_engine::{
    ecs::{
        entities::{Entity, EntityTrash, NewEntity},
        events,
    },
    framework::{
        physics,
        render_system::{self, render_data::RenderAsset, AssetManager, Renderer},
    },
    utils::{
        number::Number,
        rect::{PointF, Rect},
    },
};

use crate::{
    crystal::{CrystalNumbers, CrystalPos},
    param_dag::NumDag,
    utils::elevations::Elevations,
};

#[hyperfold_engine::component]
struct Fireball {
    pub value: Number,
}

#[hyperfold_engine::event]
struct CreateFireball {
    pub pos: PointF,
    pub value: Number,
}

#[hyperfold_engine::system]
pub fn new_fireball(
    data: &CreateFireball,
    entities: &mut dyn crate::_engine::Components,
    r: &Renderer,
    am: &mut AssetManager,
) {
    let e = Entity::new();
    hyperfold_engine::add_components!(
        entities,
        e,
        Fireball { value: data.value },
        render_system::Elevation(Elevations::Projectiles as u8),
        render_system::RenderComponent::new(RenderAsset::from_file(
            "res/projectiles/fireball.png",
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
    );
}

hyperfold_engine::components!(
    UpdateFireball,
    pos: &'a physics::Position,
    pd: &'a mut physics::PhysicsData,
    fb: &'a Fireball
);

#[hyperfold_engine::system]
fn update_fireball(
    _e: &events::core::Update,
    trash: &mut EntityTrash,
    fballs: Vec<UpdateFireball>,
    crystal: CrystalPos,
    dag: &mut NumDag,
) {
    for UpdateFireball { eid, pos, pd, fb } in fballs {
        let target = crystal.pos.0.center();
        let (dx, dy) = (target.x - pos.0.cx(), target.y - pos.0.cy());
        let mag = (dx * dx + dy * dy).sqrt();
        if mag <= 5.0 {
            trash.0.push(*eid);
            dag.0.update(CrystalNumbers::Magic, |m| *m + fb.value);
        } else {
            pd.v.x = dx * 150.0 / mag;
            pd.v.y = dy * 150.0 / mag;
        }
    }
}
